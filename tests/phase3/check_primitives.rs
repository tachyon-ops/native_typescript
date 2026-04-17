use tsnat_types::ty::{TypeArena, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN, TYPE_NULL, TYPE_UNDEFINED, TYPE_UNKNOWN, TYPE_ANY, TYPE_NEVER, Type};
use tsnat_types::assignability::AssignabilityChecker;

#[test]
fn test_primitive_assignability() {
    let mut arena = TypeArena::new();
    let checker = AssignabilityChecker::new(&arena);

    // Number
    assert!(checker.is_assignable(TYPE_NUMBER, TYPE_NUMBER));
    assert!(!checker.is_assignable(TYPE_STRING, TYPE_NUMBER));

    // String
    assert!(checker.is_assignable(TYPE_STRING, TYPE_STRING));
    assert!(!checker.is_assignable(TYPE_NUMBER, TYPE_STRING));

    // Boolean
    assert!(checker.is_assignable(TYPE_BOOLEAN, TYPE_BOOLEAN));
    assert!(!checker.is_assignable(TYPE_NUMBER, TYPE_BOOLEAN));

    // Top and bottom types
    assert!(checker.is_assignable(TYPE_NEVER, TYPE_NUMBER)); // never -> anything
    assert!(checker.is_assignable(TYPE_NUMBER, TYPE_UNKNOWN)); // anything -> unknown
    assert!(checker.is_assignable(TYPE_STRING, TYPE_ANY)); // anything -> any
    assert!(checker.is_assignable(TYPE_ANY, TYPE_NUMBER)); // any -> anything

    // Null and Undefined
    assert!(checker.is_assignable(TYPE_NULL, TYPE_NULL));
    assert!(checker.is_assignable(TYPE_UNDEFINED, TYPE_UNDEFINED));
    assert!(!checker.is_assignable(TYPE_NULL, TYPE_NUMBER));
}

#[test]
fn test_unions_and_intersections() {
    let mut arena = TypeArena::new();
    
    // (number | string)
    let num_str = arena.alloc(Type::Union(vec![TYPE_NUMBER, TYPE_STRING]));
    
    // number & string
    let num_and_str = arena.alloc(Type::Intersection(vec![TYPE_NUMBER, TYPE_STRING]));

    let checker = AssignabilityChecker::new(&arena);

    // number <: (number | string)
    assert!(checker.is_assignable(TYPE_NUMBER, num_str));
    assert!(checker.is_assignable(TYPE_STRING, num_str));
    // boolean </: (number | string)
    assert!(!checker.is_assignable(TYPE_BOOLEAN, num_str));

    // (number & string) <: number
    assert!(checker.is_assignable(num_and_str, TYPE_NUMBER));
    assert!(checker.is_assignable(num_and_str, TYPE_STRING));

    // number </: (number & string)
    assert!(!checker.is_assignable(TYPE_NUMBER, num_and_str));
}
