# TypeScript Native (`tsnat`)

`tsnat` is an experimental engine built in Rust to execute TypeScript natively, completely bypassing traditional JavaScript VM environments (V8/SpiderMonkey) and DOM abstractions. It compiles parsed TypeScript ASTs natively to execution environments and bridges to the operating system using C-ABI mechanisms.

By providing a unified TypeScript parsing/lexing frontend and an execution backend built directly against SDL3 and Yoga layout bindings, `tsnat` allows developers to write React code and deploy highly performant, directly rendered desktop applications at native speeds.

## Features Currently Supported
- **Custom Lexing & Parsing (`tsnat-lex`, `tsnat-parse`)**: Blazing fast recursive descent parser customized with AST configurations mapping explicitly to native execution contexts.
- **Dynamic FFI Engine (`tsnat-ffi`)**: Full capability to wrap closure functions directly intercepting arbitrary OS `.so`/`.dll` library functions across the sandbox barrier via `declare native function` extensions!
- **Zero-Config React 19 Embedding**: Evaluator is designed to natively reconcile JSX rendering against an embedded React 19 engine.
- **SDL3 & Yoga Native Subsystems**: UI definitions get strictly mapped onto SDL3 window frame buffers utilizing Facebook's robust flexbox layout engine.

## Prerequisites

Before bootstrapping the ecosystem, ensure your system has the standard toolchains required:

- **Rust (1.75+)** - Recommended: Rustup (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **CMake & Make**
- **Git**

### Automatically Setup SDL3

As `tsnat` targets the bleeding-edge stable branch of `SDL3` for its GUI layers, it is currently absent from most Linux package managers. 

We've provided a simple build script that effortlessly clones, configures, and installs `SDL3.4.4+` alongside its required sub-dependencies on your machine! Run this before attempting to build the crate.

```bash
# Requires sudo authentication for installing apt dependencies and binaries to /usr/local
sudo ./tools/setup-sdl3.sh
```

## Running the Codebase

All crates are isolated natively inside the Cargo Workspace. 

### Build Everything
To compile the root project along with its crates (`tsnat-react`, `tsnat-eval`, `tsnat-lex`, etc):
```bash
cargo build --release
```

### Running Tests
As `tsnat-cli` is currently an internal development stub, all language and integration validation tests are safely verified via the Rust testing suite:

**Run all AST layout, string allocations, and mathematical bounds:**
```bash
cargo test --workspace
```

**Run End-to-End Native Code Bridging Tests (FFI Boundaries):**
```bash
# Will programmatically construct a dummy c dynamic library and execute TS boundary checking
cargo test -p tsnat-eval --test eval_ffi
```

## Creating Custom React Components (Upcoming)
In the upcoming Phase 5 deliverables, you will be able to simply execute:

```bash
tsnat run ./App.tsx
```

Where `App.tsx` natively integrates `<View>` and `<Text>` structs into the background `tsnat-react` C-ABI binder.

## Project Architecture
- `crates/tsnat-lex`: Source Code Tokenization layer
- `crates/tsnat-parse`: Recursive Descent AST Parser
- `crates/tsnat-eval`: The TypeScript evaluation environment.
- `crates/tsnat-ffi`: System dependency loading and pointer allocations wrapper `libffi`.
- `crates/tsnat-react`: The Native React rendering and Yoga-Flexbox bridging SDK.
