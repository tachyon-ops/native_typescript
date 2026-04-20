use bumpalo::Bump;
use tsnat_common::interner::Interner;
use tsnat_eval::eval::Evaluator;
use tsnat_eval::value::Value;
use tsnat_lex::lexer::Lexer;
use tsnat_parse::parser::Parser;
use std::process::Command;
use std::path::PathBuf;

fn build_dummy_lib() -> PathBuf {
    let dummy_dir = PathBuf::from("../tsnat-ffi/tests/dummy_cdylib");
    let status = Command::new("cargo")
        .arg("build")
        .current_dir(&dummy_dir)
        .status()
        .expect("Failed to run cargo build for dummy lib");
    assert!(status.success(), "dummy_cdylib failed to build");

    // return the built dylib path
    let mut path = dummy_dir.join("target/debug/libdummy_cdylib");
    if cfg!(target_os = "macos") {
        path.set_extension("dylib");
    } else if cfg!(target_os = "windows") {
        path.set_extension("dll");
    } else {
        path.set_extension("so");
    }
    path
}

#[allow(dead_code)]
fn run_eval(src: &str, lib_path: &PathBuf) -> Value<'static> {
    // Inject the exact path into the string for the import statement
    let final_src = src.replace("LIB_PATH", lib_path.to_str().unwrap());
    
    let mut interner = Interner::new();
    let arena = Bump::new();
    let mut lexer = Lexer::new(&final_src, 0, &mut interner);
    let tokens = lexer.tokenise_all().expect("Failed to tokenize");

    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let ast = parser.parse_program().expect("Failed to parse");

    let mut eval = Evaluator::new(&mut interner, &arena);
    let mut last_val = Value::Undefined;
    
    for stmt in ast.stmts.iter() {
        if let tsnat_eval::eval::ControlFlow::Normal(val) = eval.exec_stmt(&stmt).expect("Eval failed") {
            last_val = val;
        }
    }
    
    // Safety: unit tests static leak
    unsafe { std::mem::transmute(last_val) }
}

#[test]
fn test_ffi_native_invocation() {
    let lib_path = build_dummy_lib();
    
    let src = r#"
        import native dummy from 'LIB_PATH';
        declare native function test_add_f64(a: number, b: number): number;
        declare native function test_negate_bool(a: boolean): number;
        
        let sum = test_add_f64(10.5, 20.0);
        let neg = test_negate_bool(false);
    "#;
    
    let final_src = src.replace("LIB_PATH", lib_path.to_str().unwrap());
    
    let mut interner = Interner::new();
    let arena = Bump::new();
    let mut lexer = Lexer::new(&final_src, 0, &mut interner);
    let tokens = lexer.tokenise_all().expect("Failed to tokenize");

    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let ast = parser.parse_program().expect("Failed to parse");

    let mut eval = Evaluator::new(&mut interner, &arena);
    for stmt in ast.stmts.iter() {
        eval.exec_stmt(&stmt).expect("Eval failed");
    }
    
    let sum_sym = eval.interner.intern("sum");
    let neg_sym = eval.interner.intern("neg");
    
    let sum_val = eval.env.borrow().get(sum_sym).unwrap();
    let neg_val = eval.env.borrow().get(neg_sym).unwrap();
    
    match sum_val {
        Value::Number(n) => assert_eq!(n, 30.5),
        _ => panic!("Expected number"),
    }
    
    match neg_val {
        Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected number"),
    }
}
