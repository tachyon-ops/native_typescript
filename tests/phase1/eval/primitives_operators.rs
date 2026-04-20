/// Evaluator tests — primitives, operators, variables.
/// ALGO: See SPECS.md §7 FR-EVAL-001 through FR-EVAL-003

mod common;
use common::*;

// ── Number operations ─────────────────────────────────────────────────────────

#[test]
fn test_eval_number_add() {
    expect_number("1 + 2", 3.0);
}

#[test]
fn test_eval_number_subtract() {
    expect_number("10 - 3", 7.0);
}

#[test]
fn test_eval_number_multiply() {
    expect_number("4 * 5", 20.0);
}

#[test]
fn test_eval_number_divide() {
    expect_number("10 / 4", 2.5);
}

#[test]
fn test_eval_number_modulo() {
    expect_number("10 % 3", 1.0);
}

#[test]
fn test_eval_number_power() {
    expect_number("2 ** 10", 1024.0);
}

#[test]
fn test_eval_number_unary_minus() {
    expect_number("-42", -42.0);
}

#[test]
fn test_eval_number_unary_plus() {
    expect_number("+'3'", 3.0);
}

#[test]
fn test_eval_number_precedence() {
    expect_number("2 + 3 * 4", 14.0);
}

#[test]
fn test_eval_number_parens() {
    expect_number("(2 + 3) * 4", 20.0);
}

#[test]
fn test_eval_number_division_by_zero() {
    expect_number("1 / 0", f64::INFINITY);
}

#[test]
fn test_eval_number_nan() {
    expect_number("0 / 0", f64::NAN);
}

#[test]
fn test_eval_number_infinity() {
    expect_number("Infinity", f64::INFINITY);
}

#[test]
fn test_eval_number_negative_infinity() {
    expect_number("-Infinity", f64::NEG_INFINITY);
}

#[test]
fn test_eval_bitwise_and() {
    expect_number("0b1100 & 0b1010", 0b1000 as f64);
}

#[test]
fn test_eval_bitwise_or() {
    expect_number("0b1100 | 0b1010", 0b1110 as f64);
}

#[test]
fn test_eval_bitwise_xor() {
    expect_number("0b1100 ^ 0b1010", 0b0110 as f64);
}

#[test]
fn test_eval_bitwise_not() {
    expect_number("~0", -1.0);
}

#[test]
fn test_eval_left_shift() {
    expect_number("1 << 3", 8.0);
}

#[test]
fn test_eval_right_shift() {
    expect_number("8 >> 1", 4.0);
}

#[test]
fn test_eval_unsigned_right_shift() {
    expect_number("-1 >>> 0", 4294967295.0);
}

// ── String operations ─────────────────────────────────────────────────────────

#[test]
fn test_eval_string_concat() {
    expect_string(r#""hello" + " " + "world""#, "hello world");
}

#[test]
fn test_eval_string_number_coerce() {
    expect_string(r#""value: " + 42"#, "value: 42");
}

#[test]
fn test_eval_template_literal_basic() {
    expect_string(r#"const name = "Alice"; `Hello, ${name}!`"#, "Hello, Alice!");
}

#[test]
fn test_eval_template_literal_expression() {
    expect_string("`${1 + 2 + 3}`", "6");
}

#[test]
fn test_eval_template_literal_nested() {
    expect_string("`a${`b${1}c`}d`", "ab1cd");
}

// ── Boolean / comparison ──────────────────────────────────────────────────────

#[test]
fn test_eval_strict_eq_number() {
    expect_bool("1 === 1", true);
}

#[test]
fn test_eval_strict_neq_number() {
    expect_bool("1 !== 2", true);
}

#[test]
fn test_eval_strict_eq_string() {
    expect_bool(r#""a" === "a""#, true);
}

#[test]
fn test_eval_loose_eq_coerce() {
    expect_bool(r#"1 == "1""#, true);
}

#[test]
fn test_eval_loose_neq_coerce() {
    expect_bool(r#"1 != "2""#, true);
}

#[test]
fn test_eval_lt() {
    expect_bool("1 < 2", true);
}

#[test]
fn test_eval_gt() {
    expect_bool("2 > 1", true);
}

#[test]
fn test_eval_lte() {
    expect_bool("2 <= 2", true);
}

#[test]
fn test_eval_gte() {
    expect_bool("3 >= 2", true);
}

#[test]
fn test_eval_logical_and_true() {
    expect_bool("true && true", true);
}

#[test]
fn test_eval_logical_and_false() {
    expect_bool("true && false", false);
}

#[test]
fn test_eval_logical_or_true() {
    expect_bool("false || true", true);
}

#[test]
fn test_eval_logical_and_short_circuit() {
    // false && anything should not evaluate right side
    expect_bool("false && (1/0 > 0)", false);
}

#[test]
fn test_eval_logical_or_short_circuit() {
    expect_bool("true || (1/0 > 0)", true);
}

#[test]
fn test_eval_logical_and_returns_value() {
    expect_number("1 && 2", 2.0);
}

#[test]
fn test_eval_logical_or_returns_value() {
    expect_number("0 || 42", 42.0);
}

#[test]
fn test_eval_nullish_coalescing_null() {
    expect_number("null ?? 42", 42.0);
}

#[test]
fn test_eval_nullish_coalescing_undefined() {
    expect_number("undefined ?? 42", 42.0);
}

#[test]
fn test_eval_nullish_coalescing_zero_not_replaced() {
    expect_number("0 ?? 42", 0.0);
}

#[test]
fn test_eval_nullish_coalescing_empty_string_not_replaced() {
    expect_string(r#""" ?? "default""#, "");
}

#[test]
fn test_eval_not_true() {
    expect_bool("!true", false);
}

#[test]
fn test_eval_not_false() {
    expect_bool("!false", true);
}

#[test]
fn test_eval_not_truthy() {
    expect_bool("!1", false);
}

#[test]
fn test_eval_not_falsy() {
    expect_bool("!0", true);
}

// ── typeof ────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_typeof_number() {
    expect_string("typeof 42", "number");
}

#[test]
fn test_eval_typeof_string() {
    expect_string(r#"typeof "hello""#, "string");
}

#[test]
fn test_eval_typeof_boolean() {
    expect_string("typeof true", "boolean");
}

#[test]
fn test_eval_typeof_undefined() {
    expect_string("typeof undefined", "undefined");
}

#[test]
fn test_eval_typeof_null() {
    // This is the famous JS quirk
    expect_string("typeof null", "object");
}

#[test]
fn test_eval_typeof_object() {
    expect_string("typeof {}", "object");
}

#[test]
fn test_eval_typeof_function() {
    expect_string("typeof function(){}", "function");
}

#[test]
fn test_eval_typeof_symbol() {
    expect_string("typeof Symbol()", "symbol");
}

#[test]
fn test_eval_typeof_bigint() {
    expect_string("typeof 42n", "bigint");
}

#[test]
fn test_eval_typeof_undeclared_no_throw() {
    // typeof on undeclared variable should return "undefined", not throw
    expect_string("typeof undeclaredVariable", "undefined");
}

// ── Variable binding ──────────────────────────────────────────────────────────

#[test]
fn test_eval_const() {
    expect_number("const x = 42; x", 42.0);
}

#[test]
fn test_eval_let() {
    expect_number("let x = 10; x += 5; x", 15.0);
}

#[test]
fn test_eval_var_hoist() {
    // var declarations are hoisted; x is undefined before assignment
    expect_bool("(function() { return x === undefined; var x = 1; })()", true);
}

#[test]
fn test_eval_const_reassign_throws() {
    let err = expect_runtime_error("const x = 1; x = 2;");
    assert!(format!("{err}").to_lowercase().contains("assignment") || format!("{err}").contains("const"));
}

#[test]
fn test_eval_let_temporal_dead_zone() {
    let err = expect_runtime_error("{ console.log(x); let x = 1; }");
    assert!(format!("{err}").to_lowercase().contains("cannot access") || format!("{err}").contains("initialization"));
}

// ── Conditional expression ────────────────────────────────────────────────────

#[test]
fn test_eval_conditional_true() {
    expect_number("true ? 1 : 2", 1.0);
}

#[test]
fn test_eval_conditional_false() {
    expect_number("false ? 1 : 2", 2.0);
}

#[test]
fn test_eval_conditional_nested() {
    expect_string(
        "const x = 5; x > 10 ? 'big' : x > 3 ? 'medium' : 'small'",
        "medium",
    );
}

// ── Optional chaining ─────────────────────────────────────────────────────────

#[test]
fn test_eval_optional_chain_defined() {
    expect_string("const obj = { name: 'Alice' }; obj?.name", "Alice");
}

#[test]
fn test_eval_optional_chain_null() {
    expect_undefined("const obj = null; obj?.name");
}

#[test]
fn test_eval_optional_chain_undefined() {
    expect_undefined("const obj = undefined; obj?.name");
}

#[test]
fn test_eval_optional_chain_deep() {
    expect_string(
        "const a = { b: { c: { d: 'deep' } } }; a?.b?.c?.d",
        "deep",
    );
}

#[test]
fn test_eval_optional_chain_deep_null_short_circuit() {
    expect_undefined("const a = { b: null }; a?.b?.c?.d");
}

#[test]
fn test_eval_optional_call() {
    expect_number("const f = (x: number) => x * 2; f?.(5)", 10.0);
}

#[test]
fn test_eval_optional_call_null() {
    expect_undefined("const f = null; (f as any)?.(5)");
}

// ── instanceof ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_instanceof_true() {
    expect_bool("[] instanceof Array", true);
}

#[test]
fn test_eval_instanceof_false() {
    expect_bool("42 instanceof Array", false);
}

#[test]
fn test_eval_instanceof_class() {
    expect_bool(
        "class Foo {} const f = new Foo(); f instanceof Foo",
        true,
    );
}

#[test]
fn test_eval_instanceof_parent_class() {
    expect_bool(
        "class Animal {} class Dog extends Animal {} const d = new Dog(); d instanceof Animal",
        true,
    );
}

// ── in operator ───────────────────────────────────────────────────────────────

#[test]
fn test_eval_in_operator_present() {
    expect_bool(r#"'name' in { name: 'Alice', age: 30 }"#, true);
}

#[test]
fn test_eval_in_operator_absent() {
    expect_bool(r#"'email' in { name: 'Alice' }"#, false);
}

// ── void operator ─────────────────────────────────────────────────────────────

#[test]
fn test_eval_void_returns_undefined() {
    expect_undefined("void 42");
}

// ── delete operator ───────────────────────────────────────────────────────────

#[test]
fn test_eval_delete_property() {
    expect_bool(
        r#"const obj: any = { x: 1, y: 2 }; delete obj.x; !('x' in obj)"#,
        true,
    );
}

// ── Increment / decrement ─────────────────────────────────────────────────────

#[test]
fn test_eval_prefix_increment() {
    expect_number("let i = 5; ++i", 6.0);
}

#[test]
fn test_eval_postfix_increment_returns_old() {
    expect_number("let i = 5; i++", 5.0);
}

#[test]
fn test_eval_postfix_increment_mutates() {
    expect_number("let i = 5; i++; i", 6.0);
}

#[test]
fn test_eval_prefix_decrement() {
    expect_number("let i = 5; --i", 4.0);
}

#[test]
fn test_eval_postfix_decrement_returns_old() {
    expect_number("let i = 5; i--", 5.0);
}

// ── Comma operator ────────────────────────────────────────────────────────────

#[test]
fn test_eval_comma_in_for() {
    expect_number("let s = 0; for (let i = 0, j = 10; i < 5; i++, j--) s += 1; s", 5.0);
}
