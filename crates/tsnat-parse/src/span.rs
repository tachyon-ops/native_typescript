use std::path::PathBuf;

/// A half-open byte range [start, end) into a source file.
/// u32 is sufficient for files up to 4 GiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: u32,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const DUMMY: Self = Self { file_id: 0, start: 0, end: 0 };
    
    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(self.file_id, other.file_id);
        Self {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub struct SourceFile {
    pub id: u32,
    pub path: PathBuf,
    pub content: String,
    /// Byte offset of each line start, for line/col computation.
    pub line_starts: Vec<u32>,
}

impl SourceFile {
    fn new(id: u32, path: PathBuf, content: String) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in content.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self {
            id,
            path,
            content,
            line_starts,
        }
    }
}

pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, path: PathBuf, content: String) -> u32 {
        let id = self.files.len() as u32;
        self.files.push(SourceFile::new(id, path, content));
        id
    }

    pub fn get_file(&self, id: u32) -> &SourceFile {
        &self.files[id as usize]
    }

    pub fn line_col(&self, span: Span) -> (u32, u32) {
        let file = self.get_file(span.file_id);
        let start = span.start;
        
        let line_idx = match file.line_starts.binary_search(&start) {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };
        
        let line = (line_idx + 1) as u32; // 1-based
        let line_start_byte = file.line_starts[line_idx] as usize;
        let prefix = &file.content[line_start_byte..start as usize];
        let col = (prefix.chars().count() + 1) as u32; // 1-based
        
        (line, col)
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(span: Span) -> Self {
        (span.start as usize, (span.end - span.start) as usize).into()
    }
}
