/// Evaluator tests — control flow: if, loops, switch, try/catch, break/continue.
/// ALGO: See SPECS.md §7 FR-EVAL-001

mod common;
use common::*;

// ── if / else ─────────────────────────────────────────────────────────────────

#[test]
fn test_eval_if_true_branch() {
    expect_number("let x = 0; if (true) { x = 1; } x", 1.0);
}

#[test]
fn test_eval_if_false_branch() {
    expect_number("let x = 0; if (false) { x = 1; } x", 0.0);
}

#[test]
fn test_eval_if_else() {
    expect_number("let x = 0; if (false) { x = 1; } else { x = 2; } x", 2.0);
}

#[test]
fn test_eval_if_else_if() {
    expect_string(
        r#"
        const n = 0;
        let s: string;
        if (n > 0) { s = 'positive'; }
        else if (n < 0) { s = 'negative'; }
        else { s = 'zero'; }
        s
        "#,
        "zero",
    );
}

#[test]
fn test_eval_if_truthy() {
    expect_number("let x = 0; if (1) { x = 1; } x", 1.0);
}

#[test]
fn test_eval_if_falsy_zero() {
    expect_number("let x = 1; if (0) { x = 99; } x", 1.0);
}

#[test]
fn test_eval_if_falsy_empty_string() {
    expect_number(r#"let x = 1; if ("") { x = 99; } x"#, 1.0);
}

#[test]
fn test_eval_if_falsy_null() {
    expect_number("let x = 1; if (null) { x = 99; } x", 1.0);
}

// ── while ─────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_while_basic() {
    expect_number(
        "let i = 0; let s = 0; while (i < 5) { s += i; i++; } s",
        10.0,
    );
}

#[test]
fn test_eval_while_never_runs() {
    expect_number("let x = 42; while (false) { x = 0; } x", 42.0);
}

// ── do...while ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_do_while_runs_once() {
    expect_number("let x = 0; do { x++; } while (false); x", 1.0);
}

#[test]
fn test_eval_do_while_multiple() {
    expect_number("let i = 0; do { i++; } while (i < 5); i", 5.0);
}

// ── for ───────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_for_basic() {
    expect_number("let s = 0; for (let i = 0; i < 10; i++) { s += i; } s", 45.0);
}

#[test]
fn test_eval_for_never_runs() {
    expect_number("let x = 0; for (let i = 0; i < 0; i++) { x++; } x", 0.0);
}

#[test]
fn test_eval_for_multiple_inits() {
    expect_number(
        "let s = 0; for (let i = 0, j = 10; i < j; i++, j--) { s++; } s",
        5.0,
    );
}

// ── for...of ─────────────────────────────────────────────────────────────────

#[test]
fn test_eval_for_of_array() {
    expect_number(
        "let s = 0; for (const x of [1, 2, 3, 4, 5]) { s += x; } s",
        15.0,
    );
}

#[test]
fn test_eval_for_of_string() {
    expect_string(
        r#"let s = ''; for (const c of 'hello') { s += c.toUpperCase(); } s"#,
        "HELLO",
    );
}

#[test]
fn test_eval_for_of_map() {
    expect_number(
        r#"
        const m = new Map([['a', 1], ['b', 2], ['c', 3]]);
        let s = 0;
        for (const [, v] of m) { s += v; }
        s
        "#,
        6.0,
    );
}

#[test]
fn test_eval_for_of_set() {
    expect_number(
        "let s = 0; for (const x of new Set([1, 2, 2, 3, 3])) { s += x; } s",
        6.0,
    );
}

#[test]
fn test_eval_for_of_destructure() {
    expect_number(
        r#"
        const pairs = [[1, 2], [3, 4], [5, 6]];
        let s = 0;
        for (const [a, b] of pairs) { s += a + b; }
        s
        "#,
        21.0,
    );
}

// ── for...in ──────────────────────────────────────────────────────────────────

#[test]
fn test_eval_for_in_keys() {
    expect_number(
        "const obj = { a: 1, b: 2, c: 3 }; let count = 0; for (const key in obj) { count++; } count",
        3.0,
    );
}

// ── break ────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_break_while() {
    expect_number(
        "let i = 0; while (true) { if (i >= 5) break; i++; } i",
        5.0,
    );
}

#[test]
fn test_eval_break_for() {
    expect_number(
        "let s = 0; for (let i = 0; i < 100; i++) { if (i === 5) break; s += i; } s",
        10.0,
    );
}

#[test]
fn test_eval_break_labeled() {
    expect_number(
        r#"
        let found = -1;
        outer: for (let i = 0; i < 5; i++) {
            for (let j = 0; j < 5; j++) {
                if (i + j === 7) { found = i * 10 + j; break outer; }
            }
        }
        found
        "#,
        34.0, // i=3, j=4 → 3*10+4=34
    );
}

// ── continue ─────────────────────────────────────────────────────────────────

#[test]
fn test_eval_continue_for() {
    // Sum only even numbers 0..9
    expect_number(
        "let s = 0; for (let i = 0; i < 10; i++) { if (i % 2 !== 0) continue; s += i; } s",
        20.0,
    );
}

#[test]
fn test_eval_continue_labeled() {
    expect_number(
        r#"
        let s = 0;
        outer: for (let i = 0; i < 3; i++) {
            for (let j = 0; j < 3; j++) {
                if (j === 1) continue outer;
                s += 1;
            }
        }
        s
        "#,
        3.0, // Only j=0 runs each outer iteration
    );
}

// ── switch ────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_switch_matches_case() {
    expect_string(
        r#"
        let result = '';
        switch ('b') {
            case 'a': result = 'A'; break;
            case 'b': result = 'B'; break;
            case 'c': result = 'C'; break;
            default:  result = 'D'; break;
        }
        result
        "#,
        "B",
    );
}

#[test]
fn test_eval_switch_default() {
    expect_string(
        r#"
        let result = '';
        switch ('z') {
            case 'a': result = 'A'; break;
            default:  result = 'DEFAULT'; break;
        }
        result
        "#,
        "DEFAULT",
    );
}

#[test]
fn test_eval_switch_fallthrough() {
    expect_string(
        r#"
        let result = '';
        switch (2) {
            case 1:
            case 2:
            case 3: result = 'one-two-three'; break;
            default: result = 'other';
        }
        result
        "#,
        "one-two-three",
    );
}

#[test]
fn test_eval_switch_no_match_no_default() {
    expect_string(
        r#"
        let result = 'unchanged';
        switch (99) {
            case 1: result = 'one'; break;
        }
        result
        "#,
        "unchanged",
    );
}

// ── try / catch / finally ─────────────────────────────────────────────────────

#[test]
fn test_eval_try_no_throw() {
    expect_number("let x = 0; try { x = 42; } catch (e) { x = -1; } x", 42.0);
}

#[test]
fn test_eval_try_catch_error() {
    expect_string(
        r#"
        let caught = '';
        try {
            throw new Error("oops");
        } catch (e) {
            caught = (e as Error).message;
        }
        caught
        "#,
        "oops",
    );
}

#[test]
fn test_eval_try_catch_type_error() {
    expect_string(
        r#"
        let kind = '';
        try {
            const x: any = null;
            x.property;
        } catch (e) {
            kind = (e as Error).constructor.name;
        }
        kind
        "#,
        "TypeError",
    );
}

#[test]
fn test_eval_try_finally_runs() {
    expect_number(
        r#"
        let x = 0;
        try { x = 1; } finally { x += 10; }
        x
        "#,
        11.0,
    );
}

#[test]
fn test_eval_try_finally_runs_even_after_throw() {
    expect_number(
        r#"
        let cleaned = 0;
        try {
            try { throw new Error("test"); }
            finally { cleaned = 1; }
        } catch (_) {}
        cleaned
        "#,
        1.0,
    );
}

#[test]
fn test_eval_try_catch_optional_binding() {
    expect_bool(
        "let ok = false; try { throw new Error(); } catch { ok = true; } ok",
        true,
    );
}

#[test]
fn test_eval_throw_non_error() {
    expect_number(
        r#"
        let caught = 0;
        try { throw 42; } catch (e) { caught = e as number; }
        caught
        "#,
        42.0,
    );
}

// ── Scope ─────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_block_scope_let() {
    // let in a block should not be visible outside
    let err = expect_runtime_error("{ let x = 1; } x;");
    let msg = format!("{err}");
    assert!(
        msg.contains("x") && (msg.contains("not defined") || msg.contains("undefined")),
        "unexpected error: {msg}"
    );
}

#[test]
fn test_eval_block_scope_const() {
    let err = expect_runtime_error("{ const x = 1; } x;");
    let msg = format!("{err}");
    assert!(msg.contains("x"));
}

#[test]
fn test_eval_var_function_scope() {
    // var declared inside an if block is visible outside
    expect_number("if (true) { var x = 42; } x", 42.0);
}
