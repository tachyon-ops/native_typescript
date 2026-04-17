use tsnat_types::ty::{TypeArena, TYPE_NUMBER, TYPE_STRING, Type, ObjectType, PropertyType};
use tsnat_types::assignability::AssignabilityChecker;
use indexmap::IndexMap;
use tsnat_common::interner::{Interner, Symbol};

#[test]
fn test_object_assignability() {
    let mut arena = TypeArena::new();
    let mut interner = Interner::new();
    
    let sym_x = interner.intern("x");
    let sym_y = interner.intern("y");

    // { x: number }
    let mut props1 = IndexMap::new();
    props1.insert(sym_x, PropertyType { ty: TYPE_NUMBER, optional: false, readonly: false });
    let obj1 = arena.alloc(Type::Object(ObjectType { properties: props1 }));

    // { x: number, y: string }
    let mut props2 = IndexMap::new();
    props2.insert(sym_x, PropertyType { ty: TYPE_NUMBER, optional: false, readonly: false });
    props2.insert(sym_y, PropertyType { ty: TYPE_STRING, optional: false, readonly: false });
    let obj2 = arena.alloc(Type::Object(ObjectType { properties: props2 }));

    // { x: number, y?: string }
    let mut props3 = IndexMap::new();
    props3.insert(sym_x, PropertyType { ty: TYPE_NUMBER, optional: false, readonly: false });
    props3.insert(sym_y, PropertyType { ty: TYPE_STRING, optional: true, readonly: false });
    let obj3 = arena.alloc(Type::Object(ObjectType { properties: props3 }));

    let checker = AssignabilityChecker::new(&arena);

    // Identical
    assert!(checker.is_assignable(obj1, obj1));

    // { x: number, y: string } <: { x: number }
    assert!(checker.is_assignable(obj2, obj1));

    // { x: number } </: { x: number, y: string }
    assert!(!checker.is_assignable(obj1, obj2));

    // { x: number } <: { x: number, y?: string }
    assert!(checker.is_assignable(obj1, obj3));

    // { x: number, y: string } <: { x: number, y?: string }
    assert!(checker.is_assignable(obj2, obj3));
}
