/// Phase 2 gate test — native FFI.
///
/// This test must pass before any Phase 3 work begins.
/// Tests that TypeScript can call a C function via FFI and receive a typed result.
/// ALGO: See SPECS.md §8 FR-FFI-001 through FR-FFI-005

#[path = "../common/mod.rs"]
mod common;
use common::*;

use std::path::PathBuf;

/// Compiles the C fixture library for the test.
/// The C source is at tests/fixtures/add.c.
fn build_fixture_lib() -> PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/test-fixtures"));
    std::fs::create_dir_all(&out_dir).unwrap();

    let lib_path = out_dir.join("libadd.so");
    if lib_path.exists() {
        return lib_path;
    }

    let status = std::process::Command::new("cc")
        .args([
            "-shared", "-fPIC", "-o",
            lib_path.to_str().unwrap(),
            "tests/fixtures/add.c",
        ])
        .status()
        .expect("failed to run cc — is a C compiler available?");

    assert!(status.success(), "C compilation failed");
    lib_path
}

// ── Gate test ─────────────────────────────────────────────────────────────────

#[test]
fn test_gate_ffi_call_c_add() {
    let lib_path = build_fixture_lib();
    let src = format!(
        r#"
        import native from '{lib}';

        declare native function add(a: number, b: number): number;

        add(3, 4)
        "#,
        lib = lib_path.display()
    );
    expect_number(&src, 7.0);
}

// ── Type mapping ──────────────────────────────────────────────────────────────

#[test]
fn test_ffi_number_roundtrip() {
    let lib_path = build_fixture_lib();
    let src = format!(
        r#"
        import native from '{lib}';
        declare native function add(a: number, b: number): number;
        add(100.5, 200.5)
        "#,
        lib = lib_path.display()
    );
    expect_number(&src, 301.0);
}

#[test]
fn test_ffi_string_roundtrip() {
    // Requires tests/fixtures/greet.c: char* greet(const char* name) { ... }
    let lib_path = build_fixture_lib();
    let src = format!(
        r#"
        import native from '{lib}';
        declare native function greet(name: string): string;
        greet("World")
        "#,
        lib = lib_path.display()
    );
    expect_string(&src, "Hello, World!");
}

#[test]
fn test_ffi_bool_roundtrip() {
    let lib_path = build_fixture_lib();
    let src = format!(
        r#"
        import native from '{lib}';
        declare native function is_positive(x: number): boolean;
        is_positive(5)
        "#,
        lib = lib_path.display()
    );
    expect_bool(&src, true);
}

// ── NativePtr ─────────────────────────────────────────────────────────────────

#[test]
fn test_ffi_native_ptr_opaque() {
    let lib_path = build_fixture_lib();
    let src = format!(
        r#"
        import native from '{lib}';
        declare native function make_counter(): NativePtr<Counter>;
        declare native function counter_increment(c: NativePtr<Counter>): void;
        declare native function counter_get(c: NativePtr<Counter>): number;

        const c = make_counter();
        counter_increment(c);
        counter_increment(c);
        counter_increment(c);
        counter_get(c)
        "#,
        lib = lib_path.display()
    );
    expect_number(&src, 3.0);
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn test_ffi_library_not_found() {
    let err = expect_runtime_error(
        r#"
        import native from '/nonexistent/library.so';
        declare native function f(): void;
        f();
        "#,
    );
    let msg = format!("{err}");
    assert!(
        msg.contains("not found") || msg.contains("cannot open") || msg.contains("Ffi"),
        "unexpected error: {msg}"
    );
}
