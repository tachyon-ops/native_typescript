use tsnat_common::diagnostic::TsnatError;
use tsnat_common::span::Span;

#[test]
fn test_diagnostic_formatting() {
    let span = Span { file_id: 0, start: 0, end: 5 };
    
    let lex_err = TsnatError::Lex { message: "bad char".into(), span };
    let parse_err = TsnatError::Parse { message: "unexpected EOF".into(), span };
    let type_err = TsnatError::Type { message: "type mismatch".into(), span, help: Some("use string".into()) };
    let rt_err = TsnatError::Runtime { message: "null pointer".into(), span: Some(span) };
    let ffi_err = TsnatError::Ffi { message: "missing symbol".into() };

    assert!(!format!("{}", lex_err).is_empty());
    assert!(!format!("{}", parse_err).is_empty());
    assert!(!format!("{}", type_err).is_empty());
    assert!(!format!("{}", rt_err).is_empty());
    assert!(!format!("{}", ffi_err).is_empty());
}
