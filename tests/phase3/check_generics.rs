use tsnat_types::ty::{TypeArena, TypeId, Type, GenericType, TypeParamDecl, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN};
use tsnat_types::assignability::AssignabilityChecker;
use tsnat_types::infer::TypeInferencer;
use tsnat_common::interner::Interner;
use std::collections::HashMap;

#[test]
fn test_generic_assignability() {
    let mut arena = TypeArena::new();

    // Array (target id placeholder)
    let array_id = arena.alloc(Type::Object(tsnat_types::ty::ObjectType { properties: indexmap::IndexMap::new() }));

    // Array<number>
    let arr_number = arena.alloc(Type::Generic(GenericType {
        target: array_id,
        args: vec![TYPE_NUMBER],
    }));

    // Array<string>
    let arr_string = arena.alloc(Type::Generic(GenericType {
        target: array_id,
        args: vec![TYPE_STRING],
    }));

    let checker = AssignabilityChecker::new(&arena);

    // Same generic and arguments should be assignable
    assert!(checker.is_assignable(arr_number, arr_number));
    assert!(checker.is_assignable(arr_string, arr_string));

    // Array<number> is NOT assignable to Array<string>
    assert!(!checker.is_assignable(arr_number, arr_string));
}

#[test]
fn test_type_param_constraints() {
    let mut arena = TypeArena::new();
    let mut interner = Interner::new();
    let t_sym = interner.intern("T");

    // T extends string
    let t_param = arena.alloc(Type::TypeParam(TypeParamDecl {
        name: t_sym,
        constraint: Some(TYPE_STRING),
        default: None,
    }));

    let checker = AssignabilityChecker::new(&arena);

    // T extends string is assignable TO string
    assert!(checker.is_assignable(t_param, TYPE_STRING));

    // T extends string is NOT assignable TO number
    assert!(!checker.is_assignable(t_param, TYPE_NUMBER));

    // string is NOT assignable TO T (unless T = string is explicitly inferred in context, which is outside basic assignability)
    assert!(!checker.is_assignable(TYPE_STRING, t_param));
}

#[test]
fn test_type_substitution() {
    let mut arena = TypeArena::new();
    let mut interner = Interner::new();
    let mut inferencer = TypeInferencer::new(&mut arena);

    let t_sym = interner.intern("T");

    let t_param_type = inferencer.arena.alloc(Type::TypeParam(TypeParamDecl { name: t_sym, constraint: None, default: None }));
    let union_t_num = inferencer.arena.alloc(Type::Union(vec![t_param_type, TYPE_NUMBER]));

    // Substitute T -> string
    let mut subs = HashMap::new();
    subs.insert(t_sym, TYPE_STRING);

    let instantiated = inferencer.substitute(union_t_num, &subs);

    // instantiated should be loosely `string | number`
    let checker = AssignabilityChecker::new(inferencer.arena);

    // Both string and number should be assignable to the new instantiated type
    assert!(checker.is_assignable(TYPE_STRING, instantiated));
    assert!(checker.is_assignable(TYPE_NUMBER, instantiated));
    // boolean should not be
    assert!(!checker.is_assignable(TYPE_BOOLEAN, instantiated));
}
