/// Phase 5 gate test — native code generation.
///
/// Compiles a TypeScript program to a native binary and executes it.
/// ALGO: See SPECS.md §10 FR-CG-001 through FR-CG-004

#[path = "../common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn tsnat_bin() -> PathBuf {
    // The CLI binary built by cargo
    let mut path = std::env::current_exe()
        .unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf();
    path.push("tsnat");
    path
}

fn compile_and_run(src: &str) -> (String, Duration) {
    let dir = TempDir::new().unwrap();
    let src_path = dir.path().join("main.ts");
    let bin_path = dir.path().join("main");
    std::fs::write(&src_path, src).unwrap();

    // Compile
    let compile_status = Command::new(tsnat_bin())
        .args(["build", src_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .status()
        .expect("tsnat build failed to run");
    assert!(compile_status.success(), "compilation failed");

    // Run and measure
    let start = Instant::now();
    let output = Command::new(&bin_path)
        .output()
        .expect("failed to run compiled binary");
    let duration = start.elapsed();

    assert!(output.status.success(), "binary exited with non-zero status");
    let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    (stdout, duration)
}

// ════════════════════════════════════════════════════════════════════════════
// Gate test
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gate_native_binary_hello_world() {
    let (output, startup) = compile_and_run(r#"
        console.log("hello from native");
    "#);
    assert_eq!(output, "hello from native");
    // Startup time < 100ms
    assert!(
        startup < Duration::from_millis(100),
        "startup time {}ms exceeds 100ms budget",
        startup.as_millis()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Correctness
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_native_arithmetic() {
    let (output, _) = compile_and_run("console.log(2 ** 10);");
    assert_eq!(output, "1024");
}

#[test]
fn test_native_string_operations() {
    let (output, _) = compile_and_run(r#"
        const words = ['hello', 'world'];
        console.log(words.join(' ').toUpperCase());
    "#);
    assert_eq!(output, "HELLO WORLD");
}

#[test]
fn test_native_class() {
    let (output, _) = compile_and_run(r#"
        class Fibonacci {
            compute(n: number): number {
                if (n <= 1) return n;
                return this.compute(n - 1) + this.compute(n - 2);
            }
        }
        console.log(new Fibonacci().compute(10));
    "#);
    assert_eq!(output, "55");
}

#[test]
fn test_native_async_await() {
    let (output, _) = compile_and_run(r#"
        async function main(): Promise<void> {
            const result = await Promise.resolve(42);
            console.log(result);
        }
        main();
    "#);
    assert_eq!(output, "42");
}

#[test]
fn test_native_map_filter_reduce() {
    let (output, _) = compile_and_run(r#"
        const result = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .filter(x => x % 2 === 0)
            .map(x => x * x)
            .reduce((a, b) => a + b, 0);
        console.log(result);
    "#);
    assert_eq!(output, "220"); // 4+16+36+64+100
}

#[test]
fn test_native_generators() {
    let (output, _) = compile_and_run(r#"
        function* range(n: number) {
            for (let i = 0; i < n; i++) yield i;
        }
        const sum = [...range(100)].reduce((a, b) => a + b, 0);
        console.log(sum);
    "#);
    assert_eq!(output, "4950");
}

// ════════════════════════════════════════════════════════════════════════════
// Performance
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_native_startup_under_100ms() {
    let (_, duration) = compile_and_run("console.log('ok');");
    assert!(
        duration < Duration::from_millis(100),
        "startup was {}ms, expected < 100ms",
        duration.as_millis()
    );
}

#[test]
fn test_native_fibonacci_performance() {
    // fib(35) should complete in < 2 seconds even without optimisation
    let start = Instant::now();
    let (output, _) = compile_and_run(r#"
        function fib(n: number): number {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        console.log(fib(35));
    "#);
    let total = start.elapsed();
    assert_eq!(output, "9227465");
    assert!(
        total < Duration::from_secs(2),
        "fib(35) took {}ms, expected < 2000ms",
        total.as_millis()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// tsnat check
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_check_exits_1_on_type_error() {
    let dir = TempDir::new().unwrap();
    let src_path = dir.path().join("bad.ts");
    std::fs::write(&src_path, r#"const x: number = "wrong";"#).unwrap();

    let status = Command::new(tsnat_bin())
        .args(["check", src_path.to_str().unwrap()])
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(1), "tsnat check should exit 1 on type errors");
}

#[test]
fn test_check_exits_0_on_clean_file() {
    let dir = TempDir::new().unwrap();
    let src_path = dir.path().join("ok.ts");
    std::fs::write(&src_path, "const x: number = 42;\nconsole.log(x);").unwrap();

    let status = Command::new(tsnat_bin())
        .args(["check", src_path.to_str().unwrap()])
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(0), "tsnat check should exit 0 on clean file");
}
