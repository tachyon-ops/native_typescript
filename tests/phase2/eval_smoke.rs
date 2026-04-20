use tsnat_parse::parser::Parser;
use tsnat_lex::lexer::Lexer;
use tsnat_lex::token::TokenKind;
use tsnat_common::interner::Interner;
use tsnat_eval::evaluate;
use bumpalo::Bump;

fn eval_string(source: &str) -> String {
    let arena = Bump::new();
    let mut interner = Interner::new();
    
    let mut lexer = Lexer::new(source, 0, &mut interner);
    let mut tokens = Vec::new();
    while let Ok(tok) = lexer.next_token() {
        let is_eof = tok.kind == TokenKind::Eof;
        tokens.push(tok);
        if is_eof { break; }
    }

    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let program = parser.parse_program().unwrap();
    let result = evaluate(&program, &mut interner, &arena).unwrap();
    format!("{:?}", result)
}

#[test]
fn test_basic_arithmetic() {
    assert_eq!(eval_string("1 + 2 * 3"), "7");
    assert_eq!(eval_string("10 / 2 - 1"), "4");
}

#[test]
fn test_variables_and_shadowing() {
    let script = "
        let x = 10;
        {
            let x = 20;
        }
        x
    ";
    assert_eq!(eval_string(script), "10");
}

#[test]
fn test_if_else() {
    assert_eq!(eval_string("if (true) { 1 } else { 2 }"), "1");
    assert_eq!(eval_string("if (false) { 1 } else { 2 }"), "2");
}

#[test]
fn test_while_loop() {
    let script = "
        let i = 0;
        let sum = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    ";
    assert_eq!(eval_string(script), "10");
}

#[test]
fn test_functions_and_closures() {
    let script = "
        function makeAdder(x) {
            return function(y) {
                return x + y;
            };
        }
        let add10 = makeAdder(10);
        add10(5)
    ";
    assert_eq!(eval_string(script), "15");
}

#[test]
fn test_objects() {
    let script = "
        let o = { x: 42, y: 'hello' };
        o.x
    ";
    assert_eq!(eval_string(script), "42");
}

#[test]
fn test_console_log() {
    // This just verifies it doesn't crash and returns undefined
    assert_eq!(eval_string("console.log('Test')"), "undefined");
}
