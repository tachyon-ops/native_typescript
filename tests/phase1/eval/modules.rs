/// Evaluator tests — ESM module system: import, export, dynamic import.
/// ALGO: See SPECS.md §7 FR-EVAL-006

mod common;
use common::*;

use std::fs;
use tempfile::TempDir;

/// Write test module files to a temp directory and run them.
/// Returns the last expression value of `main.ts`.
fn eval_modules(files: &[(&str, &str)], entry: &str) -> tsnat_eval::Value {
    let dir = TempDir::new().unwrap();
    for (name, content) in files {
        fs::write(dir.path().join(name), content).unwrap();
    }
    let main_path = dir.path().join(entry);
    let src = fs::read_to_string(&main_path).unwrap();

    let mut interp = tsnat_eval::Interpreter::new();
    interp
        .eval_file(&main_path)
        .expect(&format!("eval_file failed for {entry}"))
}

// ── Named exports ─────────────────────────────────────────────────────────────

#[test]
fn test_eval_module_named_export_import() {
    let val = eval_modules(
        &[
            ("lib.ts", "export function add(a: number, b: number): number { return a + b; }\nexport const PI = 3.14159;"),
            ("main.ts", "import { add, PI } from './lib';\nadd(1, 2) + PI"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 6.14159).abs() < 1e-4),
        _ => panic!("expected number, got {val:?}"),
    }
}

#[test]
fn test_eval_module_renamed_import() {
    let val = eval_modules(
        &[
            ("math.ts", "export function square(x: number): number { return x * x; }"),
            ("main.ts", "import { square as sq } from './math';\nsq(7)"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 49.0).abs() < 1e-10),
        _ => panic!("expected 49"),
    }
}

// ── Default export / import ───────────────────────────────────────────────────

#[test]
fn test_eval_module_default_export_function() {
    let val = eval_modules(
        &[
            ("greet.ts", "export default function greet(name: string): string { return `Hello, ${name}!`; }"),
            ("main.ts", "import greet from './greet';\ngreet('World')"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::String(s) => assert_eq!(s.as_ref(), "Hello, World!"),
        _ => panic!("expected string"),
    }
}

#[test]
fn test_eval_module_default_export_value() {
    let val = eval_modules(
        &[
            ("config.ts", "const config = { version: '1.0.0', debug: false };\nexport default config;"),
            ("main.ts", "import config from './config';\nconfig.version"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::String(s) => assert_eq!(s.as_ref(), "1.0.0"),
        _ => panic!("expected string"),
    }
}

// ── Namespace import ──────────────────────────────────────────────────────────

#[test]
fn test_eval_module_namespace_import() {
    let val = eval_modules(
        &[
            ("math.ts", "export const add = (a: number, b: number) => a + b;\nexport const mul = (a: number, b: number) => a * b;"),
            ("main.ts", "import * as math from './math';\nmath.add(2, 3) * math.mul(2, 3)"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 30.0).abs() < 1e-10),
        _ => panic!("expected 30"),
    }
}

// ── Re-exports ────────────────────────────────────────────────────────────────

#[test]
fn test_eval_module_re_export() {
    let val = eval_modules(
        &[
            ("a.ts", "export const x = 10;"),
            ("b.ts", "export { x } from './a';\nexport const y = 20;"),
            ("main.ts", "import { x, y } from './b';\nx + y"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 30.0).abs() < 1e-10),
        _ => panic!("expected 30"),
    }
}

#[test]
fn test_eval_module_re_export_all() {
    let val = eval_modules(
        &[
            ("utils.ts", "export const double = (x: number) => x * 2;\nexport const triple = (x: number) => x * 3;"),
            ("index.ts", "export * from './utils';"),
            ("main.ts", "import { double, triple } from './index';\ndouble(3) + triple(2)"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 12.0).abs() < 1e-10),
        _ => panic!("expected 12"),
    }
}

// ── Module caching ────────────────────────────────────────────────────────────

#[test]
fn test_eval_module_cached() {
    // A module with side effects should run only once even if imported twice
    let val = eval_modules(
        &[
            ("counter.ts", "export let count = 0;\ncount++;\nexport function getCount(): number { return count; }"),
            ("a.ts", "import { getCount } from './counter';\nexport const fromA = getCount();"),
            ("b.ts", "import { getCount } from './counter';\nexport const fromB = getCount();"),
            ("main.ts", "import { fromA } from './a';\nimport { fromB } from './b';\nfromA + fromB"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 2.0).abs() < 1e-10, "expected 2 (module ran once), got {n}"),
        _ => panic!("expected number"),
    }
}

// ── Dynamic import ────────────────────────────────────────────────────────────

#[test]
fn test_eval_dynamic_import() {
    let val = eval_modules(
        &[
            ("lazy.ts", "export function compute(): number { return 42; }"),
            ("main.ts", "const m = await import('./lazy');\nm.compute()"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 42.0).abs() < 1e-10),
        _ => panic!("expected 42"),
    }
}

// ── Side-effect import ────────────────────────────────────────────────────────

#[test]
fn test_eval_side_effect_import() {
    // Side-effect imports should execute the module
    let dir = TempDir::new().unwrap();
    let flag_path = dir.path().join("flag.txt");
    let flag_str = flag_path.to_string_lossy().to_string();

    let setup_src = format!(
        r#"import '{{sideeffect}}';"#,
    );

    // We can't easily test file side effects, so instead test that a global is set
    let val = eval_modules(
        &[
            ("setup.ts", "import './effect';"),
            ("effect.ts", "(globalThis as any).__effectRan = true;"),
            ("main.ts", "import './setup';\n(globalThis as any).__effectRan"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Bool(b) => assert!(b),
        _ => panic!("expected true — side effect should have run"),
    }
}

// ── Circular imports ──────────────────────────────────────────────────────────

#[test]
fn test_eval_circular_import() {
    // Circular imports should not cause infinite loops
    // In ESM, circular bindings are live — but initial values may be undefined
    let val = eval_modules(
        &[
            ("a.ts", "import { getB } from './b';\nexport function getA(): string { return 'A'; }\nexport function callB(): string { return getB(); }"),
            ("b.ts", "import { getA } from './a';\nexport function getB(): string { return 'B:' + getA(); }"),
            ("main.ts", "import { callB } from './a';\ncallB()"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::String(s) => assert_eq!(s.as_ref(), "B:A"),
        _ => panic!("expected 'B:A'"),
    }
}

// ── Index resolution ──────────────────────────────────────────────────────────

#[test]
fn test_eval_module_index_resolution() {
    // import from './utils' should resolve to './utils/index.ts'
    let val = eval_modules(
        &[
            ("utils/index.ts", "export const value = 99;"),
            ("main.ts", "import { value } from './utils';\nvalue"),
        ],
        "main.ts",
    );
    match val {
        tsnat_eval::Value::Number(n) => assert!((n - 99.0).abs() < 1e-10),
        _ => panic!("expected 99"),
    }
}

// ── Type-only imports (runtime no-op) ────────────────────────────────────────

#[test]
fn test_eval_type_import_is_erased() {
    // import type should not introduce any runtime binding
    expect_parse_ok("import type { Foo } from './types';");
}
