/// Parser tests — statement parsing.
/// ALGO: See SPECS.md §4 FR-PAR-001, FR-PAR-004

#[path = "../../common/mod.rs"]
mod common;
use common::*;

// ── Variable declarations ─────────────────────────────────────────────────────

#[test]
fn test_parse_const() {
    expect_parse_ok("const x = 1;");
}

#[test]
fn test_parse_let() {
    expect_parse_ok("let x = 1;");
}

#[test]
fn test_parse_var() {
    expect_parse_ok("var x = 1;");
}

#[test]
fn test_parse_const_typed() {
    expect_parse_ok("const x: number = 1;");
}

#[test]
fn test_parse_let_no_init() {
    expect_parse_ok("let x: string;");
}

#[test]
fn test_parse_multiple_declarators() {
    expect_parse_ok("let x = 1, y = 2, z = 3;");
}

#[test]
fn test_parse_destructure_const() {
    expect_parse_ok("const [a, b] = [1, 2];");
}

#[test]
fn test_parse_using_declaration() {
    expect_parse_ok("using resource = acquireResource();");
}

// ── Control flow ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_if() {
    expect_parse_ok("if (x > 0) { console.log(x); }");
}

#[test]
fn test_parse_if_else() {
    expect_parse_ok("if (x > 0) { return x; } else { return -x; }");
}

#[test]
fn test_parse_if_else_if() {
    expect_parse_ok("if (x > 0) { return 1; } else if (x < 0) { return -1; } else { return 0; }");
}

#[test]
fn test_parse_if_without_braces() {
    expect_parse_ok("if (x) return x;");
}

#[test]
fn test_parse_switch() {
    expect_parse_ok(r#"
        switch (x) {
            case 1: break;
            case 2: return 2;
            default: break;
        }
    "#);
}

#[test]
fn test_parse_switch_fallthrough() {
    expect_parse_ok(r#"
        switch (x) {
            case 1:
            case 2:
                return 'one or two';
        }
    "#);
}

// ── Loops ─────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_while() {
    expect_parse_ok("while (x > 0) { x--; }");
}

#[test]
fn test_parse_do_while() {
    expect_parse_ok("do { x++; } while (x < 10);");
}

#[test]
fn test_parse_for_classic() {
    expect_parse_ok("for (let i = 0; i < 10; i++) { }");
}

#[test]
fn test_parse_for_no_init() {
    expect_parse_ok("for (; i < 10; i++) { }");
}

#[test]
fn test_parse_for_no_update() {
    expect_parse_ok("for (let i = 0; i < 10;) { i++; }");
}

#[test]
fn test_parse_for_infinite() {
    expect_parse_ok("for (;;) { break; }");
}

#[test]
fn test_parse_for_in() {
    expect_parse_ok("for (const key in obj) { console.log(key); }");
}

#[test]
fn test_parse_for_of() {
    expect_parse_ok("for (const item of arr) { console.log(item); }");
}

#[test]
fn test_parse_for_of_destructure() {
    expect_parse_ok("for (const [k, v] of map) { }");
}

#[test]
fn test_parse_for_await_of() {
    expect_parse_ok("async function f() { for await (const x of stream) { } }");
}

// ── Jump statements ───────────────────────────────────────────────────────────

#[test]
fn test_parse_return() {
    expect_parse_ok("function f() { return 42; }");
}

#[test]
fn test_parse_return_void() {
    expect_parse_ok("function f() { return; }");
}

#[test]
fn test_parse_throw() {
    expect_parse_ok("throw new Error('oops');");
}

#[test]
fn test_parse_break() {
    expect_parse_ok("for (;;) { break; }");
}

#[test]
fn test_parse_break_labeled() {
    expect_parse_ok("outer: for (;;) { for (;;) { break outer; } }");
}

#[test]
fn test_parse_continue() {
    expect_parse_ok("for (;;) { continue; }");
}

#[test]
fn test_parse_continue_labeled() {
    expect_parse_ok("outer: for (;;) { for (;;) { continue outer; } }");
}

// ── Try / catch / finally ──────────────────────────────────────────────────────

#[test]
fn test_parse_try_catch() {
    expect_parse_ok("try { f(); } catch (e) { console.error(e); }");
}

#[test]
fn test_parse_try_catch_typed() {
    expect_parse_ok("try { f(); } catch (e: unknown) { }");
}

#[test]
fn test_parse_try_finally() {
    expect_parse_ok("try { f(); } finally { cleanup(); }");
}

#[test]
fn test_parse_try_catch_finally() {
    expect_parse_ok("try { f(); } catch (e) { } finally { cleanup(); }");
}

#[test]
fn test_parse_try_catch_no_binding() {
    // Optional catch binding
    expect_parse_ok("try { f(); } catch { }");
}

// ── Block ─────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_empty_block() {
    expect_parse_ok("{}");
}

#[test]
fn test_parse_nested_blocks() {
    expect_parse_ok("{ { { } } }");
}

// ── Labeled statement ─────────────────────────────────────────────────────────

#[test]
fn test_parse_labeled_statement() {
    expect_parse_ok("outer: for (;;) { }");
}

// ── Debugger ─────────────────────────────────────────────────────────────────

#[test]
fn test_parse_debugger() {
    expect_parse_ok("debugger;");
}
