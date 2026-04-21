/// Evaluator tests — destructuring, spread, objects, symbols.
/// ALGO: See SPECS.md §7 FR-EVAL-001

#[path = "../../common/mod.rs"]
mod common;
use common::*;

// ── Object literals ───────────────────────────────────────────────────────────

#[test]
fn test_eval_object_literal() {
    expect_number("const obj = { x: 1, y: 2 }; obj.x + obj.y", 3.0);
}

#[test]
fn test_eval_object_shorthand() {
    expect_number("const x = 3, y = 4; const obj = { x, y }; obj.x + obj.y", 7.0);
}

#[test]
fn test_eval_object_computed_key() {
    expect_number(
        r#"const key = 'name'; const obj = { [key]: 'Alice' }; obj.name.length"#,
        5.0,
    );
}

#[test]
fn test_eval_object_method_shorthand() {
    expect_number("const obj = { add(a: number, b: number) { return a + b; } }; obj.add(3, 4)", 7.0);
}

#[test]
fn test_eval_object_nested() {
    expect_string(
        "const obj = { a: { b: { c: 'deep' } } }; obj.a.b.c",
        "deep",
    );
}

#[test]
fn test_eval_object_dynamic_property() {
    expect_number("const obj: any = {}; const key = 'x'; obj[key] = 42; obj.x", 42.0);
}

// ── Object destructuring ──────────────────────────────────────────────────────

#[test]
fn test_eval_destructure_object_basic() {
    expect_number("const { x, y } = { x: 3, y: 4 }; x + y", 7.0);
}

#[test]
fn test_eval_destructure_object_rename() {
    expect_number("const { x: a, y: b } = { x: 10, y: 20 }; a + b", 30.0);
}

#[test]
fn test_eval_destructure_object_default() {
    expect_number("const { x = 5, y = 10 } = { x: 3 }; x + y", 13.0);
}

#[test]
fn test_eval_destructure_object_rest() {
    expect_number(
        "const { a, ...rest } = { a: 1, b: 2, c: 3 }; rest.b + rest.c",
        5.0,
    );
}

#[test]
fn test_eval_destructure_object_nested() {
    expect_number(
        "const { a: { b: { c } } } = { a: { b: { c: 42 } } }; c",
        42.0,
    );
}

// ── Array destructuring ───────────────────────────────────────────────────────

#[test]
fn test_eval_destructure_array_basic() {
    expect_number("const [a, b] = [1, 2]; a + b", 3.0);
}

#[test]
fn test_eval_destructure_array_skip() {
    expect_number("const [, second, , fourth] = [1, 2, 3, 4]; second + fourth", 6.0);
}

#[test]
fn test_eval_destructure_array_default() {
    expect_number("const [a = 10, b = 20] = [5]; a + b", 25.0);
}

#[test]
fn test_eval_destructure_array_rest() {
    expect_number("const [first, ...rest] = [1, 2, 3, 4]; first + rest.length", 4.0);
}

#[test]
fn test_eval_destructure_array_nested() {
    expect_number("const [[a, b], [c, d]] = [[1, 2], [3, 4]]; a + b + c + d", 10.0);
}

#[test]
fn test_eval_destructure_swap() {
    expect_number("let a = 1, b = 2; [a, b] = [b, a]; a + b * 10", 21.0);
}

// ── Parameter destructuring ───────────────────────────────────────────────────

#[test]
fn test_eval_param_object_destructure() {
    expect_string(
        r#"
        function greet({ name, greeting = 'Hello' }: { name: string; greeting?: string }) {
            return `${greeting}, ${name}!`;
        }
        greet({ name: 'Alice' })
        "#,
        "Hello, Alice!",
    );
}

#[test]
fn test_eval_param_array_destructure() {
    expect_number(
        "function first([a]: number[]): number { return a; } first([42, 1, 2])",
        42.0,
    );
}

// ── Spread ────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_array_spread() {
    expect_string(
        "[...[1, 2], ...[3, 4], 5].join(',')",
        "1,2,3,4,5",
    );
}

#[test]
fn test_eval_object_spread() {
    expect_number(
        "const a = { x: 1, y: 2 }; const b = { ...a, z: 3 }; b.x + b.y + b.z",
        6.0,
    );
}

#[test]
fn test_eval_object_spread_override() {
    expect_number(
        "const a = { x: 1, y: 2 }; const b = { ...a, x: 99 }; b.x",
        99.0,
    );
}

#[test]
fn test_eval_object_spread_shallow_clone() {
    expect_bool(
        r#"
        const original = { a: 1, b: 2 };
        const clone = { ...original };
        clone.a === 1 && clone !== original
        "#,
        true,
    );
}

// ── Symbols ───────────────────────────────────────────────────────────────────

#[test]
fn test_eval_symbol_typeof() {
    expect_string("typeof Symbol()", "symbol");
}

#[test]
fn test_eval_symbol_unique() {
    expect_bool("Symbol() !== Symbol()", true);
}

#[test]
fn test_eval_symbol_for_shared() {
    expect_bool("Symbol.for('key') === Symbol.for('key')", true);
}

#[test]
fn test_eval_symbol_description() {
    expect_string("Symbol('my symbol').description", "my symbol");
}

#[test]
fn test_eval_symbol_as_key() {
    expect_number(
        r#"
        const sym = Symbol('key');
        const obj: any = {};
        obj[sym] = 42;
        obj[sym]
        "#,
        42.0,
    );
}

#[test]
fn test_eval_symbol_not_enumerable_in_for_in() {
    expect_number(
        r#"
        const sym = Symbol('key');
        const obj: any = { a: 1, [sym]: 2 };
        let count = 0;
        for (const k in obj) count++;
        count
        "#,
        1.0, // Symbol key not included in for...in
    );
}

#[test]
fn test_eval_symbol_iterator() {
    expect_string(
        r#"
        const arr = [1, 2, 3];
        const iter = arr[Symbol.iterator]();
        iter.next().value + ',' + iter.next().value
        "#,
        "1,2",
    );
}

// ── Proxy ─────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_proxy_get_trap() {
    expect_number(
        r#"
        const handler = {
            get(target: any, key: string) {
                return key in target ? target[key] : 37;
            }
        };
        const p = new Proxy({} as any, handler);
        p.a
        "#,
        37.0,
    );
}

#[test]
fn test_eval_proxy_set_trap() {
    expect_number(
        r#"
        const log: string[] = [];
        const handler = {
            set(target: any, key: string, value: any) {
                log.push(key);
                target[key] = value;
                return true;
            }
        };
        const p = new Proxy({} as any, handler);
        p.x = 42;
        p.x
        "#,
        42.0,
    );
}

// ── Object.* methods ─────────────────────────────────────────────────────────

#[test]
fn test_eval_object_keys() {
    expect_string(
        "Object.keys({ a: 1, b: 2, c: 3 }).sort().join(',')",
        "a,b,c",
    );
}

#[test]
fn test_eval_object_values() {
    expect_number(
        "Object.values({ a: 1, b: 2, c: 3 }).reduce((a: number, b: number) => a + b, 0)",
        6.0,
    );
}

#[test]
fn test_eval_object_entries() {
    expect_number(
        "Object.entries({ a: 1, b: 2 }).length",
        2.0,
    );
}

#[test]
fn test_eval_object_assign() {
    expect_number(
        "const target = { a: 1 }; Object.assign(target, { b: 2, c: 3 }); target.b + target.c",
        5.0,
    );
}

#[test]
fn test_eval_object_freeze() {
    expect_bool(
        "const obj = Object.freeze({ x: 1 }); Object.isFrozen(obj)",
        true,
    );
}

#[test]
fn test_eval_object_create() {
    expect_bool(
        r#"
        const proto = { greet() { return 'hi'; } };
        const obj = Object.create(proto);
        obj.greet() === 'hi'
        "#,
        true,
    );
}

#[test]
fn test_eval_object_has_own() {
    expect_bool(
        "const obj = { a: 1 }; Object.hasOwn(obj, 'a') && !Object.hasOwn(obj, 'toString')",
        true,
    );
}

// ── Enums ─────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_numeric_enum() {
    expect_number(
        "enum Direction { Up = 0, Down = 1, Left = 2, Right = 3 } Direction.Up + Direction.Right",
        3.0,
    );
}

#[test]
fn test_eval_string_enum() {
    expect_string(
        r#"enum Color { Red = 'RED', Green = 'GREEN' } Color.Red"#,
        "RED",
    );
}

#[test]
fn test_eval_enum_reverse_mapping() {
    // Numeric enums have reverse mapping: Direction[0] === 'Up'
    expect_string(
        "enum Direction { Up = 0, Down = 1 } Direction[0]",
        "Up",
    );
}
