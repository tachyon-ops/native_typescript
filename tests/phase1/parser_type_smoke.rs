use tsnat_parse::parser::Parser;
use tsnat_parse::ast::*;
use tsnat_common::interner::Interner;
use bumpalo::Bump;

#[test]
fn test_primitive_types() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "let a: number; let b: string; let c: boolean; let d: any; let e: void; let f: never;";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 6);
    for stmt in program.stmts.iter() {
        if let Stmt::Var(v) = stmt {
            assert!(v.decls[0].ty.is_some());
        } else {
            panic!("Expected VarDecl");
        }
    }
}

#[test]
fn test_literal_types() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "let a: 42; let b: 'hello'; let c: true;";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 3);
    assert!(matches!(unwrap_var_type(&program.stmts[0]), TypeNode::LiteralNumber(42.0, _)));
    assert!(matches!(unwrap_var_type(&program.stmts[1]), TypeNode::LiteralString(_, _)));
    assert!(matches!(unwrap_var_type(&program.stmts[2]), TypeNode::LiteralBool(true, _)));
}

#[test]
fn test_complex_types() {
    let arena = Bump::new();
    let interner = Interner::new();
    let source = "
        let a: Promise<string>;
        let b: string[];
        let c: [number, string];
        let d: string | number;
        let e: { x: number } & { y: string };
        let f: (x: number) => string;
        let g: ((x: number) => string) | null;
    ";
    let mut parser = Parser::new(source, &arena, &interner).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 7);
    assert!(matches!(unwrap_var_type(&program.stmts[0]), TypeNode::TypeRef(_)));
    assert!(matches!(unwrap_var_type(&program.stmts[1]), TypeNode::Array(_)));
    assert!(matches!(unwrap_var_type(&program.stmts[2]), TypeNode::Tuple(_, _)));
    assert!(matches!(unwrap_var_type(&program.stmts[3]), TypeNode::Union(_, _)));
    assert!(matches!(unwrap_var_type(&program.stmts[4]), TypeNode::Intersection(_, _)));
    assert!(matches!(unwrap_var_type(&program.stmts[5]), TypeNode::Function(_)));
    assert!(matches!(unwrap_var_type(&program.stmts[6]), TypeNode::Union(_, _)));
}

fn unwrap_var_type<'a>(stmt: &Stmt<'a>) -> &'a TypeNode<'a> {
    if let Stmt::Var(v) = stmt {
        v.decls[0].ty.as_ref().unwrap()
    } else {
        panic!("Expected VarDecl");
    }
}
