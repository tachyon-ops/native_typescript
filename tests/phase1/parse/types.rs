/// Parser tests — TypeScript type annotations.
/// ALGO: See SPECS.md §4 FR-PAR-006

mod common;
use common::*;

// ── Primitive types ───────────────────────────────────────────────────────────

#[test]
fn test_parse_type_number() {
    expect_parse_ok("let x: number;");
}

#[test]
fn test_parse_type_string() {
    expect_parse_ok("let x: string;");
}

#[test]
fn test_parse_type_boolean() {
    expect_parse_ok("let x: boolean;");
}

#[test]
fn test_parse_type_bigint() {
    expect_parse_ok("let x: bigint;");
}

#[test]
fn test_parse_type_symbol() {
    expect_parse_ok("let x: symbol;");
}

#[test]
fn test_parse_type_null() {
    expect_parse_ok("let x: null;");
}

#[test]
fn test_parse_type_undefined() {
    expect_parse_ok("let x: undefined;");
}

#[test]
fn test_parse_type_void() {
    expect_parse_ok("function f(): void {}");
}

#[test]
fn test_parse_type_never() {
    expect_parse_ok("function f(): never { throw new Error(); }");
}

#[test]
fn test_parse_type_unknown() {
    expect_parse_ok("let x: unknown;");
}

#[test]
fn test_parse_type_any() {
    expect_parse_ok("let x: any;");
}

#[test]
fn test_parse_type_object() {
    expect_parse_ok("let x: object;");
}

// ── Literal types ─────────────────────────────────────────────────────────────

#[test]
fn test_parse_type_number_literal() {
    expect_parse_ok("let x: 42;");
}

#[test]
fn test_parse_type_string_literal() {
    expect_parse_ok(r#"let x: "hello";"#);
}

#[test]
fn test_parse_type_bool_literal() {
    expect_parse_ok("let x: true;");
}

// ── Compound types ────────────────────────────────────────────────────────────

#[test]
fn test_parse_type_union() {
    expect_parse_ok("let x: string | number | null;");
}

#[test]
fn test_parse_type_intersection() {
    expect_parse_ok("let x: A & B & C;");
}

#[test]
fn test_parse_type_union_and_intersection() {
    expect_parse_ok("let x: (A & B) | (C & D);");
}

#[test]
fn test_parse_type_array() {
    expect_parse_ok("let x: number[];");
}

#[test]
fn test_parse_type_array_generic() {
    expect_parse_ok("let x: Array<number>;");
}

#[test]
fn test_parse_type_readonly_array() {
    expect_parse_ok("let x: readonly number[];");
}

#[test]
fn test_parse_type_tuple() {
    expect_parse_ok("let x: [string, number, boolean];");
}

#[test]
fn test_parse_type_tuple_named() {
    expect_parse_ok("let x: [name: string, age: number];");
}

#[test]
fn test_parse_type_tuple_rest() {
    expect_parse_ok("let x: [string, ...number[]];");
}

#[test]
fn test_parse_type_tuple_optional() {
    expect_parse_ok("let x: [string, number?];");
}

// ── Object types ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_type_object_literal() {
    expect_parse_ok("let x: { name: string; age: number };");
}

#[test]
fn test_parse_type_object_optional() {
    expect_parse_ok("let x: { name: string; age?: number };");
}

#[test]
fn test_parse_type_object_readonly() {
    expect_parse_ok("let x: { readonly id: number };");
}

#[test]
fn test_parse_type_object_method() {
    expect_parse_ok("let x: { greet(name: string): string };");
}

#[test]
fn test_parse_type_object_call_signature() {
    expect_parse_ok("let x: { (arg: string): number };");
}

#[test]
fn test_parse_type_object_index_signature() {
    expect_parse_ok("let x: { [key: string]: number };");
}

#[test]
fn test_parse_type_object_construct_signature() {
    expect_parse_ok("let x: { new(arg: string): Foo };");
}

// ── Function types ────────────────────────────────────────────────────────────

#[test]
fn test_parse_type_function() {
    expect_parse_ok("let f: (x: number) => string;");
}

#[test]
fn test_parse_type_function_no_params() {
    expect_parse_ok("let f: () => void;");
}

#[test]
fn test_parse_type_function_optional_param() {
    expect_parse_ok("let f: (x: number, y?: string) => void;");
}

#[test]
fn test_parse_type_function_rest_param() {
    expect_parse_ok("let f: (...args: number[]) => void;");
}

#[test]
fn test_parse_type_function_generic() {
    expect_parse_ok("let f: <T>(x: T) => T;");
}

#[test]
fn test_parse_type_constructor() {
    expect_parse_ok("let C: new(x: string) => Foo;");
}

// ── Advanced types ────────────────────────────────────────────────────────────

#[test]
fn test_parse_type_conditional() {
    expect_parse_ok("type A = T extends string ? 'yes' : 'no';");
}

#[test]
fn test_parse_type_conditional_nested() {
    expect_parse_ok("type A = T extends string ? 'str' : T extends number ? 'num' : 'other';");
}

#[test]
fn test_parse_type_infer() {
    expect_parse_ok("type Ret<T> = T extends (...args: any[]) => infer R ? R : never;");
}

#[test]
fn test_parse_type_mapped() {
    expect_parse_ok("type R<T> = { [K in keyof T]: T[K] };");
}

#[test]
fn test_parse_type_mapped_optional_add() {
    expect_parse_ok("type R<T> = { [K in keyof T]+?: T[K] };");
}

#[test]
fn test_parse_type_mapped_optional_remove() {
    expect_parse_ok("type R<T> = { [K in keyof T]-?: T[K] };");
}

#[test]
fn test_parse_type_mapped_readonly_add() {
    expect_parse_ok("type R<T> = { +readonly [K in keyof T]: T[K] };");
}

#[test]
fn test_parse_type_mapped_readonly_remove() {
    expect_parse_ok("type R<T> = { -readonly [K in keyof T]: T[K] };");
}

#[test]
fn test_parse_type_indexed_access() {
    expect_parse_ok("type V = T[K];");
}

#[test]
fn test_parse_type_indexed_access_keyof() {
    expect_parse_ok("type V<T> = T[keyof T];");
}

#[test]
fn test_parse_type_template_literal() {
    expect_parse_ok(r#"type E = `on${string}`;"#);
}

#[test]
fn test_parse_type_template_literal_union() {
    expect_parse_ok(r#"type E = `on${'click' | 'focus' | 'blur'}`;"#);
}

#[test]
fn test_parse_type_typeof() {
    expect_parse_ok("let x: typeof obj;");
}

#[test]
fn test_parse_type_keyof() {
    expect_parse_ok("let x: keyof T;");
}

#[test]
fn test_parse_type_unique_symbol() {
    expect_parse_ok("const sym: unique symbol = Symbol();");
}

#[test]
fn test_parse_type_predicate() {
    expect_parse_ok("function isString(x: unknown): x is string { return typeof x === 'string'; }");
}

#[test]
fn test_parse_type_asserts_predicate() {
    expect_parse_ok("function assert(x: unknown): asserts x is string { if (typeof x !== 'string') throw new Error(); }");
}

// ── Declarations with types ───────────────────────────────────────────────────

#[test]
fn test_parse_interface_simple() {
    expect_parse_ok("interface Animal { name: string; speak(): void; }");
}

#[test]
fn test_parse_interface_extends() {
    expect_parse_ok("interface Dog extends Animal { breed: string; }");
}

#[test]
fn test_parse_interface_generic() {
    expect_parse_ok("interface Box<T> { value: T; }");
}

#[test]
fn test_parse_type_alias() {
    expect_parse_ok("type Point = { x: number; y: number };");
}

#[test]
fn test_parse_type_alias_generic() {
    expect_parse_ok("type Maybe<T> = T | null | undefined;");
}

#[test]
fn test_parse_enum() {
    expect_parse_ok("enum Direction { Up, Down, Left, Right }");
}

#[test]
fn test_parse_enum_string_values() {
    expect_parse_ok(r#"enum Color { Red = 'R', Green = 'G', Blue = 'B' }"#);
}

#[test]
fn test_parse_enum_numeric_values() {
    expect_parse_ok("enum Status { Active = 1, Inactive = 2, Pending = 4 }");
}

#[test]
fn test_parse_const_enum() {
    expect_parse_ok("const enum Direction { Up = 0, Down = 1 }");
}

#[test]
fn test_parse_namespace() {
    expect_parse_ok("namespace Utils { export function clamp(x: number, min: number, max: number): number { return Math.max(min, Math.min(max, x)); } }");
}

#[test]
fn test_parse_ambient_declare() {
    expect_parse_ok("declare const x: string;");
}

#[test]
fn test_parse_ambient_module() {
    expect_parse_ok("declare module '*.svg' { const content: string; export default content; }");
}

// ── Decorators ────────────────────────────────────────────────────────────────

#[test]
fn test_parse_class_decorator() {
    expect_parse_ok("@Injectable class Service {}");
}

#[test]
fn test_parse_class_decorator_with_args() {
    expect_parse_ok("@Injectable({ singleton: true }) class Service {}");
}

#[test]
fn test_parse_method_decorator() {
    expect_parse_ok("class Foo { @Log greet() {} }");
}

#[test]
fn test_parse_parameter_decorator() {
    expect_parse_ok("class Foo { greet(@Param name: string) {} }");
}

// ── Imports / Exports ─────────────────────────────────────────────────────────

#[test]
fn test_parse_import_named() {
    expect_parse_ok("import { foo, bar } from './module';");
}

#[test]
fn test_parse_import_default() {
    expect_parse_ok("import Foo from './foo';");
}

#[test]
fn test_parse_import_namespace() {
    expect_parse_ok("import * as ns from './ns';");
}

#[test]
fn test_parse_import_type() {
    expect_parse_ok("import type { Foo } from './types';");
}

#[test]
fn test_parse_import_side_effect() {
    expect_parse_ok("import './styles.css';");
}

#[test]
fn test_parse_dynamic_import() {
    expect_parse_ok("const m = await import('./module');");
}

#[test]
fn test_parse_export_named() {
    expect_parse_ok("export { foo, bar };");
}

#[test]
fn test_parse_export_default() {
    expect_parse_ok("export default function f() {}");
}

#[test]
fn test_parse_export_type() {
    expect_parse_ok("export type { Foo };");
}

#[test]
fn test_parse_re_export() {
    expect_parse_ok("export { foo } from './foo';");
}

#[test]
fn test_parse_re_export_all() {
    expect_parse_ok("export * from './module';");
}

// ── JSX ───────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_jsx_self_closing() {
    expect_parse_ok("const el = <View />;");
}

#[test]
fn test_parse_jsx_with_children() {
    expect_parse_ok("const el = <View><Text>hello</Text></View>;");
}

#[test]
fn test_parse_jsx_with_props() {
    expect_parse_ok(r#"const el = <Button onPress={() => {}} style={{ flex: 1 }}>Click</Button>;"#);
}

#[test]
fn test_parse_jsx_fragment() {
    expect_parse_ok("const el = <><View /><View /></>;");
}

#[test]
fn test_parse_jsx_expression() {
    expect_parse_ok("const el = <Text>{message}</Text>;");
}
