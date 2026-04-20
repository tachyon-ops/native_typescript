/// Evaluator tests — advanced TypeScript patterns: decorators, namespaces,
/// computed properties, Proxy/Reflect, tagged templates, WeakRef.
/// ALGO: See SPECS.md §7 FR-EVAL-001, FR-EVAL-003

mod common;
use common::*;

// ── Computed properties ───────────────────────────────────────────────────────

#[test]
fn test_eval_computed_property_string() {
    expect_string(
        r#"const key = 'greeting'; const obj = { [key]: 'hello' }; obj.greeting"#,
        "hello",
    );
}

#[test]
fn test_eval_computed_property_number() {
    expect_number(
        r#"const i = 2; const obj: any = { [i]: 42 }; obj[2]"#,
        42.0,
    );
}

#[test]
fn test_eval_computed_property_symbol() {
    expect_number(
        r#"
        const sym = Symbol('key');
        const obj: any = { [sym]: 99 };
        obj[sym]
        "#,
        99.0,
    );
}

// ── Property access edge cases ────────────────────────────────────────────────

#[test]
fn test_eval_property_access_inherited() {
    expect_string(
        "const arr = [1, 2, 3]; typeof arr.push",
        "function",
    );
}

#[test]
fn test_eval_property_access_numeric_string() {
    expect_number(r#"const obj: any = { '0': 42 }; obj[0]"#, 42.0);
}

// ── Reflect ───────────────────────────────────────────────────────────────────

#[test]
fn test_eval_reflect_get() {
    expect_number(
        "Reflect.get({ x: 42 }, 'x') as number",
        42.0,
    );
}

#[test]
fn test_eval_reflect_set() {
    expect_number(
        r#"
        const obj: any = {};
        Reflect.set(obj, 'x', 42);
        obj.x
        "#,
        42.0,
    );
}

#[test]
fn test_eval_reflect_has() {
    expect_bool("Reflect.has({ x: 1 }, 'x')", true);
}

#[test]
fn test_eval_reflect_delete_property() {
    expect_bool(
        r#"
        const obj: any = { x: 1, y: 2 };
        Reflect.deleteProperty(obj, 'x');
        !Reflect.has(obj, 'x')
        "#,
        true,
    );
}

#[test]
fn test_eval_reflect_own_keys() {
    expect_number(
        "Reflect.ownKeys({ a: 1, b: 2, c: 3 }).length",
        3.0,
    );
}

#[test]
fn test_eval_reflect_apply() {
    expect_number(
        "Reflect.apply(Math.max, null, [1, 5, 3, 2, 4]) as number",
        5.0,
    );
}

#[test]
fn test_eval_reflect_construct() {
    expect_bool(
        r#"
        class Foo { value: number = 42; }
        const instance = Reflect.construct(Foo, []) as Foo;
        instance instanceof Foo && instance.value === 42
        "#,
        true,
    );
}

// ── Tagged template literals ──────────────────────────────────────────────────

#[test]
fn test_eval_tagged_template_strings() {
    expect_string(
        r#"
        function tag(strings: TemplateStringsArray, ...values: any[]): string {
            return strings.raw.join('|');
        }
        tag`hello ${1} world`
        "#,
        "hello | world",
    );
}

#[test]
fn test_eval_tagged_template_values() {
    expect_number(
        r#"
        function sumValues(strings: TemplateStringsArray, ...values: number[]): number {
            return values.reduce((a, b) => a + b, 0);
        }
        sumValues`${1} + ${2} + ${3} = ${6}`
        "#,
        12.0,
    );
}

#[test]
fn test_eval_tagged_template_html_escape() {
    expect_string(
        r#"
        function html(strings: TemplateStringsArray, ...values: any[]): string {
            return strings.reduce((acc, s, i) => {
                const v = values[i - 1] ?? '';
                return acc + String(v).replace(/</g, '&lt;') + s;
            });
        }
        html`<div>${'<script>'}</div>`
        "#,
        "<div>&lt;script></div>",
    );
}

// ── Namespaces ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_namespace_member_access() {
    expect_number(
        r#"
        namespace MathUtils {
            export function square(x: number): number { return x * x; }
            export const PI = 3.14159;
        }
        MathUtils.square(4) + MathUtils.PI
        "#,
        19.14159,
    );
}

#[test]
fn test_eval_namespace_nested() {
    expect_string(
        r#"
        namespace Outer {
            export namespace Inner {
                export const value = 'nested';
            }
        }
        Outer.Inner.value
        "#,
        "nested",
    );
}

#[test]
fn test_eval_namespace_merging() {
    expect_number(
        r#"
        namespace NS {
            export const a = 1;
        }
        namespace NS {
            export const b = 2;
        }
        NS.a + NS.b
        "#,
        3.0,
    );
}

// ── Decorators ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_class_decorator() {
    expect_bool(
        r#"
        function sealed(constructor: new (...args: any[]) => any) {
            Object.seal(constructor);
            Object.seal(constructor.prototype);
        }

        @sealed
        class BugReport {
            type = 'report';
            title: string;
            constructor(t: string) { this.title = t; }
        }

        const report = new BugReport('Crash');
        report.title === 'Crash'
        "#,
        true,
    );
}

#[test]
fn test_eval_method_decorator() {
    expect_bool(
        r#"
        const log: string[] = [];

        function logMethod(
            target: any,
            propertyKey: string,
            descriptor: PropertyDescriptor
        ) {
            const original = descriptor.value;
            descriptor.value = function(...args: any[]) {
                log.push(`call:${propertyKey}`);
                return original.apply(this, args);
            };
            return descriptor;
        }

        class Greeter {
            @logMethod
            greet(name: string): string { return `Hi ${name}`; }
        }

        const g = new Greeter();
        g.greet('Alice');
        log.length === 1 && log[0] === 'call:greet'
        "#,
        true,
    );
}

// ── Iterators and iteration protocol ─────────────────────────────────────────

#[test]
fn test_eval_custom_iterator() {
    expect_string(
        r#"
        function* take<T>(iterable: Iterable<T>, n: number): Generator<T> {
            let count = 0;
            for (const x of iterable) {
                if (count++ >= n) break;
                yield x;
            }
        }

        function* naturals(): Generator<number> {
            let n = 1;
            while (true) yield n++;
        }

        [...take(naturals(), 5)].join(',')
        "#,
        "1,2,3,4,5",
    );
}

#[test]
fn test_eval_iterator_protocol_manual() {
    expect_number(
        r#"
        const range = {
            [Symbol.iterator]() {
                let i = 0;
                return {
                    next() {
                        return i < 5
                            ? { value: i++, done: false }
                            : { value: undefined, done: true };
                    }
                };
            }
        };
        let s = 0;
        for (const n of range) s += n;
        s
        "#,
        10.0,
    );
}

// ── Getters and setters on object literals ────────────────────────────────────

#[test]
fn test_eval_object_getter() {
    expect_number(
        r#"
        const obj = {
            _x: 0,
            get x() { return this._x; },
            set x(v: number) { this._x = v * 2; }
        };
        obj.x = 5;
        obj.x
        "#,
        10.0,
    );
}

// ── WeakRef and FinalizationRegistry ─────────────────────────────────────────

#[test]
fn test_eval_weakref_deref() {
    expect_bool(
        r#"
        let obj: any = { value: 42 };
        const ref = new WeakRef(obj);
        ref.deref()?.value === 42
        "#,
        true,
    );
}

// ── String.raw ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_string_raw() {
    // String.raw does not process escape sequences
    expect_string(
        r#"String.raw`Hello\nWorld`"#,
        r"Hello\nWorld",
    );
}

// ── Object.getOwnPropertyDescriptor ──────────────────────────────────────────

#[test]
fn test_eval_get_own_property_descriptor() {
    expect_bool(
        r#"
        const obj = { x: 42 };
        const desc = Object.getOwnPropertyDescriptor(obj, 'x')!;
        desc.value === 42 && desc.writable === true && desc.enumerable === true
        "#,
        true,
    );
}

// ── Array.from with length ────────────────────────────────────────────────────

#[test]
fn test_eval_array_from_length_map() {
    expect_string(
        "Array.from({ length: 5 }, (_, i) => i * 2).join(',')",
        "0,2,4,6,8",
    );
}

// ── Nullish assignment ────────────────────────────────────────────────────────

#[test]
fn test_eval_nullish_assign() {
    expect_number(
        "let x: number | null = null; x ??= 42; x",
        42.0,
    );
}

#[test]
fn test_eval_nullish_assign_no_op() {
    expect_number(
        "let x: number | null = 10; x ??= 42; x",
        10.0,
    );
}

#[test]
fn test_eval_logical_or_assign() {
    expect_number("let x = 0; x ||= 5; x", 5.0);
}

#[test]
fn test_eval_logical_and_assign() {
    expect_number("let x = 1; x &&= 99; x", 99.0);
}

// ── Numeric BigInt ────────────────────────────────────────────────────────────

#[test]
fn test_eval_bigint_arithmetic() {
    expect_bool(
        "9007199254740993n === 9007199254740992n + 1n",
        true,
    );
}

#[test]
fn test_eval_bigint_comparison() {
    expect_bool("10n > 5n && 3n < 4n", true);
}

#[test]
fn test_eval_bigint_typeof() {
    expect_string("typeof 42n", "bigint");
}

#[test]
fn test_eval_bigint_mixed_throws() {
    let err = expect_runtime_error("1n + 1;");
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("bigint") || msg.contains("type"), "unexpected: {msg}");
}
