# SPECS.md — TSNAT Functional Requirements

> All decisions are final. There are no TBDs. If something appears to be missing, consult AGENTS.md workflow before assuming.

---

## §0 — Resolved Decisions

Every architectural decision is made here. Agents do not re-decide these.

| Decision | Choice | Reason |
|---|---|---|
| Implementation language | Rust 2024 edition | Memory safety, LLVM bindings, zero-cost abstractions for hot paths |
| LLVM version | 18 (via `inkwell 0.4`) | Stable API, wide platform support |
| Async runtime (tooling) | `tokio 1` | CLI and LSP tooling only — not used in generated runtime |
| Error reporting | `miette 7` | Source-annotated diagnostics with spans |
| AST arena | `bumpalo 3` | All AST nodes arena-allocated per parse session |
| String interning | Custom `Interner` in `tsnat-parse` | Avoids repeated heap allocation for identifiers |
| Hash maps | `rustc-hash` (`FxHashMap`) | 2–3× faster than `std` for small integer/pointer keys |
| Module system | ESM only | No CommonJS. `import`/`export` only |
| Decorator spec | TC39 Stage 3 (TypeScript 5+) | No legacy `experimentalDecorators` |
| React version | 19 | Latest stable; concurrent mode features required |
| Windowing backend (Phase 4) | SDL3 via `sdl3-sys` | Cross-platform, single dependency, GPU surface |
| Layout engine (Phase 4) | Yoga (Flexbox) via `yoga-sys` | Same engine React Native uses; well-tested |
| Font rendering (Phase 4) | FreeType 2 + HarfBuzz via sys crates | Industry standard, permissive license |
| GC (Phase 5) | Conservative Boehm GC via `boehm-gc-sys` | Correct first, fast later |
| Target triple (Phase 5, first) | `x86_64-unknown-linux-gnu` | Simplest ABI; other targets added after this works |

---

## §1 — Repository Structure

The agent must create and maintain exactly this structure:

```
tsnat/
├── Cargo.toml               # Workspace manifest
├── AGENTS.md
├── SPECS.md
├── TASKS.md
├── crates/
│   ├── tsnat-lex/           # FR-LEX-*
│   ├── tsnat-parse/         # FR-PAR-*
│   ├── tsnat-types/         # FR-TYP-*
│   ├── tsnat-ir/            # FR-IR-*
│   ├── tsnat-eval/          # FR-EVAL-*
│   ├── tsnat-ffi/           # FR-FFI-*
│   ├── tsnat-react/         # FR-REACT-*
│   ├── tsnat-codegen/       # FR-CG-*
│   └── tsnat-cli/           # FR-CLI-*
├── lib/
│   └── lib.d.ts             # Built-in TypeScript declarations
└── tests/
    ├── phase1/
    ├── phase2/
    ├── phase3/
    ├── phase4/
    └── phase5/
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.dependencies]
thiserror   = "2"
miette      = { version = "7", features = ["fancy"] }
bumpalo     = "3"
rustc-hash  = "2"
indexmap    = "2"
tokio       = { version = "1", features = ["full"] }
inkwell     = { version = "0.4", features = ["llvm18-0"] }
```

---

## §2 — Cross-Cutting Data Models

These types are defined in `tsnat-parse` and used across all crates.

### 2.1 Source Location

```rust
// crates/tsnat-parse/src/span.rs

/// A half-open byte range [start, end) into a source file.
/// u32 is sufficient for files up to 4 GiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: u32,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const DUMMY: Self = Self { file_id: 0, start: 0, end: 0 };
    pub fn merge(self, other: Self) -> Self { ... }
}

/// Registry of source files. File ID 0 is reserved for built-in declarations.
pub struct SourceMap {
    files: Vec<SourceFile>,
}

pub struct SourceFile {
    pub id: u32,
    pub path: PathBuf,
    pub content: String,
    /// Byte offset of each line start, for line/col computation.
    pub line_starts: Vec<u32>,
}
```

### 2.2 Diagnostics

```rust
// crates/tsnat-parse/src/diagnostic.rs

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TsnatError {
    #[error("Lex error: {message}")]
    Lex { message: String, #[label] span: Span },

    #[error("Parse error: {message}")]
    Parse { message: String, #[label] span: Span },

    #[error("Type error: {message}")]
    Type { message: String, #[label] span: Span, #[help] help: Option<String> },

    #[error("Runtime error: {message}")]
    Runtime { message: String, #[label] span: Option<Span> },

    #[error("FFI error: {message}")]
    Ffi { message: String },
}

pub type TsnatResult<T> = Result<T, TsnatError>;
```

### 2.3 String Interning

```rust
// crates/tsnat-parse/src/interner.rs

/// Interned string. Equality is pointer equality after interning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

pub struct Interner {
    map: FxHashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> Symbol { ... }
    pub fn get(&self, sym: Symbol) -> &str { ... }
}
```

---

## §3 — Lexer (FR-LEX-*)

**Crate:** `tsnat-lex`
**Depends on:** `tsnat-parse` (for `Span`, `TsnatError`, `Symbol`, `Interner`)

### FR-LEX-001 — Token kinds

The `TokenKind` enum must cover every token in the TypeScript grammar. The complete list is:

```rust
// crates/tsnat-lex/src/token.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TokenKind {
    // ── Literals ──────────────────────────────────────
    Number,          // 42  3.14  0xFF  0b1010  1_000_000
    BigInt,          // 42n
    String,          // "foo"  'bar'
    TemplateHead,    // `before ${
    TemplateMiddle,  // } middle ${
    TemplateTail,    // } after`
    NoSubstTemplate, // `no substitutions`
    Regex,           // /pattern/flags

    // ── Identifiers & keywords ────────────────────────
    Ident,

    // Value keywords
    KwBreak, KwCase, KwCatch, KwClass, KwConst, KwContinue,
    KwDebugger, KwDefault, KwDelete, KwDo, KwElse, KwEnum,
    KwExport, KwExtends, KwFalse, KwFinally, KwFor, KwFunction,
    KwIf, KwImport, KwIn, KwInstanceof, KwLet, KwNew, KwNull,
    KwReturn, KwSuper, KwSwitch, KwThis, KwThrow, KwTrue,
    KwTry, KwTypeof, KwUndefined, KwVar, KwVoid, KwWhile, KwWith,
    KwYield, KwAsync, KwAwait, KwOf, KwFrom, KwAs, KwSatisfies,
    KwUsing, KwStatic,

    // Type keywords (contextual — valid as identifiers when not in type position)
    KwType, KwInterface, KwNamespace, KwModule, KwDeclare,
    KwAbstract, KwOverride, KwReadonly, KwKeyof, KwInfer,
    KwIs, KwAsserts, KwPublic, KwPrivate, KwProtected,
    KwNever, KwUnknown, KwAny, KwObject, KwSymbol,
    KwIntrinsic,

    // ── Punctuation ───────────────────────────────────
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, DotDotDot, QuestionDot,
    Question, QuestionQuestion,

    // ── Operators ─────────────────────────────────────
    Plus, Minus, Star, Slash, Percent, StarStar,
    Amp, Pipe, Caret, Tilde, LtLt, GtGt, GtGtGt,
    Bang, AmpAmp, PipePipe,
    Eq, EqEq, EqEqEq, BangEq, BangEqEq,
    Lt, Gt, LtEq, GtEq,
    Arrow, // =>
    PlusEq, MinusEq, StarEq, SlashEq, PercentEq, StarStarEq,
    AmpEq, PipeEq, CaretEq, LtLtEq, GtGtEq, GtGtGtEq,
    AmpAmpEq, PipePipeEq, QuestionQuestionEq,
    PlusPlus, MinusMinus,
    At,     // decorator @

    // ── JSX ───────────────────────────────────────────
    JsxText,
    JsxTagOpen,   // <TagName
    JsxTagClose,  // </TagName> or />
    JsxLBrace,    // { inside JSX
    JsxRBrace,    // } inside JSX

    // ── Special ───────────────────────────────────────
    Eof,
}
```

### FR-LEX-002 — Token struct

```rust
pub struct Token {
    pub kind: TokenKind,
    /// Interned text value (empty for punctuation/operators that have no variable content).
    pub value: Symbol,
    pub span: Span,
    pub has_preceding_newline: bool, // needed for ASI
}
```

### FR-LEX-003 — Automatic Semicolon Insertion (ASI)

The lexer must set `has_preceding_newline` on every token. The parser uses this flag to implement ASI per the ECMAScript specification. The lexer itself does not insert semicolons.

### FR-LEX-004 — Lexer modes

The lexer maintains a mode stack. Modes:
- `Normal` — default JavaScript/TypeScript
- `Template` — inside template literal `${ ... }`
- `JSX` — after `<` in expression position
- `Type` — after `:` or `<` in type annotation position (affects disambiguation of `>>` vs two `>` tokens)

### FR-LEX-005 — Numeric literal validation

Validate at lex time:
- Hex: `0[xX][0-9a-fA-F_]+`
- Binary: `0[bB][01_]+`
- Octal: `0[oO][0-7_]+`
- Decimal separators (`1_000`) — disallow leading/trailing/double separator
- BigInt suffix `n` — disallow on non-integer literals (`3.14n` is invalid)

### FR-LEX-006 — String escape sequences

Fully decode all ECMAScript escape sequences including `\uXXXX`, `\u{XXXXX}`, `\xXX`, and legacy octal escapes (which must produce a warning, not an error).

---

## §4 — Parser (FR-PAR-*)

**Crate:** `tsnat-parse`
**Produces:** A typed AST allocated in a `bumpalo::Bump` arena.

### FR-PAR-001 — Recursive descent

The parser is hand-written recursive descent. No parser generators. All disambiguation is handled by lookahead and context flags. The parser never backtracks — it is a single-pass parser with arbitrary lookahead via `peek_ahead(n)`.

### FR-PAR-002 — AST ownership model

All AST nodes are allocated in a per-parse `bumpalo::Bump` arena. Nodes use `&'arena` references, not `Box`. The arena is dropped when parsing is complete and the program moves to the interpreter or type checker.

```rust
// crates/tsnat-parse/src/arena.rs
pub type AstArena<'a> = &'a bumpalo::Bump;
pub type NodeList<'a, T> = &'a [T];
```

### FR-PAR-003 — Top-level program structure

```rust
// crates/tsnat-parse/src/ast.rs

pub struct Program<'a> {
    pub stmts: NodeList<'a, Stmt<'a>>,
    pub span: Span,
    pub source_type: SourceType,
}

pub enum SourceType {
    Module,  // has import/export → always Module in TSNAT
    Script,  // no import/export
}
```

### FR-PAR-004 — Statement node set

```rust
pub enum Stmt<'a> {
    Block(BlockStmt<'a>),
    Var(VarDecl<'a>),
    Expr(ExprStmt<'a>),
    If(IfStmt<'a>),
    Switch(SwitchStmt<'a>),
    For(ForStmt<'a>),
    ForIn(ForInStmt<'a>),
    ForOf(ForOfStmt<'a>),
    While(WhileStmt<'a>),
    DoWhile(DoWhileStmt<'a>),
    Return(ReturnStmt<'a>),
    Throw(ThrowStmt<'a>),
    Try(TryStmt<'a>),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Labeled(LabeledStmt<'a>),
    // Declarations
    Function(FunctionDecl<'a>),
    Class(ClassDecl<'a>),
    Interface(InterfaceDecl<'a>),
    TypeAlias(TypeAliasDecl<'a>),
    Enum(EnumDecl<'a>),
    Namespace(NamespaceDecl<'a>),
    Import(ImportDecl<'a>),
    Export(ExportDecl<'a>),
    // Ambient declarations (declare ...)
    Ambient(AmbientDecl<'a>),
}
```

### FR-PAR-005 — Expression node set

```rust
pub enum Expr<'a> {
    // Literals
    Number(f64, Span),
    BigInt(u128, Span),
    String(Symbol, Span),
    Bool(bool, Span),
    Null(Span),
    Undefined(Span),
    This(Span),
    Regex { pattern: Symbol, flags: Symbol, span: Span },
    Template(TemplateExpr<'a>),
    TaggedTemplate(TaggedTemplateExpr<'a>),

    // Identifiers & member access
    Ident(Symbol, Span),
    Member(MemberExpr<'a>),
    Index(IndexExpr<'a>),
    OptChain(OptChainExpr<'a>),

    // Operations
    Unary(UnaryExpr<'a>),
    Binary(BinaryExpr<'a>),
    Logical(LogicalExpr<'a>),
    Conditional(ConditionalExpr<'a>),
    Assign(AssignExpr<'a>),
    Sequence(SequenceExpr<'a>),
    Spread(SpreadExpr<'a>),

    // Calls & construction
    Call(CallExpr<'a>),
    New(NewExpr<'a>),

    // Functions
    Function(FunctionExpr<'a>),
    Arrow(ArrowExpr<'a>),
    Class(ClassExpr<'a>),

    // Async
    Await(AwaitExpr<'a>),
    Yield(YieldExpr<'a>),

    // Type
    As(AsExpr<'a>),           // expr as T
    Satisfies(SatisfiesExpr<'a>),
    TypeAssertion(TypeAssertionExpr<'a>), // <T>expr

    // Destructuring
    ArrayPattern(ArrayPattern<'a>),
    ObjectPattern(ObjectPattern<'a>),

    // JSX
    JsxElement(JsxElement<'a>),
    JsxFragment(JsxFragment<'a>),
}
```

### FR-PAR-006 — Type node set

```rust
pub enum TypeNode<'a> {
    // Primitives
    Number(Span), String(Span), Boolean(Span), BigInt(Span),
    Symbol(Span), Null(Span), Undefined(Span), Void(Span),
    Never(Span), Unknown(Span), Any(Span), Object(Span),

    // Literals
    LiteralNumber(f64, Span),
    LiteralString(Symbol, Span),
    LiteralBool(bool, Span),

    // Structural
    TypeRef(TypeRefNode<'a>),          // Identifier with optional type args
    Object(ObjectTypeNode<'a>),        // { x: T; y?: U }
    Array(ArrayTypeNode<'a>),          // T[]
    Tuple(TupleTypeNode<'a>),          // [A, B, C]
    Function(FunctionTypeNode<'a>),    // (x: T) => U
    Constructor(ConstructorTypeNode<'a>),

    // Combinators
    Union(UnionTypeNode<'a>),           // A | B
    Intersection(IntersectionTypeNode<'a>), // A & B

    // Advanced
    Conditional(ConditionalTypeNode<'a>), // T extends U ? X : Y
    Infer(InferTypeNode<'a>),             // infer T
    Mapped(MappedTypeNode<'a>),           // { [K in keyof T]: ... }
    IndexedAccess(IndexedAccessNode<'a>), // T[K]
    TemplateLiteral(TemplateLiteralTypeNode<'a>),
    Typeof(TypeofTypeNode<'a>),
    Keyof(KeyofTypeNode<'a>),
    Unique(UniqueTypeNode<'a>),           // unique symbol

    // Type predicate
    Predicate(TypePredicateNode<'a>),     // x is T / asserts x is T

    // Parenthesised (preserved for pretty-printing)
    Paren(ParenTypeNode<'a>),
}
```

### FR-PAR-007 — Error recovery

On a syntax error the parser:
1. Emits a `TsnatError::Parse` diagnostic to the error collector (does not panic).
2. Advances tokens until a synchronisation point: `;`, `}`, `)`, `]`, or a statement-starting keyword.
3. Resumes parsing from that point.
4. Returns a `Program` even on errors. The interpreter will not run a program with parse errors, but the type checker can still analyse it for IDE use.

### FR-PAR-008 — Operator precedence

Precedence table (high = tight binding):

| Level | Operators |
|---|---|
| 20 | `(expr)`, `expr.x`, `expr[x]`, `expr?.x`, `new expr(args)`, `expr(args)` |
| 19 | `new expr` (no args), `expr++`, `expr--` |
| 18 | `++expr`, `--expr`, `!expr`, `~expr`, `+expr`, `-expr`, `typeof`, `void`, `delete`, `await` |
| 17 | `**` (right-associative) |
| 16 | `*`, `/`, `%` |
| 15 | `+`, `-` |
| 14 | `<<`, `>>`, `>>>` |
| 13 | `<`, `>`, `<=`, `>=`, `instanceof`, `in` |
| 12 | `==`, `!=`, `===`, `!==` |
| 11 | `&` |
| 10 | `^` |
| 9 | `\|` |
| 8 | `&&` |
| 7 | `\|\|`, `??` |
| 6 | `? :` (right-associative) |
| 5 | `=`, `+=`, `-=`, … (right-associative) |
| 4 | `yield`, `yield*` |
| 3 | `...expr` |
| 2 | `,` |

---

## §5 — Type Checker (FR-TYP-*)

**Crate:** `tsnat-types`
**Input:** `Program<'a>` from `tsnat-parse`
**Output:** `TypedProgram` with every expression annotated with a resolved `TypeId`

### FR-TYP-001 — Type representation

```rust
// crates/tsnat-types/src/ty.rs

/// A cheap handle to a type. Types are stored in the TypeArena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

pub enum Type {
    // Primitives
    Number, String, Boolean, BigInt, Symbol, Null, Undefined,
    Void, Never, Unknown, Any,

    // Literals
    LiteralNumber(ordered_float::OrderedFloat<f64>),
    LiteralString(Symbol),
    LiteralBool(bool),
    UniqueSymbol(u32), // unique symbol — identified by creation site

    // Composite
    Object(ObjectType),
    Array(TypeId),
    Tuple(TupleType),
    Function(FunctionType),
    Constructor(ConstructorType),

    // Combinators
    Union(Vec<TypeId>),
    Intersection(Vec<TypeId>),

    // Generic
    TypeParam(TypeParamType),    // T in generic context
    Generic(GenericType),        // T<A, B>

    // Advanced
    Conditional(ConditionalType),// T extends U ? X : Y
    Infer(Symbol),               // infer T
    Mapped(MappedType),
    IndexedAccess(TypeId, TypeId),// T[K]
    TemplateLiteral(TemplateLiteralType),

    // Intrinsic helpers
    Keyof(TypeId),
    Typeof(TypeId),
}

pub struct ObjectType {
    pub properties: IndexMap<Symbol, PropertyType>,
    pub index_signatures: Vec<IndexSignature>,
    pub call_signatures: Vec<FunctionType>,
    pub construct_signatures: Vec<ConstructorType>,
}

pub struct PropertyType {
    pub ty: TypeId,
    pub optional: bool,
    pub readonly: bool,
}

pub struct FunctionType {
    pub type_params: Vec<TypeParamDecl>,
    pub params: Vec<ParamType>,
    pub return_ty: TypeId,
}
```

### FR-TYP-002 — Structural assignability

The core operation is `is_assignable(source: TypeId, target: TypeId) -> bool`.
Rules:
- `never` is assignable to every type.
- Every type is assignable to `unknown` and `any`.
- `any` is assignable to every type.
- `null` and `undefined` are assignable to `T | null` and `T | undefined` respectively.
- Object types: `source` is assignable to `target` if `source` has at least all required properties of `target` with compatible types.
- Union: `A | B` is assignable to `T` if both `A` and `T` and `B` and `T` are assignable.
- Intersection: `A & B` is assignable to `T` if `A` and `T` or `B` and `T` is assignable.

### FR-TYP-003 — Type inference

The checker performs bidirectional type inference:
- **Check mode:** An expected type is pushed down into an expression. Used when the expression is in a typed context (variable declaration with annotation, argument with parameter type).
- **Infer mode:** The expression's type is inferred bottom-up. Used when no context type is available.

Generic type argument inference: when calling `f<T>(x: T)` without explicit `<T>`, the checker infers `T` from the argument type. Uses a `TypeInferencer` that solves a system of type equations.

### FR-TYP-004 — Control flow narrowing

The checker maintains a `FlowNode` graph derived from the CFG of each function. At each point in the program, the type of a variable is the intersection of the types narrowed by all reachable flow nodes.

Narrowing triggers:
- `typeof x === "string"` → narrows to `string`
- `typeof x === "number"` → narrows to `number`
- `x instanceof Foo` → narrows to `Foo`
- `x != null` → removes `null | undefined`
- `x !== undefined` → removes `undefined`
- Discriminant property checks on union types
- User-defined type guards: `function isString(x: unknown): x is string`
- `asserts` predicates: `function assert(x: unknown): asserts x is string`

### FR-TYP-005 — Generics and constraints

```typescript
function pick<T, K extends keyof T>(obj: T, key: K): T[K]
```

Type parameters with `extends` constraints are checked at call sites. The checker must:
1. Verify the supplied type argument satisfies the constraint.
2. Substitute the type parameter with the inferred or supplied argument throughout the function's type signature.

### FR-TYP-006 — Conditional types

`T extends U ? X : Y`

When `T` is a concrete type, evaluate immediately. When `T` is a type parameter, defer — the conditional type remains unevaluated and is distributed if `T` is later substituted with a union.

Distribution: `(A | B) extends U ? X : Y` evaluates to `(A extends U ? X : Y) | (B extends U ? X : Y)`.

### FR-TYP-007 — Mapped types

```typescript
type Readonly<T> = { readonly [K in keyof T]: T[K] }
```

The checker must:
1. Enumerate the key type (`keyof T` = union of string/number/symbol literal types).
2. For each key `K`, construct the value type by substituting `K` into the value expression.
3. Apply `+readonly`, `-readonly`, `+?`, `-?` modifiers.

### FR-TYP-008 — Template literal types

```typescript
type EventName<T extends string> = `on${Capitalize<T>}`;
```

The checker must:
1. Normalise template literal types to a canonical `TemplateLiteralType` representation.
2. Resolve intrinsic string manipulation types: `Uppercase<T>`, `Lowercase<T>`, `Capitalize<T>`, `Uncapitalize<T>`.
3. Perform assignability checks on template literal types against string literal unions.

### FR-TYP-009 — Built-in utility types

These must be defined in `lib/lib.d.ts` and resolved by the type checker via normal mapped/conditional type evaluation — NOT hardcoded in the checker:

`Partial<T>`, `Required<T>`, `Readonly<T>`, `Pick<T, K>`, `Omit<T, K>`,
`Record<K, V>`, `Exclude<T, U>`, `Extract<T, U>`, `NonNullable<T>`,
`ReturnType<F>`, `Parameters<F>`, `InstanceType<C>`, `Awaited<T>`,
`ConstructorParameters<C>`, `ThisType<T>`.

### FR-TYP-010 — Error messages

Every type error must include:
- The span of the offending expression.
- The source type (what was given).
- The target type (what was expected).
- A one-sentence plain-English explanation.

```
error[TS2322]: Type 'string' is not assignable to type 'number'
  --> src/main.ts:4:7
   |
 4 |   x = "hello";
   |       ^^^^^^^ expected `number`, found `string`
```

---

## §6 — IR Lowering (FR-IR-*)

**Crate:** `tsnat-ir`
**Input:** Typed AST from `tsnat-types`
**Output:** A `Module` of `Function` objects in three-address SSA form

### FR-IR-001 — Instruction set

```rust
pub enum Instr {
    // Constants
    Const    { dst: Reg, val: ConstValue },

    // Arithmetic
    Add      { dst: Reg, lhs: Reg, rhs: Reg },
    Sub      { dst: Reg, lhs: Reg, rhs: Reg },
    Mul      { dst: Reg, lhs: Reg, rhs: Reg },
    Div      { dst: Reg, lhs: Reg, rhs: Reg },
    Mod      { dst: Reg, lhs: Reg, rhs: Reg },
    Pow      { dst: Reg, lhs: Reg, rhs: Reg },
    Neg      { dst: Reg, src: Reg },

    // Bitwise
    BitAnd   { dst: Reg, lhs: Reg, rhs: Reg },
    BitOr    { dst: Reg, lhs: Reg, rhs: Reg },
    BitXor   { dst: Reg, lhs: Reg, rhs: Reg },
    BitNot   { dst: Reg, src: Reg },
    Shl      { dst: Reg, lhs: Reg, rhs: Reg },
    Shr      { dst: Reg, lhs: Reg, rhs: Reg },
    UShr     { dst: Reg, lhs: Reg, rhs: Reg },

    // Comparison
    Eq       { dst: Reg, lhs: Reg, rhs: Reg }, // ===
    Ne       { dst: Reg, lhs: Reg, rhs: Reg }, // !==
    Lt       { dst: Reg, lhs: Reg, rhs: Reg },
    Le       { dst: Reg, lhs: Reg, rhs: Reg },
    Gt       { dst: Reg, lhs: Reg, rhs: Reg },
    Ge       { dst: Reg, lhs: Reg, rhs: Reg },

    // Object operations
    AllocObj { dst: Reg, shape_id: ShapeId },
    GetProp  { dst: Reg, obj: Reg, key: PropKey },
    SetProp  { obj: Reg, key: PropKey, val: Reg },
    DeleteProp { obj: Reg, key: PropKey },
    GetIndex { dst: Reg, obj: Reg, idx: Reg },
    SetIndex { obj: Reg, idx: Reg, val: Reg },

    // Array operations
    AllocArr { dst: Reg, capacity: u32 },
    Push     { arr: Reg, val: Reg },
    Len      { dst: Reg, arr: Reg },

    // Function operations
    MakeClosure { dst: Reg, fn_id: FnId, captures: Vec<Reg> },
    Call        { dst: Reg, callee: Reg, this: Reg, args: Vec<Reg> },
    CallMethod  { dst: Reg, obj: Reg, method: Symbol, args: Vec<Reg> },
    Return      { val: Reg },
    Throw       { val: Reg },

    // Control flow
    Jump   { target: BlockId },
    Branch { cond: Reg, then_: BlockId, else_: BlockId },

    // Async/generator state machine
    Suspend { resume_block: BlockId, yield_val: Reg },
    Resume  { dst: Reg },

    // GC
    GcAlloc  { dst: Reg, size: Reg },
    GcRoot   { ptr: Reg },       // register as GC root
    GcUnroot { ptr: Reg },

    // FFI
    FfiCall  { dst: Reg, symbol: Symbol, args: Vec<Reg>, convention: CallConv },
    FfiLoad  { dst: Reg, ptr: Reg, ty: FfiType },
    FfiStore { ptr: Reg, val: Reg, ty: FfiType },
}
```

### FR-IR-002 — Async lowering

`async function f() { const x = await g(); }` lowers to:

1. A state struct holding all locals live across `await` points.
2. A `resume(state, resolved_value)` function that matches on `state.phase` and jumps to the appropriate `Suspend`/`Resume` pair.
3. The original `f()` returns a `Promise` that calls `resume(state, undefined)` to start execution.

### FR-IR-003 — Closure lowering

Every captured variable that is mutated after capture is heap-allocated into a `Cell<Value>` struct. The closure captures a pointer to the cell. Immutable captures are copied by value.

---

## §7 — Interpreter (FR-EVAL-*)

**Crate:** `tsnat-eval`
**Input:** `Program<'a>` + `TypedProgram` from `tsnat-types`
**Executes:** Directly on the typed AST. No IR in Phase 1. Speed is not a goal.

### FR-EVAL-001 — Value type

```rust
// crates/tsnat-eval/src/value.rs

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    BigInt(i128),
    String(Rc<str>),
    Symbol(SymbolId),
    Object(Rc<RefCell<JsObject>>),
    Function(Rc<JsFunction>),
    NativeFunction(Rc<dyn Fn(Vec<Value>, Option<Value>) -> EvalResult<Value>>),
    Promise(Rc<RefCell<JsPromise>>),
    Regex(Rc<JsRegex>),
}

pub struct JsObject {
    pub shape: ShapeId,
    pub properties: IndexMap<Symbol, Value>,
    pub prototype: Option<Rc<RefCell<JsObject>>>,
    pub exotic: Option<ExoticKind>, // Array, Arguments, etc.
}

pub struct JsFunction {
    pub name: Symbol,
    pub params: Vec<ParamDecl>,
    pub body: FunctionBody,
    pub closure: Environment,
    pub is_async: bool,
    pub is_generator: bool,
    pub prototype: Rc<RefCell<JsObject>>,
}

pub enum FunctionBody {
    AstNode(FunctionNode), // Phase 1
    IrFn(FnId),            // Phase 5
}
```

### FR-EVAL-002 — Environment

```rust
pub struct Environment {
    bindings: FxHashMap<Symbol, Binding>,
    parent: Option<Rc<RefCell<Environment>>>,
}

pub struct Binding {
    pub value: Value,
    pub kind: BindingKind,
}

pub enum BindingKind {
    Const,    // cannot reassign
    Let,      // can reassign
    Var,      // hoisted to function scope
    Function, // hoisted + initialised
    Class,    // temporal dead zone until declaration
}
```

### FR-EVAL-003 — Global object

At startup the interpreter populates the global environment with:

```
undefined, null, true, false, Infinity, NaN, globalThis
console   { log, error, warn, info, debug, assert, time, timeEnd, group, groupEnd }
Math      { abs, ceil, floor, round, sqrt, pow, min, max, log, exp, random, sign, trunc, ... }
JSON      { parse, stringify }
Date      { new Date(), Date.now(), Date.parse() }
Array     { new Array(), Array.from(), Array.isArray(), Array.of() }
Object    { keys, values, entries, assign, create, freeze, isFrozen, getPrototypeOf, ... }
String    { fromCharCode, fromCodePoint }
Number    { isInteger, isFinite, isNaN, parseInt, parseFloat, MAX_SAFE_INTEGER, ... }
Boolean
Symbol    { for, keyFor, iterator, asyncIterator, toPrimitive, ... }
Map, Set, WeakMap, WeakSet
Promise   { resolve, reject, all, allSettled, any, race }
Error, TypeError, RangeError, ReferenceError, SyntaxError, URIError, EvalError
Proxy, Reflect
RegExp
ArrayBuffer, Uint8Array, Int32Array, Float64Array, ... (all TypedArrays)
```

### FR-EVAL-004 — Promise and microtask queue

```rust
pub struct EventLoop {
    microtask_queue: VecDeque<Microtask>,
    timer_queue: BinaryHeap<TimerTask>,
    io_queue: VecDeque<IoCompletion>,
}
```

After each top-level statement execution:
1. Drain the microtask queue completely (each resolved Promise schedules microtasks).
2. Check timers.
3. Check I/O completions.

`await` inside an async function:
1. Evaluates the awaited expression to a `Value`.
2. If the value is a `Promise`, suspends the current frame and registers a continuation.
3. Returns `Value::Undefined` to the event loop.
4. When the Promise settles, the continuation is pushed to the microtask queue.

### FR-EVAL-005 — Prototype chain

Property lookup (`get_property(obj, key)`) algorithm:
1. Check `obj.properties` (own properties).
2. If not found, check `obj.prototype` recursively.
3. If no prototype, return `Value::Undefined`.

`Object.create(proto)` creates an object with the given prototype.

### FR-EVAL-006 — Module loading

```rust
pub struct ModuleLoader {
    cache: FxHashMap<PathBuf, Rc<Module>>,
    resolver: Box<dyn ModuleResolver>,
}

pub trait ModuleResolver {
    fn resolve(&self, specifier: &str, from: &Path) -> TsnatResult<PathBuf>;
}
```

`import { x } from './foo'` triggers:
1. Resolve the path via `ModuleResolver`.
2. Check the cache. If hit, return cached exports.
3. Parse and execute the module. Cache the result.
4. Bind the imported names in the current environment.

Dynamic `import()` returns a `Promise` that resolves to the module namespace object.

---

## §8 — Native FFI (FR-FFI-*)

**Crate:** `tsnat-ffi`

### FR-FFI-001 — Type mapping (TypeScript → C)

| TypeScript type | C type | Notes |
|---|---|---|
| `number` | `double` | IEEE 754 |
| `bigint` | `int64_t` | Truncates to 64 bits |
| `string` | `const char*` | UTF-8, null-terminated, caller owns |
| `boolean` | `uint8_t` | 0 or 1 |
| `null` | `void*` | NULL pointer |
| `undefined` | `void*` | NULL pointer |
| `NativePtr<T>` | `void*` | Opaque pointer, no dereference from TS |
| `ArrayBuffer` | `void*, size_t` | Two-argument C convention |
| `() => void` | `void (*fn)(void*)` + `void*` | Function pointer + closure env |

### FR-FFI-002 — Import syntax

```typescript
// In a .d.ts file or a .ts file with declare
declare native function SDL_CreateWindow(
    title: string,
    x: number, y: number,
    w: number, h: number,
    flags: number
): NativePtr<SDLWindow>;
```

The interpreter resolves `declare native function` bindings by:
1. Looking up the symbol name in the loaded shared libraries.
2. Wrapping the raw function pointer in a `Value::NativeFunction` with type-checked argument marshalling.

### FR-FFI-003 — Export syntax

```typescript
@NativeExport
export function onEvent(event: NativePtr<SDLEvent>): void {
    // ...
}
```

`@NativeExport` generates:
1. A C-callable wrapper function with the appropriate ABI.
2. A symbol table entry in the compiled output.

### FR-FFI-004 — Library loading

```typescript
import native SDL from 'sdl3'; // resolves to libSDL3.so / SDL3.dll / libSDL3.dylib
```

The interpreter calls `dlopen` (Linux/macOS) or `LoadLibraryW` (Windows) to load the shared library. Symbol resolution uses `dlsym` / `GetProcAddress`.

### FR-FFI-005 — NativePtr<T> type

`NativePtr<T>` is a branded opaque value:
```rust
pub struct NativePtr {
    pub ptr: *mut std::ffi::c_void,
    pub type_name: Symbol, // for runtime type checking
}
```

Dereferencing a `NativePtr<T>` from TypeScript requires a corresponding `declare native` accessor. Attempting to dereference an undeclared pointer is a runtime `TsnatError::Ffi`.

---

## §9 — React Renderer (FR-REACT-*)

**Crate:** `tsnat-react`
**Strategy:** React 19 runs *inside the interpreter*. The custom reconciler's `HostConfig` is implemented in Rust and exposed to the interpreter via FFI.

### FR-REACT-001 — Bundled React

The crate bundles a pre-compiled ESM build of `react@19` and `react-reconciler@0.29`. These are loaded by the interpreter at startup as virtual modules at the paths `react` and `react-reconciler`. They do not come from npm at runtime.

### FR-REACT-002 — Host config

```typescript
// This TypeScript interface is implemented in Rust via @NativeExport functions
// and exposed to the JS reconciler as a host config object.

interface HostConfig {
    // Creation
    createInstance(type: string, props: object): NativePtr<Widget>;
    createTextInstance(text: string): NativePtr<Widget>;

    // Tree mutations
    appendInitialChild(parent: NativePtr<Widget>, child: NativePtr<Widget>): void;
    appendChild(parent: NativePtr<Widget>, child: NativePtr<Widget>): void;
    insertBefore(parent: NativePtr<Widget>, child: NativePtr<Widget>, before: NativePtr<Widget>): void;
    removeChild(parent: NativePtr<Widget>, child: NativePtr<Widget>): void;

    // Updates
    prepareUpdate(
        instance: NativePtr<Widget>,
        type: string,
        oldProps: object,
        newProps: object
    ): object | null; // return non-null if update is needed

    commitUpdate(
        instance: NativePtr<Widget>,
        updatePayload: object,
        type: string,
        oldProps: object,
        newProps: object
    ): void;

    commitTextUpdate(instance: NativePtr<Widget>, oldText: string, newText: string): void;

    // Containers
    createContainer(): NativePtr<Window>;
    appendChildToContainer(container: NativePtr<Window>, child: NativePtr<Widget>): void;
    removeChildFromContainer(container: NativePtr<Window>, child: NativePtr<Widget>): void;

    // Scheduling
    scheduleCallback(priority: number, fn: () => void): void;
    cancelCallback(id: number): void;
    now(): number;
}
```

### FR-REACT-003 — Widget types

| JSX type | SDL3 widget | Yoga node type |
|---|---|---|
| `Window` | `SDL_Window` + `SDL_Renderer` | Root |
| `View` | Rendered rectangle | `YGNodeRef` (flex container) |
| `Text` | FreeType-rendered glyph atlas | Leaf |
| `Input` | Rendered rectangle + cursor + text buffer | Leaf |
| `Button` | `View` + click handler | `YGNodeRef` |
| `Image` | SDL texture | Leaf |
| `ScrollView` | `View` + scroll offset + clipping | Container |
| `Canvas` | `SDL_Renderer` + user draw callback | Leaf |

### FR-REACT-004 — Layout pass

After every reconciler commit, the renderer:
1. Calls `YGNodeCalculateLayout(root, window_w, window_h, YGDirectionLTR)`.
2. Walks the widget tree depth-first.
3. Calls `SDL_RenderFillRect` (views), FreeType glyph rendering (text), or `SDL_RenderTexture` (images) with the computed layout.
4. Calls `SDL_RenderPresent`.

### FR-REACT-005 — Event dispatch

The SDL event loop (`SDL_PollEvent`) runs on the main thread at 60 fps. Events are translated:

| SDL event | React synthetic event |
|---|---|
| `SDL_EVENT_MOUSE_BUTTON_DOWN` over a widget | `onClick` |
| `SDL_EVENT_MOUSE_MOTION` | `onMouseMove` |
| `SDL_EVENT_KEY_DOWN` | `onKeyDown` |
| `SDL_EVENT_TEXT_INPUT` | `onChange` (on Input) |
| `SDL_EVENT_WINDOW_RESIZED` | Layout recalculation |
| `SDL_EVENT_QUIT` | `process.exit(0)` |

### FR-REACT-006 — Entry point

```typescript
// This function is the only TSNAT-specific API the user calls.
// Everything else is standard React.
import { renderApp } from 'tsnat/react';
import React from 'react';

renderApp(<App />, {
    title: 'My App',
    width: 800,
    height: 600,
    resizable: true,
});
```

---

## §10 — Code Generator (FR-CG-*)

**Crate:** `tsnat-codegen`
**Phase:** 5 — Do not begin until Phase 4 exit tests are green.

### FR-CG-001 — LLVM IR generation

The code generator lowers `tsnat-ir` `Module` → LLVM IR via `inkwell`. Each IR `Function` becomes an LLVM function. Each IR `Reg` becomes an LLVM `alloca` (promoted to registers by `mem2reg`).

### FR-CG-002 — Garbage collector

Phase 5 GC: Boehm conservative GC (`libgc`). All heap allocations go through `GC_malloc(size)`. The GC scans the stack conservatively for pointers into the heap. The Rust code must not store GC-managed pointers on the stack through any `unsafe` operation that LLVM may eliminate.

### FR-CG-003 — Shape-based object layout

Each `ShapeId` describes a fixed struct layout for objects of that shape:
```
struct Shape_42 {
    GcHeader header;        // 8 bytes: shape_id (u32) + ref_count (u32)
    JsValue  prototype;     // 8 bytes
    JsValue  props[N];      // 8 bytes × N (fixed by shape)
}
```

`GetProp` on a known shape compiles to a direct field offset load — one instruction, no hash lookup.

### FR-CG-004 — Inline cache

For call sites where the shape is unknown at compile time, an inline cache is emitted:
```
; IC stub — patched after first execution
call void @ic_miss_handler(i64 %shape_id, i64 %prop_key, ptr %obj)
```
After the first miss the stub is patched (via LLVM's `patchpoint` intrinsic) to a direct offset load for the observed shape.

---

## §11 — CLI (FR-CLI-*)

**Crate:** `tsnat-cli`
**Binary:** `tsnat`

```
tsnat run    <file.ts>              Execute TypeScript (interpreter mode)
tsnat build  <file.ts> -o <out>    Compile to native binary (Phase 5)
tsnat check  <file.ts>             Type-check only; exit 1 on errors
tsnat repl                         Interactive REPL
tsnat fmt    <file.ts>             Format (lossless, CST-based)
```

Every subcommand:
- Accepts `--tsconfig <path>` (default: `tsconfig.json` in CWD or ancestor).
- Outputs diagnostics to stderr in `miette` format.
- Exits 0 on success, 1 on error.

---

## §12 — Built-in Library (FR-LIB-*)

**Location:** `lib/lib.d.ts`

This file is parsed and loaded as file ID 0 in the `SourceMap`. It declares the types of all built-in globals. It is TypeScript declaration syntax only — no implementations. Implementations are in `tsnat-eval`'s built-in object setup (FR-EVAL-003).

The file must declare (at minimum):
- All primitive types and their prototype methods.
- `Array<T>`, `ReadonlyArray<T>`, `Map<K, V>`, `Set<T>`, `WeakMap<K, V>`, `WeakSet<T>`.
- `Promise<T>`, `PromiseLike<T>`.
- `Error`, `TypeError`, `RangeError`, `ReferenceError`, `SyntaxError`.
- `Proxy<T>`, `Reflect`.
- All TypedArrays (`Uint8Array`, `Int32Array`, `Float64Array`, etc.).
- All utility types from FR-TYP-009.
- `NativePtr<T>`, `NativeExport` decorator type.
- React types: `React.FC<P>`, `React.ReactElement`, `React.ReactNode`, all hook signatures.
- TSNAT renderer types: `renderApp`, `ViewStyle`, `TextStyle`, all host element prop types.
