/// Evaluator tests — functions, closures, parameters, prototypes.
/// ALGO: See SPECS.md §7 FR-EVAL-001, FR-EVAL-002, FR-EVAL-005

#[path = "../../common/mod.rs"]
mod common;
use common::*;

// ── Function declarations ─────────────────────────────────────────────────────

#[test]
fn test_eval_function_declaration() {
    expect_number("function add(a: number, b: number): number { return a + b; } add(3, 4)", 7.0);
}

#[test]
fn test_eval_function_hoisting() {
    // Function declarations are hoisted — callable before the declaration
    expect_number("add(3, 4); function add(a: number, b: number) { return a + b; }", 7.0);
}

#[test]
fn test_eval_function_expression() {
    expect_number("const f = function(x: number) { return x * 2; }; f(5)", 10.0);
}

#[test]
fn test_eval_named_function_expression_internal() {
    // Named function expressions can refer to themselves by name internally
    expect_number(
        r#"
        const factorial = function fact(n: number): number {
            return n <= 1 ? 1 : n * fact(n - 1);
        };
        factorial(5)
        "#,
        120.0,
    );
}

// ── Arrow functions ───────────────────────────────────────────────────────────

#[test]
fn test_eval_arrow_concise() {
    expect_number("const double = (x: number) => x * 2; double(7)", 14.0);
}

#[test]
fn test_eval_arrow_block_body() {
    expect_number("const abs = (x: number) => { return x < 0 ? -x : x; }; abs(-5)", 5.0);
}

#[test]
fn test_eval_arrow_no_parens() {
    expect_number("const inc = (n: number) => n + 1; inc(9)", 10.0);
}

#[test]
fn test_eval_arrow_returns_object() {
    // Parenthesised object literal in concise arrow
    expect_number(
        "const makePoint = (x: number, y: number) => ({ x, y }); makePoint(1, 2).x",
        1.0,
    );
}

// ── Default parameters ────────────────────────────────────────────────────────

#[test]
fn test_eval_default_param_used() {
    expect_number(
        "function greet(name: string = 'World') { return name.length; } greet()",
        5.0,
    );
}

#[test]
fn test_eval_default_param_overridden() {
    expect_number(
        "function add(a: number, b: number = 10) { return a + b; } add(5, 20)",
        25.0,
    );
}

#[test]
fn test_eval_default_param_expression() {
    expect_number(
        "let base = 10; function f(x: number = base * 2) { return x; } f()",
        20.0,
    );
}

// ── Rest parameters ───────────────────────────────────────────────────────────

#[test]
fn test_eval_rest_param() {
    expect_number(
        "function sum(...nums: number[]): number { return nums.reduce((a, b) => a + b, 0); } sum(1, 2, 3, 4, 5)",
        15.0,
    );
}

#[test]
fn test_eval_rest_param_mixed() {
    expect_number(
        "function f(first: number, ...rest: number[]) { return first + rest.length; } f(10, 20, 30, 40)",
        13.0,
    );
}

// ── Destructuring parameters ──────────────────────────────────────────────────

#[test]
fn test_eval_destructure_array_param() {
    expect_number(
        "function f([a, b]: number[]) { return a + b; } f([3, 4])",
        7.0,
    );
}

#[test]
fn test_eval_destructure_object_param() {
    expect_number(
        "function f({ x, y }: { x: number; y: number }) { return x + y; } f({ x: 10, y: 20 })",
        30.0,
    );
}

#[test]
fn test_eval_destructure_param_default() {
    expect_string(
        r#"function f({ name = 'Anonymous' }: { name?: string }) { return name; } f({})"#,
        "Anonymous",
    );
}

// ── Closures ──────────────────────────────────────────────────────────────────

#[test]
fn test_eval_closure_captures() {
    expect_number(
        r#"
        function makeAdder(x: number) {
            return (y: number) => x + y;
        }
        const add5 = makeAdder(5);
        add5(3)
        "#,
        8.0,
    );
}

#[test]
fn test_eval_closure_mutable_capture() {
    expect_number(
        r#"
        function makeCounter() {
            let count = 0;
            return {
                inc: () => ++count,
                get: () => count,
            };
        }
        const c = makeCounter();
        c.inc();
        c.inc();
        c.inc();
        c.get()
        "#,
        3.0,
    );
}

#[test]
fn test_eval_closure_shared_mutable() {
    expect_number(
        r#"
        function makeShared() {
            let n = 0;
            const inc = () => { n++; };
            const get = () => n;
            return { inc, get };
        }
        const { inc, get } = makeShared();
        inc(); inc(); inc();
        get()
        "#,
        3.0,
    );
}

#[test]
fn test_eval_iife() {
    expect_number("((x: number) => x * x)(7)", 49.0);
}

#[test]
fn test_eval_closure_loop_capture() {
    // Classic let-in-loop closure test — each closure captures its own i
    expect_number(
        r#"
        const funcs: (() => number)[] = [];
        for (let i = 0; i < 5; i++) {
            funcs.push(() => i);
        }
        funcs[3]()
        "#,
        3.0,
    );
}

#[test]
fn test_eval_var_loop_shares_binding() {
    // var doesn't create per-iteration binding — all closures see final value
    expect_number(
        r#"
        const funcs: (() => number)[] = [];
        for (var i = 0; i < 5; i++) {
            funcs.push(() => i);
        }
        funcs[0]()
        "#,
        5.0,
    );
}

// ── Recursion ─────────────────────────────────────────────────────────────────

#[test]
fn test_eval_recursion_factorial() {
    expect_number(
        "function factorial(n: number): number { return n <= 1 ? 1 : n * factorial(n - 1); } factorial(6)",
        720.0,
    );
}

#[test]
fn test_eval_recursion_fibonacci() {
    expect_number(
        r#"
        function fib(n: number): number {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
        "#,
        55.0,
    );
}

#[test]
fn test_eval_mutual_recursion() {
    expect_bool(
        r#"
        function isEven(n: number): boolean { return n === 0 ? true : isOdd(n - 1); }
        function isOdd(n: number): boolean  { return n === 0 ? false : isEven(n - 1); }
        isEven(4) && isOdd(7)
        "#,
        true,
    );
}

// ── this binding ──────────────────────────────────────────────────────────────

#[test]
fn test_eval_method_this_binding() {
    expect_number(
        r#"
        const obj = {
            value: 42,
            getValue() { return this.value; }
        };
        obj.getValue()
        "#,
        42.0,
    );
}

#[test]
fn test_eval_arrow_no_this_binding() {
    expect_number(
        r#"
        const obj = {
            value: 42,
            getArrow() {
                const f = () => this.value; // arrow captures outer 'this'
                return f();
            }
        };
        obj.getArrow()
        "#,
        42.0,
    );
}

#[test]
fn test_eval_function_call_this_undefined_strict() {
    // In a plain function call (non-method), 'this' is undefined in strict mode
    // TypeScript files are always strict modules
    expect_undefined(
        r#"
        function getThis() { return this; }
        getThis()
        "#,
    );
}

// ── Prototype chain ───────────────────────────────────────────────────────────

#[test]
fn test_eval_prototype_lookup() {
    expect_string(
        r#"
        const proto = { greet() { return 'hello'; } };
        const obj = Object.create(proto);
        obj.greet()
        "#,
        "hello",
    );
}

#[test]
fn test_eval_prototype_override() {
    expect_string(
        r#"
        const proto = { greet() { return 'hello'; } };
        const obj = Object.create(proto);
        obj.greet = () => 'hi';
        obj.greet()
        "#,
        "hi",
    );
}

// ── Spread in calls ───────────────────────────────────────────────────────────

#[test]
fn test_eval_spread_call_args() {
    expect_number(
        "function sum(a: number, b: number, c: number) { return a + b + c; } sum(...[1, 2, 3])",
        6.0,
    );
}

#[test]
fn test_eval_spread_mixed_args() {
    expect_number(
        "function f(a: number, b: number, c: number, d: number) { return a+b+c+d; } f(1, ...[2, 3], 4)",
        10.0,
    );
}

// ── Higher-order functions ────────────────────────────────────────────────────

#[test]
fn test_eval_higher_order_compose() {
    expect_number(
        r#"
        function compose<A, B, C>(f: (b: B) => C, g: (a: A) => B): (a: A) => C {
            return (a: A) => f(g(a));
        }
        const double = (x: number) => x * 2;
        const addOne = (x: number) => x + 1;
        const doubleThenAdd = compose(addOne, double);
        doubleThenAdd(5)
        "#,
        11.0,
    );
}

#[test]
fn test_eval_higher_order_currying() {
    expect_number(
        r#"
        const curry = (f: (a: number, b: number) => number) =>
            (a: number) => (b: number) => f(a, b);
        const add = (a: number, b: number) => a + b;
        curry(add)(3)(4)
        "#,
        7.0,
    );
}
