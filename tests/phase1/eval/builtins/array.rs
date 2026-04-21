/// Evaluator tests — Array built-in methods.
/// ALGO: See SPECS.md §7 FR-EVAL-003

#[path = "../../../common/mod.rs"]
mod common;
use common::*;

// ── Construction ──────────────────────────────────────────────────────────────

#[test]
fn test_eval_array_literal() {
    expect_number("[1, 2, 3].length", 3.0);
}

#[test]
fn test_eval_array_constructor() {
    expect_number("new Array(5).length", 5.0);
}

#[test]
fn test_eval_array_from_iterable() {
    expect_string("Array.from('hello').join(',')", "h,e,l,l,o");
}

#[test]
fn test_eval_array_from_map() {
    expect_string("Array.from([1, 2, 3], (x: number) => x * 2).join(',')", "2,4,6");
}

#[test]
fn test_eval_array_of() {
    expect_string("Array.of(1, 2, 3).join(',')", "1,2,3");
}

#[test]
fn test_eval_array_is_array() {
    expect_bool("Array.isArray([]) && !Array.isArray({})", true);
}

// ── Mutation ─────────────────────────────────────────────────────────────────

#[test]
fn test_eval_array_push() {
    expect_number("const a = [1, 2]; a.push(3, 4); a.length", 4.0);
}

#[test]
fn test_eval_array_pop() {
    expect_number("const a = [1, 2, 3]; a.pop()", 3.0);
}

#[test]
fn test_eval_array_shift() {
    expect_number("const a = [1, 2, 3]; a.shift()", 1.0);
}

#[test]
fn test_eval_array_unshift() {
    expect_number("const a = [3, 4]; a.unshift(1, 2); a[0]", 1.0);
}

#[test]
fn test_eval_array_splice_remove() {
    expect_string("const a = [1, 2, 3, 4]; a.splice(1, 2); a.join(',')", "1,4");
}

#[test]
fn test_eval_array_splice_insert() {
    expect_string("const a = [1, 4]; a.splice(1, 0, 2, 3); a.join(',')", "1,2,3,4");
}

#[test]
fn test_eval_array_sort_numbers() {
    expect_string(
        "[3, 1, 4, 1, 5, 9, 2, 6].sort((a: number, b: number) => a - b).join(',')",
        "1,1,2,3,4,5,6,9",
    );
}

#[test]
fn test_eval_array_sort_strings() {
    expect_string(
        r#"['banana', 'apple', 'cherry'].sort().join(',')"#,
        "apple,banana,cherry",
    );
}

#[test]
fn test_eval_array_reverse() {
    expect_string("[1, 2, 3].reverse().join(',')", "3,2,1");
}

#[test]
fn test_eval_array_fill() {
    expect_string("[1, 2, 3, 4, 5].fill(0, 1, 3).join(',')", "1,0,0,4,5");
}

#[test]
fn test_eval_array_copy_within() {
    expect_string("[1, 2, 3, 4, 5].copyWithin(0, 3).join(',')", "4,5,3,4,5");
}

// ── Non-mutation ──────────────────────────────────────────────────────────────

#[test]
fn test_eval_array_map() {
    expect_string("[1, 2, 3].map((x: number) => x * 2).join(',')", "2,4,6");
}

#[test]
fn test_eval_array_filter() {
    expect_string(
        "[1, 2, 3, 4, 5, 6].filter((x: number) => x % 2 === 0).join(',')",
        "2,4,6",
    );
}

#[test]
fn test_eval_array_reduce() {
    expect_number("[1, 2, 3, 4, 5].reduce((acc: number, x: number) => acc + x, 0)", 15.0);
}

#[test]
fn test_eval_array_reduce_right() {
    expect_string(
        r#"['a','b','c'].reduceRight((acc: string, x: string) => acc + x, '')"#,
        "cba",
    );
}

#[test]
fn test_eval_array_for_each() {
    expect_number(
        "let s = 0; [1, 2, 3].forEach((x: number) => { s += x; }); s",
        6.0,
    );
}

#[test]
fn test_eval_array_find() {
    expect_number("[1, 2, 3, 4].find((x: number) => x > 2) as number", 3.0);
}

#[test]
fn test_eval_array_find_index() {
    expect_number("[1, 2, 3, 4].findIndex((x: number) => x > 2)", 2.0);
}

#[test]
fn test_eval_array_find_last() {
    expect_number("[1, 2, 3, 4].findLast((x: number) => x % 2 === 0) as number", 4.0);
}

#[test]
fn test_eval_array_some() {
    expect_bool("[1, 2, 3].some((x: number) => x > 2)", true);
}

#[test]
fn test_eval_array_every() {
    expect_bool("[2, 4, 6].every((x: number) => x % 2 === 0)", true);
}

#[test]
fn test_eval_array_includes() {
    expect_bool("[1, 2, 3].includes(2)", true);
}

#[test]
fn test_eval_array_includes_nan() {
    expect_bool("[NaN].includes(NaN)", true);
}

#[test]
fn test_eval_array_index_of() {
    expect_number("[1, 2, 3, 2, 1].indexOf(2)", 1.0);
}

#[test]
fn test_eval_array_last_index_of() {
    expect_number("[1, 2, 3, 2, 1].lastIndexOf(2)", 3.0);
}

#[test]
fn test_eval_array_slice() {
    expect_string("[1, 2, 3, 4, 5].slice(1, 3).join(',')", "2,3");
}

#[test]
fn test_eval_array_slice_negative() {
    expect_string("[1, 2, 3, 4, 5].slice(-2).join(',')", "4,5");
}

#[test]
fn test_eval_array_concat() {
    expect_string("[1, 2].concat([3, 4], [5]).join(',')", "1,2,3,4,5");
}

#[test]
fn test_eval_array_join() {
    expect_string("[1, 2, 3].join(' - ')", "1 - 2 - 3");
}

#[test]
fn test_eval_array_join_default() {
    expect_string("[1, 2, 3].join()", "1,2,3");
}

#[test]
fn test_eval_array_flat() {
    expect_string("[1, [2, [3, [4]]]].flat().join(',')", "1,2,3,4"); // default depth = 1
}

#[test]
fn test_eval_array_flat_deep() {
    expect_string("[1, [2, [3, [4]]]].flat(Infinity).join(',')", "1,2,3,4");
}

#[test]
fn test_eval_array_flat_map() {
    expect_string(
        "[1, 2, 3].flatMap((x: number) => [x, x * 2]).join(',')",
        "1,2,2,4,3,6",
    );
}

#[test]
fn test_eval_array_at() {
    expect_number("[1, 2, 3, 4, 5].at(-1) as number", 5.0);
}

#[test]
fn test_eval_array_keys() {
    expect_string("[...['a', 'b', 'c'].keys()].join(',')", "0,1,2");
}

#[test]
fn test_eval_array_values() {
    expect_string("[...['a', 'b', 'c'].values()].join(',')", "a,b,c");
}

#[test]
fn test_eval_array_entries() {
    expect_string(
        "[...['a', 'b'].entries()].map((entry: any) => `${entry[0]}:${entry[1]}`).join(',')",
        "0:a,1:b",
    );
}

// ── Typed arrays ──────────────────────────────────────────────────────────────

#[test]
fn test_eval_uint8array() {
    expect_number("new Uint8Array([1, 2, 3]).reduce((a: number, b: number) => a + b, 0)", 6.0);
}

#[test]
fn test_eval_int32array() {
    expect_number("new Int32Array(3).length", 3.0);
}

#[test]
fn test_eval_float64array() {
    expect_number("new Float64Array([1.5, 2.5]).reduce((a: number, b: number) => a + b, 0)", 4.0);
}
