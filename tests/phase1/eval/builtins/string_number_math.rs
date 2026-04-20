/// Evaluator tests — String, Number, Math built-in methods.
/// ALGO: See SPECS.md §7 FR-EVAL-003

mod common;
use common::*;

// ── String methods ────────────────────────────────────────────────────────────

#[test]
fn test_eval_string_length() {
    expect_number(r#""hello".length"#, 5.0);
}

#[test]
fn test_eval_string_at() {
    expect_string(r#""hello".at(-1) as string"#, "o");
}

#[test]
fn test_eval_string_char_at() {
    expect_string(r#""hello".charAt(1)"#, "e");
}

#[test]
fn test_eval_string_char_code_at() {
    expect_number(r#""A".charCodeAt(0)"#, 65.0);
}

#[test]
fn test_eval_string_from_char_code() {
    expect_string("String.fromCharCode(72, 101, 108, 108, 111)", "Hello");
}

#[test]
fn test_eval_string_index_of() {
    expect_number(r#""hello world".indexOf("world")"#, 6.0);
}

#[test]
fn test_eval_string_last_index_of() {
    expect_number(r#""abcabc".lastIndexOf("b")"#, 4.0);
}

#[test]
fn test_eval_string_includes() {
    expect_bool(r#""hello world".includes("world")"#, true);
}

#[test]
fn test_eval_string_starts_with() {
    expect_bool(r#""hello".startsWith("hel")"#, true);
}

#[test]
fn test_eval_string_ends_with() {
    expect_bool(r#""hello".endsWith("llo")"#, true);
}

#[test]
fn test_eval_string_slice() {
    expect_string(r#""hello world".slice(6)"#, "world");
}

#[test]
fn test_eval_string_slice_negative() {
    expect_string(r#""hello world".slice(-5)"#, "world");
}

#[test]
fn test_eval_string_substring() {
    expect_string(r#""hello".substring(1, 4)"#, "ell");
}

#[test]
fn test_eval_string_to_lower_case() {
    expect_string(r#""HELLO".toLowerCase()"#, "hello");
}

#[test]
fn test_eval_string_to_upper_case() {
    expect_string(r#""hello".toUpperCase()"#, "HELLO");
}

#[test]
fn test_eval_string_trim() {
    expect_string(r#""  hello  ".trim()"#, "hello");
}

#[test]
fn test_eval_string_trim_start() {
    expect_string(r#""  hello  ".trimStart()"#, "hello  ");
}

#[test]
fn test_eval_string_trim_end() {
    expect_string(r#""  hello  ".trimEnd()"#, "  hello");
}

#[test]
fn test_eval_string_split() {
    expect_string(r#""a,b,c".split(",").join("-")"#, "a-b-c");
}

#[test]
fn test_eval_string_split_limit() {
    expect_number(r#""a,b,c,d".split(",", 2).length"#, 2.0);
}

#[test]
fn test_eval_string_replace() {
    expect_string(r#""hello world".replace("world", "Alice")"#, "hello Alice");
}

#[test]
fn test_eval_string_replace_all() {
    expect_string(r#""aabbaa".replaceAll("a", "x")"#, "xxbbxx");
}

#[test]
fn test_eval_string_replace_regex() {
    expect_string(r#""hello 42 world 7".replace(/\d+/g, "#")"#, "hello # world #");
}

#[test]
fn test_eval_string_match() {
    expect_number(r#"("hello 42 world").match(/\d+/)![0].length"#, 2.0);
}

#[test]
fn test_eval_string_match_all() {
    expect_number(
        r#"[...("a1b2c3").matchAll(/[a-z](\d)/g)].length"#,
        3.0,
    );
}

#[test]
fn test_eval_string_pad_start() {
    expect_string(r#""5".padStart(3, "0")"#, "005");
}

#[test]
fn test_eval_string_pad_end() {
    expect_string(r#""hi".padEnd(5, ".")"#, "hi...");
}

#[test]
fn test_eval_string_repeat() {
    expect_string(r#""ab".repeat(3)"#, "ababab");
}

#[test]
fn test_eval_string_concat_method() {
    expect_string(r#""hello".concat(" ", "world")"#, "hello world");
}

// ── Number methods / globals ──────────────────────────────────────────────────

#[test]
fn test_eval_number_is_integer_true() {
    expect_bool("Number.isInteger(42)", true);
}

#[test]
fn test_eval_number_is_integer_false() {
    expect_bool("Number.isInteger(42.5)", false);
}

#[test]
fn test_eval_number_is_finite_true() {
    expect_bool("Number.isFinite(42)", true);
}

#[test]
fn test_eval_number_is_finite_false() {
    expect_bool("Number.isFinite(Infinity)", false);
}

#[test]
fn test_eval_number_is_nan_true() {
    expect_bool("Number.isNaN(NaN)", true);
}

#[test]
fn test_eval_number_is_nan_false() {
    expect_bool("Number.isNaN(42)", false);
}

#[test]
fn test_eval_number_parse_int() {
    expect_number("Number.parseInt('42abc')", 42.0);
}

#[test]
fn test_eval_number_parse_float() {
    expect_number("Number.parseFloat('3.14xyz')", 3.14);
}

#[test]
fn test_eval_number_to_fixed() {
    expect_string("(3.14159).toFixed(2)", "3.14");
}

#[test]
fn test_eval_number_max_safe_integer() {
    expect_number("Number.MAX_SAFE_INTEGER", 9007199254740991.0);
}

#[test]
fn test_eval_number_min_safe_integer() {
    expect_number("Number.MIN_SAFE_INTEGER", -9007199254740991.0);
}

#[test]
fn test_eval_global_parse_int() {
    expect_number("parseInt('16', 16)", 22.0);
}

#[test]
fn test_eval_global_parse_float() {
    expect_number("parseFloat('2.718')", 2.718);
}

#[test]
fn test_eval_global_is_nan() {
    expect_bool("isNaN(NaN)", true);
}

#[test]
fn test_eval_global_is_finite() {
    expect_bool("isFinite(42)", true);
}

// ── Math ──────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_math_pi() {
    expect_bool("Math.abs(Math.PI - 3.14159265358979) < 1e-10", true);
}

#[test]
fn test_eval_math_e() {
    expect_bool("Math.abs(Math.E - 2.71828182845905) < 1e-10", true);
}

#[test]
fn test_eval_math_abs() {
    expect_number("Math.abs(-42)", 42.0);
}

#[test]
fn test_eval_math_ceil() {
    expect_number("Math.ceil(4.1)", 5.0);
}

#[test]
fn test_eval_math_floor() {
    expect_number("Math.floor(4.9)", 4.0);
}

#[test]
fn test_eval_math_round() {
    expect_number("Math.round(4.5)", 5.0);
}

#[test]
fn test_eval_math_round_down() {
    expect_number("Math.round(4.4)", 4.0);
}

#[test]
fn test_eval_math_sqrt() {
    expect_number("Math.sqrt(16)", 4.0);
}

#[test]
fn test_eval_math_cbrt() {
    expect_number("Math.cbrt(27)", 3.0);
}

#[test]
fn test_eval_math_pow() {
    expect_number("Math.pow(2, 10)", 1024.0);
}

#[test]
fn test_eval_math_min() {
    expect_number("Math.min(3, 1, 4, 1, 5, 9)", 1.0);
}

#[test]
fn test_eval_math_max() {
    expect_number("Math.max(3, 1, 4, 1, 5, 9)", 9.0);
}

#[test]
fn test_eval_math_min_spread() {
    expect_number("Math.min(...[3, 1, 4, 1, 5, 9])", 1.0);
}

#[test]
fn test_eval_math_log() {
    expect_bool("Math.abs(Math.log(Math.E) - 1) < 1e-10", true);
}

#[test]
fn test_eval_math_log2() {
    expect_number("Math.log2(8)", 3.0);
}

#[test]
fn test_eval_math_log10() {
    expect_number("Math.log10(1000)", 3.0);
}

#[test]
fn test_eval_math_sin_cos() {
    expect_bool("Math.abs(Math.sin(Math.PI / 2) - 1) < 1e-10", true);
}

#[test]
fn test_eval_math_trunc() {
    expect_number("Math.trunc(-4.9)", -4.0);
}

#[test]
fn test_eval_math_sign_positive() {
    expect_number("Math.sign(42)", 1.0);
}

#[test]
fn test_eval_math_sign_negative() {
    expect_number("Math.sign(-7)", -1.0);
}

#[test]
fn test_eval_math_sign_zero() {
    expect_number("Math.sign(0)", 0.0);
}

#[test]
fn test_eval_math_hypot() {
    expect_number("Math.hypot(3, 4)", 5.0);
}

#[test]
fn test_eval_math_random_range() {
    expect_bool("const r = Math.random(); r >= 0 && r < 1", true);
}

#[test]
fn test_eval_math_clz32() {
    expect_number("Math.clz32(1)", 31.0);
}

#[test]
fn test_eval_math_fround() {
    expect_bool("typeof Math.fround(1.337) === 'number'", true);
}
