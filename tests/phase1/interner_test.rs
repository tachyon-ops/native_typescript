use tsnat_common::interner::Interner;

#[test]
fn test_interner() {
    let mut interner = Interner::new();
    let a = interner.intern("hello");
    let b = interner.intern("hello");
    let c = interner.intern("world");
    
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(interner.get(a), "hello");
}
