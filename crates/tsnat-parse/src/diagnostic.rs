use crate::span::Span;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TsnatError {
    #[error("Lex error: {message}")]
    Lex { message: String, #[label] span: Span },

    #[error("Parse error: {message}")]
    Parse { message: String, #[label] span: Span },

    #[error("Type error: {message}")]
    Type { message: String, #[label] span: Span, #[help] help: Option<String> },

    #[error("Runtime error: {message}")]
    Runtime { message: String, #[label] span: Option<Span> },

    #[error("FFI error: {message}")]
    Ffi { message: String },
}

pub type TsnatResult<T> = Result<T, TsnatError>;
