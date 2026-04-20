pub mod ty;
pub mod assignability;
pub mod checker;
pub mod infer;

use tsnat_parse::ast::Program;
use tsnat_common::span::Span;

pub struct TypeChecker {
    pub diagnostics: Vec<Diagnostic>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    pub fn check(&mut self, _program: &Program) {
        // Pass-through for now
    }

    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    TS2304,
    TS2322,
    TS2339,
    TS2345,
    TS2366,
    TS2540,
    TS2554,
    TS7006,
}
