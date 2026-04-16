# TASKS.md — Ordered Implementation Tasks

> Work tasks in order. Do not skip ahead. Each task has an acceptance test — the task is not done until the test passes.
> Reference SPECS.md for all data models and FR details. Reference AGENTS.md for all workflow rules.

---

## Phase 1 — Interpreter

### TASK-001 — Workspace scaffold
**FR:** none (setup)
**Deliverable:** The repository compiles with `cargo build` and all crates exist.

Create:
- `Cargo.toml` (workspace, see §1 of SPECS.md for exact content)
- `crates/tsnat-lex/`, `crates/tsnat-parse/`, `crates/tsnat-types/`, `crates/tsnat-ir/`
- `crates/tsnat-eval/`, `crates/tsnat-ffi/`, `crates/tsnat-react/`, `crates/tsnat-codegen/`, `crates/tsnat-cli/`
- Each crate has a minimal `Cargo.toml` and `src/lib.rs` (or `src/main.rs` for `tsnat-cli`).
- `lib/lib.d.ts` — empty for now, placeholder only.
- `tests/phase1/`, `tests/phase2/`, `tests/phase3/`, `tests/phase4/`, `tests/phase5/` — each with a `.gitkeep`.

**Acceptance test:** `cargo build --workspace` exits 0. `cargo test --workspace` exits 0 (no tests yet, just compiles).

---

### TASK-002 — SourceMap and Span
**FR:** FR-LEX-002 (span), cross-cutting §2.1
**Crate:** `tsnat-parse`

Implement `crates/tsnat-parse/src/span.rs` exactly as specified in §2.1 of SPECS.md.
Implement `SourceMap` with methods:
- `fn add_file(&mut self, path: PathBuf, content: String) -> u32` — returns file_id
- `fn get_file(&self, id: u32) -> &SourceFile`
- `fn line_col(&self, span: Span) -> (u32, u32)` — 1-based line and column

**Acceptance test:** `tests/phase1/span_test.rs`
```rust
let mut sm = SourceMap::new();
let id = sm.add_file("test.ts".into(), "hello\nworld".to_string());
let span = Span { file_id: id, start: 6, end: 11 };
let (line, col) = sm.line_col(span);
assert_eq!(line, 2);
assert_eq!(col, 1);
```

---

### TASK-003 — String interner
**FR:** §2.3
**Crate:** `tsnat-parse`

Implement `crates/tsnat-parse/src/interner.rs` exactly as specified in §2.3.
Pre-intern the following symbols at index 0–N and expose them as constants:
`SYM_EMPTY`, `SYM_CONSTRUCTOR`, `SYM_PROTOTYPE`, `SYM_LENGTH`, `SYM_UNDEFINED`,
`SYM_NULL`, `SYM_NUMBER`, `SYM_STRING`, `SYM_BOOLEAN`, `SYM_OBJECT`, `SYM_FUNCTION`,
`SYM_SYMBOL`, `SYM_BIGINT`.

**Acceptance test:**
```rust
let mut interner = Interner::new();
let a = interner.intern("hello");
let b = interner.intern("hello");
let c = interner.intern("world");
assert_eq!(a, b);
assert_ne!(a, c);
assert_eq!(interner.get(a), "hello");
```

---

### TASK-004 — Diagnostics
**FR:** §2.2
**Crate:** `tsnat-parse`

Implement `crates/tsnat-parse/src/diagnostic.rs` exactly as specified in §2.2.
Add `miette = { version = "7", features = ["fancy"] }` and `thiserror = "2"` to the crate's `Cargo.toml`.

The `TsnatError` enum must implement `miette::Diagnostic`. Each variant must carry a `#[label]` span for source annotation.

**Acceptance test:** Constructing each error variant compiles. `format!("{}", err)` produces a non-empty string for each variant.

---

### TASK-005 — Lexer: token kinds and struct
**FR:** FR-LEX-001, FR-LEX-002
**Crate:** `tsnat-lex`

Implement `crates/tsnat-lex/src/token.rs` with the full `TokenKind` enum and `Token` struct exactly as specified in FR-LEX-001 and FR-LEX-002.

`TokenKind` must implement `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Display`.

**Acceptance test:** All 100+ variants compile. `TokenKind::KwConst.to_string() == "const"`. `TokenKind::Eof.to_string() == "EOF"`.

---

### TASK-006 — Lexer: core implementation
**FR:** FR-LEX-001 through FR-LEX-006
**Crate:** `tsnat-lex`

Implement `crates/tsnat-lex/src/lexer.rs`:

```rust
pub struct Lexer<'src> {
    source: &'src str,
    file_id: u32,
    pos: u32,
    mode_stack: Vec<LexMode>,
    interner: &'src mut Interner,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, file_id: u32, interner: &'src mut Interner) -> Self;
    pub fn next_token(&mut self) -> TsnatResult<Token>;
    pub fn tokenise_all(&mut self) -> TsnatResult<Vec<Token>>;
}
```

The lexer must handle all FR-LEX-001 token kinds. Implement in this order:
1. Whitespace and comment skipping (set `has_preceding_newline`)
2. Single-character punctuation and operators
3. Multi-character operators (`===`, `!==`, `**=`, `??=`, `?.`, `...`, `=>`)
4. Keywords (scan identifier, then check keyword table)
5. Number literals (decimal, hex, binary, octal, bigint, separators)
6. String literals (single and double quote, with escape sequences per FR-LEX-006)
7. Template literals (with mode stack push/pop per FR-LEX-004)
8. Regex literals (context-sensitive: only valid after `=`, `(`, `,`, `[`, `!`, `&`, `|`, `?`, `:`, `{`, `}`, `;`, start-of-file)

**Acceptance test:** `tests/phase1/lex_smoke.rs` — tokenise each of these strings and assert the expected token sequences:
```
"42"             → [Number("42"), Eof]
"'hello'"        → [String("hello"), Eof]
"`a${x}b`"       → [TemplateHead("a"), Ident("x"), TemplateTail("b"), Eof]
"const x = 1;"   → [KwConst, Ident("x"), Eq, Number("1"), Semicolon, Eof]
"x?.y"           → [Ident("x"), QuestionDot, Ident("y"), Eof]
"x ?? y"         → [Ident("x"), QuestionQuestion, Ident("y"), Eof]
"x **= 2"        → [Ident("x"), StarStarEq, Number("2"), Eof]
```

---

### TASK-007 — Parser: program and variable declarations
**FR:** FR-PAR-001, FR-PAR-002, FR-PAR-003, FR-PAR-004 (VarDecl only)
**Crate:** `tsnat-parse`

Implement the parser struct and parse `Program`, `VarDecl` (`const`, `let`, `var`), and `ExprStmt` for simple expressions (identifiers, number literals, binary operators, assignment).

```rust
pub struct Parser<'src, 'arena> {
    tokens: &'src [Token],
    pos: usize,
    arena: &'arena bumpalo::Bump,
    interner: &'src mut Interner,
    errors: Vec<TsnatError>,
}

impl<'src, 'arena> Parser<'src, 'arena> {
    pub fn new(...) -> Self;
    pub fn parse_program(&mut self) -> Program<'arena>;
    fn peek(&self) -> &Token;
    fn peek_ahead(&self, n: usize) -> &Token;
    fn advance(&mut self) -> &Token;
    fn expect(&mut self, kind: TokenKind) -> TsnatResult<&Token>;
    fn sync(&mut self); // error recovery: advance to sync point
}
```

**Acceptance test:** Parse `"const x = 42;"` → `Program { stmts: [Stmt::Var(VarDecl { kind: Const, name: "x", init: Some(Expr::Number(42.0)) })] }`.

---

### TASK-008 — Parser: full expression set
**FR:** FR-PAR-005, FR-PAR-008
**Crate:** `tsnat-parse`

Extend the parser to handle all expression types from FR-PAR-005. Implement the precedence climbing algorithm from FR-PAR-008. This task is large — implement and test each expression type before moving to the next:

Order of implementation:
1. Literals (number, string, bool, null, undefined, this, regex)
2. Identifiers
3. Parenthesised expressions
4. Unary operators
5. Binary operators (use the precedence table from FR-PAR-008)
6. Conditional (`? :`)
7. Assignment and compound assignment
8. Member access (`a.b`, `a[b]`, `a?.b`)
9. Call expressions (`f(a, b)`)
10. New expressions
11. Arrow functions
12. Function expressions
13. Template literals
14. Spread
15. Await / yield
16. `as`, `satisfies`, type assertions `<T>expr`

**Acceptance test:** Each of these must parse without error:
```typescript
a + b * c - d
a ? b : c
a.b.c
a?.b?.c
a[b]
f(1, 2, ...rest)
new Foo(a, b)
(x: number) => x * 2
async (x) => await fetch(x)
`hello ${name}!`
x as string
<string>x
```

---

### TASK-009 — Parser: statements
**FR:** FR-PAR-004 (full)
**Crate:** `tsnat-parse`

Implement all statement types from FR-PAR-004.

Order:
1. Block `{}`
2. `if`/`else`
3. `while`, `do`/`while`
4. `for`, `for...in`, `for...of`
5. `return`, `throw`, `break`, `continue`
6. `try`/`catch`/`finally`
7. `switch`/`case`
8. Labeled statement
9. `function` declaration
10. `class` declaration (basic — fields, methods, constructor, extends)
11. `import`/`export`

**Acceptance test:** Parse this snippet without errors:
```typescript
import { foo } from './foo';
class Counter {
    private count: number = 0;
    increment(): void { this.count++; }
    get value() { return this.count; }
}
async function run() {
    for (const x of [1, 2, 3]) {
        try {
            await foo(x);
        } catch (e) {
            console.error(e);
        }
    }
}
export { run };
```

---

### TASK-010 — Parser: type annotations
**FR:** FR-PAR-006
**Crate:** `tsnat-parse`

Implement all type nodes from FR-PAR-006.

Order:
1. Primitive type keywords
2. Literal types
3. `TypeRef` (identifier + optional type arguments `<A, B>`)
4. Array type `T[]`
5. Tuple type `[A, B]`
6. Object type `{ x: T; y?: U }`
7. Function type `(x: T) => U`
8. Union `A | B` and intersection `A & B`
9. Conditional type `T extends U ? X : Y`
10. `infer T`
11. Mapped type `{ [K in keyof T]: V }`
12. Indexed access `T[K]`
13. Template literal type `` `${T}` ``
14. `typeof`, `keyof`, `unique symbol`
15. Type predicate `x is T`, `asserts x is T`

**Acceptance test:** Parse without error:
```typescript
type A = string | number | null;
type B = { x: number; y?: string; readonly z: boolean };
type C<T> = T extends string ? 'yes' : 'no';
type D<T> = { [K in keyof T]: T[K] | null };
type E = `on${Capitalize<string>}`;
type F<T> = T[keyof T];
```

---

### TASK-011 — Parser: interfaces, type aliases, enums, namespaces
**FR:** FR-PAR-004 (interface, type alias, enum, namespace)
**Crate:** `tsnat-parse`

Implement:
- `interface Foo extends Bar { ... }`
- `type Alias<T> = ...`
- `enum Direction { Up, Down, Left = 'L', Right = 'R' }`
- `namespace Foo { export const x = 1; }`
- `declare` ambient declarations

**Acceptance test:** Parse without error:
```typescript
interface Animal { name: string; speak(): void }
interface Dog extends Animal { breed: string }
enum Color { Red = 0, Green = 1, Blue = 2 }
namespace Utils {
    export function clamp(x: number, min: number, max: number): number {
        return Math.max(min, Math.min(max, x));
    }
}
declare module '*.svg' { const content: string; export default content; }
```

---

### TASK-012 — Parser: decorators and JSX
**FR:** FR-PAR-005 (JSX), FR-PAR-005 (decorators)
**Crate:** `tsnat-parse`

Implement:
- TC39 decorators: `@Decorator class Foo {}`, `@decorator method() {}`
- JSX: `<View>`, `<View />`, `<View key={expr}>`, `<>fragments</>`, `{expression}`

Enable JSX via a parser flag `--jsx` passed to `Parser::new`.

**Acceptance test:**
```typescript
@Injectable({ singleton: true })
class Service {
    @Log
    greet(@Param name: string): string { return `hello ${name}`; }
}
```
```tsx
const el = (
    <View style={{ flex: 1 }}>
        <Text>{message}</Text>
        <Button onPress={() => setCount(c => c + 1)}>
            <Text>+1</Text>
        </Button>
    </View>
);
```

---

### TASK-013 — Interpreter: evaluator core
**FR:** FR-EVAL-001, FR-EVAL-002
**Crate:** `tsnat-eval`

Implement the `Value` enum, `JsObject`, `JsFunction`, `Environment`, and `Binding` types exactly as specified in FR-EVAL-001 and FR-EVAL-002.

Implement the core evaluator:
```rust
pub struct Interpreter {
    pub global: Rc<RefCell<Environment>>,
    pub call_stack: Vec<CallFrame>,
    pub event_loop: EventLoop,
    pub interner: Interner,
    pub source_map: SourceMap,
}

impl Interpreter {
    pub fn new() -> Self;
    pub fn eval_program(&mut self, program: &Program) -> EvalResult<Value>;
    fn eval_stmt(&mut self, stmt: &Stmt, env: EnvRef) -> EvalResult<StmtResult>;
    fn eval_expr(&mut self, expr: &Expr, env: EnvRef) -> EvalResult<Value>;
}
```

Implement `eval_expr` for: literals, identifiers, binary operators, unary operators, assignment, member access, call expressions, `new` expressions.

Implement `eval_stmt` for: `ExprStmt`, `VarDecl`, `BlockStmt`, `IfStmt`, `ReturnStmt`.

**Acceptance test:**
```typescript
const x = 1 + 2 * 3;   // x === 7
const s = "hello" + " " + "world";  // s === "hello world"
function add(a, b) { return a + b; }
const r = add(3, 4);  // r === 7
```

---

### TASK-014 — Interpreter: control flow
**FR:** FR-EVAL-001 (all stmt types)
**Crate:** `tsnat-eval`

Extend `eval_stmt` for all remaining statement types: `WhileStmt`, `DoWhileStmt`, `ForStmt`, `ForInStmt`, `ForOfStmt`, `SwitchStmt`, `ThrowStmt`, `TryStmt`, `BreakStmt`, `ContinueStmt`, `LabeledStmt`.

Implement `BreakSignal`, `ContinueSignal`, `ReturnSignal`, `ThrowSignal` as `StmtResult` variants (not Rust panics or `Result::Err`).

**Acceptance test:**
```typescript
let sum = 0;
for (let i = 0; i < 10; i++) { sum += i; }
// sum === 45

let found = null;
outer: for (const x of [1,2,3]) {
    for (const y of [4,5,6]) {
        if (x + y === 7) { found = [x, y]; break outer; }
    }
}
// found === [1, 6]

function div(a, b) {
    if (b === 0) throw new Error("div by zero");
    return a / b;
}
let caught = null;
try { div(1, 0); } catch(e) { caught = e.message; }
// caught === "div by zero"
```

---

### TASK-015 — Interpreter: built-in globals
**FR:** FR-EVAL-003
**Crate:** `tsnat-eval`

Implement all globals listed in FR-EVAL-003. Each global object must expose the correct methods as `Value::NativeFunction`.

Implement in this order:
1. `console.log` (writes to stdout), `console.error` (stderr), `console.warn`, `console.assert`
2. `Math` (all methods)
3. `JSON.stringify` and `JSON.parse`
4. `Array` (constructor + `Array.from`, `Array.isArray`, prototype methods: `push`, `pop`, `shift`, `unshift`, `map`, `filter`, `reduce`, `forEach`, `find`, `findIndex`, `includes`, `indexOf`, `slice`, `splice`, `join`, `sort`, `reverse`, `flat`, `flatMap`, `some`, `every`, `concat`)
5. `Object` (constructor + `keys`, `values`, `entries`, `assign`, `create`, `freeze`, `isFrozen`, `hasOwn`)
6. `String` prototype methods (`split`, `trim`, `trimStart`, `trimEnd`, `startsWith`, `endsWith`, `includes`, `indexOf`, `lastIndexOf`, `slice`, `substring`, `replace`, `replaceAll`, `match`, `matchAll`, `padStart`, `padEnd`, `repeat`, `charAt`, `charCodeAt`, `codePointAt`, `toLowerCase`, `toUpperCase`, `at`)
7. `Number` (constructor + `isInteger`, `isFinite`, `isNaN`, `parseInt`, `parseFloat`, prototype: `toFixed`, `toString`)
8. `Map`, `Set`, `WeakMap`, `WeakSet`
9. `Symbol`
10. `Error`, `TypeError`, `RangeError`, `ReferenceError`, `SyntaxError`
11. `Date` (`Date.now()`, `new Date()`, prototype methods)
12. `RegExp` (constructor, `test`, `exec`, `match`)

**Acceptance test:** `tests/phase1/builtins.rs` — test each global with its common usage patterns. Specifically:
```typescript
const arr = [3, 1, 4, 1, 5, 9];
arr.sort((a, b) => a - b);
// arr === [1, 1, 3, 4, 5, 9]

const m = new Map();
m.set('a', 1); m.set('b', 2);
// m.get('a') === 1, m.size === 2

const json = JSON.stringify({ x: 1, y: [2, 3] });
const parsed = JSON.parse(json);
// parsed.x === 1, parsed.y[1] === 3
```

---

### TASK-016 — Interpreter: functions, closures, classes
**FR:** FR-EVAL-001 (JsFunction), FR-EVAL-002, FR-EVAL-005
**Crate:** `tsnat-eval`

Implement:
- Function declaration and function expression evaluation
- Arrow functions (no `this` rebinding)
- Closures (correct capture of lexical environment)
- `class` declaration: constructor, methods, getters/setters, `static` members, `extends` and `super()`
- Prototype chain lookup (FR-EVAL-005)
- `this` binding: method calls bind `this` to the receiver; plain function calls bind `this` to `undefined` (strict mode)
- `instanceof` operator

**Acceptance test:**
```typescript
class Animal {
    constructor(public name: string) {}
    speak() { return `${this.name} makes a noise.`; }
}
class Dog extends Animal {
    speak() { return `${this.name} barks.`; }
}
const d = new Dog('Rex');
// d.speak() === "Rex barks."
// d instanceof Dog === true
// d instanceof Animal === true

function makeCounter() {
    let count = 0;
    return {
        inc: () => ++count,
        get: () => count,
    };
}
const c = makeCounter();
c.inc(); c.inc();
// c.get() === 2
```

---

### TASK-017 — Interpreter: async/await and Promises
**FR:** FR-EVAL-004
**Crate:** `tsnat-eval`

Implement:
- `Promise` constructor, `.then()`, `.catch()`, `.finally()`
- `Promise.resolve()`, `Promise.reject()`, `Promise.all()`, `Promise.allSettled()`, `Promise.any()`, `Promise.race()`
- `async function` evaluation (returns a Promise)
- `await` expression (suspends and resumes)
- Microtask queue (drain after each top-level task)
- `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`

**Acceptance test:**
```typescript
async function delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}
async function run(): Promise<string> {
    await delay(0);
    const results = await Promise.all([
        Promise.resolve(1),
        Promise.resolve(2),
        Promise.resolve(3),
    ]);
    return results.join(',');
}
// run() resolves to "1,2,3"
```

---

### TASK-018 — Interpreter: generators
**FR:** FR-EVAL-001
**Crate:** `tsnat-eval`

Implement `function*` generators, `yield`, `yield*`, and the iterator protocol (`Symbol.iterator`, `next()`, `return()`, `throw()`).

`for...of` must work with:
- Arrays
- Strings (iterates code points)
- Map / Set
- Custom iterators
- Generators

**Acceptance test:**
```typescript
function* range(start: number, end: number) {
    for (let i = start; i < end; i++) yield i;
}
const nums = [...range(0, 5)];
// nums === [0, 1, 2, 3, 4]

function* fibonacci() {
    let [a, b] = [0, 1];
    while (true) { yield a; [a, b] = [b, a + b]; }
}
const fib = fibonacci();
const first6 = Array.from({ length: 6 }, () => fib.next().value);
// first6 === [0, 1, 1, 2, 3, 5]
```

---

### TASK-019 — Module loader
**FR:** FR-EVAL-006
**Crate:** `tsnat-eval`

Implement the module loader. ESM only. Modules are parsed, type-checked (if type checker is available), and evaluated on first import. Results are cached by resolved path.

Implement a `FileSystemResolver` that resolves:
- Relative paths (`./foo`, `../bar`)
- Extension resolution: try `.ts`, then `.tsx`, then `/index.ts`

**Acceptance test:** Two-file test in `tests/phase1/module_loader.rs`:
```
// lib.ts
export function add(a: number, b: number): number { return a + b; }
export const PI = 3.14159;

// main.ts
import { add, PI } from './lib';
console.log(add(1, 2));    // 3
console.log(PI);            // 3.14159
```

---

### TASK-020 — CLI: `tsnat run` and `tsnat repl`
**FR:** FR-CLI-001
**Crate:** `tsnat-cli`

Implement `tsnat run <file.ts>`:
1. Read the source file.
2. Lex → parse → eval (skip type checking in Phase 1 — that's Phase 3).
3. Print any diagnostics to stderr using `miette` formatting.
4. Exit 0 on success, 1 on runtime error.

Implement `tsnat repl`:
1. Print a prompt (`> `).
2. Read a line.
3. Lex → parse → eval.
4. Print the result value (if not `undefined`) using the `display_value` formatter.
5. Repeat.

`display_value` rules:
- `undefined` → print nothing
- `null` → `null`
- `number` → use `{:?}` for NaN/Infinity, otherwise trim trailing `.0`
- `string` → print without quotes (like Node.js REPL)
- `boolean` → `true` / `false`
- `object` → `{ key: value, ... }` (2 levels deep max, circular = `[Circular]`)
- `array` → `[ items, ... ]`
- `function` → `[Function: name]`

**Phase 1 exit test:** `tests/phase1/repl_smoke.rs`
```
Input:  const x: number = 42;\nconst greet = (name: string): string => `Hello, ${name}!`;\ngreet("world")
Output: Hello, world!
```
This test must pass. It is the Phase 1 exit gate.

---

## Phase 2 — Native FFI

### TASK-021 — FFI type mapping and NativePtr
**FR:** FR-FFI-001, FR-FFI-005
**Crate:** `tsnat-ffi`

Implement the `NativePtr` value type and the marshalling functions:
```rust
pub fn ts_to_c(val: &Value, expected: FfiType) -> TsnatResult<CValue>;
pub fn c_to_ts(val: CValue, ty: FfiType) -> TsnatResult<Value>;
```

`FfiType` is an enum of all types in the FR-FFI-001 mapping table.

---

### TASK-022 — Dynamic library loading
**FR:** FR-FFI-004
**Crate:** `tsnat-ffi`

Implement `DynLib`:
```rust
pub struct DynLib {
    handle: *mut std::ffi::c_void, // from dlopen
    symbols: FxHashMap<Symbol, *mut std::ffi::c_void>,
}

impl DynLib {
    pub fn open(path: &Path) -> TsnatResult<Self>;
    pub fn symbol(&mut self, name: &str) -> TsnatResult<*mut std::ffi::c_void>;
    pub fn call(&self, symbol: Symbol, args: Vec<CValue>, ret_type: FfiType) -> TsnatResult<CValue>;
}
```

Use `libloading` crate for platform-portable `dlopen`/`dlsym`.

---

### TASK-023 — `declare native function` import
**FR:** FR-FFI-002
**Crates:** `tsnat-parse` (AST), `tsnat-eval` (resolution)

Parse `declare native function name(params): ReturnType` as an `AmbientDecl::NativeFn` AST node.
In the interpreter, when binding this declaration, look up the symbol in loaded native libraries and wrap it as `Value::NativeFunction`.

---

### TASK-024 — `import native` statement
**FR:** FR-FFI-004
**Crates:** `tsnat-parse`, `tsnat-eval`

Parse `import native LibName from 'library-path'` as a new `ImportDecl::Native` AST variant.
The evaluator calls `DynLib::open` on the resolved library path and registers it in the module's namespace object.

**Phase 2 exit test:** `tests/phase2/ffi_roundtrip.rs`
A test that loads a small C shared library (`tests/fixtures/add.c`, compiled to `libadd.so`), calls `add(3, 4)` from TypeScript via `declare native function`, and asserts the result is `7`.

---

## Phase 3 — Type Checker

### TASK-025 — Type arena and primitive types
**FR:** FR-TYP-001
**Crate:** `tsnat-types`

Implement `TypeArena`, `TypeId`, and the `Type` enum from FR-TYP-001. Intern the primitive types so `TypeArena::number()` always returns the same `TypeId`.

---

### TASK-026 — Type checker: declarations
**FR:** FR-TYP-001
**Crate:** `tsnat-types`

Implement the first pass: collect all declarations in a `TypeEnv` (type environment). This pass runs before expression type checking. After this pass, every `interface`, `type`, `class`, `function`, and `variable` declaration has a `TypeId` entry in the env.

---

### TASK-027 — Type checker: expression types
**FR:** FR-TYP-002, FR-TYP-003
**Crate:** `tsnat-types`

Implement `check_expr(expr, expected: Option<TypeId>, env) -> TypeId` for all expression types. Implement `is_assignable` (FR-TYP-002). Implement bidirectional inference (FR-TYP-003).

---

### TASK-028 — Type checker: control flow narrowing
**FR:** FR-TYP-004
**Crate:** `tsnat-types`

Implement the `FlowNode` graph and narrowing for all triggers listed in FR-TYP-004.

---

### TASK-029 — Type checker: generics
**FR:** FR-TYP-005
**Crate:** `tsnat-types`

Implement type parameter substitution and the type argument inference algorithm.

---

### TASK-030 — Type checker: conditional types
**FR:** FR-TYP-006
**Crate:** `tsnat-types`

Implement conditional type evaluation, deferral for type parameters, and union distribution.

---

### TASK-031 — Type checker: mapped types and template literals
**FR:** FR-TYP-007, FR-TYP-008
**Crate:** `tsnat-types`

Implement mapped type evaluation and template literal type resolution, including intrinsic string manipulation types.

---

### TASK-032 — lib.d.ts declarations
**FR:** FR-LIB-001
**Files:** `lib/lib.d.ts`

Write the full built-in declaration file. Every global from FR-EVAL-003 must be declared with correct TypeScript types. All utility types from FR-TYP-009 must be defined. React types and TSNAT renderer types are added in TASK-040.

---

### TASK-033 — CLI: `tsnat check`
**FR:** FR-CLI-001
**Crate:** `tsnat-cli`

Implement `tsnat check <file.ts>`: lex → parse → type-check → report errors. Exit 1 if any type errors. Exit 0 if clean.

**Phase 3 exit test:** `tests/phase3/type_errors.rs`
A file with known type errors:
```typescript
const x: number = "hello";          // TS2322
function f(a: number): string { return a; } // TS2322
const y = { a: 1 };
const z = y.nonexistent;            // TS2339
```
`tsnat check` must emit all three errors with correct line numbers and exit 1.
A clean file must exit 0.

---

## Phase 4 — React Renderer

### TASK-034 — SDL3 window creation
**FR:** FR-REACT-003, FR-REACT-005
**Crate:** `tsnat-react`

Add `sdl3-sys` to `Cargo.toml`. Implement:
```rust
pub struct Window {
    sdl_window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    width: u32,
    height: u32,
}

impl Window {
    pub fn create(title: &str, width: u32, height: u32) -> TsnatResult<Self>;
    pub fn poll_events(&mut self) -> Vec<NativeEvent>;
    pub fn clear(&mut self);
    pub fn present(&mut self);
    pub fn destroy(self);
}
```

Verify a blank window opens, stays open for 2 seconds, and closes cleanly.

---

### TASK-035 — Yoga layout integration
**FR:** FR-REACT-004
**Crate:** `tsnat-react`

Add `yoga-sys` to `Cargo.toml`. Implement `LayoutTree`:
```rust
pub struct LayoutTree {
    root: YGNodeRef,
    nodes: FxHashMap<WidgetId, YGNodeRef>,
}

impl LayoutTree {
    pub fn calculate(&mut self, width: f32, height: f32);
    pub fn get_layout(&self, id: WidgetId) -> LayoutRect;
}
```

---

### TASK-036 — Text rendering with FreeType
**FR:** FR-REACT-008
**Crate:** `tsnat-react`

Add `freetype-rs` to `Cargo.toml`. Implement a glyph atlas:
```rust
pub struct GlyphAtlas {
    texture: *mut SDL_Texture,
    glyphs: FxHashMap<(char, u32 /* size */), GlyphMetrics>,
}

impl GlyphAtlas {
    pub fn load_font(path: &Path, sizes: &[u32]) -> TsnatResult<Self>;
    pub fn render_text(&self, renderer: *mut SDL_Renderer, text: &str, x: f32, y: f32, size: u32, color: Color);
}
```

---

### TASK-037 — Widget render pipeline
**FR:** FR-REACT-003, FR-REACT-004
**Crate:** `tsnat-react`

Implement the widget tree and render pass. Each `WidgetKind` (View, Text, Input, Button, Image, ScrollView) maps to a `render(renderer, layout)` function.

---

### TASK-038 — Host config FFI bridge
**FR:** FR-REACT-002
**Crate:** `tsnat-react`

Implement the host config Rust functions that the React reconciler will call via FFI. Export them as C-callable functions with `@NativeExport` equivalents. The JS reconciler receives a host config object whose methods are backed by these Rust functions.

---

### TASK-039 — Bundle React 19 and react-reconciler
**FR:** FR-REACT-001
**Crate:** `tsnat-react`

Bundle the pre-compiled ESM build of `react@19.0.0` and `react-reconciler@0.29.0` as `include_str!()` constants. Register them as virtual modules in the interpreter's module loader at the paths `"react"` and `"react-reconciler"`.

---

### TASK-040 — React types in lib.d.ts and renderApp entry point
**FR:** FR-REACT-006, FR-LIB-001 (React types)
**Files:** `lib/lib.d.ts`, `crates/tsnat-react/src/entry.rs`

Add React 19 type declarations to `lib/lib.d.ts`. Implement the `renderApp` function exposed as `tsnat/react`.

---

### TASK-041 — Event dispatch
**FR:** FR-REACT-005
**Crate:** `tsnat-react`

Translate SDL events to React synthetic events per the mapping table in FR-REACT-005. Dispatch events through the reconciler's event system.

**Phase 4 exit test:** `tests/phase4/render_window.rs`

This program must render a window, show a counter, and respond to a click:
```tsx
import { renderApp } from 'tsnat/react';
import React, { useState } from 'react';

function Counter() {
    const [n, setN] = useState(0);
    return (
        <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
            <Text style={{ fontSize: 32 }}>{n}</Text>
            <Button onPress={() => setN(c => c + 1)}>
                <Text>Increment</Text>
            </Button>
        </View>
    );
}
renderApp(<Counter />, { title: 'Counter', width: 400, height: 300 });
```
The test runner programmatically injects a click event at the button's layout rect and asserts that the Text node updates to "1".

---

## Phase 5 — Native Code Generator

### TASK-042 — IR generation from typed AST
**FR:** FR-IR-001 through FR-IR-003
**Crate:** `tsnat-ir`

Implement the IR lowering pass. Input: `TypedProgram`. Output: `IrModule` with `FnId`-indexed `IrFunction` objects in three-address form. Implement async lowering (FR-IR-002) and closure lowering (FR-IR-003).

---

### TASK-043 — LLVM IR codegen
**FR:** FR-CG-001
**Crate:** `tsnat-codegen`

Add `inkwell = { version = "0.4", features = ["llvm18-0"] }`. Implement `IrModule → inkwell::module::Module`. Each IR instruction maps to LLVM IR as documented in FR-CG-001.

---

### TASK-044 — Boehm GC integration
**FR:** FR-CG-002
**Crate:** `tsnat-codegen`

Add `boehm-gc-sys`. Route all heap allocations through `GC_malloc`. Implement `GcAlloc`, `GcRoot`, `GcUnroot` IR instructions in the LLVM backend.

---

### TASK-045 — Shape system and inline cache
**FR:** FR-CG-003, FR-CG-004
**Crate:** `tsnat-codegen`

Implement shape-indexed struct layouts (FR-CG-003) and IC stubs (FR-CG-004).

---

### TASK-046 — CLI: `tsnat build`
**FR:** FR-CLI-001
**Crate:** `tsnat-cli`

Implement `tsnat build <file.ts> -o <out>`: lex → parse → type-check → IR → LLVM IR → native binary. Link against `libgc` and `libSDL3`.

**Phase 5 exit test:** `tests/phase5/native_binary.rs`
Compile and run the Counter app from TASK-041 as a native binary. Measure: startup time < 100ms. Memory usage < 50MB at idle.

---

## Task Summary

| Phase | Tasks | Gate test |
|---|---|---|
| 1 — Interpreter | TASK-001 → TASK-020 | `tests/phase1/repl_smoke.rs` |
| 2 — FFI | TASK-021 → TASK-024 | `tests/phase2/ffi_roundtrip.rs` |
| 3 — Type Checker | TASK-025 → TASK-033 | `tests/phase3/type_errors.rs` |
| 4 — React Renderer | TASK-034 → TASK-041 | `tests/phase4/render_window.rs` |
| 5 — Codegen | TASK-042 → TASK-046 | `tests/phase5/native_binary.rs` |

**Total: 46 tasks. Each is one focused, testable unit of work.**
