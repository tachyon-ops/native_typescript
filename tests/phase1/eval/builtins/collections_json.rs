/// Evaluator tests — Map, Set, WeakMap, WeakSet, JSON, Date, RegExp, Error.
/// ALGO: See SPECS.md §7 FR-EVAL-003

#[path = "../../../common/mod.rs"]
mod common;
use common::*;

// ── Map ───────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_map_set_get() {
    expect_number(
        "const m = new Map<string, number>(); m.set('a', 1); m.get('a') as number",
        1.0,
    );
}

#[test]
fn test_eval_map_size() {
    expect_number("new Map([['a', 1], ['b', 2], ['c', 3]]).size", 3.0);
}

#[test]
fn test_eval_map_has() {
    expect_bool("const m = new Map([['a', 1]]); m.has('a') && !m.has('b')", true);
}

#[test]
fn test_eval_map_delete() {
    expect_bool("const m = new Map([['a', 1]]); m.delete('a'); !m.has('a')", true);
}

#[test]
fn test_eval_map_clear() {
    expect_number("const m = new Map([['a', 1], ['b', 2]]); m.clear(); m.size", 0.0);
}

#[test]
fn test_eval_map_iteration_order() {
    expect_string(
        r#"const m = new Map([['c', 3], ['a', 1], ['b', 2]]); [...m.keys()].join(',')"#,
        "c,a,b",
    );
}

#[test]
fn test_eval_map_for_of() {
    expect_number(
        r#"
        const m = new Map([['a', 1], ['b', 2], ['c', 3]]);
        let s = 0;
        for (const [, v] of m) s += v;
        s
        "#,
        6.0,
    );
}

#[test]
fn test_eval_map_object_key() {
    expect_number(
        r#"
        const m = new Map<object, number>();
        const key = {};
        m.set(key, 42);
        m.get(key) as number
        "#,
        42.0,
    );
}

#[test]
fn test_eval_map_from_entries() {
    expect_number(
        "new Map(Object.entries({ a: 1, b: 2 })).get('b') as number",
        2.0,
    );
}

// ── Set ───────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_set_add_size() {
    expect_number("new Set([1, 2, 2, 3, 3, 3]).size", 3.0);
}

#[test]
fn test_eval_set_has() {
    expect_bool("new Set([1, 2, 3]).has(2)", true);
}

#[test]
fn test_eval_set_delete() {
    expect_bool("const s = new Set([1, 2, 3]); s.delete(2); !s.has(2)", true);
}

#[test]
fn test_eval_set_clear() {
    expect_number("const s = new Set([1, 2, 3]); s.clear(); s.size", 0.0);
}

#[test]
fn test_eval_set_iteration() {
    expect_string(
        "[...new Set([3, 1, 2])].join(',')",
        "3,1,2",
    );
}

#[test]
fn test_eval_set_for_each() {
    expect_number(
        "let s = 0; new Set([1, 2, 3]).forEach((v: number) => { s += v; }); s",
        6.0,
    );
}

#[test]
fn test_eval_set_deduplication() {
    expect_number("[...new Set([1,1,2,2,3,3,3])].length", 3.0);
}

// ── WeakMap ───────────────────────────────────────────────────────────────────

#[test]
fn test_eval_weakmap_set_get() {
    expect_number(
        r#"
        const wm = new WeakMap<object, number>();
        const key = {};
        wm.set(key, 99);
        wm.get(key) as number
        "#,
        99.0,
    );
}

#[test]
fn test_eval_weakmap_has() {
    expect_bool(
        "const wm = new WeakMap(); const k = {}; wm.set(k, 1); wm.has(k)",
        true,
    );
}

#[test]
fn test_eval_weakmap_delete() {
    expect_bool(
        "const wm = new WeakMap(); const k = {}; wm.set(k, 1); wm.delete(k); !wm.has(k)",
        true,
    );
}

// ── WeakSet ───────────────────────────────────────────────────────────────────

#[test]
fn test_eval_weakset_add_has() {
    expect_bool(
        "const ws = new WeakSet(); const o = {}; ws.add(o); ws.has(o)",
        true,
    );
}

#[test]
fn test_eval_weakset_delete() {
    expect_bool(
        "const ws = new WeakSet(); const o = {}; ws.add(o); ws.delete(o); !ws.has(o)",
        true,
    );
}

// ── JSON ──────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_json_stringify_object() {
    expect_string(
        r#"JSON.stringify({ a: 1, b: 2 })"#,
        r#"{"a":1,"b":2}"#,
    );
}

#[test]
fn test_eval_json_stringify_array() {
    expect_string("JSON.stringify([1, 2, 3])", "[1,2,3]");
}

#[test]
fn test_eval_json_stringify_null() {
    expect_string("JSON.stringify(null)", "null");
}

#[test]
fn test_eval_json_stringify_nested() {
    expect_bool(
        r#"typeof JSON.stringify({ a: { b: [1, 2] } }) === 'string'"#,
        true,
    );
}

#[test]
fn test_eval_json_parse_object() {
    expect_number(r#"JSON.parse('{"x":42}').x"#, 42.0);
}

#[test]
fn test_eval_json_parse_array() {
    expect_number(r#"JSON.parse('[1,2,3]')[1]"#, 2.0);
}

#[test]
fn test_eval_json_roundtrip() {
    expect_bool(
        r#"
        const obj = { name: 'Alice', scores: [10, 20, 30] };
        const parsed = JSON.parse(JSON.stringify(obj));
        parsed.name === obj.name && parsed.scores[2] === 30
        "#,
        true,
    );
}

#[test]
fn test_eval_json_parse_invalid_throws() {
    let err = expect_runtime_error(r#"JSON.parse("not json")"#);
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("json") || msg.contains("SyntaxError"), "unexpected error: {msg}");
}

// ── Date ──────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_date_now_is_number() {
    expect_bool("typeof Date.now() === 'number'", true);
}

#[test]
fn test_eval_date_new() {
    expect_bool("new Date() instanceof Date", true);
}

#[test]
fn test_eval_date_from_timestamp() {
    expect_number("new Date(0).getFullYear()", 1970.0);
}

#[test]
fn test_eval_date_get_time() {
    expect_bool("typeof new Date().getTime() === 'number'", true);
}

// ── RegExp ────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_regexp_test() {
    expect_bool(r#"/^\d+$/.test("123")"#, true);
}

#[test]
fn test_eval_regexp_test_false() {
    expect_bool(r#"/^\d+$/.test("12a")"#, false);
}

#[test]
fn test_eval_regexp_exec() {
    expect_string(r#"/(\w+)/.exec("hello world")![1]"#, "hello");
}

#[test]
fn test_eval_regexp_global_match() {
    expect_number(r#""a1b2c3".match(/\d/g)!.length"#, 3.0);
}

#[test]
fn test_eval_regexp_constructor() {
    expect_bool(
        r#"new RegExp("^\\d+$").test("123")"#,
        true,
    );
}

// ── Error types ───────────────────────────────────────────────────────────────

#[test]
fn test_eval_error_message() {
    expect_string(r#"new Error("oops").message"#, "oops");
}

#[test]
fn test_eval_error_instanceof() {
    expect_bool("new Error() instanceof Error", true);
}

#[test]
fn test_eval_type_error() {
    expect_bool("new TypeError('type') instanceof TypeError && new TypeError('type') instanceof Error", true);
}

#[test]
fn test_eval_range_error() {
    expect_bool("new RangeError('range') instanceof RangeError", true);
}

#[test]
fn test_eval_reference_error_thrown() {
    let err = expect_runtime_error("undeclaredVar;");
    let msg = format!("{err}");
    assert!(msg.contains("undeclaredVar") || msg.to_lowercase().contains("reference"));
}

#[test]
fn test_eval_error_stack_is_string() {
    expect_bool("typeof new Error('x').stack === 'string'", true);
}
