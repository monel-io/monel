# 7. Modules, Visibility, and Dependencies

## 7.1 Overview

Monel uses a file-based module system where each module consists of an intent file and an implementation file. The module system is designed around three principles:

1. **Greppability** -- every symbol usage can be traced to its source with a simple text search.
2. **Context window efficiency** -- module boundaries are sized so that an LLM can hold a complete module (intent + implementation) within a single context window.
3. **Intent as API** -- a module's public API is defined entirely by its intent file; the implementation file is an internal concern.

## 7.2 File Extensions

| Extension      | Purpose                          | Written by  | Read by     |
|----------------|----------------------------------|-------------|-------------|
| `.mn.intent`   | Module specification (API, contracts, effects) | Humans      | Compiler, LLMs, humans |
| `.mn`          | Module implementation            | LLMs or humans | Compiler   |
| `.mn.test`     | Module tests                     | LLMs or humans | Test runner |

Every module MUST have exactly one `.mn.intent` file and exactly one `.mn` file. The `.mn.test` file is optional but strongly recommended. A `.mn` file without a corresponding `.mn.intent` file is a compilation error. A `.mn.intent` file without a corresponding `.mn` file is valid during development (the compiler reports it as "unimplemented").

## 7.3 Directory Structure

A Monel project follows a fixed directory layout:

```
my-project/
  monel.project              # required: project manifest
  monel.team                 # optional: team/org conventions
  monel.policy               # optional: enforced rules
  .monel/
    index.json               # auto-generated: query index
    cache/                   # auto-generated: compilation cache
  src/
    main.mn.intent           # entry point intent
    main.mn                  # entry point implementation
    terminal/
      mod.mn.intent          # terminal module root intent
      mod.mn                 # terminal module root implementation
      mod.mn.test            # terminal module tests
      pty.mn.intent          # terminal/pty submodule intent
      pty.mn                 # terminal/pty submodule implementation
      pty.mn.test            # terminal/pty submodule tests
    renderer/
      mod.mn.intent          # renderer module root intent
      mod.mn                 # renderer module root implementation
      mod.mn.test            # renderer module tests
```

### 7.3.1 Module Root Files

Each directory under `src/` that contains Monel source files MUST have a `mod.mn.intent` and `mod.mn` pair. These serve as the directory's module root and define what the directory exports. The top-level entry point uses `main.mn.intent` and `main.mn` instead of `mod`.

### 7.3.2 Nested Modules

Modules nest according to directory structure. The module path mirrors the filesystem path:

| Filesystem path                | Module path         |
|--------------------------------|---------------------|
| `src/main.mn`                  | `main`              |
| `src/terminal/mod.mn`          | `terminal`          |
| `src/terminal/pty.mn`          | `terminal/pty`      |
| `src/renderer/mod.mn`         | `renderer`          |

## 7.4 Visibility

Visibility is declared exclusively in the intent file. The implementation file does not contain visibility modifiers.

### 7.4.1 Exports

The `exports` block in a `.mn.intent` file declares the module's public API:

```
module terminal

exports:
  Terminal
  Config
  spawn_shell
  resize
```

Only items listed in `exports` are accessible from outside the module. Everything else is module-internal.

### 7.4.2 Internal Declarations

Items not listed in `exports` are internal. They can be explicitly grouped under an `internal` block for documentation clarity, but this is optional -- unlisted items are internal by default:

```
module terminal

exports:
  Terminal
  Config
  spawn_shell

internal:
  parse_escape_sequence
  buffer_pool
```

### 7.4.3 Re-exports

A module root (`mod.mn.intent`) can re-export items from submodules:

```
module terminal

exports:
  Terminal         # defined in this module
  Pty              # re-exported from terminal/pty
  PtyConfig        # re-exported from terminal/pty

re-exports:
  pty: [Pty, PtyConfig]
```

This allows `use terminal {Pty}` instead of requiring `use terminal/pty {Pty}`.

### 7.4.4 Visibility Rules

1. `exports` items are visible to any module that imports this module.
2. Internal items are visible only within the same module file and its tests.
3. A `mod.mn` file can access internal items from submodules within the same directory only if those submodules explicitly grant `visible_to: parent` in their intent.
4. Test files (`.mn.test`) have access to all internal items of the module they test.

```
# In pty.mn.intent
module terminal/pty

exports:
  Pty
  PtyConfig

visible_to:
  parent: [RawPty, pty_buffer_size]
```

## 7.5 Import Syntax

Monel uses the `use` keyword for imports. All imports are explicit -- there are no wildcard imports.

### 7.5.1 Basic Import

```
use http/server {Config, serve}
use terminal/pty {Pty, PtyConfig}
use std/collections {Map, Vec}
```

The syntax is:

```
use <module-path> {<item1>, <item2>, ...}
```

### 7.5.2 Import Rules

1. **No wildcard imports.** `use http/server {*}` is a syntax error. Every imported symbol must be named explicitly.
2. **No aliased module imports.** You cannot `use http/server as srv`. Module paths are always fully qualified.
3. **Type and function imports share the same syntax.** There is no distinction between importing a type and importing a function.
4. **Aliased item imports** are permitted when name collisions would otherwise occur:

```
use http/server {Config as ServerConfig}
use db/pool {Config as DbConfig}
```

5. **Self-referential imports are forbidden.** A module cannot import from itself.

### 7.5.3 Greppability Guarantee

Because every import names its items explicitly, any occurrence of a symbol in the codebase can be traced to its definition by searching for either:
- Its declaration in an intent file, or
- Its import via `use ... {SymbolName}`

This property is maintained by the compiler. If a symbol is used but not imported or locally defined, the compiler emits an error with a suggestion of which `use` statement to add.

### 7.5.4 Standard Library Imports

The standard library is imported from the `std` namespace:

```
use std/io {read_file, write_file}
use std/collections {BTreeMap, Deque}
use std/sync {Mutex, RwLock}
use std/async {spawn, Channel}
```

A minimal prelude is automatically in scope for every module. The prelude includes: `Int`, `Float`, `Bool`, `String`, `Char`, `Byte`, `Unit`, `Never`, `Option`, `Result`, `Vec`, `Array`, `Map`, `Set`. All other types must be imported explicitly.

## 7.6 Intent as Public API

When a dependency is added to a project, only the dependency's intent layer is visible to the consuming project. The implementation files of dependencies are never accessible.

This means:
1. All public API documentation lives in the intent file.
2. The intent file is the single source of truth for a module's contract.
3. LLMs generating code against a dependency need only the intent file for full context.
4. `monel query "how do I make an HTTP request?"` searches intent files, not implementations.

### 7.6.1 Intent File as Context

Intent files are designed to fit within LLM context windows. The compiler enforces advisory limits:

| File type      | Warning threshold | Hard limit | Rationale                        |
|----------------|-------------------|------------|----------------------------------|
| `.mn.intent`   | 300 lines         | None       | Should fit in one context window |
| `.mn`          | 1000 lines        | None       | Larger but still bounded         |

**Target range for intent files: 100-200 lines.** If an intent file grows beyond 300 lines, the compiler emits a warning suggesting the module be split. These limits are advisory (warnings, not errors) unless a `monel.policy` file upgrades them to errors.

## 7.7 Project Manifest: `monel.project`

The project manifest is a TOML file at the project root.

### 7.7.1 Full Schema

```toml
[package]
name = "my-terminal"
version = "0.3.1"
edition = "2026"
description = "A GPU-accelerated terminal emulator"
license = "MIT"
repository = "https://github.com/example/my-terminal"
authors = ["Jane Doe <jane@example.com>"]

[dependencies]
http = "1.2.0"
json = { version = "2.0", features = ["streaming"] }
gpu-render = { git = "https://github.com/example/gpu-render", rev = "abc123" }
local-lib = { path = "../local-lib" }

[dev-dependencies]
test-utils = "0.1.0"
bench = "1.0.0"

[targets]
default = "native"
wasm = { features = ["no-threads"] }
native = { opt-level = 3 }

[llm]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
```

### 7.7.2 `[package]` Section

| Field         | Required | Description                                    |
|---------------|----------|------------------------------------------------|
| `name`        | Yes      | Package name, must match `[a-z][a-z0-9-]*`    |
| `version`     | Yes      | Semver version string                          |
| `edition`     | Yes      | Language edition year                          |
| `description` | No       | Human-readable description                     |
| `license`     | No       | SPDX license identifier                        |
| `repository`  | No       | Source repository URL                          |
| `authors`     | No       | List of author strings                         |

### 7.7.3 `[dependencies]` Section

Dependencies can be specified in several forms:

```toml
# Version string (from registry)
http = "1.2.0"

# Table with options
json = { version = "2.0", features = ["streaming"] }

# Git dependency
gpu-render = { git = "https://github.com/example/gpu-render", branch = "main" }
gpu-render = { git = "https://github.com/example/gpu-render", rev = "abc123" }
gpu-render = { git = "https://github.com/example/gpu-render", tag = "v1.0" }

# Path dependency (for local development)
local-lib = { path = "../local-lib" }
```

Version strings follow semver. The version requirement syntax supports:
- `"1.2.0"` -- exactly 1.2.0
- `"^1.2.0"` -- compatible with 1.2.0 (>=1.2.0, <2.0.0)
- `"~1.2.0"` -- approximately 1.2.0 (>=1.2.0, <1.3.0)
- `">=1.0, <2.0"` -- range

### 7.7.4 `[targets]` Section

Monel supports multiple compilation targets:

| Target    | Description                              |
|-----------|------------------------------------------|
| `native`  | Native binary via LLVM backend           |
| `wasm`    | WebAssembly module                       |
| `wasi`    | WebAssembly System Interface             |

Each target can have per-target configuration:

```toml
[targets.native]
opt-level = 3            # 0-3
lto = true               # link-time optimization
debug-info = false

[targets.wasm]
features = ["no-threads"]
opt-level = "size"       # optimize for size
```

### 7.7.5 `[llm]` Section

The LLM section is optional. When absent, the compiler operates in offline mode -- all compilation stages except Stage 4 (semantic parity checking) function normally.

```toml
[llm]
provider = "anthropic"          # LLM provider
model = "claude-sonnet-4-20250514"    # model identifier
api_key_env = "ANTHROPIC_API_KEY"  # env var containing the API key
base_url = "https://api.anthropic.com"  # optional: custom endpoint
timeout_seconds = 30            # optional: request timeout
max_retries = 3                 # optional: retry count
```

The compiler never stores API keys directly. The `api_key_env` field names an environment variable that the compiler reads at invocation time.

Supported providers:
- `anthropic` -- Anthropic API
- `openai` -- OpenAI-compatible API
- `local` -- Local model (e.g., ollama)
- `custom` -- Custom endpoint (requires `base_url`)

## 7.8 Team Configuration: `monel.team`

The team configuration file captures organizational conventions. It is not enforced by the compiler directly but is used by `monel generate` and `monel refactor` to produce code matching team standards.

### 7.8.1 Full Schema

```toml
[org]
name = "Acme Corp"
architecture = "hexagonal"
database = "postgresql"
auth = "oauth2"
observability = "opentelemetry"

[patterns]
prefer = [
  "repository pattern for data access",
  "event sourcing for state changes",
  "circuit breaker for external calls",
]
avoid = [
  "global mutable state",
  "raw SQL outside repository modules",
  "synchronous I/O in request handlers",
]

[style]
naming = "snake_case"
type_naming = "PascalCase"
max_intent_lines = 200
max_impl_lines = 800
max_function_lines = 50

[effects.custom]
Billing = { description = "charges money", severity = "high" }
Audit = { description = "writes audit log", severity = "medium" }
Analytics = { description = "sends analytics event", severity = "low" }

[generation]
system_prompt_extras = """
Always use our internal error types from shared/errors.
Prefer Result over Option when the absence is an error condition.
"""
style_references = [
  "src/users/mod.mn",
  "src/billing/mod.mn",
]
```

### 7.8.2 Section Details

**`[org]`** -- Organizational metadata used by the code generator to make architecture-appropriate decisions. These are freeform strings; the generator interprets them contextually.

**`[patterns]`** -- Lists of patterns to prefer or avoid. The generator uses these as soft constraints when producing implementations.

**`[style]`** -- Naming conventions and size limits. The `max_intent_lines` and `max_impl_lines` values override the compiler's default warning thresholds.

**`[effects.custom]`** -- Organization-specific effects beyond the built-in set. Each custom effect has a description and severity level (`low`, `medium`, `high`, `critical`). Custom effects are tracked by the compiler just like built-in effects.

**`[generation]`** -- Configuration for `monel generate`. `system_prompt_extras` is appended to the system prompt when generating implementations. `style_references` lists files whose style should be used as examples.

## 7.9 Policy Configuration: `monel.policy`

The policy file defines enforced rules. Unlike `monel.team`, policy rules produce compiler errors (not just warnings).

### 7.9.1 Full Schema

```toml
[effects]
forbidden_combinations = [
  ["Db.write", "Http.send"],    # never do DB writes and HTTP sends in same function
  ["Fs.write", "Crypto.sign"],  # separation of concerns
]
require_explicit_effects = true  # every function must declare effects (no inference)
max_effect_depth = 3             # max transitive effect chain depth

[parity]
require_parity = true            # all modules must pass parity checking
strict_required_for = [
  "auth/*",                      # authentication modules must use @strict intent
  "billing/*",                   # billing modules must use @strict intent
  "crypto/*",                    # crypto modules must use @strict intent
]

[intent]
require_intent = true            # every .mn file must have a .mn.intent file
strict_modules = [
  "auth/*",
  "billing/*",
]

[generation]
require_review = true            # generated code must be reviewed before commit
auto_approve_threshold = 0.95    # auto-approve if parity score >= threshold

[security]
taint_tracking = true            # enable taint analysis for untrusted inputs
require_auth_effect = [
  "billing/*",                   # these modules must declare Auth effect
  "admin/*",
]
pii_fields = [
  "email",
  "phone",
  "ssn",
  "address",
  "date_of_birth",
]
forbid_logging = [
  "password",
  "api_key",
  "secret",
  "token",
  "ssn",
]
```

### 7.9.2 Section Details

**`[effects]`** -- Rules about effect usage.
- `forbidden_combinations`: pairs of effects that must never appear in the same function. If a function transitively produces both effects, the compiler emits an error.
- `require_explicit_effects`: when `true`, every function must declare its effects in the intent file. The compiler will not infer effects.
- `max_effect_depth`: limits how deep transitive effect chains can go. A function calling a function calling a function with `Db.write` has depth 3.

**`[parity]`** -- Rules about intent-implementation parity.
- `require_parity`: when `true`, the build fails if any module has unresolved parity violations.
- `strict_required_for`: glob patterns matching module paths that must use `@strict` intent (formal contracts, pre/post conditions, invariants).

**`[intent]`** -- Rules about intent files.
- `require_intent`: when `true`, every `.mn` file must have a corresponding `.mn.intent` file. Bare implementation files are an error.
- `strict_modules`: glob patterns matching modules that must use `@strict` intent.

**`[generation]`** -- Rules about AI-generated code.
- `require_review`: when `true`, `monel generate` marks generated files as "pending review" and they do not pass compilation until reviewed.
- `auto_approve_threshold`: parity score (0.0-1.0) above which generated code is auto-approved without human review.

**`[security]`** -- Security-related rules.
- `taint_tracking`: enables compile-time taint analysis. Values from untrusted sources (HTTP input, file reads, environment variables) are marked as tainted and must be sanitized before use in sensitive operations.
- `require_auth_effect`: glob patterns matching modules that must declare the `Auth` effect.
- `pii_fields`: field names that the compiler flags as personally identifiable information. These fields trigger additional checks (e.g., must not be logged, must be encrypted at rest).
- `forbid_logging`: field names that must never appear in log statements. The compiler scans for these in string interpolations and log function calls.

## 7.10 Query Index: `.monel/index.json`

The `.monel/index.json` file is a pre-built search index generated by `monelc` during compilation. It enables instant responses from `monel query` without requiring a full project scan.

### 7.10.1 Index Contents

The index contains:
- All exported symbols with their module paths
- All intent descriptions (`does:`, `ensures:`, etc.)
- All type signatures
- All effect declarations
- All error variants
- Module dependency graph
- Symbol cross-references

### 7.10.2 Index Format

```json
{
  "version": 1,
  "generated_at": "2026-03-12T10:00:00Z",
  "modules": {
    "terminal": {
      "path": "src/terminal/mod.mn.intent",
      "exports": ["Terminal", "Config", "spawn_shell", "resize"],
      "types": {
        "Terminal": {
          "kind": "struct",
          "description": "a terminal emulator instance",
          "fields": ["width: UInt32", "height: UInt32", "buffer: TerminalBuffer"]
        },
        "Config": {
          "kind": "struct",
          "description": "terminal configuration",
          "fields": ["font_size: f32", "color_scheme: ColorScheme"]
        }
      },
      "functions": {
        "spawn_shell": {
          "signature": "fn spawn_shell(config: Config) -> Result<Terminal, PtyError>",
          "does": "spawns a new shell process and connects it to a terminal",
          "effects": ["async", "Process.spawn", "Pty.open"]
        }
      },
      "dependencies": ["terminal/pty", "std/io", "std/async"]
    }
  },
  "symbols": {
    "Terminal": "terminal",
    "Pty": "terminal/pty",
    "Config": "terminal"
  }
}
```

### 7.10.3 Index Lifecycle

1. The index is generated as a side effect of compilation.
2. `monel query` reads the index and does not require recompilation.
3. The index is invalidated when any `.mn.intent` file changes.
4. The `.monel/` directory SHOULD be added to `.gitignore`.
5. CI/CD pipelines can generate the index via `monel build` and cache it.

## 7.11 Dependency Resolution and Versioning

### 7.11.1 Version Resolution

Monel uses a SAT-solver-based dependency resolver (similar to Cargo/pub). The resolver:

1. Reads all `[dependencies]` from `monel.project`.
2. Fetches available versions from the registry.
3. Finds a compatible version set satisfying all constraints.
4. Writes the resolved versions to `monel.lock`.

The `monel.lock` file MUST be committed to version control for applications. Libraries SHOULD NOT commit `monel.lock`.

### 7.11.2 Automatic Semver from Intent Diffs

When publishing a package, the compiler can automatically determine the correct semver bump by diffing the current intent files against the previously published version:

| Change type                          | Semver bump |
|--------------------------------------|-------------|
| New exported symbol                  | Minor       |
| Removed exported symbol              | Major       |
| Changed function signature           | Major       |
| Changed type definition              | Major       |
| Added field to struct (with default) | Minor       |
| Added error variant                  | Minor       |
| Removed error variant                | Major       |
| Changed `does:` description only     | Patch       |
| Changed implementation only          | Patch       |
| New module added                     | Minor       |
| Module removed                       | Major       |
| Added effect to function             | Major       |
| Removed effect from function         | Minor       |

This analysis is performed by `monel publish` and can be previewed with `monel semver-check`.

### 7.11.3 Intent-Only Dependencies

When a dependency is fetched, only its intent files and compiled artifacts are downloaded. Source implementation files are never distributed unless explicitly opted in. This means:

1. Dependency consumers see only the public API (intent layer).
2. Proprietary implementations can be distributed as compiled artifacts with public intents.
3. The total download size is minimized.
4. LLMs generating code against a dependency need only the intent files.

### 7.11.4 Dependency Auditing

`monel audit` scans the dependency tree for:
- Known vulnerabilities (from a registry advisory database)
- Effect escalation (a dependency introduces effects not declared in its intent)
- Policy violations (dependencies that violate the project's `monel.policy`)

## 7.12 Module Path Resolution

### 7.12.1 Resolution Algorithm

Given a `use` statement like `use terminal/pty {Pty}`, the compiler resolves the module path as follows:

1. **Local resolution:** Check `src/terminal/pty.mn.intent` relative to the project root.
2. **Parent module re-exports:** Check if `terminal/mod.mn.intent` re-exports `Pty`.
3. **Dependency resolution:** Check if `terminal` is a dependency name in `monel.project`.
4. **Standard library:** Check if the path begins with `std/`.
5. **Error:** If none match, emit a "module not found" error with suggestions.

### 7.12.2 Ambiguity Rules

- A local module and a dependency MUST NOT share the same root name. The compiler emits an error if `src/http/` exists and `http` is also listed in `[dependencies]`.
- If disambiguation is needed, local modules take precedence. Dependencies can be accessed via their full registry name: `use registry/http/server {serve}`.

### 7.12.3 Circular Dependencies

Circular dependencies between modules are forbidden. The compiler builds a module dependency graph and rejects cycles with a clear error message showing the cycle path:

```
error: circular dependency detected
  --> src/a/mod.mn.intent
  |
  | a -> b -> c -> a
  |
  = help: extract shared types into a new module that both can depend on
```

## 7.13 Module Initialization

Modules do not have implicit initialization code. There are no `init()` functions that run at import time. All initialization is explicit:

```
# In intent
intent fn create_terminal(config: Config) -> Result<Terminal, InitError>
  does: "creates and initializes a terminal instance"
  effects: [Pty.open, Process.spawn]
```

If a module requires one-time setup, it must expose a factory function. The caller decides when initialization happens.

## 7.14 Conditional Compilation

Monel supports target-conditional code through the `when` attribute in intent:

```
intent fn get_terminal_size() -> (UInt32, UInt32)
  does: "returns the current terminal dimensions"
  when: target.os != "wasm"
  effects: [Pty.query]

intent fn get_terminal_size() -> (UInt32, UInt32)
  does: "returns fixed dimensions for web terminals"
  when: target.os == "wasm"
  effects: []
```

Available conditions:
- `target.os` -- `"linux"`, `"macos"`, `"windows"`, `"wasm"`
- `target.arch` -- `"x86_64"`, `"aarch64"`, `"wasm32"`
- `target.feature` -- feature flags from `monel.project`

Conditional compilation is declared in intent so that consumers of the module can see platform-specific behavior without reading the implementation.

## 7.15 Module Compilation Order

The compiler determines compilation order from the dependency graph:

1. Parse all `.mn.intent` files to build the module graph.
2. Topologically sort the modules.
3. Compile modules in dependency order (leaves first).
4. Parity checking runs per-module after both intent and implementation are compiled.

Parallel compilation is supported for modules that are not interdependent. The compiler automatically parallelizes across available CPU cores.
