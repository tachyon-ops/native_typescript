/// Phase 3 gate test — type checker.
///
/// This test must pass before any Phase 4 work begins.
/// Tests that the type checker correctly catches type errors and accepts valid programs.
/// ALGO: See SPECS.md §5 FR-TYP-001 through FR-TYP-010

mod common;
use common::*;
use tsnat_types::DiagnosticCode::*;

// ════════════════════════════════════════════════════════════════════════════
// Gate test — used as phase exit criterion
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gate_type_errors_emitted_with_correct_locations() {
    // Three errors, each on a known line
    let src = r#"
const x: number = "hello";
function f(a: number): string { return a; }
const y = { a: 1 };
const z = y.nonexistent;
    "#;
    let diags = type_check(src);
    assert_eq!(diags.len(), 3, "expected exactly 3 diagnostics, got: {diags:#?}");
    assert!(diags.iter().any(|d| d.code == TS2322));
    assert!(diags.iter().any(|d| d.code == TS2339));
}

#[test]
fn test_gate_clean_program_no_errors() {
    expect_type_ok(r#"
const x: number = 42;
const s: string = "hello";
function add(a: number, b: number): number { return a + b; }
const result = add(1, 2);
    "#);
}

// ════════════════════════════════════════════════════════════════════════════
// TS2322 — Type is not assignable
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_2322_string_to_number() {
    expect_type_error(r#"const x: number = "hello";"#, TS2322);
}

#[test]
fn test_type_2322_number_to_string() {
    expect_type_error("const x: string = 42;", TS2322);
}

#[test]
fn test_type_2322_null_to_non_nullable() {
    expect_type_error("const x: number = null;", TS2322);
}

#[test]
fn test_type_2322_return_type_mismatch() {
    expect_type_error("function f(): string { return 42; }", TS2322);
}

#[test]
fn test_type_2322_object_missing_property() {
    expect_type_error(
        r#"
        interface Point { x: number; y: number; }
        const p: Point = { x: 1 };
        "#,
        TS2322,
    );
}

#[test]
fn test_type_2322_ok_subtype() {
    expect_type_ok(
        r#"
        interface Animal { name: string; }
        interface Dog extends Animal { breed: string; }
        const d: Dog = { name: 'Fido', breed: 'Lab' };
        const a: Animal = d;
        "#,
    );
}

// ════════════════════════════════════════════════════════════════════════════
// TS2339 — Property does not exist
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_2339_nonexistent_property() {
    expect_type_error(
        "const y = { a: 1 }; const z = y.nonexistent;",
        TS2339,
    );
}

#[test]
fn test_type_2339_nonexistent_method() {
    expect_type_error(
        r#"const s: string = "hello"; s.nonexistentMethod();"#,
        TS2339,
    );
}

// ════════════════════════════════════════════════════════════════════════════
// TS2304 — Cannot find name
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_2304_undeclared_variable() {
    expect_type_error("console.log(undeclaredVar);", TS2304);
}

// ════════════════════════════════════════════════════════════════════════════
// TS2345 — Argument type not assignable to parameter type
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_2345_wrong_argument_type() {
    expect_type_error(
        r#"function f(x: number): void {} f("hello");"#,
        TS2345,
    );
}

#[test]
fn test_type_2345_too_many_args_is_different_error() {
    // Too many arguments is TS2554, not TS2345
    expect_type_error("function f(x: number): void {} f(1, 2);", TS2554);
}

// ════════════════════════════════════════════════════════════════════════════
// TS7006 — Implicit any parameter
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_7006_implicit_any() {
    expect_type_error("function f(x) { return x; }", TS7006);
}

// ════════════════════════════════════════════════════════════════════════════
// Structural typing — valid cases
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_structural_duck_typing() {
    // Object with extra properties is assignable to a narrower interface
    expect_type_ok(r#"
        interface Printable { toString(): string; }
        function print(p: Printable): void { console.log(p.toString()); }
        print({ toString: () => "hello", extra: 42 } as any);
    "#);
}

#[test]
fn test_type_structural_compatible_objects() {
    expect_type_ok(r#"
        function area(shape: { width: number; height: number }): number {
            return shape.width * shape.height;
        }
        const rect = { width: 10, height: 5, color: 'red' };
        area(rect);
    "#);
}

// ════════════════════════════════════════════════════════════════════════════
// Union types
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_union_assignable() {
    expect_type_ok("const x: string | number = 42;");
}

#[test]
fn test_type_union_assignable_string() {
    expect_type_ok(r#"const x: string | number = "hello";"#);
}

#[test]
fn test_type_union_not_assignable() {
    expect_type_error("const x: string | number = true;", TS2322);
}

#[test]
fn test_type_union_narrowing_typeof() {
    expect_type_ok(r#"
        function f(x: string | number): string {
            if (typeof x === 'string') {
                return x.toUpperCase(); // x is string here
            }
            return x.toFixed(2); // x is number here
        }
    "#);
}

#[test]
fn test_type_union_narrowing_discriminant() {
    expect_type_ok(r#"
        type Shape =
            | { kind: 'circle'; radius: number }
            | { kind: 'square'; side: number };

        function area(s: Shape): number {
            if (s.kind === 'circle') {
                return Math.PI * s.radius ** 2;
            } else {
                return s.side ** 2;
            }
        }
    "#);
}

#[test]
fn test_type_union_exhaustive_check() {
    expect_type_error(r#"
        type Direction = 'north' | 'south' | 'east' | 'west';
        function move(d: Direction): number {
            switch (d) {
                case 'north': return 1;
                case 'south': return -1;
                // Missing east and west — should error on return type
            }
        }
    "#, TS2366); // TS2366: Function lacks ending return statement
}

// ════════════════════════════════════════════════════════════════════════════
// Generics
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_generic_identity() {
    expect_type_ok(r#"
        function identity<T>(x: T): T { return x; }
        const n: number = identity(42);
        const s: string = identity("hello");
    "#);
}

#[test]
fn test_type_generic_constraint() {
    expect_type_ok(r#"
        function getLength<T extends { length: number }>(x: T): number {
            return x.length;
        }
        getLength("hello");
        getLength([1, 2, 3]);
    "#);
}

#[test]
fn test_type_generic_constraint_violation() {
    expect_type_error(r#"
        function getLength<T extends { length: number }>(x: T): number {
            return x.length;
        }
        getLength(42);
    "#, TS2345);
}

#[test]
fn test_type_generic_pick() {
    expect_type_ok(r#"
        function pick<T, K extends keyof T>(obj: T, key: K): T[K] {
            return obj[key];
        }
        const obj = { x: 1, y: 'hello' };
        const x: number = pick(obj, 'x');
        const y: string = pick(obj, 'y');
    "#);
}

// ════════════════════════════════════════════════════════════════════════════
// Control flow narrowing
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_narrowing_null_check() {
    expect_type_ok(r#"
        function f(x: string | null): number {
            if (x !== null) {
                return x.length; // x is string here, not string | null
            }
            return 0;
        }
    "#);
}

#[test]
fn test_type_narrowing_instanceof() {
    expect_type_ok(r#"
        class Foo { foo(): void {} }
        class Bar { bar(): void {} }

        function f(x: Foo | Bar): void {
            if (x instanceof Foo) {
                x.foo();
            } else {
                x.bar();
            }
        }
    "#);
}

#[test]
fn test_type_narrowing_type_guard() {
    expect_type_ok(r#"
        function isString(x: unknown): x is string {
            return typeof x === 'string';
        }
        function f(x: unknown): number {
            if (isString(x)) {
                return x.length; // x is string
            }
            return 0;
        }
    "#);
}

#[test]
fn test_type_narrowing_truthy() {
    expect_type_ok(r#"
        function f(x: string | undefined): number {
            if (x) {
                return x.length; // x is string (not undefined)
            }
            return 0;
        }
    "#);
}

// ════════════════════════════════════════════════════════════════════════════
// Conditional types
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_conditional_basic() {
    expect_type_ok(r#"
        type IsString<T> = T extends string ? true : false;
        const a: IsString<string> = true;
        const b: IsString<number> = false;
    "#);
}

#[test]
fn test_type_conditional_infer() {
    expect_type_ok(r#"
        type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;
        type N = ReturnType<() => number>;
        const x: N = 42;
    "#);
}

// ════════════════════════════════════════════════════════════════════════════
// Mapped types
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_mapped_partial() {
    expect_type_ok(r#"
        type MyPartial<T> = { [K in keyof T]?: T[K] };
        interface User { name: string; age: number; }
        const u: MyPartial<User> = {}; // all optional
        const v: MyPartial<User> = { name: 'Alice' };
    "#);
}

#[test]
fn test_type_mapped_readonly() {
    expect_type_ok(r#"
        type Frozen<T> = { readonly [K in keyof T]: T[K] };
        interface Point { x: number; y: number; }
        const p: Frozen<Point> = { x: 1, y: 2 };
    "#);
}

#[test]
fn test_type_mapped_readonly_violation() {
    expect_type_error(r#"
        type Frozen<T> = { readonly [K in keyof T]: T[K] };
        interface Point { x: number; y: number; }
        const p: Frozen<Point> = { x: 1, y: 2 };
        p.x = 3;
    "#, TS2540); // TS2540: Cannot assign to 'x' because it is a read-only property
}

// ════════════════════════════════════════════════════════════════════════════
// Template literal types
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_template_literal() {
    expect_type_ok(r#"
        type EventName = `on${'Click' | 'Focus' | 'Blur'}`;
        const e: EventName = 'onClick';
    "#);
}

#[test]
fn test_type_template_literal_invalid() {
    expect_type_error(r#"
        type EventName = `on${'Click' | 'Focus'}`;
        const e: EventName = 'onChange';
    "#, TS2322);
}

// ════════════════════════════════════════════════════════════════════════════
// Utility types
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_partial() {
    expect_type_ok(r#"
        interface User { name: string; age: number; }
        const u: Partial<User> = { name: 'Alice' };
    "#);
}

#[test]
fn test_type_required() {
    expect_type_error(r#"
        interface User { name?: string; age?: number; }
        const u: Required<User> = { name: 'Alice' }; // missing age
    "#, TS2322);
}

#[test]
fn test_type_readonly() {
    expect_type_error(r#"
        interface Point { x: number; y: number; }
        const p: Readonly<Point> = { x: 1, y: 2 };
        p.x = 3;
    "#, TS2540);
}

#[test]
fn test_type_pick() {
    expect_type_ok(r#"
        interface User { name: string; age: number; email: string; }
        type UserPreview = Pick<User, 'name' | 'email'>;
        const u: UserPreview = { name: 'Alice', email: 'alice@example.com' };
    "#);
}

#[test]
fn test_type_omit() {
    expect_type_ok(r#"
        interface User { name: string; age: number; password: string; }
        type SafeUser = Omit<User, 'password'>;
        const u: SafeUser = { name: 'Alice', age: 30 };
    "#);
}

#[test]
fn test_type_record() {
    expect_type_ok(r#"
        const scores: Record<string, number> = { alice: 100, bob: 95 };
    "#);
}

#[test]
fn test_type_exclude() {
    expect_type_ok(r#"
        type T = Exclude<string | number | boolean, boolean>;
        const x: T = 42;
        const y: T = "hello";
    "#);
}

#[test]
fn test_type_extract() {
    expect_type_ok(r#"
        type T = Extract<string | number | boolean, string | boolean>;
        const x: T = "hello";
        const y: T = true;
    "#);
}

#[test]
fn test_type_nonnullable() {
    expect_type_error(r#"
        type T = NonNullable<string | null | undefined>;
        const x: T = null;
    "#, TS2322);
}

#[test]
fn test_type_return_type() {
    expect_type_ok(r#"
        function getUser(): { name: string; age: number } { return { name: 'Alice', age: 30 }; }
        type UserType = ReturnType<typeof getUser>;
        const u: UserType = { name: 'Bob', age: 25 };
    "#);
}

#[test]
fn test_type_parameters() {
    expect_type_ok(r#"
        function f(x: number, y: string): void {}
        type P = Parameters<typeof f>;
        const args: P = [42, 'hello'];
    "#);
}

#[test]
fn test_type_awaited() {
    expect_type_ok(r#"
        type T = Awaited<Promise<string>>;
        const x: T = 'hello';
    "#);
}
