use tsnat_lex::token::TokenKind;

#[test]
fn test_token_display() {
    assert_eq!(TokenKind::KwConst.to_string(), "const");
    assert_eq!(TokenKind::Eof.to_string(), "EOF");
}
