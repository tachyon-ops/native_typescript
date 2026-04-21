/// Shared test harness for TypeScriptNative.
///
/// All tests import from this module. The actual interpreter, parser, and
/// type checker are called through these helpers so that if an internal API
/// changes, only this file needs updating — not every individual test.

use tsnat_eval::{Interpreter, Value};
use tsnat_common::diagnostic::{TsnatError, TsnatResult};

// ── Interpreter helpers ──────────────────────────────────────────────────────

/// Evaluate a TypeScript source string and return the last expression value.
pub fn eval(src: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.eval_str(src).expect(&format!("eval failed on:\n{src}"))
}

/// Evaluate and expect a runtime error. Panics if evaluation succeeds.
pub fn expect_runtime_error(src: &str) -> TsnatError {
    let mut interp = Interpreter::new();
    interp
        .eval_str(src)
        .expect_err(&format!("expected runtime error but evaluation succeeded on:\n{src}"))
}

/// Evaluate and assert the result displays as the given string.
pub fn expect_display(src: &str, expected: &str) {
    let val = eval(src);
    assert_eq!(
        val.display(),
        expected,
        "source:\n{src}"
    );
}

/// Evaluate and assert the result is a number equal to `expected`.
pub fn expect_number(src: &str, expected: f64) {
    match eval(src) {
        Value::Number(n) => {
            assert!(
                (n - expected).abs() < 1e-10 || (n.is_nan() && expected.is_nan()),
                "expected number {expected}, got {n}\nsource:\n{src}"
            )
        }
        other => panic!("expected Number({expected}), got {other:?}\nsource:\n{src}"),
    }
}

/// Evaluate and assert the result is a string equal to `expected`.
pub fn expect_string(src: &str, expected: &str) {
    match eval(src) {
        Value::String(s) => assert_eq!(s.as_ref(), expected, "source:\n{src}"),
        other => panic!("expected String({expected:?}), got {other:?}\nsource:\n{src}"),
    }
}

/// Evaluate and assert the result is a boolean equal to `expected`.
pub fn expect_bool(src: &str, expected: bool) {
    match eval(src) {
        Value::Bool(b) => assert_eq!(b, expected, "source:\n{src}"),
        other => panic!("expected Bool({expected}), got {other:?}\nsource:\n{src}"),
    }
}

/// Evaluate and assert the result is `null`.
pub fn expect_null(src: &str) {
    match eval(src) {
        Value::Null => {}
        other => panic!("expected Null, got {other:?}\nsource:\n{src}"),
    }
}

/// Evaluate and assert the result is `undefined`.
pub fn expect_undefined(src: &str) {
    match eval(src) {
        Value::Undefined => {}
        other => panic!("expected Undefined, got {other:?}\nsource:\n{src}"),
    }
}

// ── Lexer helpers ────────────────────────────────────────────────────────────

use tsnat_lex::lexer::Lexer;
use tsnat_lex::token::{Token, TokenKind};
use tsnat_common::interner::Interner;
use tsnat_common::span::SourceMap;

pub fn lex(src: &str) -> Vec<Token> {
    let mut sm = SourceMap::new();
    let file_id = sm.add_file("test.ts".into(), src.to_string());
    let mut interner = Interner::new();
    let mut lexer = Lexer::new(src, file_id, &mut interner);
    lexer.tokenise_all().expect(&format!("lex failed on:\n{src}"))
}

pub fn lex_kinds(src: &str) -> Vec<TokenKind> {
    lex(src).into_iter().map(|t| t.kind).collect()
}

// ── Parser helpers ───────────────────────────────────────────────────────────

use tsnat_parse::parser::Parser;
use tsnat_parse::ast::Program;
use bumpalo::Bump;

pub fn parse(src: &str) -> (Program<'_>, Vec<TsnatError>) {
    // Note: Program borrows from arena. In tests we use a leaked arena for simplicity.
    let arena = Box::leak(Box::new(Bump::new()));
    let mut sm = SourceMap::new();
    let file_id = sm.add_file("test.ts".into(), src.to_string());
    let mut interner = Interner::new();
    let tokens = Lexer::new(src, file_id, &mut interner)
        .tokenise_all()
        .unwrap();
    let mut parser = Parser::new(&tokens, arena, &mut interner);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            let p = Program { 
                stmts: &[],
                span: tsnat_common::span::Span::DUMMY,
                source_type: tsnat_parse::ast::SourceType::Script,
            };
            return (p, vec![e]);
        }
    };
    let errors = vec![];
    (program, errors)
}

pub fn expect_parse_ok(src: &str) {
    let (_, errors) = parse(src);
    assert!(
        errors.is_empty(),
        "expected clean parse but got {} error(s):\n{:?}\nsource:\n{src}",
        errors.len(),
        errors
    );
}

pub fn expect_parse_error(src: &str) {
    let (_, errors) = parse(src);
    assert!(
        !errors.is_empty(),
        "expected parse error but parsing succeeded\nsource:\n{src}"
    );
}

// ── Type checker helpers ─────────────────────────────────────────────────────

use tsnat_types::{TypeChecker, Diagnostic, DiagnosticCode};

pub fn type_check(src: &str) -> Vec<Diagnostic> {
    let (program, _) = parse(src);
    let mut checker = TypeChecker::new();
    checker.check(&program);
    checker.take_diagnostics()
}

pub fn expect_type_ok(src: &str) {
    let diags = type_check(src);
    assert!(
        diags.is_empty(),
        "expected clean type check but got {} error(s):\n{:#?}\nsource:\n{src}",
        diags.len(),
        diags
    );
}

pub fn expect_type_error(src: &str, code: DiagnosticCode) {
    let diags = type_check(src);
    assert!(
        diags.iter().any(|d| d.code == code),
        "expected diagnostic {code:?} but got:\n{diags:#?}\nsource:\n{src}"
    );
}

pub fn expect_type_error_count(src: &str, count: usize) {
    let diags = type_check(src);
    assert_eq!(
        diags.len(),
        count,
        "expected {count} type error(s) but got {}:\n{diags:#?}\nsource:\n{src}",
        diags.len()
    );
}
