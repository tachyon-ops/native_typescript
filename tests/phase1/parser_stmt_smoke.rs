use tsnat_parse::parser::Parser;
use tsnat_parse::ast::*;
use tsnat_common::interner::Interner;
use bumpalo::Bump;

#[test]
fn test_if_else() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "if (x) { a; } else if (y) { b; } else { c; }";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);
    match program.stmts[0] {
        Stmt::If(if_stmt) => {
            assert!(matches!(if_stmt.test, Expr::Ident(_, _)));
            match if_stmt.consequent {
                Stmt::Block(b) => assert_eq!(b.stmts.len(), 1),
                _ => panic!("Expected block"),
            }
            match if_stmt.alternate.as_ref().unwrap() {
                Stmt::If(inner_if) => {
                    assert!(matches!(inner_if.test, Expr::Ident(_, _)));
                    assert!(inner_if.alternate.is_some());
                }
                _ => panic!("Expected else if"),
            }
        }
        _ => panic!("Expected IfStmt"),
    }
}

#[test]
fn test_loops() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "
        while (true) { break; }
        do { continue; } while (false);
        for (let i = 0; i < 10; i++) { i; }
        for (const x of items) { x; }
        for (const k in obj) { k; }
    ";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 5);
    assert!(matches!(program.stmts[0], Stmt::While(_)));
    assert!(matches!(program.stmts[1], Stmt::DoWhile(_)));
    assert!(matches!(program.stmts[2], Stmt::For(_)));
    assert!(matches!(program.stmts[3], Stmt::ForOf(_)));
    assert!(matches!(program.stmts[4], Stmt::ForIn(_)));
}

#[test]
fn test_try_catch_finally() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "try { throw 1; } catch (e) { log(e); } finally { done(); }";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);
    match program.stmts[0] {
        Stmt::Try(try_stmt) => {
            assert!(try_stmt.handler.is_some());
            assert!(try_stmt.finalizer.is_some());
            assert_eq!(try_stmt.handler.unwrap().param, Some(interner.intern("e")));
        }
        _ => panic!("Expected TryStmt"),
    }
}

#[test]
fn test_switch() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "switch (x) { case 1: a; break; default: b; }";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);
    match program.stmts[0] {
        Stmt::Switch(s) => {
            assert_eq!(s.cases.len(), 2);
            assert!(s.cases[0].test.is_some());
            assert!(s.cases[1].test.is_none());
        }
        _ => panic!("Expected SwitchStmt"),
    }
}

#[test]
fn test_class_decl() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "class Foo extends Bar {
        private x: number = 0;
        static y = 1;
        constructor(x: number) { this.x = x; }
        public getX(): number { return this.x; }
    }";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);
    match program.stmts[0] {
        Stmt::Class(c) => {
            assert_eq!(c.id, Some(interner.intern("Foo")));
            assert!(c.super_class.is_some());
            assert_eq!(c.body.len(), 4);
        }
        _ => panic!("Expected ClassDecl"),
    }
}

#[test]
fn test_import_export() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "
        import { a as b, c } from 'mod';
        import defaultExport from 'mod';
        import * as ns from 'mod';
        export const x = 1;
        export default function f() {}
        export { y as z };
    ";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 6);
    assert!(matches!(program.stmts[0], Stmt::Import(_)));
    assert!(matches!(program.stmts[1], Stmt::Import(_)));
    assert!(matches!(program.stmts[2], Stmt::Import(_)));
    assert!(matches!(program.stmts[3], Stmt::Export(_)));
    assert!(matches!(program.stmts[4], Stmt::Export(_)));
    assert!(matches!(program.stmts[5], Stmt::Export(_)));
}
