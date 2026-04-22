/// Lexer tests — numeric and string literals.
/// ALGO: See SPECS.md §3 FR-LEX-001, FR-LEX-005, FR-LEX-006

#[path = "../../common/mod.rs"]
mod common;
use common::*;
use tsnat_lex::token::TokenKind::*;

// ── Numeric literals ──────────────────────────────────────────────────────────

#[test]
fn test_lex_integer() {
    assert_eq!(lex_kinds("42"), vec![Number, Eof]);
}

#[test]
fn test_lex_float() {
    assert_eq!(lex_kinds("3.14"), vec![Number, Eof]);
}

#[test]
fn test_lex_hex() {
    assert_eq!(lex_kinds("0xFF"), vec![Number, Eof]);
}

#[test]
fn test_lex_hex_upper() {
    assert_eq!(lex_kinds("0XFF"), vec![Number, Eof]);
}

#[test]
fn test_lex_binary() {
    assert_eq!(lex_kinds("0b1010"), vec![Number, Eof]);
}

#[test]
fn test_lex_octal() {
    assert_eq!(lex_kinds("0o777"), vec![Number, Eof]);
}

#[test]
fn test_lex_numeric_separator() {
    assert_eq!(lex_kinds("1_000_000"), vec![Number, Eof]);
}

#[test]
fn test_lex_bigint() {
    assert_eq!(lex_kinds("42n"), vec![BigInt, Eof]);
}

#[test]
fn test_lex_bigint_hex() {
    assert_eq!(lex_kinds("0xFFn"), vec![BigInt, Eof]);
}

#[test]
fn test_lex_numeric_separator_invalid_leading() {
    // _42 is an identifier, not a number with a leading separator
    let kinds = lex_kinds("_42");
    assert_eq!(kinds, vec![Ident, Eof]);
}

#[test]
fn test_lex_numeric_separator_invalid_double() {
    // 1__0 should produce a lex error
    let mut sm = tsnat_common::span::SourceMap::new();
    let id = sm.add_file("t.ts".into(), "1__0".to_string());
    let mut interner = tsnat_common::interner::Interner::new();
    let result = tsnat_lex::lexer::Lexer::new("1__0", id, &mut interner).tokenise_all();
    assert!(result.is_err(), "expected lex error on double numeric separator");
}

#[test]
fn test_lex_float_bigint_invalid() {
    let mut sm = tsnat_common::span::SourceMap::new();
    let id = sm.add_file("t.ts".into(), "3.14n".to_string());
    let mut interner = tsnat_common::interner::Interner::new();
    let result = tsnat_lex::lexer::Lexer::new("3.14n", id, &mut interner).tokenise_all();
    assert!(result.is_err(), "expected lex error on float bigint");
}

// ── String literals ───────────────────────────────────────────────────────────

#[test]
fn test_lex_double_quote_string() {
    let tokens = lex(r#""hello""#);
    assert_eq!(tokens[0].kind, String);
}

#[test]
fn test_lex_single_quote_string() {
    let tokens = lex("'world'");
    assert_eq!(tokens[0].kind, String);
}

#[test]
fn test_lex_escape_newline() {
    let tokens = lex(r#""\n""#);
    assert_eq!(tokens[0].kind, String);
}

#[test]
fn test_lex_escape_unicode_4digit() {
    let tokens = lex(r#""\u0041""#); // 'A'
    assert_eq!(tokens[0].kind, String);
}

#[test]
fn test_lex_escape_unicode_brace() {
    let tokens = lex(r#""\u{1F600}""#); // 😀
    assert_eq!(tokens[0].kind, String);
}

#[test]
fn test_lex_escape_hex() {
    let tokens = lex(r#""\x41""#); // 'A'
    assert_eq!(tokens[0].kind, String);
}

// ── Template literals ─────────────────────────────────────────────────────────

#[test]
fn test_lex_no_subst_template() {
    assert_eq!(lex_kinds("`hello`"), vec![NoSubstTemplate, Eof]);
}

#[test]
fn test_lex_template_with_substitution() {
    assert_eq!(
        lex_kinds("`a${x}b`"),
        vec![TemplateHead, Ident, TemplateTail, Eof]
    );
}

#[test]
fn test_lex_template_multiple_substitutions() {
    assert_eq!(
        lex_kinds("`${a}+${b}`"),
        vec![TemplateHead, Ident, TemplateMiddle, Ident, TemplateTail, Eof]
    );
}

#[test]
fn test_lex_nested_template() {
    // `outer${`inner${x}`}end`
    assert_eq!(
        lex_kinds("`outer${`inner${x}`}end`"),
        vec![
            TemplateHead,
            TemplateHead,
            Ident,
            TemplateTail,
            TemplateTail,
            Eof
        ]
    );
}

// ── Regex literals ────────────────────────────────────────────────────────────

#[test]
fn test_lex_regex_after_assign() {
    assert_eq!(lex_kinds("x = /pattern/g"), vec![Ident, Eq, Regex, Eof]);
}

#[test]
fn test_lex_regex_after_lparen() {
    assert_eq!(lex_kinds("(/test/)"), vec![LParen, Regex, RParen, Eof]);
}

#[test]
fn test_lex_division_not_regex() {
    // After an identifier, / is division not regex
    assert_eq!(lex_kinds("a / b"), vec![Ident, Slash, Ident, Eof]);
}
