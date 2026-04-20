/// Lexer tests — keywords, operators, punctuation, ASI flag.
/// ALGO: See SPECS.md §3 FR-LEX-001, FR-LEX-003

mod common;
use common::*;
use tsnat_lex::TokenKind::*;

// ── Value keywords ────────────────────────────────────────────────────────────

#[test]
fn test_lex_keywords_value() {
    let cases = [
        ("break", KwBreak), ("case", KwCase), ("catch", KwCatch),
        ("class", KwClass), ("const", KwConst), ("continue", KwContinue),
        ("debugger", KwDebugger), ("default", KwDefault), ("delete", KwDelete),
        ("do", KwDo), ("else", KwElse), ("enum", KwEnum),
        ("export", KwExport), ("extends", KwExtends), ("false", KwFalse),
        ("finally", KwFinally), ("for", KwFor), ("function", KwFunction),
        ("if", KwIf), ("import", KwImport), ("in", KwIn),
        ("instanceof", KwInstanceof), ("let", KwLet), ("new", KwNew),
        ("null", KwNull), ("return", KwReturn), ("super", KwSuper),
        ("switch", KwSwitch), ("this", KwThis), ("throw", KwThrow),
        ("true", KwTrue), ("try", KwTry), ("typeof", KwTypeof),
        ("undefined", KwUndefined), ("var", KwVar), ("void", KwVoid),
        ("while", KwWhile), ("yield", KwYield),
        ("async", KwAsync), ("await", KwAwait), ("of", KwOf),
        ("from", KwFrom), ("as", KwAs),
    ];
    for (src, expected_kind) in cases {
        let kinds = lex_kinds(src);
        assert_eq!(
            kinds[0], expected_kind,
            "keyword '{src}' should produce {expected_kind:?}"
        );
    }
}

// ── Type-position keywords ────────────────────────────────────────────────────

#[test]
fn test_lex_keywords_type() {
    let cases = [
        ("type", KwType), ("interface", KwInterface), ("namespace", KwNamespace),
        ("declare", KwDeclare), ("abstract", KwAbstract), ("override", KwOverride),
        ("readonly", KwReadonly), ("keyof", KwKeyof), ("infer", KwInfer),
        ("is", KwIs), ("asserts", KwAsserts), ("public", KwPublic),
        ("private", KwPrivate), ("protected", KwProtected),
        ("never", KwNever), ("unknown", KwUnknown), ("any", KwAny),
        ("satisfies", KwSatisfies),
    ];
    for (src, expected_kind) in cases {
        let kinds = lex_kinds(src);
        assert_eq!(
            kinds[0], expected_kind,
            "keyword '{src}' should produce {expected_kind:?}"
        );
    }
}

// ── Contextual keywords are identifiers outside type position ─────────────────

#[test]
fn test_lex_type_as_identifier_in_value_position() {
    // `type` used as a variable name is valid
    let kinds = lex_kinds("const type = 1;");
    assert_eq!(kinds, vec![KwConst, KwType, Eq, Number, Semicolon, Eof]);
    // The parser disambiguates; lexer emits KwType always
}

// ── Operators ─────────────────────────────────────────────────────────────────

#[test]
fn test_lex_arithmetic_operators() {
    assert_eq!(
        lex_kinds("a + b - c * d / e % f ** g"),
        vec![Ident, Plus, Ident, Minus, Ident, Star, Ident, Slash, Ident, Percent, Ident, StarStar, Ident, Eof]
    );
}

#[test]
fn test_lex_comparison_operators() {
    assert_eq!(
        lex_kinds("== != === !== < > <= >="),
        vec![EqEq, BangEq, EqEqEq, BangEqEq, Lt, Gt, LtEq, GtEq, Eof]
    );
}

#[test]
fn test_lex_logical_operators() {
    assert_eq!(
        lex_kinds("&& || ?? !"),
        vec![AmpAmp, PipePipe, QuestionQuestion, Bang, Eof]
    );
}

#[test]
fn test_lex_bitwise_operators() {
    assert_eq!(
        lex_kinds("& | ^ ~ << >> >>>"),
        vec![Amp, Pipe, Caret, Tilde, LtLt, GtGt, GtGtGt, Eof]
    );
}

#[test]
fn test_lex_assignment_operators() {
    assert_eq!(
        lex_kinds("+= -= *= /= %= **= &&= ||= ??="),
        vec![PlusEq, MinusEq, StarEq, SlashEq, PercentEq, StarStarEq, AmpAmpEq, PipePipeEq, QuestionQuestionEq, Eof]
    );
}

#[test]
fn test_lex_increment_decrement() {
    assert_eq!(
        lex_kinds("++ --"),
        vec![PlusPlus, MinusMinus, Eof]
    );
}

#[test]
fn test_lex_arrow() {
    assert_eq!(lex_kinds("=>"), vec![Arrow, Eof]);
}

#[test]
fn test_lex_optional_chain() {
    assert_eq!(lex_kinds("?."), vec![QuestionDot, Eof]);
}

#[test]
fn test_lex_spread() {
    assert_eq!(lex_kinds("..."), vec![DotDotDot, Eof]);
}

#[test]
fn test_lex_decorator_at() {
    assert_eq!(lex_kinds("@"), vec![At, Eof]);
}

// ── Punctuation ───────────────────────────────────────────────────────────────

#[test]
fn test_lex_punctuation() {
    assert_eq!(
        lex_kinds("( ) { } [ ] ; : , ."),
        vec![LParen, RParen, LBrace, RBrace, LBracket, RBracket, Semicolon, Colon, Comma, Dot, Eof]
    );
}

// ── ASI flag ─────────────────────────────────────────────────────────────────

#[test]
fn test_lex_asi_flag_set_after_newline() {
    let tokens = lex("a\nb");
    // Token 'b' should have has_preceding_newline = true
    assert!(
        tokens[1].has_preceding_newline,
        "token after newline should have has_preceding_newline=true"
    );
}

#[test]
fn test_lex_asi_flag_not_set_same_line() {
    let tokens = lex("a b");
    assert!(
        !tokens[1].has_preceding_newline,
        "token on same line should have has_preceding_newline=false"
    );
}

// ── Full statement tokenisation ───────────────────────────────────────────────

#[test]
fn test_lex_const_declaration() {
    assert_eq!(
        lex_kinds("const x = 1;"),
        vec![KwConst, Ident, Eq, Number, Semicolon, Eof]
    );
}

#[test]
fn test_lex_optional_chain_full() {
    assert_eq!(
        lex_kinds("x?.y"),
        vec![Ident, QuestionDot, Ident, Eof]
    );
}

#[test]
fn test_lex_nullish_coalescing() {
    assert_eq!(
        lex_kinds("x ?? y"),
        vec![Ident, QuestionQuestion, Ident, Eof]
    );
}

#[test]
fn test_lex_power_assign() {
    assert_eq!(
        lex_kinds("x **= 2"),
        vec![Ident, StarStarEq, Number, Eof]
    );
}
