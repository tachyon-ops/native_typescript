use tsnat_types::ty::{TypeArena, Type, TypeParamDecl, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN};
use tsnat_types::infer::TypeInferencer;
use tsnat_types::assignability::AssignabilityChecker;
use tsnat_common::interner::Interner;

#[test]
fn test_concrete_conditional_evaluation() {
    let mut arena = TypeArena::new();
    let mut inferencer = TypeInferencer::new(&mut arena);

    // number extends number ? string : boolean
    let evaluated = inferencer.evaluate_conditional(TYPE_NUMBER, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN);

    // Should evaluate to string
    assert_eq!(evaluated, TYPE_STRING);

    // string extends number ? string : boolean
    let evaluated2 = inferencer.evaluate_conditional(TYPE_STRING, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN);

    // Should evaluate to boolean
    assert_eq!(evaluated2, TYPE_BOOLEAN);
}

#[test]
fn test_lazy_conditional_evaluation() {
    let mut arena = TypeArena::new();
    let mut interner = Interner::new();
    let mut inferencer = TypeInferencer::new(&mut arena);

    let t_sym = interner.intern("T");
    let t_param = inferencer.arena.alloc(Type::TypeParam(TypeParamDecl { name: t_sym, constraint: None, default: None }));

    // T extends number ? string : boolean
    let conditional = inferencer.evaluate_conditional(t_param, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN);

    // It should defer evaluation and create a ConditionalType
    match inferencer.arena.get(conditional) {
        Type::Conditional(c) => {
            assert_eq!(c.check_type, t_param);
            assert_eq!(c.extends_type, TYPE_NUMBER);
            assert_eq!(c.true_type, TYPE_STRING);
            assert_eq!(c.false_type, TYPE_BOOLEAN);
        }
        _ => panic!("Expected a Deferred ConditionalType!"),
    }
}

#[test]
fn test_distributive_conditional() {
    let mut arena = TypeArena::new();
    let mut inferencer = TypeInferencer::new(&mut arena);

    // (number | string)
    let union_num_str = inferencer.arena.alloc(Type::Union(vec![TYPE_NUMBER, TYPE_STRING]));

    // (number | string) extends number ? string : boolean
    let dist = inferencer.evaluate_conditional(union_num_str, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN);

    let checker = AssignabilityChecker::new(inferencer.arena);

    // Result should be (string | boolean)
    // Both string and boolean should be assignable to the distributed result.
    assert!(checker.is_assignable(TYPE_STRING, dist));
    assert!(checker.is_assignable(TYPE_BOOLEAN, dist));
    assert!(!checker.is_assignable(TYPE_NUMBER, dist));
}
