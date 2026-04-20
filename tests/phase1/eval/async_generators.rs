/// Evaluator tests — async/await, Promises, generators.
/// ALGO: See SPECS.md §7 FR-EVAL-004, FR-EVAL-005

mod common;
use common::*;

// ── Promise basics ────────────────────────────────────────────────────────────

#[test]
fn test_eval_promise_resolve() {
    expect_number("await Promise.resolve(42)", 42.0);
}

#[test]
fn test_eval_promise_reject_caught() {
    expect_string(
        r#"
        let msg = '';
        try {
            await Promise.reject(new Error('fail'));
        } catch (e) {
            msg = (e as Error).message;
        }
        msg
        "#,
        "fail",
    );
}

#[test]
fn test_eval_promise_then() {
    expect_number("await Promise.resolve(10).then(x => x * 2)", 20.0);
}

#[test]
fn test_eval_promise_chain() {
    expect_number(
        r#"
        await Promise.resolve(1)
            .then(x => x + 1)
            .then(x => x * 3)
            .then(x => x - 1)
        "#,
        5.0,
    );
}

#[test]
fn test_eval_promise_catch() {
    expect_string(
        r#"
        await Promise.reject(new Error('bad'))
            .catch(e => (e as Error).message)
        "#,
        "bad",
    );
}

#[test]
fn test_eval_promise_finally() {
    expect_number(
        r#"
        let side = 0;
        const result = await Promise.resolve(42).finally(() => { side = 1; });
        result + side
        "#,
        43.0,
    );
}

// ── Promise combinators ───────────────────────────────────────────────────────

#[test]
fn test_eval_promise_all() {
    expect_string(
        r#"
        const results = await Promise.all([
            Promise.resolve(1),
            Promise.resolve(2),
            Promise.resolve(3),
        ]);
        results.join(',')
        "#,
        "1,2,3",
    );
}

#[test]
fn test_eval_promise_all_rejects_on_any_failure() {
    expect_string(
        r#"
        let caught = '';
        try {
            await Promise.all([
                Promise.resolve(1),
                Promise.reject(new Error('fail')),
                Promise.resolve(3),
            ]);
        } catch (e) {
            caught = (e as Error).message;
        }
        caught
        "#,
        "fail",
    );
}

#[test]
fn test_eval_promise_allsettled() {
    expect_number(
        r#"
        const results = await Promise.allSettled([
            Promise.resolve(1),
            Promise.reject(new Error('x')),
            Promise.resolve(3),
        ]);
        results.filter(r => r.status === 'fulfilled').length
        "#,
        2.0,
    );
}

#[test]
fn test_eval_promise_race() {
    // Both resolve, first one wins (order is synchronous here)
    expect_number(
        r#"
        await Promise.race([
            Promise.resolve(1),
            Promise.resolve(2),
        ])
        "#,
        1.0,
    );
}

#[test]
fn test_eval_promise_any() {
    expect_number(
        r#"
        await Promise.any([
            Promise.reject(new Error('a')),
            Promise.resolve(42),
            Promise.resolve(99),
        ])
        "#,
        42.0,
    );
}

#[test]
fn test_eval_promise_any_all_rejected() {
    expect_bool(
        r#"
        let isAggregateError = false;
        try {
            await Promise.any([Promise.reject(1), Promise.reject(2)]);
        } catch (e) {
            isAggregateError = (e as any).constructor.name === 'AggregateError';
        }
        isAggregateError
        "#,
        true,
    );
}

// ── async / await ─────────────────────────────────────────────────────────────

#[test]
fn test_eval_async_function_returns_promise() {
    expect_bool(
        "async function f() { return 1; } f() instanceof Promise",
        true,
    );
}

#[test]
fn test_eval_async_await_basic() {
    expect_number(
        r#"
        async function fetchNum(): Promise<number> {
            return 42;
        }
        await fetchNum()
        "#,
        42.0,
    );
}

#[test]
fn test_eval_async_await_chain() {
    expect_number(
        r#"
        async function step1(): Promise<number> { return 1; }
        async function step2(n: number): Promise<number> { return n + 1; }
        async function step3(n: number): Promise<number> { return n * 3; }
        async function run(): Promise<number> {
            const a = await step1();
            const b = await step2(a);
            return await step3(b);
        }
        await run()
        "#,
        6.0,
    );
}

#[test]
fn test_eval_async_error_propagation() {
    expect_string(
        r#"
        async function fails(): Promise<never> {
            throw new Error("async fail");
        }
        async function run(): Promise<string> {
            try {
                await fails();
                return 'no error';
            } catch (e) {
                return (e as Error).message;
            }
        }
        await run()
        "#,
        "async fail",
    );
}

#[test]
fn test_eval_async_parallel() {
    expect_number(
        r#"
        async function delay(n: number): Promise<number> {
            return new Promise(resolve => setTimeout(() => resolve(n), 0));
        }
        async function run(): Promise<number> {
            const [a, b, c] = await Promise.all([delay(1), delay(2), delay(3)]);
            return a + b + c;
        }
        await run()
        "#,
        6.0,
    );
}

#[test]
fn test_eval_async_arrow() {
    expect_number(
        r#"
        const double = async (x: number): Promise<number> => x * 2;
        await double(21)
        "#,
        42.0,
    );
}

#[test]
fn test_eval_async_for_await_of() {
    expect_number(
        r#"
        async function* generate() {
            yield 1;
            yield 2;
            yield 3;
        }
        async function run(): Promise<number> {
            let s = 0;
            for await (const x of generate()) { s += x; }
            return s;
        }
        await run()
        "#,
        6.0,
    );
}

// ── Generators ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_generator_basic() {
    expect_string(
        r#"
        function* gen() {
            yield 1;
            yield 2;
            yield 3;
        }
        [...gen()].join(',')
        "#,
        "1,2,3",
    );
}

#[test]
fn test_eval_generator_manual_iteration() {
    expect_number(
        r#"
        function* counter() {
            let i = 0;
            while (true) yield i++;
        }
        const g = counter();
        g.next().value + g.next().value + g.next().value
        "#,
        3.0, // 0 + 1 + 2
    );
}

#[test]
fn test_eval_generator_return_value() {
    expect_bool(
        r#"
        function* gen() { yield 1; return 'done'; }
        const g = gen();
        g.next(); // { value: 1, done: false }
        const last = g.next(); // { value: 'done', done: true }
        last.done === true && last.value === 'done'
        "#,
        true,
    );
}

#[test]
fn test_eval_generator_yield_star() {
    expect_string(
        r#"
        function* inner() { yield 'a'; yield 'b'; }
        function* outer() { yield 'before'; yield* inner(); yield 'after'; }
        [...outer()].join(',')
        "#,
        "before,a,b,after",
    );
}

#[test]
fn test_eval_generator_next_with_value() {
    expect_number(
        r#"
        function* accumulator() {
            let total = 0;
            while (true) {
                const n: number = yield total;
                total += n;
            }
        }
        const g = accumulator();
        g.next(0);   // start
        g.next(10);  // total = 10
        g.next(20);  // total = 30
        g.next(12).value // total = 42
        "#,
        42.0,
    );
}

#[test]
fn test_eval_generator_throw() {
    expect_string(
        r#"
        function* gen() {
            try {
                yield 1;
            } catch (e) {
                yield (e as Error).message;
            }
        }
        const g = gen();
        g.next();
        g.throw(new Error('thrown')).value as string
        "#,
        "thrown",
    );
}

#[test]
fn test_eval_generator_return_early() {
    expect_bool(
        r#"
        function* gen() { yield 1; yield 2; yield 3; }
        const g = gen();
        g.next();
        const r = g.return('early');
        r.done && r.value === 'early'
        "#,
        true,
    );
}

#[test]
fn test_eval_generator_fibonacci() {
    expect_string(
        r#"
        function* fibonacci() {
            let [a, b] = [0, 1];
            while (true) { yield a; [a, b] = [b, a + b]; }
        }
        const fib = fibonacci();
        Array.from({ length: 8 }, () => fib.next().value).join(',')
        "#,
        "0,1,1,2,3,5,8,13",
    );
}

#[test]
fn test_eval_generator_range() {
    expect_string(
        r#"
        function* range(start: number, end: number, step = 1) {
            for (let i = start; i < end; i += step) yield i;
        }
        [...range(0, 10, 2)].join(',')
        "#,
        "0,2,4,6,8",
    );
}

// ── Microtask ordering ────────────────────────────────────────────────────────

#[test]
fn test_eval_microtask_order() {
    expect_string(
        r#"
        const log: string[] = [];
        Promise.resolve().then(() => log.push('micro1'));
        Promise.resolve().then(() => log.push('micro2'));
        log.push('sync');
        await Promise.resolve();
        log.join(',')
        "#,
        "sync,micro1,micro2",
    );
}
