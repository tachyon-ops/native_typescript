use tsnat_lex::lexer::Lexer;
use tsnat_lex::token::TokenKind;
use tsnat_parse::interner::Interner;

#[test]
fn test_lex_smoke() {
    let mut interner = Interner::new();

    let cases = vec![
        ("42", vec![TokenKind::Number, TokenKind::Eof]),
        ("'hello'", vec![TokenKind::String, TokenKind::Eof]),
        ("`a${x}b`", vec![
            TokenKind::TemplateHead,
            TokenKind::Ident,
            TokenKind::TemplateTail,
            TokenKind::Eof,
        ]),
        ("const x = 1;", vec![
            TokenKind::KwConst,
            TokenKind::Ident,
            TokenKind::Eq,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]),
        ("x?.y", vec![
            TokenKind::Ident,
            TokenKind::QuestionDot,
            TokenKind::Ident,
            TokenKind::Eof,
        ]),
        ("x ?? y", vec![
            TokenKind::Ident,
            TokenKind::QuestionQuestion,
            TokenKind::Ident,
            TokenKind::Eof,
        ]),
        ("x **= 2", vec![
            TokenKind::Ident,
            TokenKind::StarStarEq,
            TokenKind::Number,
            TokenKind::Eof,
        ]),
    ];

    for (src, expected) in cases {
        let mut lexer = Lexer::new(src, 0, &mut interner);
        let tokens = lexer.tokenise_all().expect("failed to lex");
        let token_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(token_kinds, expected, "Failed on input: {}", src);
    }
}
