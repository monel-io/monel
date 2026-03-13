# Monel Language Specification: Chapter 2 — Intent Syntax

**Version:** 0.1.0-draft
**Status:** Working Draft
**Domain:** monel.io
**Date:** 2026-03-12

---

## 2.1 Purpose of This Chapter

This chapter defines the complete syntax and semantics of Monel intent files (`.mn.intent`). Intent files declare *what* a program should do without specifying *how*. They are the primary artifact written and reviewed by humans and the primary input to the parity compiler's verification stages.

---

## 2.2 File Structure

An intent file has the extension `.mn.intent`. Its path within the project determines the module it belongs to. The file `src/auth/session.mn.intent` declares intent for the module `auth/session`, and corresponds to the implementation file `src/auth/session.mn`.

An intent file consists of a sequence of top-level declarations, separated by blank lines. Declarations may appear in any order. The file may begin with `use` declarations to import types referenced in signatures.

---

## 2.3 Tiered Verification Design

Monel intent uses a tiered system that controls the depth of compiler verification. The tier is specified per-declaration, not per-file — a single file may contain both lightweight and strict declarations.

### 2.3.1 Lightweight Tier (Default)

The lightweight tier is the default for all intent declarations. It requires minimal annotation and provides structural and effect-based verification.

**Keywords available:** `does:`, `fails:`, `effects:`, `edge_cases:`

**Compiler verification:**
- Structural parity (Stage 2): matching implementation exists with identical signature
- Effect verification (Stage 3): implementation effects are a subset of declared effects
- Error exhaustiveness (Stage 3): declared `fails:` variants are reachable and no undeclared variants are produced
- Semantic check (Stage 4, optional): `does:` and `edge_cases:` verified by LLM when configured

**Example:**
```
intent fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
  does: "adds element to top of stack"
  fails: Overflow
  effects: [Atomic.write]
```

### 2.3.2 Strict Tier (`@strict`)

The strict tier activates formal verification. It is applied by adding `@strict` after the function signature. Strict intent includes all lightweight keywords plus additional formal contract keywords.

**Additional keywords:** `requires:`, `ensures:`, `errors:`, `invariant:`, `panics:`, `complexity:`

**Compiler verification (in addition to lightweight):**
- Precondition verification (Stage 3): `requires:` clauses are translated to SMT queries and verified via Z3
- Postcondition verification (Stage 3): `ensures:` clauses are translated to SMT queries and verified via Z3
- Panic freedom (Stage 3): when `panics: never` is declared, the compiler proves no code path can panic
- Invariant preservation (Stage 3): `invariant:` clauses are verified at all public API boundaries
- Complexity verification (Stage 3): when `complexity:` is declared, benchmark harnesses are generated to validate bounds

**Example:**
```
intent fn push(self: mut Stack<T>, val: T) -> Result<(), StackError> @strict
  requires: true
  ensures:
    ok => self.size() == old(self.size()) + 1
    ok => self.peek() == Some(val)
    err => self.unchanged()
  errors:
    Overflow: "stack is at capacity"
  effects: [Atomic.write]
```

---

## 2.4 Lightweight Intent Keywords

### 2.4.1 `does:`

**Syntax:** `does: STRING`

**Semantics:** A single-line natural-language string describing the function's purpose. The string must be enclosed in double quotes.

**Verification:**
- Stage 2: presence checked (every `intent fn` should have a `does:` clause; warning if absent)
- Stage 4 (LLM, optional): the LLM reads the implementation and confirms it fulfills the stated purpose

**Constraints:**
- Maximum one `does:` clause per intent declaration
- The string should describe *what* the function does, not *how*
- The string should be a complete sentence fragment (no trailing period required)

**Examples:**
```
does: "authenticates a user and creates a session"
does: "returns the element at the top of the stack without removing it"
does: "splits a string into tokens using the given delimiter"
```

### 2.4.2 `fails:`

**Syntax:** `fails: IDENT ( , IDENT )*`

**Semantics:** A comma-separated list of error variant identifiers. These are the error cases that the function may produce. Each identifier must correspond to a variant of the function's error type (the `E` in `Result<T, E>`).

**Verification:**
- Stage 3: every listed variant must be reachable in the implementation (the compiler proves that at least one code path produces each variant)
- Stage 3: the implementation must not produce error variants not listed in `fails:` (exhaustiveness in both directions)
- If the function's return type is not `Result`, `fails:` is a compilation error

**Examples:**
```
fails: Overflow
fails: InvalidCredentials, AccountLocked, RateLimited
fails: NotFound, PermissionDenied, Timeout
```

### 2.4.3 `effects:`

**Syntax:** `effects: [ EFFECT ( , EFFECT )* ]` or `effects: []`

**Semantics:** A list of side effect kinds that the function may perform. The list uses bracket notation. An empty list `[]` declares a pure function.

**Effect vocabulary (built-in — see Chapter 5 for the canonical reference):**

| Effect | Meaning |
|---|---|
| `Fs.read` | Read from filesystem |
| `Fs.write` | Write to filesystem |
| `Net.connect` | Open network connection |
| `Net.listen` | Listen for network connections |
| `Http.send` | Send HTTP request |
| `Http.receive` | Receive HTTP response |
| `Db.read` | Read from database |
| `Db.write` | Write to database |
| `Atomic.read` | Read shared atomic state |
| `Atomic.write` | Write shared atomic state |
| `Log.write` | Write to log output |
| `Env.read` | Read environment variables |
| `Env.write` | Write environment variables |
| `Process.spawn` | Spawn child process |
| `Signal` | Signal handling |
| `Crypto.hash` | Cryptographic hashing |
| `Crypto.encrypt` | Encryption |
| `Crypto.decrypt` | Decryption |
| `Crypto.sign` | Cryptographic signing |
| `Crypto.verify` | Signature verification |
| `Crypto.random` | Cryptographic random generation |
| `Auth.check` | Authorization check |
| `Time.read` | Read system clock |
| `Time.sleep` | Blocking sleep |
| `Gpu.draw` | Issue GPU draw calls |
| `Gpu.compute` | Issue GPU compute calls |
| `async` | Asynchronous operation |
| `unsafe` | Raw pointer / FFI operations |

Projects may define custom effect kinds in `monel.project` (see Section 7.8).

**Verification:**
- Stage 3: the compiler analyzes the implementation's call graph and determines the actual effects. Every actual effect must appear in the declared `effects:` list. If the implementation performs an effect not in the list, it is a compilation error.
- Effects are transitive: if `fn a` calls `fn b`, and `b` has `effects: [Db.read]`, then `a` must also declare `Db.read` (or a superset).

**Examples:**
```
effects: []
effects: [Db.read]
effects: [Db.read, Db.write, Log.write]
effects: [Fs.read, Fs.write, Net.connect]
```

### 2.4.4 `edge_cases:`

**Syntax:** `edge_cases:` followed by an indented list of `- STRING` entries.

**Semantics:** A list of natural-language descriptions of edge cases that the function must handle. Each entry is a string describing a scenario.

**Verification:**
- Stage 4 (LLM, optional): the LLM reads the implementation and confirms that each described edge case is handled
- No compiler verification in Stages 1-3 (edge cases are semi-formal)

**Example:**
```
edge_cases:
  - "empty input returns empty result"
  - "input exceeding MAX_SIZE is rejected before allocation"
  - "concurrent modification during iteration is detected"
```

---

## 2.5 Strict Intent Keywords

The following keywords are available only in `@strict` declarations. Using them without `@strict` is a compilation error.

### 2.5.1 `requires:`

**Syntax:** `requires: EXPR`

**Semantics:** A precondition expression that must hold when the function is called. The expression is written in Monel's expression syntax (a subset — see Section 2.13 for the contract expression grammar). It may reference function parameters and `self`.

**Verification:**
- Stage 3: the precondition is translated to an SMT formula. The compiler verifies that every call site satisfies the precondition, or that the precondition is checked at runtime (depending on compiler mode).

**Special value:** `requires: true` means "no precondition" — the function may be called in any state. This is distinct from omitting `requires:` entirely (which is only valid in lightweight intent).

**Examples:**
```
requires: true
requires: index < self.len()
requires: amount > 0 and balance >= amount
requires: not self.is_empty()
```

### 2.5.2 `ensures:`

**Syntax:** `ensures:` followed by an indented block of `CONDITION => EXPR` entries.

**Semantics:** Postcondition expressions that must hold when the function returns. Conditions distinguish between success and error paths.

**Condition forms:**
- `ok =>` — the postcondition holds when the function returns `Ok`
- `err =>` — the postcondition holds when the function returns `Err`
- `ok(val) =>` — binds the success value to `val` for use in the expression
- `err(e) =>` — binds the error value to `e` for use in the expression
- `always =>` — the postcondition holds regardless of return path

**Special functions in postconditions:**
- `old(expr)` — the value of `expr` before the function executed (captures pre-state)
- `self.unchanged()` — equivalent to `self == old(self)` (the receiver is not modified)
- `result` — the return value (in `always` clauses)

**Verification:**
- Stage 3: each postcondition is translated to an SMT formula. The compiler verifies that every return path through the implementation satisfies the relevant postconditions.

**Example:**
```
ensures:
  ok => self.size() == old(self.size()) + 1
  ok => self.peek() == Some(val)
  err => self.unchanged()
```

### 2.5.3 `errors:`

**Syntax:** `errors:` followed by an indented block of `IDENT : STRING` entries.

**Semantics:** Error variants with human-readable descriptions. This is the `@strict` equivalent of `fails:` — it provides the same exhaustiveness checking but adds documentation.

**Verification:**
- Stage 3: same as `fails:` — exhaustive checking in both directions
- Each variant identifier must correspond to a variant of the function's error type

**Note:** `errors:` and `fails:` are mutually exclusive in a single declaration. Use `fails:` in lightweight intent, `errors:` in strict intent.

**Example:**
```
errors:
  Overflow: "stack is at capacity"
  InvalidIndex: "index is out of bounds"
  Empty: "operation requires non-empty collection"
```

### 2.5.4 `invariant:`

**Syntax:** `invariant: EXPR`

This keyword appears in `intent type` declarations (not function declarations). It declares a property that must hold for all values of the type at all public API boundaries.

**Semantics:** The invariant expression may reference `self` (the value of the type). It must evaluate to a boolean. The compiler verifies that:
- Every constructor produces a value satisfying the invariant
- Every public method that returns `self` (or modifies `self` in place) preserves the invariant
- Internal methods may temporarily violate the invariant, but it must be restored before the function returns

**Example:**
```
intent type SortedList<T: Ord> @strict
  invariant: self.is_sorted()
  does: "a list that maintains sorted order at all times"
```

### 2.5.5 `panics: never`

**Syntax:** `panics: never`

**Semantics:** Declares that the function will never panic under any input. The compiler must prove this statically.

**Verification:**
- Stage 3: the compiler analyzes every code path in the implementation and proves that none can trigger a panic. This includes:
  - No unchecked array/slice indexing (must use `.get()` or bounds-check)
  - No `.unwrap()` on `Option` or `Result` (must pattern match)
  - No integer overflow in default (checked) mode
  - No stack overflow (recursive functions must have provably bounded depth, or the compiler rejects them)
  - No calls to functions that may panic (transitively)

**Example:**
```
intent fn lookup(self: Map<K, V>, key: K) -> Option<V> @strict
  requires: true
  ensures:
    ok(val) => self.contains_key(key) and self.get(key) == Some(val)
  panics: never
  effects: []
```

### 2.5.6 `complexity:`

**Syntax:** `complexity: COMPLEXITY_EXPR`

**Semantics:** An optional declaration of the function's time complexity. The complexity expression uses standard asymptotic notation.

**Complexity expression forms:**
- `O(1)` — constant time
- `O(n)` — linear time, where `n` is an implicit parameter (the "size" of the input)
- `O(n log n)` — linearithmic time
- `O(n^2)` — quadratic time
- `O(log n)` — logarithmic time
- Named parameters: `O(n * m)` where `n` and `m` are bound to specific parameters

**Verification:**
- Stage 3: the compiler generates benchmark harnesses that run the function with varying input sizes and checks that the measured growth rate is consistent with the declared complexity
- This is empirical verification, not proof — it may produce false positives on small inputs
- Complexity verification is optional and can be disabled with `--no-complexity-check`

**Example:**
```
intent fn sort(self: mut Vec<T>) -> () @strict
  requires: true
  ensures:
    always => self.is_sorted()
    always => self.len() == old(self.len())
  complexity: O(n log n)
  panics: never
  effects: []
```

---

## 2.6 Intent for Types

### 2.6.1 Refinement Types

Refinement types restrict a base type with a predicate. Values of a refinement type are guaranteed to satisfy the predicate at all times.

**Syntax:**
```
type NAME = BASE_TYPE where PREDICATE
```

**Semantics:** The predicate is a boolean expression over `value` (the implicit name for the value being constrained). The compiler verifies at every assignment point that the predicate is satisfied — either statically (via SMT) or by inserting a runtime check.

**Examples:**
```
type Port = Int where value >= 1 and value <= 65535
type NonEmptyString = String where value.len() > 0
type Percentage = Float where value >= 0.0 and value <= 100.0
type EvenInt = Int where value % 2 == 0
```

### 2.6.2 Algebraic Data Types

Intent may declare the shape of algebraic data types (enums) without specifying representation.

**Syntax:**
```
intent type NAME
  does: STRING
  variants:
    IDENT ( ( FIELDS ) )?
    ...
```

**Semantics:** The `variants:` block lists the data type's variants. Each variant may have named or positional fields. The compiler verifies that the implementation defines a type with exactly these variants.

**Example:**
```
intent type AuthError
  does: "errors that can occur during authentication"
  variants:
    InvalidCredentials
    AccountLocked(until: DateTime)
    RateLimited(retry_after: Duration)
    InternalError(message: String)
```

### 2.6.3 Struct Types

Intent may declare the fields of a struct type.

**Syntax:**
```
intent type NAME
  does: STRING
  fields:
    IDENT : TYPE
    ...
```

**Example:**
```
intent type Session
  does: "an authenticated user session"
  fields:
    user_id: UserId
    token: SessionToken
    created_at: DateTime
    expires_at: DateTime
```

### 2.6.4 Distinct Types

Distinct types are new types that are structurally identical to a base type but nominally distinct. They prevent accidental mixing of values with the same representation but different meanings.

**Syntax:**
```
type NAME = distinct BASE_TYPE
```

**Semantics:** A distinct type has the same in-memory representation as its base type but is a separate type in the type system. Explicit conversion functions are required to move between the distinct type and its base.

**Examples:**
```
type UserId = distinct Int
type SessionToken = distinct String
type PortNumber = distinct Int where value >= 1 and value <= 65535
```

---

## 2.7 Intent for Modules

Module intent declares the public API surface and internal boundaries of a module.

**Syntax:**
```
intent module NAME
  does: STRING
  exports:
    IDENT ( : KIND )?
    ...
  internal:
    IDENT ( : KIND )?
    ...
```

**Fields:**
- `does:` — purpose of the module
- `exports:` — names visible to other modules. Optional `KIND` annotation: `fn`, `type`, `trait`, `const`
- `internal:` — names that exist in the implementation but are not exported. This makes the boundary explicit — the compiler verifies that internal names are not accidentally exported

**Verification:**
- Stage 2: every name in `exports:` must exist in the implementation as a public item
- Stage 2: every name in `internal:` must exist in the implementation as a non-public item
- Stage 2: the implementation must not export names not listed in `exports:`

**Example:**
```
intent module auth/session
  does: "manages authenticated user sessions"
  exports:
    Session: type
    create: fn
    validate: fn
    revoke: fn
  internal:
    token_hash: fn
    cleanup_expired: fn
```

---

## 2.8 Intent for Traits

Trait intent declares the contract that implementations of a trait must satisfy.

**Syntax:**
```
intent trait NAME
  does: STRING
  methods:
    INTENT_FN_DECL
    ...
```

**Semantics:** Each method listed under `methods:` is a full `intent fn` declaration (without the `intent` keyword prefix — it is implied). The compiler verifies that every `impl` of the trait provides implementations for all declared methods with matching signatures.

**Example:**
```
intent trait Serializable
  does: "types that can be converted to and from a byte representation"
  methods:
    fn serialize(self) -> Result<Bytes, SerializeError>
      does: "converts this value to bytes"
      fails: SerializeError
      effects: []
    fn deserialize(data: Bytes) -> Result<Self, DeserializeError>
      does: "reconstructs a value from bytes"
      fails: DeserializeError
      effects: []
```

---

## 2.9 Diagrammatic Intent

Monel supports structured diagrammatic intent for expressing state machines, data flows, and protocol sequences. These are not comments or documentation — they are compiler-verified specifications.

### 2.9.1 State Machine Intent

**Syntax:**
```
intent state_machine NAME
  does: STRING
  states:
    IDENT ( : STRING )?
    ...
  initial: IDENT
  terminal:
    IDENT
    ...
  transitions:
    IDENT -> IDENT : STRING ( [ GUARD ] )?
    ...
```

**Semantics:**
- `states:` — the complete set of states. Optional string describes each state.
- `initial:` — the starting state (must be listed in `states:`)
- `terminal:` — states from which no transitions are required (must be listed in `states:`)
- `transitions:` — allowed state transitions. Each transition has a source state, target state, trigger description, and optional guard condition.

**Verification:**
- Stage 2: the implementation must define a type (enum) with variants corresponding to states
- Stage 2: transition functions must exist for each declared transition
- Stage 3: the compiler verifies:
  - **Reachability:** every state is reachable from the initial state
  - **Completeness:** every non-terminal state has at least one outgoing transition
  - **Determinism:** for each state, transitions with overlapping guards are flagged as warnings
  - **Terminal correctness:** terminal states have no outgoing transitions (warning if they do)
  - **No orphan transitions:** every transition references valid states

**Example:**
```
intent state_machine ConnectionState
  does: "lifecycle of a TCP connection"
  states:
    Idle: "not connected"
    Connecting: "handshake in progress"
    Connected: "established and ready"
    Draining: "graceful shutdown initiated"
    Closed: "connection terminated"
  initial: Idle
  terminal:
    Closed
  transitions:
    Idle -> Connecting: "initiate handshake"
    Connecting -> Connected: "handshake complete"
    Connecting -> Closed: "handshake failed"
    Connected -> Draining: "shutdown requested"
    Connected -> Closed: "connection lost"
    Draining -> Closed: "drain complete"
```

### 2.9.2 Data Flow Intent

**Syntax:**
```
intent data_flow NAME
  does: STRING
  nodes:
    IDENT : STRING
    ...
  edges:
    IDENT -> IDENT : TYPE ( [ ANNOTATION ] )?
    ...
```

**Semantics:**
- `nodes:` — processing stages in the data flow
- `edges:` — data flowing between stages, with type annotations
- Optional annotations: `[buffered]`, `[streaming]`, `[batched(N)]`

**Verification:**
- Stage 2: each node must correspond to a function or module in the implementation
- Stage 3: the types on edges must match the input/output types of the corresponding functions

**Example:**
```
intent data_flow RequestPipeline
  does: "HTTP request processing pipeline"
  nodes:
    parse: "parse raw bytes into request"
    authenticate: "verify credentials"
    authorize: "check permissions"
    handle: "execute business logic"
    serialize: "convert response to bytes"
  edges:
    parse -> authenticate: Request
    authenticate -> authorize: AuthenticatedRequest
    authorize -> handle: AuthorizedRequest
    handle -> serialize: Response
```

### 2.9.3 Protocol Sequence Intent

**Syntax:**
```
intent protocol NAME
  does: STRING
  participants:
    IDENT : STRING
    ...
  sequence:
    IDENT -> IDENT : TYPE ( : STRING )?
    ...
  constraints:
    STRING
    ...
```

**Semantics:**
- `participants:` — the actors in the protocol
- `sequence:` — ordered message exchanges, with types and optional descriptions
- `constraints:` — ordering and timing constraints expressed in natural language (LLM-verified)

**Example:**
```
intent protocol TlsHandshake
  does: "TLS 1.3 handshake sequence"
  participants:
    client: "initiating party"
    server: "responding party"
  sequence:
    client -> server: ClientHello: "supported cipher suites and random"
    server -> client: ServerHello: "selected cipher suite and random"
    server -> client: Certificate: "server certificate chain"
    server -> client: Finished: "server handshake complete"
    client -> server: Finished: "client handshake complete"
  constraints:
    - "ServerHello must be sent before Certificate"
    - "both Finished messages must be sent before application data"
```

---

## 2.10 Layout Intent (Designers)

Layout intent declares the spatial structure of a user interface without specifying implementation details.

**Syntax:**
```
intent layout NAME
  does: STRING
  regions:
    IDENT : STRING
      proportion: PROPORTION_EXPR
      min: SIZE_EXPR
      max: SIZE_EXPR
      overflow: OVERFLOW_MODE
    ...
  focus_order:
    IDENT
    ...
  responsive:
    CONDITION : RULE
    ...
```

**Fields:**
- `regions:` — named regions with sizing constraints
- `proportion:` — relative size (e.g., `1fr`, `2fr`, `auto`, `fixed(80)`)
- `min:` / `max:` — minimum and maximum sizes (e.g., `10 cols`, `24 rows`, `200 px`)
- `overflow:` — behavior when content exceeds region: `scroll`, `truncate`, `wrap`, `hide`
- `focus_order:` — ordered list of region names defining keyboard navigation order
- `responsive:` — rules that adapt layout to available space

**Example:**
```
intent layout EditorMain
  does: "main editor layout with sidebar and editing area"
  regions:
    sidebar: "file tree and outline"
      proportion: auto
      min: 20 cols
      max: 40 cols
      overflow: scroll
    editor: "primary editing area"
      proportion: 1fr
      min: 60 cols
      overflow: scroll
    statusbar: "status and mode indicator"
      proportion: fixed(1)
      overflow: truncate
  focus_order:
    editor
    sidebar
    statusbar
  responsive:
    width < 80: hide sidebar
    width < 40: hide statusbar
```

---

## 2.11 Interaction Intent (Designers)

Interaction intent declares state machines for user interface interactions.

**Syntax:**
```
intent interaction NAME
  does: STRING
  states:
    IDENT : STRING
    ...
  initial: IDENT
  transitions:
    IDENT -> IDENT : STRING
      trigger: TRIGGER_EXPR
      feedback: STRING
    ...
  accessibility:
    CONSTRAINT
    ...
```

**Fields:**
- `states:`, `initial:`, `transitions:` — same structure as `intent state_machine` (Section 2.9.1)
- `trigger:` — the user action that causes the transition (keyboard, mouse, gesture, timer)
- `feedback:` — the visible feedback the user receives during the transition
- `accessibility:` — accessibility requirements

**Trigger expression forms:**
- `key(KEYNAME)` — keyboard key press
- `key(MOD + KEYNAME)` — modified key press
- `click(REGION)` — mouse click on a region
- `hover(REGION)` — mouse hover over a region
- `timer(DURATION)` — time-based trigger
- `focus(REGION)` — region receives focus

**Example:**
```
intent interaction CommandPalette
  does: "command palette open/filter/select lifecycle"
  states:
    closed: "palette not visible"
    open: "palette visible, accepting input"
    filtered: "results filtered by input"
    selected: "item highlighted for execution"
  initial: closed
  transitions:
    closed -> open: "activate palette"
      trigger: key(Ctrl + P)
      feedback: "palette appears with empty input"
    open -> filtered: "type filter text"
      trigger: key(any)
      feedback: "results update in real time"
    filtered -> filtered: "refine filter"
      trigger: key(any)
      feedback: "results update in real time"
    filtered -> selected: "highlight item"
      trigger: key(Down) or key(Up)
      feedback: "highlight moves to next/previous item"
    selected -> closed: "execute command"
      trigger: key(Enter)
      feedback: "palette closes, command executes"
    open -> closed: "cancel"
      trigger: key(Escape)
      feedback: "palette closes, no action"
    filtered -> closed: "cancel"
      trigger: key(Escape)
      feedback: "palette closes, no action"
    selected -> closed: "cancel"
      trigger: key(Escape)
      feedback: "palette closes, no action"
  accessibility:
    - "all commands reachable via keyboard"
    - "palette announces result count to screen reader"
    - "selected item announced to screen reader"
```

---

## 2.12 Theme Intent (Designers)

Theme intent declares visual theming constraints.

**Syntax:**
```
intent theme NAME
  does: STRING
  palette:
    IDENT : COLOR_EXPR
    ...
  contrast:
    CONSTRAINT
    ...
  spacing:
    IDENT : SIZE_EXPR
    ...
```

**Fields:**
- `palette:` — named colors. Color expressions: hex (`#RRGGBB`, `#RRGGBBAA`), named (`red`, `blue`), HSL (`hsl(H, S, L)`), reference (`palette.NAME`)
- `contrast:` — minimum contrast ratio requirements (WCAG-aligned)
- `spacing:` — named spacing values for consistent layout

**Example:**
```
intent theme MonelDark
  does: "default dark theme for Monel terminal applications"
  palette:
    background: #1e1e2e
    foreground: #cdd6f4
    accent: #89b4fa
    error: #f38ba8
    warning: #fab387
    success: #a6e3a1
    surface: #313244
    overlay: #45475a
  contrast:
    - foreground on background >= 7.0
    - accent on background >= 4.5
    - error on background >= 4.5
  spacing:
    xs: 2 px
    sm: 4 px
    md: 8 px
    lg: 16 px
    xl: 32 px
```

---

## 2.13 Deploy Intent (SRE)

Deploy intent declares operational requirements for deployment.

**Syntax:**
```
intent deploy NAME
  does: STRING
  replicas: RANGE_EXPR
  health_check:
    endpoint: STRING
    interval: DURATION
    timeout: DURATION
    threshold: INT
  circuit_breaker:
    threshold: INT
    window: DURATION
    reset: DURATION
  resource_limits:
    cpu: RESOURCE_EXPR
    memory: RESOURCE_EXPR
    disk: RESOURCE_EXPR
  effects_budget:
    EFFECT : BUDGET_EXPR
    ...
  sla:
    availability: PERCENTAGE
    latency_p99: DURATION
    error_rate: PERCENTAGE
```

**Fields:**
- `replicas:` — range expression (e.g., `2..5`, `3`, `1..`)
- `health_check:` — health check configuration
- `circuit_breaker:` — circuit breaker parameters
- `resource_limits:` — resource constraints per instance
- `effects_budget:` — rate limits on effects (e.g., `Db.write: 1000/s`, `Net.connect: 100/s`)
- `sla:` — service level agreement targets

**Example:**
```
intent deploy AuthService
  does: "authentication service deployment constraints"
  replicas: 3..10
  health_check:
    endpoint: "/health"
    interval: 10s
    timeout: 2s
    threshold: 3
  circuit_breaker:
    threshold: 5
    window: 60s
    reset: 30s
  resource_limits:
    cpu: 2 cores
    memory: 512 MB
    disk: 1 GB
  effects_budget:
    Db.read: 10000/s
    Db.write: 1000/s
    Net.connect: 500/s
  sla:
    availability: 99.95%
    latency_p99: 200ms
    error_rate: 0.1%
```

---

## 2.14 Build Intent (DevOps)

Build intent declares the build pipeline and environment requirements.

**Syntax:**
```
intent build NAME
  does: STRING
  stages:
    IDENT : STRING
      depends_on: [ IDENT ( , IDENT )* ]
      effects: [ EFFECT ( , EFFECT )* ]
    ...
  env_requires:
    IDENT : CONSTRAINT
    ...
```

**Example:**
```
intent build MonelCI
  does: "continuous integration pipeline for Monel project"
  stages:
    lint: "check formatting and style"
      depends_on: []
      effects: [Fs.read]
    test: "run all tests"
      depends_on: [lint]
      effects: [Fs.read, Fs.write, Net.connect]
    build: "compile release binary"
      depends_on: [test]
      effects: [Fs.read, Fs.write]
    package: "create distributable artifact"
      depends_on: [build]
      effects: [Fs.read, Fs.write]
  env_requires:
    MONEL_VERSION: ">= 0.1.0"
    LLVM_VERSION: ">= 17.0"
    TARGET_ARCH: "aarch64 or x86_64"
```

---

## 2.15 Block-Level `@intent` Tags

Within implementation files (`.mn`), blocks may be annotated with `@intent` tags that cross-reference intent declarations. But intent files may also declare block-level intent for annotating sections within a function.

**Syntax (in `.mn.intent`):**
```
intent block NAME
  does: STRING
  within: FN_NAME
```

**Semantics:** Block-level intent provides documentation and semantic verification for subsections of a function. The implementation must contain a corresponding `@intent("NAME")` tag at the appropriate block.

**Example (in `.mn.intent`):**
```
intent block validate_password_strength
  does: "ensures password meets minimum complexity requirements"
  within: create_account
```

**Corresponding implementation (in `.mn`):**
```
fn create_account @intent("create_account")
  ...
  body:
    @intent("validate_password_strength")
    let strength = Password.check_strength(password)
    match strength
      | Weak => Err(AccountError.WeakPassword)
      | Strong => ...
```

---

## 2.16 Formal Grammar (EBNF)

The following EBNF grammar defines the complete syntax of `.mn.intent` files. This grammar is normative — any construct not derivable from this grammar is not valid Monel intent syntax.

### 2.16.1 Notation

```
(* EBNF notation used in this specification:                *)
(*   =       defines a production                            *)
(*   ,       concatenation                                   *)
(*   |       alternation                                     *)
(*   [ X ]   optional (zero or one X)                        *)
(*   { X }   repetition (zero or more X)                     *)
(*   ( X )   grouping                                        *)
(*   "x"     terminal string                                 *)
(*   'x'     terminal string (alternate quoting)             *)
(*   (* *)   comment                                         *)
(*   ? X ?   special sequence (described in prose)           *)
(*   - X     exception (everything except X)                 *)
(*   INDENT  increase indentation level by 2 spaces          *)
(*   DEDENT  decrease indentation level by 2 spaces          *)
(*   NL      newline character                               *)
```

### 2.16.2 Lexical Elements

```
letter          = "A" | "B" | ... | "Z" | "a" | "b" | ... | "z" ;
digit           = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
underscore      = "_" ;
ident           = ( letter | underscore ) , { letter | digit | underscore } ;
upper_ident     = ( "A" | "B" | ... | "Z" ) , { letter | digit | underscore } ;
lower_ident     = ( "a" | "b" | ... | "z" | underscore ) , { letter | digit | underscore } ;
path_ident      = ident , { "/" , ident } ;

(* Strings *)
string_char     = ? any Unicode character except '"' and '\' ?
                | "\\" , ( '"' | "\\" | "n" | "r" | "t" ) ;
string_literal  = '"' , { string_char } , '"' ;

(* Numbers *)
int_literal     = [ "-" ] , digit , { digit } ;
float_literal   = [ "-" ] , digit , { digit } , "." , digit , { digit } ;
number_literal  = int_literal | float_literal ;

(* Duration *)
duration_unit   = "ms" | "s" | "m" | "h" | "d" ;
duration        = number_literal , duration_unit ;

(* Size *)
size_unit       = "px" | "cols" | "rows" | "em" | "rem" | "%" ;
size_expr       = number_literal , size_unit ;

(* Resource *)
resource_unit   = "cores" | "MB" | "GB" | "TB" | "KB" ;
resource_expr   = number_literal , resource_unit ;

(* Percentage *)
percentage      = number_literal , "%" ;

(* Rate *)
rate_expr       = int_literal , "/" , ( "s" | "m" | "h" ) ;

(* Color *)
hex_color       = "#" , 6 * ( digit | "a" | "b" | "c" | "d" | "e" | "f"
                | "A" | "B" | "C" | "D" | "E" | "F" )
                , [ 2 * ( digit | "a" | "b" | "c" | "d" | "e" | "f"
                | "A" | "B" | "C" | "D" | "E" | "F" ) ] ;
hsl_color       = "hsl(" , number_literal , "," , number_literal , "," , number_literal , ")" ;
named_color     = ident ;
palette_ref     = "palette." , ident ;
color_expr      = hex_color | hsl_color | named_color | palette_ref ;

(* Range *)
range_expr      = int_literal , ".." , [ int_literal ] ;

(* Proportion *)
proportion_expr = ( int_literal , "fr" ) | "auto" | ( "fixed(" , int_literal , ")" ) ;

(* Overflow mode *)
overflow_mode   = "scroll" | "truncate" | "wrap" | "hide" ;

(* Comment *)
comment         = "#" , { ? any character except newline ? } ;
```

### 2.16.3 Type Expressions

```
type_expr       = simple_type
                | generic_type
                | tuple_type
                | function_type
                | result_type
                | option_type
                | reference_type ;

simple_type     = upper_ident
                | path_ident , "." , upper_ident ;

generic_type    = simple_type , "<" , type_expr , { "," , type_expr } , ">" ;

tuple_type      = "(" , type_expr , "," , type_expr , { "," , type_expr } , ")" ;

function_type   = "Fn" , "(" , [ type_expr , { "," , type_expr } ] , ")" , "->" , type_expr ;

result_type     = "Result" , "<" , type_expr , "," , type_expr , ">" ;

option_type     = "Option" , "<" , type_expr , ">" ;

reference_type  = ( "ref" | "mut" ) , type_expr ;

(* Self types *)
self_param      = "self"
                | "mut" , "self"
                | "self" , ":" , type_expr
                | "self" , ":" , "mut" , type_expr ;
```

### 2.16.4 Contract Expressions

Contract expressions are a subset of Monel expressions used in `requires:`, `ensures:`, `invariant:`, and refinement type predicates.

```
contract_expr   = or_expr ;

or_expr         = and_expr , { "or" , and_expr } ;

and_expr        = not_expr , { "and" , not_expr } ;

not_expr        = "not" , not_expr
                | comparison_expr ;

comparison_expr = additive_expr , [ comparison_op , additive_expr ] ;

comparison_op   = "==" | "!=" | "<" | "<=" | ">" | ">=" ;

additive_expr   = multiplicative_expr , { ( "+" | "-" ) , multiplicative_expr } ;

multiplicative_expr = unary_expr , { ( "*" | "/" | "%" ) , unary_expr } ;

unary_expr      = "-" , unary_expr
                | postfix_expr ;

postfix_expr    = primary_expr , { "." , ident , [ "(" , [ arg_list ] , ")" ] } ;

primary_expr    = ident
                | "self"
                | "value"                   (* used in refinement type predicates *)
                | "result"                  (* used in ensures: always clauses *)
                | "true"
                | "false"
                | number_literal
                | string_literal
                | "old" , "(" , contract_expr , ")"
                | "Some" , "(" , contract_expr , ")"
                | "None"
                | "(" , contract_expr , ")" ;

arg_list        = contract_expr , { "," , contract_expr } ;
```

### 2.16.5 Effect Expressions

```
effect_name     = upper_ident , "." , lower_ident ;
effect_list     = "[" , [ effect_name , { "," , effect_name } ] , "]" ;
```

### 2.16.6 Parameter Declarations

```
param           = ident , ":" , [ "mut" ] , type_expr ;
param_list      = param , { "," , param } ;
full_param_list = [ self_param , [ "," , param_list ] ]
                | param_list ;
```

### 2.16.7 Top-Level Intent File

```
intent_file     = { NL } , { use_decl , NL } , { NL }
                , { top_level_decl , { NL } } ;

use_decl        = "use" , path_ident , "{" , ident , { "," , ident } , "}" ;
```

### 2.16.8 Top-Level Declarations

```
top_level_decl  = intent_fn_decl
                | intent_type_decl
                | intent_module_decl
                | intent_trait_decl
                | intent_state_machine_decl
                | intent_data_flow_decl
                | intent_protocol_decl
                | intent_layout_decl
                | intent_interaction_decl
                | intent_theme_decl
                | intent_deploy_decl
                | intent_build_decl
                | intent_block_decl
                | refinement_type_decl
                | distinct_type_decl
                | comment ;
```

### 2.16.9 Function Intent

```
intent_fn_decl  = "intent" , "fn" , ident
                , "(" , [ full_param_list ] , ")"
                , [ "->" , type_expr ]
                , [ "@strict" ]
                , NL , INDENT
                , intent_fn_body
                , DEDENT ;

intent_fn_body  = { intent_fn_clause , NL } ;

intent_fn_clause = does_clause
                 | fails_clause
                 | effects_clause
                 | edge_cases_clause
                 | requires_clause         (* @strict only *)
                 | ensures_clause          (* @strict only *)
                 | errors_clause           (* @strict only *)
                 | panics_clause           (* @strict only *)
                 | complexity_clause       (* @strict only *)
                 | comment ;

does_clause     = "does:" , string_literal ;

fails_clause    = "fails:" , upper_ident , { "," , upper_ident } ;

effects_clause  = "effects:" , effect_list ;

edge_cases_clause = "edge_cases:" , NL , INDENT
                  , { "-" , string_literal , NL }
                  , DEDENT ;

requires_clause = "requires:" , contract_expr ;

ensures_clause  = "ensures:" , NL , INDENT
                , { ensures_arm , NL }
                , DEDENT ;

ensures_arm     = ensures_condition , "=>" , contract_expr ;

ensures_condition = "ok"
                  | "err"
                  | "ok" , "(" , ident , ")"
                  | "err" , "(" , ident , ")"
                  | "always" ;

errors_clause   = "errors:" , NL , INDENT
                , { upper_ident , ":" , string_literal , NL }
                , DEDENT ;

panics_clause   = "panics:" , "never" ;

complexity_clause = "complexity:" , complexity_expr ;

complexity_expr = "O" , "(" , complexity_term , ")" ;

complexity_term = "1"
                | ident
                | ident , "^" , int_literal
                | "log" , ident
                | ident , "log" , ident
                | complexity_term , "*" , complexity_term ;
```

### 2.16.10 Type Intent

```
intent_type_decl = "intent" , "type" , upper_ident
                 , [ "<" , type_param_list , ">" ]
                 , [ "@strict" ]
                 , NL , INDENT
                 , intent_type_body
                 , DEDENT ;

type_param_list = type_param , { "," , type_param } ;

type_param      = upper_ident , [ ":" , upper_ident , { "+" , upper_ident } ] ;

intent_type_body = { intent_type_clause , NL } ;

intent_type_clause = does_clause
                   | variants_clause
                   | fields_clause
                   | invariant_clause     (* @strict only *)
                   | comment ;

variants_clause = "variants:" , NL , INDENT
                , { variant_decl , NL }
                , DEDENT ;

variant_decl    = upper_ident , [ "(" , variant_fields , ")" ] ;

variant_fields  = variant_field , { "," , variant_field } ;

variant_field   = ident , ":" , type_expr ;

fields_clause   = "fields:" , NL , INDENT
                , { ident , ":" , type_expr , NL }
                , DEDENT ;

invariant_clause = "invariant:" , contract_expr ;
```

### 2.16.11 Refinement and Distinct Types

```
refinement_type_decl = "type" , upper_ident , "=" , type_expr
                     , "where" , contract_expr ;

distinct_type_decl   = "type" , upper_ident , "=" , "distinct" , type_expr
                     , [ "where" , contract_expr ] ;
```

### 2.16.12 Module Intent

```
intent_module_decl = "intent" , "module" , path_ident
                   , NL , INDENT
                   , intent_module_body
                   , DEDENT ;

intent_module_body = { intent_module_clause , NL } ;

intent_module_clause = does_clause
                     | exports_clause
                     | internal_clause
                     | comment ;

exports_clause  = "exports:" , NL , INDENT
                , { ident , [ ":" , kind_keyword ] , NL }
                , DEDENT ;

internal_clause = "internal:" , NL , INDENT
                , { ident , [ ":" , kind_keyword ] , NL }
                , DEDENT ;

kind_keyword    = "fn" | "type" | "trait" | "const" ;
```

### 2.16.13 Trait Intent

```
intent_trait_decl = "intent" , "trait" , upper_ident
                  , [ "<" , type_param_list , ">" ]
                  , NL , INDENT
                  , intent_trait_body
                  , DEDENT ;

intent_trait_body = { intent_trait_clause , NL } ;

intent_trait_clause = does_clause
                    | methods_clause
                    | comment ;

methods_clause  = "methods:" , NL , INDENT
                , { trait_method_decl , NL }
                , DEDENT ;

trait_method_decl = "fn" , ident
                  , "(" , [ full_param_list ] , ")"
                  , [ "->" , type_expr ]
                  , [ "@strict" ]
                  , NL , INDENT
                  , intent_fn_body
                  , DEDENT ;
```

### 2.16.14 State Machine Intent

```
intent_state_machine_decl = "intent" , "state_machine" , upper_ident
                          , NL , INDENT
                          , intent_sm_body
                          , DEDENT ;

intent_sm_body  = { intent_sm_clause , NL } ;

intent_sm_clause = does_clause
                 | sm_states_clause
                 | sm_initial_clause
                 | sm_terminal_clause
                 | sm_transitions_clause
                 | comment ;

sm_states_clause = "states:" , NL , INDENT
                 , { ident , [ ":" , string_literal ] , NL }
                 , DEDENT ;

sm_initial_clause = "initial:" , ident ;

sm_terminal_clause = "terminal:" , NL , INDENT
                   , { ident , NL }
                   , DEDENT ;

sm_transitions_clause = "transitions:" , NL , INDENT
                      , { sm_transition , NL }
                      , DEDENT ;

sm_transition   = ident , "->" , ident , ":" , string_literal
                , [ "[" , contract_expr , "]" ] ;
```

### 2.16.15 Data Flow Intent

```
intent_data_flow_decl = "intent" , "data_flow" , upper_ident
                      , NL , INDENT
                      , intent_df_body
                      , DEDENT ;

intent_df_body  = { intent_df_clause , NL } ;

intent_df_clause = does_clause
                 | df_nodes_clause
                 | df_edges_clause
                 | comment ;

df_nodes_clause = "nodes:" , NL , INDENT
                , { ident , ":" , string_literal , NL }
                , DEDENT ;

df_edges_clause = "edges:" , NL , INDENT
                , { df_edge , NL }
                , DEDENT ;

df_edge         = ident , "->" , ident , ":" , type_expr
                , [ "[" , ident , "]" ] ;
```

### 2.16.16 Protocol Intent

```
intent_protocol_decl = "intent" , "protocol" , upper_ident
                     , NL , INDENT
                     , intent_proto_body
                     , DEDENT ;

intent_proto_body = { intent_proto_clause , NL } ;

intent_proto_clause = does_clause
                    | proto_participants_clause
                    | proto_sequence_clause
                    | proto_constraints_clause
                    | comment ;

proto_participants_clause = "participants:" , NL , INDENT
                          , { ident , ":" , string_literal , NL }
                          , DEDENT ;

proto_sequence_clause = "sequence:" , NL , INDENT
                      , { proto_message , NL }
                      , DEDENT ;

proto_message   = ident , "->" , ident , ":" , type_expr
                , [ ":" , string_literal ] ;

proto_constraints_clause = "constraints:" , NL , INDENT
                         , { "-" , string_literal , NL }
                         , DEDENT ;
```

### 2.16.17 Layout Intent

```
intent_layout_decl = "intent" , "layout" , upper_ident
                   , NL , INDENT
                   , intent_layout_body
                   , DEDENT ;

intent_layout_body = { intent_layout_clause , NL } ;

intent_layout_clause = does_clause
                     | layout_regions_clause
                     | layout_focus_order_clause
                     | layout_responsive_clause
                     | comment ;

layout_regions_clause = "regions:" , NL , INDENT
                      , { layout_region , NL }
                      , DEDENT ;

layout_region   = ident , ":" , string_literal , NL , INDENT
                , [ "proportion:" , proportion_expr , NL ]
                , [ "min:" , size_expr , NL ]
                , [ "max:" , size_expr , NL ]
                , [ "overflow:" , overflow_mode , NL ]
                , DEDENT ;

layout_focus_order_clause = "focus_order:" , NL , INDENT
                          , { ident , NL }
                          , DEDENT ;

layout_responsive_clause = "responsive:" , NL , INDENT
                         , { responsive_rule , NL }
                         , DEDENT ;

responsive_rule = contract_expr , ":" , ident , ident ;
    (* e.g., "width < 80: hide sidebar" *)
```

### 2.16.18 Interaction Intent

```
intent_interaction_decl = "intent" , "interaction" , upper_ident
                        , NL , INDENT
                        , intent_interaction_body
                        , DEDENT ;

intent_interaction_body = { intent_interaction_clause , NL } ;

intent_interaction_clause = does_clause
                          | sm_states_clause
                          | sm_initial_clause
                          | interaction_transitions_clause
                          | interaction_accessibility_clause
                          | comment ;

interaction_transitions_clause = "transitions:" , NL , INDENT
                               , { interaction_transition , NL }
                               , DEDENT ;

interaction_transition = ident , "->" , ident , ":" , string_literal
                       , NL , INDENT
                       , "trigger:" , trigger_expr , NL
                       , "feedback:" , string_literal , NL
                       , DEDENT ;

trigger_expr    = trigger_atom , { "or" , trigger_atom } ;

trigger_atom    = "key" , "(" , key_expr , ")"
                | "click" , "(" , ident , ")"
                | "hover" , "(" , ident , ")"
                | "timer" , "(" , duration , ")"
                | "focus" , "(" , ident , ")" ;

key_expr        = ident , { "+" , ident }
                | "any" ;

interaction_accessibility_clause = "accessibility:" , NL , INDENT
                                 , { "-" , string_literal , NL }
                                 , DEDENT ;
```

### 2.16.19 Theme Intent

```
intent_theme_decl = "intent" , "theme" , upper_ident
                  , NL , INDENT
                  , intent_theme_body
                  , DEDENT ;

intent_theme_body = { intent_theme_clause , NL } ;

intent_theme_clause = does_clause
                    | theme_palette_clause
                    | theme_contrast_clause
                    | theme_spacing_clause
                    | comment ;

theme_palette_clause = "palette:" , NL , INDENT
                     , { ident , ":" , color_expr , NL }
                     , DEDENT ;

theme_contrast_clause = "contrast:" , NL , INDENT
                      , { "-" , string_literal , NL }
                          (* contrast rules are structured strings:       *)
                          (* "foreground on background >= 7.0"             *)
                      , DEDENT ;

theme_spacing_clause = "spacing:" , NL , INDENT
                     , { ident , ":" , size_expr , NL }
                     , DEDENT ;
```

### 2.16.20 Deploy Intent

```
intent_deploy_decl = "intent" , "deploy" , upper_ident
                   , NL , INDENT
                   , intent_deploy_body
                   , DEDENT ;

intent_deploy_body = { intent_deploy_clause , NL } ;

intent_deploy_clause = does_clause
                     | deploy_replicas_clause
                     | deploy_health_check_clause
                     | deploy_circuit_breaker_clause
                     | deploy_resource_limits_clause
                     | deploy_effects_budget_clause
                     | deploy_sla_clause
                     | comment ;

deploy_replicas_clause = "replicas:" , ( int_literal | range_expr ) ;

deploy_health_check_clause = "health_check:" , NL , INDENT
                           , "endpoint:" , string_literal , NL
                           , "interval:" , duration , NL
                           , "timeout:" , duration , NL
                           , "threshold:" , int_literal , NL
                           , DEDENT ;

deploy_circuit_breaker_clause = "circuit_breaker:" , NL , INDENT
                              , "threshold:" , int_literal , NL
                              , "window:" , duration , NL
                              , "reset:" , duration , NL
                              , DEDENT ;

deploy_resource_limits_clause = "resource_limits:" , NL , INDENT
                              , { ident , ":" , resource_expr , NL }
                              , DEDENT ;

deploy_effects_budget_clause = "effects_budget:" , NL , INDENT
                             , { effect_name , ":" , rate_expr , NL }
                             , DEDENT ;

deploy_sla_clause = "sla:" , NL , INDENT
                  , { ident , ":" , ( percentage | duration ) , NL }
                  , DEDENT ;
```

### 2.16.21 Build Intent

```
intent_build_decl = "intent" , "build" , upper_ident
                  , NL , INDENT
                  , intent_build_body
                  , DEDENT ;

intent_build_body = { intent_build_clause , NL } ;

intent_build_clause = does_clause
                    | build_stages_clause
                    | build_env_requires_clause
                    | comment ;

build_stages_clause = "stages:" , NL , INDENT
                    , { build_stage , NL }
                    , DEDENT ;

build_stage     = ident , ":" , string_literal , NL , INDENT
                , "depends_on:" , "[" , [ ident , { "," , ident } ] , "]" , NL
                , "effects:" , effect_list , NL
                , DEDENT ;

build_env_requires_clause = "env_requires:" , NL , INDENT
                          , { ident , ":" , string_literal , NL }
                          , DEDENT ;
```

### 2.16.22 Block Intent

```
intent_block_decl = "intent" , "block" , ident
                  , NL , INDENT
                  , does_clause , NL
                  , "within:" , ident , NL
                  , DEDENT ;
```

---

## 2.17 Semantic Constraints (Non-Syntactic Rules)

The following constraints are enforced by the compiler but are not expressible in the EBNF grammar:

1. **`@strict` exclusivity.** The keywords `requires:`, `ensures:`, `errors:`, `invariant:`, `panics:`, and `complexity:` are only valid inside `@strict` declarations. Using them in lightweight declarations is a compilation error.

2. **`fails:` / `errors:` mutual exclusion.** A single `intent fn` declaration may use `fails:` or `errors:` but not both. `errors:` is the strict equivalent of `fails:` with added descriptions.

3. **Clause uniqueness.** Each clause keyword may appear at most once per declaration, with the exception of `edge_cases:` list items (which are multiple entries under a single `edge_cases:` keyword).

4. **Type consistency.** Type expressions in intent signatures must be valid types — either built-in types, types defined in imported modules, or types defined in the same intent file.

5. **Effect vocabulary.** Effect names in `effects:` lists must be drawn from the built-in effect vocabulary (Section 2.4.3) or from custom effects defined in `monel.policy`.

6. **Identifier correspondence.** Identifiers in `fails:` and `errors:` must correspond to variants of the function's declared error type. The compiler resolves this through the return type.

7. **Contract expression purity.** Expressions in `requires:`, `ensures:`, and `invariant:` must be pure — they may not call functions with effects, perform I/O, or modify state. The `old()` function is a compile-time construct, not a runtime call.

8. **Indentation consistency.** All indentation must use exactly 2 spaces per level. Tabs are a lexical error. Mixed indentation is a lexical error.

9. **File-module correspondence.** The path of a `.mn.intent` file determines the module it describes. The file `src/foo/bar.mn.intent` must declare intent for module `foo/bar`. Mismatches are compilation errors.

10. **Refinement type predicates.** The variable `value` in refinement type `where` clauses refers to the value being constrained. No other free variables are allowed (parameters of the enclosing scope are not in scope).

---

## 2.18 Reserved Keywords

The following identifiers are reserved in `.mn.intent` files and may not be used as user-defined names:

```
intent    fn        type      module    trait     state_machine
data_flow protocol  layout    interaction         theme
deploy    build     block     does      fails     effects
edge_cases          requires  ensures   errors    invariant
panics    complexity          exports   internal  variants
fields    methods   states    initial   terminal  transitions
nodes     edges     participants        sequence  constraints
regions   focus_order         responsive          palette
contrast  spacing   replicas  health_check        circuit_breaker
resource_limits     effects_budget      sla       stages
env_requires        within    where     distinct  use
self      mut       ref       true      false     never
ok        err       always    old       Some      None
value     result    strict
```
