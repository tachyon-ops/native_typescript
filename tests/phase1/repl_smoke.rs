/// Phase 1 gate test.
///
/// This test must pass before any Phase 2 work begins.
/// It proves the full pipeline: source → lex → parse → eval → value.
///
/// ALGO: See SPECS.md §7 (Interpreter), §3 (Lexer), §4 (Parser)

#[path = "../common/mod.rs"]
mod common;
use common::*;

#[test]
fn test_gate_const_declaration() {
    expect_number("const x: number = 42; x", 42.0);
}

#[test]
fn test_gate_string_concatenation() {
    expect_string(r#"const s = "hello" + " " + "world"; s"#, "hello world");
}

#[test]
fn test_gate_arrow_function() {
    expect_string(
        r#"
        const greet = (name: string): string => `Hello, ${name}!`;
        greet("world")
        "#,
        "Hello, world!",
    );
}

#[test]
fn test_gate_generics_identity() {
    expect_number(
        r#"
        function identity<T>(x: T): T { return x; }
        identity<number>(99)
        "#,
        99.0,
    );
}

#[test]
fn test_gate_class_basic() {
    expect_string(
        r#"
        class Greeter {
            constructor(private name: string) {}
            greet(): string { return `Hi, ${this.name}`; }
        }
        new Greeter("Alice").greet()
        "#,
        "Hi, Alice",
    );
}

#[test]
fn test_gate_async_await() {
    expect_string(
        r#"
        async function run(): Promise<string> {
            const result = await Promise.resolve("async works");
            return result;
        }
        await run()
        "#,
        "async works",
    );
}

#[test]
fn test_gate_map() {
    expect_number(
        r#"
        const m = new Map<string, number>();
        m.set("a", 1);
        m.set("b", 2);
        m.get("b")
        "#,
        2.0,
    );
}

#[test]
fn test_gate_set() {
    expect_number(
        r#"
        const s = new Set<number>([1, 2, 2, 3, 3, 3]);
        s.size
        "#,
        3.0,
    );
}
