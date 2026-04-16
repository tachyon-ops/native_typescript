use std::path::PathBuf;
use tsnat_parse::span::{SourceMap, Span};

#[test]
fn test_span_line_col() {
    let mut sm = SourceMap::new();
    let id = sm.add_file(PathBuf::from("test.ts"), "hello\nworld".to_string());
    let span = Span { file_id: id, start: 6, end: 11 };
    let (line, col) = sm.line_col(span);
    assert_eq!(line, 2);
    assert_eq!(col, 1);
}
