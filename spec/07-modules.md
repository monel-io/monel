# 7. Modules

A module is a single `.mn` file. The module system maps filesystem paths to module paths and controls which symbols are visible across module boundaries.

---

## 7.1 File Extensions

| Extension   | Purpose                                        |
|-------------|------------------------------------------------|
| `.mn`       | Module source (contracts, types, implementation) |
| `.mn.test`  | Module tests                                   |

Every module is a single `.mn` file containing both contracts and implementation. The `.mn.test` file is optional.

---

## 7.2 Directory Structure

A Monel project follows a fixed directory layout:

```
my-project/
  monel.project              # required: project manifest
  monel.team                 # optional: team/org conventions
  monel.policy               # optional: enforced rules
  .monel/
    index.json               # auto-generated: query index
    cache/                   # auto-generated: compilation cache
    tools/                   # auto-generated: project-scoped tool binaries
  src/
    main.mn                  # entry point
    terminal/
      mod.mn                 # terminal module root
      mod.mn.test            # terminal module tests
      pty.mn                 # terminal/pty submodule
      pty.mn.test            # terminal/pty submodule tests
    renderer/
      mod.mn                 # renderer module root
      mod.mn.test            # renderer module tests
```

### 7.2.1 Module Root Files

Each directory under `src/` that contains Monel source files MUST have a `mod.mn` file. This file serves as the directory's module root and defines what the directory exports. The top-level entry point uses `main.mn` instead of `mod.mn`.

### 7.2.2 Nested Modules

Modules nest according to directory structure. The module path mirrors the filesystem path:

| Filesystem path              | Module path    |
|------------------------------|----------------|
| `src/main.mn`                | `main`         |
| `src/terminal/mod.mn`        | `terminal`     |
| `src/terminal/pty.mn`        | `terminal/pty` |
| `src/renderer/mod.mn`        | `renderer`     |

---

## 7.3 Visibility

Visibility is controlled by the `exports` block declared in the `.mn` file.

### 7.3.1 Exports

The `exports` block at the top of a `.mn` file declares the module's public API:

```
module terminal

exports:
  Terminal
  Config
  spawn_shell
  resize
```

Only items listed in `exports` are accessible from outside the module. Everything else is module-internal.

### 7.3.2 Internal Items

Items not listed in `exports` are internal. They can be grouped under an `internal` block for documentation clarity, but this is optional -- unlisted items are internal by default:

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

### 7.3.3 Re-exports

A module root (`mod.mn`) can re-export items from submodules:

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

### 7.3.4 Visibility Rules

1. `exports` items are visible to any module that imports this module.
2. Internal items are visible only within the same module file and its tests.
3. A `mod.mn` file can access internal items from submodules within the same directory only if those submodules explicitly grant `visible_to: parent`.
4. Test files (`.mn.test`) have access to all items of the module they test, including internal items.

```
# In pty.mn
module terminal/pty

exports:
  Pty
  PtyConfig

visible_to:
  parent: [RawPty, pty_buffer_size]
```

---

## 7.4 Import Syntax

Monel uses the `use` keyword for imports. All imports are explicit -- there are no wildcard imports.

### 7.4.1 Basic Import

```
use http/server {Config, serve}
use terminal/pty {Pty, PtyConfig}
use std/collections {Map, Vec}
```

The syntax is:

```
use <module-path> {<item1>, <item2>, ...}
```

### 7.4.2 Import Rules

1. **No wildcard imports.** `use http/server {*}` is a syntax error. Every imported symbol must be named explicitly.
2. **No aliased module imports.** You cannot `use http/server as srv`. Module paths are always fully qualified.
3. **Type and function imports share the same syntax.** There is no distinction between importing a type and importing a function.
4. **Aliased item imports** are permitted when name collisions would otherwise occur:

```
use http/server {Config as ServerConfig}
use db/pool {Config as DbConfig}
```

5. **Self-referential imports are forbidden.** A module cannot import from itself.

### 7.4.3 Greppability Guarantee

Because every import names its items explicitly, any occurrence of a symbol in the codebase can be traced to its definition by searching for either:
- Its declaration in a `.mn` file, or
- Its import via `use ... {SymbolName}`

The compiler maintains this property. If a symbol is used but not imported or locally defined, the compiler emits an error with a suggestion of which `use` statement to add.

### 7.4.4 Standard Library Imports

The standard library is imported from the `std` namespace:

```
use std/io {read_file, write_file}
use std/collections {BTreeMap, Deque}
use std/sync {Mutex, RwLock}
use std/async {spawn, Channel}
```

A minimal prelude is automatically in scope for every module. The prelude includes: `Int`, `Float`, `Bool`, `String`, `Char`, `Byte`, `Unit`, `Never`, `Option`, `Result`, `Vec`, `Array`, `Map`, `Set`. All other types must be imported explicitly.

---

## 7.5 Module as Public API

When a dependency is added to a project, consumers see only the dependency's exported function signatures and their contracts. Implementation details of dependencies are not accessible.

This means:
1. All public API documentation lives in the contracts attached to exported functions.
2. Exported signatures and contracts are the single source of truth for a module's API.
3. LLMs generating code against a dependency need only the exported contracts for full context.
4. `monel query "how do I make an HTTP request?"` searches exported contracts, not implementations.

### 7.5.1 Module Size Guidelines

The compiler enforces advisory size limits to keep modules reviewable:

| Threshold         | Default  | Description                          |
|-------------------|----------|--------------------------------------|
| Warning           | 500 lines | Module is getting large              |
| Suggested split   | 800 lines | Compiler suggests splitting          |

**Target range: 200-400 lines.** If a module grows beyond 500 lines, the compiler emits a warning suggesting it be split. These limits are advisory (warnings, not errors) unless a `monel.policy` file upgrades them to errors.

A `monel.team` file can override these thresholds via `max_module_lines` in the `[style]` section.

---

## 7.6 Project Manifest: `monel.project`

The project manifest is a TOML file at the project root.

### 7.6.1 Full Schema

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

[tools]
protoc = "25.1"
sqlx-cli = { version = "0.7", features = ["postgres"] }

[tools.dev]
cargo-flamegraph = "0.6"

[targets]
default = "native"
wasm = { features = ["no-threads"] }
native = { opt-level = 3 }

[llm]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
```

### 7.6.2 `[package]` Section

| Field         | Required | Description                                    |
|---------------|----------|------------------------------------------------|
| `name`        | Yes      | Package name, must match `[a-z][a-z0-9-]*`    |
| `version`     | Yes      | Semver version string                          |
| `edition`     | Yes      | Language edition year                          |
| `description` | No       | Human-readable description                     |
| `license`     | No       | SPDX license identifier                        |
| `repository`  | No       | Source repository URL                          |
| `authors`     | No       | List of author strings                         |

### 7.6.3 `[dependencies]` Section

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

### 7.6.4 `[tools]` and `[tools.dev]` Sections

The `[tools]` section declares project-level CLI tool dependencies -- executable binaries that the project's workflow requires (code generators, database migrators, linters, etc.). The `[tools.dev]` subsection declares tools needed only during development (profilers, benchmarking harnesses, etc.).

#### Manifest Syntax

```toml
[tools]
protoc = "25.1"
sqlx-cli = { version = "0.7", features = ["postgres"] }
flatc = { version = "24.3", source = "github:google/flatbuffers" }

[tools.dev]
cargo-flamegraph = "0.6"
tokei = "12.1"
```

Version strings follow the same semver syntax as `[dependencies]` (see Section 7.7.3). Each tool entry may use either the short form (version string) or the table form with additional fields:

| Field       | Required | Description                                          |
|-------------|----------|------------------------------------------------------|
| `version`   | Yes      | Semver version requirement                           |
| `source`    | No       | Registry, Git repository, or URL for the tool binary |
| `features`  | No       | Feature flags to enable when compiling from source   |
| `artifact`  | No       | Artifact name if it differs from the tool name       |

#### EBNF Grammar Fragment

```ebnf
tools_section      = "[tools]" , newline , { tool_entry , newline } ;
tools_dev_section  = "[tools.dev]" , newline , { tool_entry , newline } ;

tool_entry         = tool_name , "=" , ( version_string | tool_table ) ;
tool_name          = identifier ;
version_string     = quoted_string ;   (* semver requirement, same as dep versions *)

tool_table         = "{" , tool_field , { "," , tool_field } , "}" ;
tool_field         = version_field | source_field | features_field | artifact_field ;
version_field      = "version" , "=" , quoted_string ;
source_field       = "source" , "=" , quoted_string ;
features_field     = "features" , "=" , "[" , quoted_string , { "," , quoted_string } , "]" ;
artifact_field     = "artifact" , "=" , quoted_string ;
```

#### Installation and Scoping

Tools are project-scoped. `monel sync` installs them into `.monel/tools/` relative to the project root -- never into a global location. Different projects can pin different versions of the same tool without conflict.

```
.monel/
  tools/
    protoc           # v25.1, project-pinned
    sqlx-cli         # v0.7
    cargo-flamegraph # v0.6 (dev-only)
```

The `.monel/tools/` directory SHOULD be added to `.gitignore`. It can be reconstructed from `monel.lock`.

#### Binary Resolution Strategy

When installing a tool, `monel sync` uses the following strategy:

1. **Check cache.** If the exact version is already present in the global binary cache (`~/.monel/cache/tools/`), hardlink or copy it.
2. **Download prebuilt binary.** Query the registry or `source` for a prebuilt binary matching the host platform and architecture.
3. **Compile from source.** If no prebuilt binary is available, fetch the source and compile it. Feature flags from the manifest are passed to the build.

Prebuilt binaries are verified against SHA-256 checksums recorded in the registry.

#### `monel sync`

The `monel sync` command is the single entry point for making the project ready to build and run:

```bash
monel sync              # install deps + dev-deps + tools + dev-tools
monel sync --prod       # install deps only (no dev-deps, no tools)
```

Behavior:
- Resolves and installs library dependencies from `[dependencies]`.
- Resolves and installs dev dependencies from `[dev-dependencies]`.
- Resolves and installs tool binaries from `[tools]` and `[tools.dev]`.
- Updates `monel.lock` if the lock file is absent or stale.
- **Idempotent.** Running `monel sync` when everything is already installed is a fast no-op (sub-100ms). It compares lock file hashes against installed artifacts and skips work that is already done.

The `--prod` flag restricts installation to `[dependencies]` only. Dev dependencies, tools, and dev tools are skipped. This is intended for production container builds and CI release pipelines.

#### `monel run <tool>`

Invokes a tool using the project-pinned version from `.monel/tools/`:

```bash
monel run protoc --proto_path=proto/ --monel_out=src/generated/ schema.proto
monel run sqlx-cli migrate run
monel run cargo-flamegraph -- --bin my-terminal
```

`monel run <tool>` prepends `.monel/tools/` to `PATH` so that the tool resolves to the project-pinned version. Arguments after the tool name are forwarded verbatim.

If the tool is not installed, `monel run` prints an error directing the user to run `monel sync`.

#### Lockfile Coverage

`monel.lock` records the resolved versions and integrity hashes for everything:

- Library dependencies (`[dependencies]`).
- Dev dependencies (`[dev-dependencies]`).
- Tool binaries (`[tools]` and `[tools.dev]`).

A single lockfile produces reproducible builds and environments. See Section 7.10.1 for the lockfile lifecycle.

### 7.6.5 `[targets]` Section

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

### 7.6.6 `[llm]` Section

The LLM section configures AI-assisted authoring tools (`monel test --gen-llm-tests`, `monel suggest`, etc.).

```toml
[llm]
provider = "anthropic"          # LLM provider
model = "claude-sonnet-4-20250514"    # model identifier
api_key_env = "ANTHROPIC_API_KEY"  # env var containing the API key
base_url = "https://api.anthropic.com"  # optional: custom endpoint
timeout_seconds = 30            # optional: request timeout
max_retries = 3                 # optional: retry count
```

The toolchain never stores API keys directly. The `api_key_env` field names an environment variable read at invocation time.

Supported providers:
- `anthropic` -- Anthropic API
- `openai` -- OpenAI-compatible API
- `local` -- Local model (e.g., ollama)
- `custom` -- Custom endpoint (requires `base_url`)

---

## 7.7 Team Configuration: `monel.team`

The team configuration file captures organizational conventions. It is not enforced by the compiler directly but is used by `monel generate` and `monel refactor` to produce code matching team standards.

### 7.7.1 Full Schema

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
max_module_lines = 500
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

### 7.7.2 Section Details

**`[org]`** -- Organizational metadata used by the code generator to make architecture-appropriate decisions. These are freeform strings; the generator interprets them contextually.

**`[patterns]`** -- Lists of patterns to prefer or avoid. The generator uses these as soft constraints when producing implementations.

**`[style]`** -- Naming conventions and size limits. The `max_module_lines` value overrides the compiler's default warning threshold for module size.

**`[effects.custom]`** -- Organization-specific effects beyond the built-in set. Each custom effect has a description and severity level (`low`, `medium`, `high`, `critical`). Custom effects are tracked by the compiler just like built-in effects.

**`[generation]`** -- Configuration for `monel generate`. `system_prompt_extras` is appended to the system prompt when generating implementations. `style_references` lists files whose style should be used as examples.

---

## 7.8 Policy Configuration: `monel.policy`

The policy file defines enforced rules. Unlike `monel.team`, policy rules produce compiler errors (not just warnings).

### 7.8.1 Full Schema

```toml
[effects]
forbidden_combinations = [
  ["Db.write", "Http.send"],    # never do DB writes and HTTP sends in same function
  ["Fs.write", "Crypto.sign"],  # separation of concerns
]
require_explicit_effects = true  # every function must declare effects (no inference)
max_effect_depth = 3             # max transitive effect chain depth

[contracts]
require_contracts_for = [
  "auth/*",                      # authentication modules must have contracts
  "billing/*",                   # billing modules must have contracts
  "crypto/*",                    # crypto modules must have contracts
]

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

### 7.8.2 Section Details

**`[effects]`** -- Rules about effect usage.
- `forbidden_combinations`: pairs of effects that must never appear in the same function. If a function transitively produces both effects, the compiler emits an error.
- `require_explicit_effects`: when `true`, every function must declare its effects. The compiler will not infer effects.
- `max_effect_depth`: limits how deep transitive effect chains can go. A function calling a function calling a function with `Db.write` has depth 3.

**`[contracts]`** -- Rules about contract annotations.
- `require_contracts_for`: glob patterns matching modules where all public functions must have contracts (`requires:`, `ensures:`, `effects:`).

**`[security]`** -- Security-related rules.
- `taint_tracking`: enables compile-time taint analysis. Values from untrusted sources (HTTP input, file reads, environment variables) are marked as tainted and must be sanitized before use in sensitive operations.
- `require_auth_effect`: glob patterns matching modules that must declare the `Auth` effect.
- `pii_fields`: field names that the compiler flags as personally identifiable information. These fields trigger additional checks (e.g., must not be logged, must be encrypted at rest).
- `forbid_logging`: field names that must never appear in log statements. The compiler scans for these in string interpolations and log function calls.

---

## 7.9 Query Index: `.monel/index.json`

The `.monel/index.json` file is a pre-built search index generated by `monelc` during compilation. It enables instant responses from `monel query` without requiring a full project scan.

### 7.9.1 Index Contents

The index contains:
- All exported symbols with their module paths
- All contract clauses (`requires:`, `ensures:`, `effects:`, etc.)
- All type signatures
- All effect declarations
- All error variants
- Module dependency graph
- Symbol cross-references

### 7.9.2 Index Format

```json
{
  "version": 1,
  "generated_at": "2026-03-12T10:00:00Z",
  "modules": {
    "terminal": {
      "path": "src/terminal/mod.mn",
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

### 7.9.3 Index Lifecycle

1. The index is generated as a side effect of compilation.
2. `monel query` reads the index and does not require recompilation.
3. The index is invalidated when any `.mn` file's exported contracts change.
4. The `.monel/` directory SHOULD be added to `.gitignore`.
5. CI/CD pipelines can generate the index via `monel build` and cache it.

---

## 7.10 Dependency Resolution and Versioning

### 7.10.1 Version Resolution

Monel uses a SAT-solver-based dependency resolver (similar to Cargo/pub). The resolver:

1. Reads all `[dependencies]` from `monel.project`.
2. Fetches available versions from the registry.
3. Finds a compatible version set satisfying all constraints.
4. Writes the resolved versions to `monel.lock`.

The `monel.lock` file covers library dependencies, dev dependencies, and tool binaries (see Section 7.7.4). It MUST be committed to version control for applications. Libraries SHOULD NOT commit `monel.lock`.

### 7.10.2 Automatic Semver from Export Diffs

When publishing a package, the compiler can automatically determine the correct semver bump by diffing the current exported signatures and contracts against the previously published version:

| Change type                          | Semver bump |
|--------------------------------------|-------------|
| New exported symbol                  | Minor       |
| Removed exported symbol              | Major       |
| Changed function signature           | Major       |
| Changed type definition              | Major       |
| Added field to struct (with default) | Minor       |
| Added error variant                  | Minor       |
| Removed error variant                | Major       |
| Changed `doc:` description only      | Patch       |
| Changed implementation only          | Patch       |
| New module added                     | Minor       |
| Module removed                       | Major       |
| Added effect to function             | Major       |
| Removed effect from function         | Minor       |

This analysis is performed by `monel publish` and can be previewed with `monel semver-check`.

### 7.10.3 Export-Only Dependencies

When a dependency is fetched, only its exported signatures, contracts, and compiled artifacts are downloaded. Source implementation files are never distributed unless explicitly opted in. This means:

1. Dependency consumers see only the public API (exported contracts).
2. Proprietary implementations can be distributed as compiled artifacts with public exports.
3. The total download size is minimized.
4. LLMs generating code against a dependency need only the exported contracts.

### 7.10.4 Dependency Auditing

`monel audit` scans the dependency tree for:
- Known vulnerabilities (from a registry advisory database)
- Effect escalation (a dependency introduces effects not declared in its contracts)
- Policy violations (dependencies that violate the project's `monel.policy`)

---

## 7.11 Module Path Resolution

### 7.11.1 Resolution Algorithm

Given a `use` statement like `use terminal/pty {Pty}`, the compiler resolves the module path as follows:

1. **Local resolution:** Check `src/terminal/pty.mn` relative to the project root.
2. **Parent module re-exports:** Check if `terminal/mod.mn` re-exports `Pty`.
3. **Dependency resolution:** Check if `terminal` is a dependency name in `monel.project`.
4. **Standard library:** Check if the path begins with `std/`.
5. **Error:** If none match, emit a "module not found" error with suggestions.

### 7.11.2 Ambiguity Rules

- A local module and a dependency MUST NOT share the same root name. The compiler emits an error if `src/http/` exists and `http` is also listed in `[dependencies]`.
- If disambiguation is needed, local modules take precedence. Dependencies can be accessed via their full registry name: `use registry/http/server {serve}`.

### 7.11.3 Circular Dependencies

Circular dependencies between modules are forbidden. The compiler builds a module dependency graph and rejects cycles with a clear error message showing the cycle path:

```
error: circular dependency detected
  --> src/a/mod.mn
  |
  | a -> b -> c -> a
  |
  = help: extract shared types into a new module that both can depend on
```

---

## 7.12 Module Initialization

Modules do not have implicit initialization code. There are no `init()` functions that run at import time. All initialization is explicit:

```
fn create_terminal(config: Config) -> Result<Terminal, InitError>
  doc: "creates and initializes a terminal instance"
  effects: [Pty.open, Process.spawn]
  ...
```

If a module requires one-time setup, it exposes a factory function. The caller decides when initialization happens.

---

## 7.13 Conditional Compilation

Monel supports target-conditional code through the `when` attribute:

```
fn get_terminal_size() -> (UInt32, UInt32)
  doc: "returns the current terminal dimensions"
  when: target.os != "wasm"
  effects: [Pty.query]
  ...

fn get_terminal_size() -> (UInt32, UInt32)
  doc: "returns fixed dimensions for web terminals"
  when: target.os == "wasm"
  effects: []
  ...
```

Available conditions:
- `target.os` -- `"linux"`, `"macos"`, `"windows"`, `"wasm"`
- `target.arch` -- `"x86_64"`, `"aarch64"`, `"wasm32"`
- `target.feature` -- feature flags from `monel.project`

Conditional compilation is declared alongside contracts so that consumers of the module can see platform-specific behavior without reading the implementation.

---

## 7.14 Module Compilation Order

The compiler determines compilation order from the dependency graph:

1. Parse all `.mn` files to extract module declarations, exports, and imports.
2. Topologically sort the modules.
3. Compile modules in dependency order (leaves first).
4. Parity checking runs per-module after contracts and implementation are both compiled.

Parallel compilation is supported for modules that are not interdependent. The compiler automatically parallelizes across available CPU cores.
