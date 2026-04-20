/// Parser tests — expression parsing and operator precedence.
/// ALGO: See SPECS.md §4 FR-PAR-001, FR-PAR-005, FR-PAR-008

mod common;
use common::*;

// ── Literals ──────────────────────────────────────────────────────────────────

#[test]
fn test_parse_number_literal() {
    expect_parse_ok("42;");
}

#[test]
fn test_parse_string_literal_double() {
    expect_parse_ok(r#""hello";"#);
}

#[test]
fn test_parse_string_literal_single() {
    expect_parse_ok("'world';");
}

#[test]
fn test_parse_bool_true() {
    expect_parse_ok("true;");
}

#[test]
fn test_parse_bool_false() {
    expect_parse_ok("false;");
}

#[test]
fn test_parse_null() {
    expect_parse_ok("null;");
}

#[test]
fn test_parse_undefined() {
    expect_parse_ok("undefined;");
}

#[test]
fn test_parse_this() {
    expect_parse_ok("this;");
}

#[test]
fn test_parse_bigint() {
    expect_parse_ok("42n;");
}

#[test]
fn test_parse_regex() {
    expect_parse_ok("const r = /pattern/gi;");
}

// ── Template literals ─────────────────────────────────────────────────────────

#[test]
fn test_parse_no_subst_template() {
    expect_parse_ok("`hello`;");
}

#[test]
fn test_parse_template_with_expr() {
    expect_parse_ok("`hello ${name}!`;");
}

#[test]
fn test_parse_tagged_template() {
    expect_parse_ok("html`<div>${content}</div>`;");
}

#[test]
fn test_parse_nested_template() {
    expect_parse_ok("`outer ${ `inner ${x}` } end`;");
}

// ── Operator precedence ───────────────────────────────────────────────────────

#[test]
fn test_parse_precedence_mul_over_add() {
    // a + b * c should parse as a + (b * c)
    expect_parse_ok("a + b * c;");
}

#[test]
fn test_parse_precedence_pow_right_assoc() {
    // 2 ** 3 ** 2 should parse as 2 ** (3 ** 2)
    expect_parse_ok("2 ** 3 ** 2;");
}

#[test]
fn test_parse_precedence_comparison() {
    expect_parse_ok("a < b === c > d;");
}

#[test]
fn test_parse_precedence_logical_and_or() {
    // a || b && c = a || (b && c)
    expect_parse_ok("a || b && c;");
}

#[test]
fn test_parse_precedence_nullish() {
    expect_parse_ok("a ?? b ?? c;");
}

#[test]
fn test_parse_precedence_conditional() {
    expect_parse_ok("a ? b : c ? d : e;");
}

#[test]
fn test_parse_precedence_assignment_right_assoc() {
    expect_parse_ok("a = b = c;");
}

#[test]
fn test_parse_compound_assignment() {
    expect_parse_ok("x += 1; x -= 2; x *= 3; x /= 4; x **= 2; x ??= 0; x &&= true; x ||= false;");
}

// ── Member access ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_member_dot() {
    expect_parse_ok("a.b.c;");
}

#[test]
fn test_parse_member_index() {
    expect_parse_ok("a[b];");
}

#[test]
fn test_parse_optional_chain_member() {
    expect_parse_ok("a?.b;");
}

#[test]
fn test_parse_optional_chain_index() {
    expect_parse_ok("a?.[b];");
}

#[test]
fn test_parse_optional_chain_call() {
    expect_parse_ok("a?.();");
}

#[test]
fn test_parse_optional_chain_deep() {
    expect_parse_ok("a?.b?.c?.d;");
}

// ── Call expressions ──────────────────────────────────────────────────────────

#[test]
fn test_parse_call_no_args() {
    expect_parse_ok("f();");
}

#[test]
fn test_parse_call_with_args() {
    expect_parse_ok("f(1, 2, 3);");
}

#[test]
fn test_parse_call_spread_arg() {
    expect_parse_ok("f(...args);");
}

#[test]
fn test_parse_call_type_args() {
    expect_parse_ok("f<string>(x);");
}

#[test]
fn test_parse_new_expression() {
    expect_parse_ok("new Foo(1, 2);");
}

#[test]
fn test_parse_new_no_args() {
    expect_parse_ok("new Foo;");
}

#[test]
fn test_parse_new_type_args() {
    expect_parse_ok("new Map<string, number>();");
}

// ── Unary operators ───────────────────────────────────────────────────────────

#[test]
fn test_parse_unary_bang() {
    expect_parse_ok("!x;");
}

#[test]
fn test_parse_unary_minus() {
    expect_parse_ok("-x;");
}

#[test]
fn test_parse_unary_plus() {
    expect_parse_ok("+x;");
}

#[test]
fn test_parse_unary_tilde() {
    expect_parse_ok("~x;");
}

#[test]
fn test_parse_unary_typeof() {
    expect_parse_ok("typeof x;");
}

#[test]
fn test_parse_unary_void() {
    expect_parse_ok("void x;");
}

#[test]
fn test_parse_unary_delete() {
    expect_parse_ok("delete obj.key;");
}

#[test]
fn test_parse_prefix_increment() {
    expect_parse_ok("++i;");
}

#[test]
fn test_parse_postfix_increment() {
    expect_parse_ok("i++;");
}

#[test]
fn test_parse_prefix_decrement() {
    expect_parse_ok("--i;");
}

#[test]
fn test_parse_postfix_decrement() {
    expect_parse_ok("i--;");
}

// ── Functions ──────────────────────────────────────────────────────────────────

#[test]
fn test_parse_arrow_concise() {
    expect_parse_ok("const f = x => x * 2;");
}

#[test]
fn test_parse_arrow_with_types() {
    expect_parse_ok("const f = (x: number): number => x * 2;");
}

#[test]
fn test_parse_arrow_block_body() {
    expect_parse_ok("const f = (x: number): number => { return x * 2; };");
}

#[test]
fn test_parse_arrow_async() {
    expect_parse_ok("const f = async (x: string) => await fetch(x);");
}

#[test]
fn test_parse_function_expression() {
    expect_parse_ok("const f = function(x: number) { return x; };");
}

#[test]
fn test_parse_function_named_expression() {
    expect_parse_ok("const f = function named(x: number) { return x; };");
}

// ── Type assertions ───────────────────────────────────────────────────────────

#[test]
fn test_parse_as_expression() {
    expect_parse_ok("const x = value as string;");
}

#[test]
fn test_parse_as_const() {
    expect_parse_ok("const x = [1, 2, 3] as const;");
}

#[test]
fn test_parse_satisfies() {
    expect_parse_ok("const x = { a: 1 } satisfies Record<string, number>;");
}

#[test]
fn test_parse_non_null_assertion() {
    expect_parse_ok("const x = maybeNull!;");
}

// ── Destructuring ─────────────────────────────────────────────────────────────

#[test]
fn test_parse_array_destructure() {
    expect_parse_ok("const [a, b, c] = arr;");
}

#[test]
fn test_parse_array_destructure_with_default() {
    expect_parse_ok("const [a = 1, b = 2] = arr;");
}

#[test]
fn test_parse_array_destructure_rest() {
    expect_parse_ok("const [first, ...rest] = arr;");
}

#[test]
fn test_parse_object_destructure() {
    expect_parse_ok("const { x, y } = obj;");
}

#[test]
fn test_parse_object_destructure_rename() {
    expect_parse_ok("const { x: a, y: b } = obj;");
}

#[test]
fn test_parse_object_destructure_default() {
    expect_parse_ok("const { x = 0, y = 0 } = obj;");
}

#[test]
fn test_parse_object_destructure_rest() {
    expect_parse_ok("const { a, b, ...rest } = obj;");
}

#[test]
fn test_parse_nested_destructure() {
    expect_parse_ok("const { a: { b: { c } } } = obj;");
}

// ── Spread ────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_array_spread() {
    expect_parse_ok("const arr = [...a, ...b];");
}

#[test]
fn test_parse_object_spread() {
    expect_parse_ok("const obj = { ...a, ...b, x: 1 };");
}

// ── Async / await ─────────────────────────────────────────────────────────────

#[test]
fn test_parse_await_expression() {
    expect_parse_ok("async function f() { return await g(); }");
}

#[test]
fn test_parse_await_in_for_of() {
    expect_parse_ok("async function f() { for await (const x of gen()) {} }");
}

// ── Yield ────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_yield_expression() {
    expect_parse_ok("function* f() { yield 1; }");
}

#[test]
fn test_parse_yield_star() {
    expect_parse_ok("function* f() { yield* other(); }");
}

// ── Comma / sequence ──────────────────────────────────────────────────────────

#[test]
fn test_parse_sequence_in_for() {
    expect_parse_ok("for (let i = 0, j = 10; i < j; i++, j--) {}");
}

// ── Error recovery ────────────────────────────────────────────────────────────

#[test]
fn test_parse_recovery_missing_semicolon() {
    // Should parse with ASI, not error
    expect_parse_ok("const x = 1\nconst y = 2");
}

#[test]
fn test_parse_error_unclosed_paren() {
    expect_parse_error("const x = (1 + 2;");
}
