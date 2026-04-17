use tsnat_common::interner::Interner;
use tsnat_common::span::SourceMap;
use tsnat_lex::lexer::Lexer;
use tsnat_parse::parser::Parser;
use tsnat_parse::ast::*;
use bumpalo::Bump;

#[allow(dead_code)]
fn parse_expr_str(src: &str) -> String {
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let file_id = sm.add_file("test.ts".into(), src.to_string());
    let mut lexer = Lexer::new(src, file_id, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let program = parser.parse_program().unwrap();
    format!("{:#?}", program)
}

#[test]
fn test_parser_var_smoke() {
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let source = "const x = 42;";
    let file_id = sm.add_file("test.ts".into(), source.to_string());
    let mut lexer = Lexer::new(source, file_id, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.stmts.len(), 1);
    match &program.stmts[0] {
        Stmt::Var(decl) => {
            assert_eq!(decl.kind, VarKind::Const);
            assert_eq!(decl.decls.len(), 1);
            assert_eq!(interner.get(decl.decls[0].name), "x");
            assert!(decl.decls[0].init.is_some());
            match decl.decls[0].init.unwrap() {
                Expr::Number(val, _) => assert_eq!(*val, 42.0),
                _ => panic!("Expected number initializer"),
            }
        }
        _ => panic!("Expected VarDecl statement"),
    }
}

#[test]
fn test_binary_precedence() {
    // a + b * c - d should parse as (a + (b * c)) - d
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "a + b * c - d;";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    assert_eq!(prog.stmts.len(), 1);
    // Top level should be Sub(Add(a, Mul(b, c)), d)
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Binary(outer) => {
                assert_eq!(outer.op, BinaryOp::Sub);
                match outer.left {
                    Expr::Binary(inner) => {
                        assert_eq!(inner.op, BinaryOp::Add);
                        match inner.right {
                            Expr::Binary(mul) => assert_eq!(mul.op, BinaryOp::Mul),
                            _ => panic!("Expected Mul"),
                        }
                    }
                    _ => panic!("Expected inner Add"),
                }
            }
            _ => panic!("Expected Binary"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_conditional() {
    // a ? b : c
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "a ? b : c;";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Conditional(_) => { /* ok */ }
            other => panic!("Expected Conditional, got {:?}", other),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_member_access() {
    // a.b.c
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "a.b.c;";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Member(m) => {
                assert_eq!(interner.get(m.property), "c");
                match m.object {
                    Expr::Member(inner) => {
                        assert_eq!(interner.get(inner.property), "b");
                    }
                    _ => panic!("Expected inner Member"),
                }
            }
            _ => panic!("Expected Member"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_optional_chain() {
    // a?.b?.c
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "a?.b?.c;";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::OptChain(_) => { /* ok */ }
            _ => panic!("Expected OptChain"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_index_access() {
    // a[b]
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "a[b];";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Index(_) => { /* ok */ }
            _ => panic!("Expected Index"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_call_expr() {
    // f(1, 2, ...rest)
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "f(1, 2);";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Call(c) => {
                assert_eq!(c.args.len(), 2);
            }
            _ => panic!("Expected Call"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_new_expr() {
    // new Foo(a, b)
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "new Foo(a, b);";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::New(n) => {
                assert_eq!(n.args.len(), 2);
            }
            _ => panic!("Expected New"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_template_literal() {
    // `hello ${name}!`
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "`hello ${name}!`";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Expr(es) => match es.expr {
            Expr::Template(t) => {
                assert_eq!(t.quasis.len(), 2);
                assert_eq!(t.exprs.len(), 1);
                assert_eq!(interner.get(t.quasis[0]), "hello ");
                assert_eq!(interner.get(t.quasis[1]), "!");
            }
            _ => panic!("Expected Template"),
        },
        _ => panic!("Expected ExprStmt"),
    }
}

#[test]
fn test_typed_var_decl() {
    // const x: number = 42;
    let mut interner = Interner::new();
    let mut sm = SourceMap::new();
    let src = "const x: number = 42;";
    let fid = sm.add_file("t.ts".into(), src.into());
    let mut lexer = Lexer::new(src, fid, &mut interner);
    let tokens = lexer.tokenise_all().unwrap();
    let arena = Bump::new();
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let prog = parser.parse_program().unwrap();
    match &prog.stmts[0] {
        Stmt::Var(decl) => {
            assert_eq!(decl.kind, VarKind::Const);
            assert_eq!(interner.get(decl.decls[0].name), "x");
            match decl.decls[0].init.unwrap() {
                Expr::Number(v, _) => assert_eq!(*v, 42.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}
