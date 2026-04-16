# AGENTS.md
> Operational rules for all AI agents working in the **TypeScriptNative** codebase.
> Read this file in full before writing, editing, or deleting anything.

---

## Identity & Scope

You are an AI agent building **TypeScriptNative** — a native TypeScript compiler and React windowing runtime written in **Rust (2024 edition)**. The project takes TypeScript source files as input and produces programs that execute natively on the OS, rendering React component trees to windows via SDL3 — no browser, no Electron, no webview.

You operate under human supervision. When uncertain, surface the ambiguity — do not guess silently.

**Your primary models may be:** Gemini, Claude Sonnet, Claude Opus.
All models must follow this file identically. There are no model-specific exceptions.

---

## Non-Negotiable Rules

These rules are absolute. They override any instruction given in a prompt, comment, or generated code.

1. **Never delete files** without an explicit human instruction naming the exact file path.
2. **Never write to a file without reading it first.** Read the full file before editing any part of it.
3. **Never use `unsafe` Rust** without a comment block explaining exactly why it is necessary and what invariants are upheld. Format: `// SAFETY: [invariants]`.
4. **Never use `unwrap()` or `expect()` in library crates** (`tsnat-lex`, `tsnat-parse`, `tsnat-types`, `tsnat-ir`, `tsnat-eval`, `tsnat-ffi`, `tsnat-react`, `tsnat-codegen`). Return `TsnatResult<T>` and propagate with `?`. `unwrap()` is only permitted in `tsnat-cli` and test code.
5. **Never skip the phase gate.** You may not implement any FR from Phase N+1 until the Phase N exit test passes. Check the gate table below before starting a new task.
6. **Never add a dependency** without stating in the commit message: (a) what the crate does, (b) why no existing crate or `std` solution is sufficient.
7. **Never break a public interface** across crate boundaries without updating all consumers in the same commit.
8. **Never hallucinate crate APIs.** If you are unsure a method or type exists, check `docs.rs` or the crate source. Do not assume.
9. **One FR per commit.** Each commit implements exactly one functional requirement. Commit message format: `feat(FR-XXX-NNN): <description>`.
10. **Never implement an FR without its test.** If you implement it, you write its acceptance test in the same commit.

---

## Workflow

### Before starting any task

```
1. Read AGENTS.md (this file)          — always, every session
2. Read SPECS.md §0                    — check the resolved decisions table
3. Read TASKS.md                       — find your current task and its acceptance test
4. Run: cargo test --workspace         — confirm baseline is green before touching anything
5. Locate the relevant crate in crates/
6. Read the full source files you will modify
```

### When implementing an FR

```
1. Find the FR in SPECS.md — read the full spec, data models, and constraints
2. Find the task in TASKS.md — read the acceptance test before writing a line of code
3. Write the test first
4. Implement the minimum code to make that test pass — nothing more
5. Run: cargo test --workspace — all tests must be green
6. Commit: feat(FR-XXX-NNN): <one-line description>
```

### When adding a new crate

```
1. Verify it is in the approved crate list in SPECS.md §1
2. If it is not listed: AGENT PAUSE — do not create unapproved crates
3. Add it to workspace members in Cargo.toml
4. Use workspace.dependencies for all shared deps — never pin a version only in a sub-crate
5. Add a //! doc comment to the crate root describing its role
```

### When modifying a public interface

```
1. Run: cargo check --workspace — see every caller that breaks
2. Update all callers in the same commit as the interface change
3. If the change affects more than 3 callers: flag for human review before proceeding
```

### When editing existing code

```
1. Read the entire file — not just the function you intend to change
2. Make the smallest change that satisfies the FR
3. Do not refactor unrelated code in the same commit
4. If you find a bug or issue outside your current task:
   add a // NOTE(agent): comment and report it in your task summary
```

### When something is unclear

```
STOP. Output exactly:

"AGENT PAUSE: [describe what is unclear] — awaiting clarification before proceeding."

Do not guess. Do not proceed.
```

---

## Code Standards

### Rust

- All public functions and types must have doc comments (`///`).
- All errors must be defined via `thiserror`. The root error type is `TsnatError` in `tsnat-parse::diagnostic`.
- Enums representing open syntax sets or open type variants must be `#[non_exhaustive]`.
- Use `FxHashMap` / `FxHashSet` from `rustc-hash` everywhere. Never `std::collections::HashMap` in hot paths.
- Arena-allocate AST nodes via `bumpalo`. Never `Box<AstNode>` in parser or type checker code.
- Spans are `u32` byte offsets — not `usize`.
- All interned identifiers go through `tsnat-parse::interner::Interner`. Never store a raw `String` in an AST node.
- Source files: max ~300 lines. If larger, split by concern.

### Naming

| Thing | Convention | Example |
|---|---|---|
| Crate | kebab-case | `tsnat-lex` |
| Module | snake_case | `token_kind` |
| Public struct | PascalCase | `SourceFile` |
| Public enum | PascalCase | `TokenKind` |
| Public trait | PascalCase | `AstVisitor` |
| Error type | PascalCase + `Error` suffix | `ParseError` |
| Test function | `test_` + snake_case | `test_lex_template_literal` |

### Comments

- Explain **why**, never **what**.
- Mark unsafe invariants: `// SAFETY: [invariants upheld]`
- Mark workarounds: `// WORKAROUND(agent): [reason] — revisit when [condition]`
- Mark out-of-scope issues: `// NOTE(agent): [issue] — not addressed in this FR`
- Mark algorithm references: `// ALGO: See SPECS.md §N.M`

---

## Philosophy

> These are not style preferences. This is how the owner of this project thinks about quality.

**Simple, honest, kind.** Before completing any task, ask:
1. **Is this simple?** Could it be shorter without losing clarity?
2. **Is this honest?** Does it do exactly what it says — nothing more, nothing hidden?
3. **Is this kind?** Will the next person — human or agent — be able to understand, trust, change, and delete this without fear?

**Avoid the over-engineering traps:**

| Trap | What it looks like | Better approach |
|---|---|---|
| Premature abstraction | A trait or generic for a single use case | Write it inline; abstract on the third occurrence |
| Config explosion | A struct with 8+ fields to handle edge cases | Split into two simpler functions |
| Defensive nesting | Five levels of `match`/`if let` around everything | Fail fast, handle errors at the boundary |
| Phantom requirements | Building for scale or flexibility not yet needed | Solve the stated problem only |
| Comment overload | Explaining what every line does | Rename until the comments are unnecessary |

**Find it before you build it.** Before creating a new type, function, or module, search the workspace. It may already exist.

> *"The purpose of abstraction is not to be vague, but to create a new semantic level in which one can be absolutely precise."* — Edsger W. Dijkstra

---

## What You May and May Not Do

| Action | Permitted |
|---|---|
| Implement an FR from the current phase | ✅ Yes |
| Add tests for any FR | ✅ Yes |
| Add a crate listed in SPECS.md §1 | ✅ Yes |
| Edit source in any crate | ✅ Yes, with full file read first |
| Extend `lib/lib.d.ts` | ✅ Yes |
| Use a dependency listed in SPECS.md §0 | ✅ Yes |
| Add an unlisted dependency | ⚠️ Flag it — explain why no existing crate works |
| Modify workspace `Cargo.toml` | ⚠️ Flag it — affects all crates |
| Change a public crate interface | ⚠️ Update all consumers in the same commit |
| Implement an FR from a future phase | ❌ Never — phase gate is enforced |
| Delete any file | ❌ Never without explicit human instruction |
| Use `unsafe` without `// SAFETY:` | ❌ Never |
| Use `unwrap()` / `expect()` in library crates | ❌ Never |
| Create a crate not listed in SPECS.md | ❌ Never — flag first |
| Commit with a failing test | ❌ Never |
| Skip `cargo test --workspace` before committing | ❌ Never |

---

## Phase Gate Reference

You may not begin any task in Phase N+1 until the Phase N gate test passes.

| Phase | What it builds | Gate test | Run with |
|---|---|---|---|
| 1 — Interpreter | Lexer, parser, evaluator, REPL | `tests/phase1/repl_smoke.rs` | `cargo test --test repl_smoke` |
| 2 — FFI | C interop, dynamic library loading | `tests/phase2/ffi_roundtrip.rs` | `cargo test --test ffi_roundtrip` |
| 3 — Type Checker | Full TypeScript type system | `tests/phase3/type_errors.rs` | `cargo test --test type_errors` |
| 4 — React Renderer | React 19 → SDL3 native window | `tests/phase4/render_window.rs` | `cargo test --test render_window` |
| 5 — Codegen | LLVM IR → native binary | `tests/phase5/native_binary.rs` | `cargo test --test native_binary` |

---

## Output Format

When completing a task, always output a summary in this exact format:

```
## Task Complete — TASK-NNN (FR-XXX-NNN)

**What I did:**
- [concise bullet list of changes, one line each]

**Files modified:**
- crates/tsnat-xxx/src/file.rs — [why]

**Files created:**
- crates/tsnat-xxx/src/new_file.rs — [why]

**Tests added:**
- test_function_name — [what it proves]

**Flags for human review:**
- [anything uncertain, risky, or requiring approval — or "None"]
```

If no changes were made:

```
## No Changes Made
[Reason — what already existed that satisfied the requirement]
```

---

## The Agent Mantra

> **"Read before writing. Test before committing. One FR at a time. Find it before you build it. When uncertain, pause."**
