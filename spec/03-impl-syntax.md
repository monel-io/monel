# 3. Implementation Syntax

**Version:** 0.1.0-draft
**Status:** Working Draft
**Domain:** monel.io
**Date:** 2026-03-18

---

## 3.1 Syntax Overview

### 3.1.1 Hybrid Syntax

Monel uses a hybrid syntax that combines indentation-based structure with minimal delimiters:

- **Indentation defines scope.** Blocks are delimited by increased indentation (2 spaces per level), similar to Python. There are no braces and no `end` keywords.
- **`|` for pattern match arms.** Pattern matching uses the pipe character, following the ML family tradition. Each arm is written `| PATTERN => EXPR`.
- **Dot notation for field access and method calls.** `object.field` and `object.method(args)`.
- **Parentheses only for function calls.** `f(x, y)`, never for grouping in expressions (use explicit `let` bindings to break up complex expressions).
- **Arrow `=>` for match arm bodies.** Each pattern match arm uses `=>` to separate the pattern from its body.

### 3.2.2 Small Keyword Set

Monel uses a deliberately small set of keywords to reduce the learning surface:

| Keyword | Purpose |
|---|---|
| `fn` | Function declaration |
| `let` | Immutable binding |
| `let mut` | Mutable binding |
| `match` | Pattern matching |
| `if` / `else` | Conditional expressions |
| `use` | Import declaration |
| `module` | Module declaration |
| `type` | Type declaration (struct, enum) |
| `trait` | Trait declaration |
| `impl` | Trait implementation |
| `try` | Error propagation |
| `unsafe` | Unsafe block |
| `async` / `await` | Asynchronous operations |
| `return` | Early return (discouraged; prefer expression-as-last-value) |
| `loop` | Infinite loop |
| `while` | Conditional loop |
| `for` / `in` | Iterator loop |
| `break` / `continue` | Loop control |
| `pub` | Public visibility |
| `self` | Receiver parameter |
| `mut` | Mutability marker |
| `ref` | Reference marker |
| `true` / `false` | Boolean literals |

### 3.2.3 Expression-Oriented

Every construct in Monel is an expression that produces a value. There are no statements. The value of a block is the value of its last expression. The value of an `if`/`else` is the value of the taken branch. The value of a `match` is the value of the matched arm.

### 3.2.4 One Canonical Form

There is exactly one way to express each construct. No syntactic sugar exists. No alternative spellings or shorthand forms are provided. This means:

- No `unless`: use `if not`
- No ternary operator: use `if`/`else`
- No implicit returns: the last expression in a block is its value (this is the only form)
- No operator overloading beyond trait-defined operators
- String interpolation via `"{expr}"` syntax: expressions inside braces within double-quoted strings are evaluated and converted to strings via the `Display` trait
- No list comprehensions: use `iter().map().collect()`
- No `do` notation: use explicit `let` bindings with `try`

### 3.2.5 Type Inference Within Functions

Type annotations are required on:
- Function parameters
- Function return types
- Type declarations (struct fields, enum variant fields)
- `const` declarations

Type annotations are inferred within:
- `let` bindings (may be annotated optionally)
- Closure parameters (when context provides enough information)
- Generic type arguments (when inferable from usage)

---

## 3.3 Edit-Friendly Syntax Rules

The following rules make `.mn` files easy to edit programmatically (by LLMs and refactoring tools):

### 3.3.1 Function Signatures as Unique Anchors

A function declaration always begins with `fn NAME` on its own line. The function name combined with its module path is globally unique. This means any function can be located by searching for the pattern `fn NAME`.

### 3.3.2 No Wildcard Imports

Imports always list the specific names being imported:

```
use http/server {Config, serve}
use std/collections {Map}
```

Never:
```
use http/server *       (* INVALID -- wildcard imports are a syntax error *)
```

This makes dependency analysis trivial: search for the name in `use` declarations.

### 3.3.3 Each `let` Binding on Its Own Line

```
let x = compute_x()
let y = compute_y()
```

Never:
```
let x = compute_x(); let y = compute_y()    (* INVALID -- no semicolons *)
```

### 3.3.4 No Significant Trailing Commas

- Single-line lists use commas: `[a, b, c]`
- Multi-line lists use newlines as separators (no trailing comma required or allowed)
- The formatter (`monel fmt`) determines whether a list is single-line or multi-line based on line length thresholds

### 3.3.5 Binary Operators Stay on the Same Line

When an expression spans multiple lines, the binary operator stays on the same line as the left operand:

```
let total = base_price +
  tax +
  shipping
```

Never:
```
let total = base_price
  + tax             (* INVALID -- operator must be on left operand's line *)
  + shipping
```

---

## 3.4 File Structure

An implementation file has the extension `.mn`. Its path within the project determines the module it belongs to. The file `src/auth/session.mn` defines the module `auth/session`.

A `.mn` file consists of:
1. An optional `module` declaration (if the module name differs from the file path, which is rare)
2. Zero or more `use` declarations
3. Zero or more top-level declarations (functions, types, traits, implementations, constants)

Top-level declarations are separated by blank lines. The order of declarations within a file does not affect semantics (forward references are allowed).

---

## 3.5 Module Declarations and Imports

### 3.5.1 Module Declaration

**Syntax:**
```
module PATH_IDENT
```

Optional. If omitted, the module name is derived from the file path relative to `src/`. Most files omit this declaration.

### 3.5.2 Import Declarations

**Syntax:**
```
use PATH_IDENT { IDENT , IDENT , ... }
```

Imports specific names from another module. Multiple names are comma-separated within braces. Each `use` declaration imports from exactly one module.

**Examples:**
```
use std/collections {Vec, Map, Entry}
use http/server {Config, serve, Request, Response}
use auth/session {Session, SessionError}
```

**Rules:**
- Wildcard imports (`use module *`) are a syntax error
- Aliased imports use `as`: `use std/collections {BTreeMap as OrderedMap}`
- Re-exports use `pub use`: `pub use auth/session {Session}`
- Relative imports use `self/` prefix: `use self/helpers {validate}`

---

## 3.6 Function Declarations

Functions are the primary building block of Monel programs. Every function declaration follows a fixed structure: signature, optional contract block, then body.

### 3.6.1 Basic Form

```
fn name(param: Type, param: Type) -> ReturnType
  contract_clause*
  body_expr
```

- `fn NAME(PARAMS) -> TYPE`: the function signature. Always on its own line. Parameters are comma-separated within parentheses. The return type follows `->`.
- **Contract clauses**: zero or more of `requires:`, `ensures:`, `effects:`, `panics:`, `doc:`. These appear between the signature and the body, each on its own indented line.
- **Body**: an indented block of expressions. The value of the last expression is the return value. No `body:` keyword is required; the body begins at the first indented line that is not a contract clause.

**Example:**

```
fn authenticate(creds: Credentials) -> Result<Session, AuthError>
  requires:
    creds.username.len > 0
    creds.password.len > 0
  ensures:
    ok => result.user_id > 0
    ok => result.expires_at > Clock.now()
    err(Locked) => Db.failed_attempts(creds.username) >= 5
  effects: [Db.read, Db.write, Crypto.verify, Log.write]
  panics: never

  let user = Db.find_user(creds.username)
  match user
    | None => Err(AuthError.InvalidCreds)
    | Some(u) =>
      if u.is_locked
        Err(AuthError.Locked)
      else
        let valid = Crypto.verify(creds.password, u.hash)
        if valid
          let session = try Session.create(u)
          Log.info("user authenticated", user_id: u.id)
          Ok(session)
        else
          Log.warn("failed login attempt", user_id: u.id)
          Err(AuthError.InvalidCreds)
```

### 3.6.2 Contract Clauses

Contract clauses appear between the signature and the body. They are optional and may appear in any order, though the conventional order is `doc:`, `requires:`, `ensures:`, `effects:`, `panics:`.

#### `doc:`

A natural-language description of the function's purpose:

```
fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
  doc: "adds element to top of stack"
  ...
```

The `doc:` clause is a quoted string. It describes *what* the function does, not *how*.

#### `requires:`

Preconditions that must hold when the function is called. Each condition is on its own indented line:

```
fn withdraw(account: Account, amount: Money) -> Result<Money, BankError>
  requires:
    amount > Money(0)
    account.balance >= amount
  ...
```

When a function declares `requires:` clauses, the compiler verifies (via SMT) that all call sites satisfy the preconditions.

#### `ensures:`

Postconditions that the function guarantees on return. Postconditions may be qualified by `ok` or `err(Variant)` for `Result`-returning functions:

```
fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
  ensures:
    ok => self.size() == old(self.size()) + 1
    ok => self.peek() == Some(val)
    err => self.unchanged()
  ...
```

The `old(expr)` function refers to the value of `expr` at function entry. The compiler verifies postconditions via SMT.

#### `effects:`

Declares the side effects the function may perform:

```
fn save_user(user: User) -> Result<Unit, DbError>
  effects: [Db.write, Log.write]
  ...
```

The `effects:` clause is optional. When omitted, the compiler infers effects from the function body. When present, the compiler checks that the inferred effects are a subset of the declared effects. Undeclared effects are a compilation error. A function with no `effects:` clause and no effectful calls in its body is pure.

See Chapter 5 for the full effect vocabulary and checking rules.

#### `panics:`

Declares panic behavior:

```
fn safe_divide(a: Int, b: Int) -> Result<Int, MathError>
  panics: never
  ...
```

`panics: never` instructs the compiler to prove that no code path in the function can panic. If the proof fails, compilation fails.

### 3.6.3 Compact Form

Trivial functions may use the compact one-liner syntax with `=`:

```
fn len(self: Stack<T>) -> Int = self.elements.len()

fn is_empty(self: Stack<T>) -> Bool = self.len() == 0

fn double(x: Int) -> Int = x * 2
```

The compact form places the entire function on a single line: `fn NAME(PARAMS) -> TYPE = EXPR`. It cannot include contract clauses. Use the block form for functions that need contracts.

### 3.6.4 Self Parameters

Methods (functions associated with a type) use `self` as the first parameter:

```
fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
  effects: [Atomic.write]

  if self.elements.len() >= self.capacity
    Err(StackError.Overflow)
  else
    self.elements.push(val)
    Ok(())
```

Self parameter forms:
- `self: TYPE`: immutable borrow
- `self: mut TYPE`: mutable borrow
- `self: owned TYPE`: takes ownership

### 3.6.5 Generic Functions

Generic type parameters are declared after the function name:

```
fn map<A, B>(self: List<A>, f: Fn(A) -> B) -> List<B>
  ...
```

Type parameter constraints use `:` syntax:

```
fn sort<T: Ord>(self: mut Vec<T>) -> ()
  ...
```

Multiple constraints use `+`:

```
fn serialize<T: Serialize + Debug>(value: T) -> Result<Bytes, SerializeError>
  ...
```

### 3.6.6 Visibility

Functions are private to their module by default. Public functions use the `pub` keyword:

```
pub fn authenticate(creds: Credentials) -> Result<Session, AuthError>
  ...
```

### 3.6.7 Async Functions

Async functions use the `async` keyword before `fn`:

```
async fn fetch(url: Url) -> Result<Response, HttpError>
  effects: [Net.connect]

  let conn = await Net.connect(url.host)
  let response = await conn.send(Request.get(url))
  Ok(response)
```

- `async fn` declares an asynchronous function that returns a `Future`
- `await EXPR` suspends execution until the future completes
- Async functions may only be called with `await` or by spawning a task

---

## 3.7 Expressions

All constructs in Monel bodies are expressions. The following sections define each expression form.

### 3.7.1 `let` Bindings

```
let NAME = EXPR
let NAME: TYPE = EXPR
let mut NAME = EXPR
let mut NAME: TYPE = EXPR
```

A `let` binding introduces a new name in the current scope. The type annotation is optional (inferred by default). `mut` allows reassignment.

Each `let` binding must be on its own line. The bound name is in scope for all subsequent expressions in the same block.

**Examples:**
```
let user = Db.find_user(username)
let mut count: Int = 0
let hash = Crypto.sha256(data)
```

### 3.7.2 `match` Expressions

```
match EXPR
  | PATTERN => EXPR
  | PATTERN => EXPR
  ...
```

A `match` expression evaluates the scrutinee expression and compares it against patterns in order. The first matching pattern's body is evaluated and becomes the value of the `match` expression.

**Pattern forms:**
- `IDENT`: variable binding (matches anything, binds the value)
- `_`: wildcard (matches anything, discards the value)
- `LITERAL`: literal match (integer, string, boolean)
- `CONSTRUCTOR(PATTERNS)`: constructor pattern with sub-patterns
- `CONSTRUCTOR`: nullary constructor
- `(P1, P2, ...)`: tuple pattern
- `IDENT if EXPR`: guarded pattern

**Rules:**
- Match must be exhaustive. The compiler verifies that all possible values of the scrutinee type are covered.
- Each arm is on its own line, prefixed with `|`
- Multi-line arm bodies are indented under the `=>`:

```
match result
  | Ok(value) =>
    let processed = transform(value)
    emit(processed)
  | Err(e) =>
    Log.error(e.message)
    Err(e)
```

**Examples:**
```
match user
  | None => Err(AuthError.InvalidCreds)
  | Some(u) =>
    if u.is_locked
      Err(AuthError.Locked)
    else
      Ok(Session.new(u))
```

### 3.7.3 `if` / `else` Expressions

```
if CONDITION
  EXPR
else
  EXPR
```

An `if`/`else` expression evaluates the condition and takes the corresponding branch. Both branches must have the same type (since `if`/`else` is an expression).

`if` without `else` is only allowed when the body has type `()`.

**Chained conditionals:**
```
if condition_a
  expr_a
else if condition_b
  expr_b
else
  expr_c
```

There is no `elif` keyword. Use `else if` (two keywords).

**Examples:**
```
if u.is_locked
  Err(AuthError.Locked)
else
  let valid = Crypto.verify(creds.password, u.hash)
  if valid
    Ok(Session.new(u))
  else
    Err(AuthError.InvalidCreds)
```

### 3.7.4 Function Calls

```
NAME(ARGS)
EXPR.NAME(ARGS)
MODULE.NAME(ARGS)
```

Function calls use parentheses around arguments. Arguments are comma-separated.

**Method call syntax:**
```
user.validate()
list.push(element)
Crypto.verify(password, hash)
```

**Named arguments (optional, for clarity):**
```
Server.configure(port: 8080, host: "localhost")
```

Named arguments are syntactic and do not change semantics. The compiler verifies that named arguments match parameter names.

### 3.7.5 Field Access

```
EXPR.FIELD
```

Dot notation accesses struct fields. No parentheses (fields are not function calls).

```
let name = user.name
let port = config.server.port
```

### 3.7.6 Closures / Lambdas

```
fn(PARAMS) -> EXPR
fn(PARAMS) ->
  EXPR
  ...
```

Closures use the `fn` keyword with an argument list and arrow. Closures capture variables from the enclosing scope.

**Parameter types in closures:**
- When the expected type is known (e.g., passed to a function with a typed parameter), parameter types may be omitted
- Otherwise, parameter types are required

**Examples:**
```
let doubled = list.map(fn(x) -> x * 2)

let filtered = list.filter(fn(item) ->
  item.score > threshold
)

let handler = fn(request: Request) ->
  let body = parse(request.body)
  Response.ok(body)
```

### 3.7.7 `try` Expressions

```
try EXPR
```

The `try` keyword propagates errors from `Result`-returning expressions. If the expression evaluates to `Err(e)`, the enclosing function immediately returns `Err(e)`. If it evaluates to `Ok(v)`, the `try` expression evaluates to `v`.

`try` is equivalent to Rust's `?` operator but uses a keyword for readability and edit-friendliness (keywords are easier for LLMs to generate and tools to parse than symbolic operators).

**Examples:**
```
let user = try Db.find_user(username)
let token = try generate_token(user)
let session = try Session.create(user, token)
Ok(session)
```

**Chained `try`:**
```
let result = try validate(try parse(try read_file(path)))
```

### 3.7.8 Literals

```
42                    (* integer *)
3.14                  (* float *)
"hello"               (* string *)
true                  (* boolean *)
false                 (* boolean *)
[1, 2, 3]            (* list literal *)
(1, "two", 3.0)      (* tuple literal *)
```

**Integer literal forms:**
```
42                    (* decimal *)
0xFF                  (* hexadecimal *)
0b1010                (* binary *)
0o755                 (* octal *)
1_000_000             (* underscores for readability *)
```

**String literal forms:**
```
"hello world"                   (* regular string *)
"line one\nline two"            (* escape sequences *)
"""                             (* multi-line string *)
  This is a multi-line
  string literal.
"""
```

### 3.7.9 Operators

Binary operators (all are left-associative unless noted):

| Operator | Meaning | Precedence |
|---|---|---|
| `*`, `/`, `%` | Multiply, divide, remainder | 7 |
| `+`, `-` | Add, subtract | 6 |
| `==`, `!=`, `<`, `<=`, `>`, `>=` | Comparison | 5 |
| `and` | Logical AND | 4 |
| `or` | Logical OR | 3 |
| `=` | Assignment (mutable bindings only) | 1 |

Unary operators:
| Operator | Meaning |
|---|---|
| `-` | Arithmetic negation |
| `not` | Logical negation |

There is no bitwise operator syntax at the language level. Bitwise operations use function calls: `Bits.and(a, b)`, `Bits.or(a, b)`, `Bits.shift_left(a, n)`.

### 3.7.10 Block Expressions

A block is a sequence of expressions at the same indentation level. The value of the block is the value of its last expression.

```
let result =
  let a = compute_a()
  let b = compute_b()
  combine(a, b)
```

### 3.7.11 `loop`, `while`, `for` Expressions

**Infinite loop:**
```
loop
  let event = try read_event()
  match event
    | Quit => break
    | Input(data) => process(data)
```

**Conditional loop:**
```
while not queue.is_empty()
  let item = queue.pop()
  process(item)
```

**Iterator loop:**
```
for item in collection
  process(item)
```

**Loop with accumulator (loop produces a value via `break`):**
```
let found = loop
  let candidate = try next()
  if candidate.matches(predicate)
    break candidate
```

### 3.7.12 `return` Expression

```
return EXPR
```

Early return from a function. Discouraged in favor of expression-as-last-value, but available when control flow requires it (e.g., early exit from a loop in a function).

### 3.7.13 `unsafe` Blocks

```
unsafe
  let ptr = Memory.allocate(size)
  Memory.write(ptr, data)
```

Unsafe blocks permit operations that the compiler cannot verify for safety:
- Raw pointer manipulation
- Foreign function calls
- Type reinterpretation

The `effects:` declaration must include any effects performed in unsafe blocks. The compiler tracks that unsafe operations are contained within `unsafe` blocks and does not allow them outside.

---

## 3.8 Type Declarations

### 3.8.1 Struct Types

```
type NAME
  FIELD: TYPE
  FIELD: TYPE
  ...
```

Structs are product types with named fields. Each field is on its own line.

**Example:**
```
type Session
  user_id: UserId
  token: SessionToken
  created_at: DateTime
  expires_at: DateTime
```

**Generic structs:**
```
type Stack<T>
  elements: Vec<T>
  capacity: Int
```

**Visibility:** Fields are private by default. Use `pub` to make them accessible outside the module:

```
type Config
  pub host: String
  pub port: Port
  secret_key: String
```

### 3.8.2 Enum Types (Algebraic Data Types)

```
type NAME
  | VARIANT
  | VARIANT(FIELDS)
  ...
```

Enums are sum types. Each variant is prefixed with `|`.

**Example:**
```
type AuthError
  | InvalidCredentials
  | AccountLocked(until: DateTime)
  | RateLimited(retry_after: Duration)
  | InternalError(message: String)
```

**Generic enums:**
```
type Result<T, E>
  | Ok(value: T)
  | Err(error: E)

type Option<T>
  | Some(value: T)
  | None
```

### 3.8.3 Type Aliases

```
type NAME = TYPE
```

Type aliases create a new name for an existing type. They are interchangeable with the original type (not distinct types).

```
type UserList = Vec<User>
type Callback = Fn(Event) -> ()
```

---

## 3.9 Trait Declarations

```
trait NAME
  fn METHOD_NAME(PARAMS) -> TYPE
  fn METHOD_NAME(PARAMS) -> TYPE
```

Traits declare a set of methods that types may implement. Trait method declarations have signatures but no bodies.

**Example:**
```
trait Serializable
  fn serialize(self: Self) -> Result<Bytes, SerializeError>
  fn deserialize(data: Bytes) -> Result<Self, DeserializeError>
```

**Trait with associated types:**
```
trait Iterator
  type Item

  fn next(self: mut Self) -> Option<Self.Item>
```

**Trait bounds:**
```
trait Sortable: Ord + Clone
  fn sort(self: mut Self) -> ()
```

---

## 3.10 Trait Implementations

```
impl TRAIT for TYPE
  fn METHOD_NAME(PARAMS) -> TYPE
    contract_clauses*
    body_expr
```

Implementations provide method bodies for a trait on a specific type.

**Example:**
```
impl Serializable for Session
  fn serialize(self: Session) -> Result<Bytes, SerializeError>
    let mut buf = Bytes.new()
    try buf.write_string(self.user_id.to_string())
    try buf.write_string(self.token.to_string())
    try buf.write_i64(self.created_at.timestamp())
    try buf.write_i64(self.expires_at.timestamp())
    Ok(buf)

  fn deserialize(data: Bytes) -> Result<Session, DeserializeError>
    let mut reader = Bytes.reader(data)
    let user_id = try UserId.parse(try reader.read_string())
    let token = try SessionToken.parse(try reader.read_string())
    let created_at = try DateTime.from_timestamp(try reader.read_i64())
    let expires_at = try DateTime.from_timestamp(try reader.read_i64())
    Ok(Session
      user_id: user_id
      token: token
      created_at: created_at
      expires_at: expires_at
    )
```

### 3.10.1 Inherent Implementations

Methods may be defined directly on a type without a trait:

```
impl Stack<T>
  fn new(capacity: Int) -> Stack<T>
    Stack
      elements: Vec.with_capacity(capacity)
      capacity: capacity

  fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
    effects: [Atomic.write]

    if self.elements.len() >= self.capacity
      Err(StackError.Overflow)
    else
      self.elements.push(val)
      Ok(())

  fn len(self: Stack<T>) -> Int = self.elements.len()

  fn is_empty(self: Stack<T>) -> Bool = self.len() == 0
```

---

## 3.11 Constants

```
const NAME: TYPE = EXPR
```

Constants are compile-time values. The expression must be evaluable at compile time.

```
const MAX_RETRIES: Int = 3
const DEFAULT_PORT: Port = Port(8080)
const VERSION: String = "0.1.0"
```

---

## 3.12 Struct Construction

Structs are constructed by name with field assignments:

```
let session = Session
  user_id: user.id
  token: generated_token
  created_at: DateTime.now()
  expires_at: DateTime.now().add(Duration.hours(24))
```

Single-line form for small structs:

```
let point = Point(x: 1.0, y: 2.0)
```

---

## 3.13 Full Example

The following example demonstrates a complete `.mn` file with its major features:

```
use auth/types {Credentials, AuthError, Session}
use db {Db}
use crypto {Crypto}
use log {Log}

impl AuthService
  pub fn authenticate(creds: Credentials) -> Result<Session, AuthError>
    doc: "authenticates a user and creates a session"
    requires:
      creds.username.len > 0
      creds.password.len > 0
    ensures:
      ok => result.user_id > 0
    effects: [Db.read, Db.write, Log.write]
    panics: never

    let user = Db.find_user(creds.username)
    match user
      | None => Err(AuthError.InvalidCreds)
      | Some(u) =>
        if u.is_locked
          Err(AuthError.Locked)
        else
          let valid = Crypto.verify(creds.password, u.hash)
          if valid
            let session = try Session.create(u)
            Log.info("user authenticated", user_id: u.id)
            Ok(session)
          else
            Log.warn("failed login attempt", user_id: u.id)
            Err(AuthError.InvalidCreds)

  pub fn revoke_session(self: mut AuthService, token: SessionToken) -> Result<(), AuthError>
    effects: [Db.write, Log.write]

    let session = try Db.find_session(token)
    match session
      | None => Err(AuthError.SessionNotFound)
      | Some(s) =>
        try Db.delete_session(s.id)
        Log.info("session revoked", session_id: s.id)
        Ok(())

  fn validate_token(token: SessionToken) -> Bool = token.len() == 64 and token.is_alphanumeric()
```

---

## 3.14 Formal Grammar (EBNF)

The following EBNF grammar defines the complete syntax of `.mn` files. This grammar is normative.

### 3.14.1 Notation

```
(* EBNF notation -- same conventions as Chapter 2, Section 2.16.1 *)
(*   INDENT  increase indentation level by 2 spaces               *)
(*   DEDENT  decrease indentation level by 2 spaces               *)
(*   NL      newline character                                     *)
```

### 3.14.2 Lexical Elements

```
letter          = "A" | "B" | ... | "Z" | "a" | "b" | ... | "z" ;
digit           = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
hex_digit       = digit | "a" | "b" | "c" | "d" | "e" | "f"
                | "A" | "B" | "C" | "D" | "E" | "F" ;
underscore      = "_" ;

ident           = ( letter | underscore ) , { letter | digit | underscore } ;
upper_ident     = ( "A" | "B" | ... | "Z" ) , { letter | digit | underscore } ;
lower_ident     = ( "a" | "b" | ... | "z" | underscore ) , { letter | digit | underscore } ;
path_ident      = ident , { "/" , ident } ;

(* String literals *)
string_char     = ? any Unicode character except '"' and '\' ?
                | "\\" , ( '"' | "\\" | "n" | "r" | "t" | "0" ) ;
string_literal  = '"' , { string_char } , '"'
                | '"""' , NL , { ? any character ? } , NL , '"""' ;

(* Interpolated strings -- expressions inside braces are evaluated *)
interpolated_string = '"' , { string_char | ( "{" , expr , "}" ) } , '"' ;

(* Integer literals *)
dec_literal     = digit , { digit | "_" } ;
hex_literal     = "0x" , hex_digit , { hex_digit | "_" } ;
bin_literal     = "0b" , ( "0" | "1" ) , { "0" | "1" | "_" } ;
oct_literal     = "0o" , ( "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" )
                , { "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "_" } ;
int_literal     = [ "-" ] , ( dec_literal | hex_literal | bin_literal | oct_literal ) ;

(* Float literals *)
float_literal   = [ "-" ] , dec_literal , "." , dec_literal ;

(* Number literals *)
number_literal  = float_literal | int_literal ;

(* Boolean literals *)
bool_literal    = "true" | "false" ;

(* Comments *)
comment         = "#" , { ? any character except newline ? } ;
```

### 3.14.3 Type Expressions

```
type_expr       = simple_type
                | qualified_type
                | generic_type
                | tuple_type
                | function_type
                | ref_type
                | mut_type ;

simple_type     = upper_ident ;

qualified_type  = path_ident , "." , upper_ident ;

generic_type    = ( simple_type | qualified_type )
                , "<" , type_expr , { "," , type_expr } , ">" ;

tuple_type      = "(" , type_expr , "," , type_expr , { "," , type_expr } , ")" ;

function_type   = "Fn" , "(" , [ type_expr , { "," , type_expr } ] , ")"
                , "->" , type_expr ;

ref_type        = "ref" , type_expr ;

mut_type        = "mut" , type_expr ;

(* Type parameters for generic declarations *)
type_param      = upper_ident , [ ":" , type_constraint ] ;
type_constraint = upper_ident , { "+" , upper_ident } ;
type_param_list = type_param , { "," , type_param } ;

(* Self type *)
self_type       = "Self" ;
```

### 3.14.4 Effect Expressions

```
effect_name     = upper_ident , "." , lower_ident ;
effect_list     = "[" , [ effect_name , { "," , effect_name } ] , "]" ;
```

### 3.14.5 Top-Level File Structure

```
source_file     = { NL }
                , [ module_decl , NL , { NL } ]
                , { use_decl , NL }
                , { NL }
                , { top_level_decl , { NL } } ;

module_decl     = "module" , path_ident ;

use_decl        = [ "pub" ] , "use" , use_path , "{" , use_names , "}" ;

use_path        = [ "self/" ] , path_ident ;

use_names       = use_name , { "," , use_name } ;

use_name        = ident , [ "as" , ident ] ;
```

### 3.14.6 Top-Level Declarations

```
top_level_decl  = fn_decl
                | type_decl
                | trait_decl
                | impl_decl
                | const_decl
                | comment ;
```

### 3.14.7 Function Declarations

```
fn_decl         = [ "pub" ] , [ "async" ] , "fn" , ident
                , [ "<" , type_param_list , ">" ]
                , "(" , [ param_list ] , ")" , "->" , type_expr
                , ( compact_body | block_body ) ;

compact_body    = "=" , expr , NL ;

block_body      = NL , INDENT
                , { contract_clause }
                , expr_block
                , DEDENT ;

param_list      = full_param , { "," , full_param } ;

full_param      = self_param | param ;

param           = ident , ":" , [ "mut" ] , type_expr ;

self_param      = "self" , ":" , [ "mut" | "owned" ] , type_expr ;

contract_clause = doc_clause
                | requires_clause
                | ensures_clause
                | effects_clause
                | panics_clause ;

doc_clause      = "doc:" , string_literal , NL ;

requires_clause = "requires:" , NL , INDENT
                , { expr , NL }
                , DEDENT ;

ensures_clause  = "ensures:" , NL , INDENT
                , { ensures_cond , NL }
                , DEDENT ;

ensures_cond    = [ ( "ok" | "err" | "err" , "(" , upper_ident , ")" ) , "=>" ] , expr ;

effects_clause  = "effects:" , effect_list , NL ;

panics_clause   = "panics:" , "never" , NL ;
```

### 3.14.8 Expressions

```
expr_block      = { ( let_expr | expr ) , NL } ;

expr            = assign_expr ;

assign_expr     = or_expr , [ "=" , or_expr ] ;

or_expr         = and_expr , { "or" , and_expr } ;

and_expr        = not_expr , { "and" , not_expr } ;

not_expr        = "not" , not_expr
                | comparison_expr ;

comparison_expr = additive_expr , [ comparison_op , additive_expr ] ;

comparison_op   = "==" | "!=" | "<" | "<=" | ">" | ">=" ;

additive_expr   = multiplicative_expr , { ( "+" | "-" ) , multiplicative_expr } ;

multiplicative_expr = unary_expr , { ( "*" | "/" | "%" ) , unary_expr } ;

unary_expr      = "-" , unary_expr
                | try_expr ;

try_expr        = "try" , postfix_expr
                | await_expr ;

await_expr      = "await" , postfix_expr
                | postfix_expr ;

postfix_expr    = primary_expr , { postfix_op } ;

postfix_op      = "." , ident                          (* field access *)
                | "." , ident , "(" , [ arg_list ] , ")"  (* method call *)
                | "(" , [ arg_list ] , ")"             (* function call *) ;

primary_expr    = ident
                | upper_ident                          (* type name / constructor *)
                | qualified_name
                | number_literal
                | string_literal
                | bool_literal
                | list_literal
                | tuple_literal
                | struct_literal
                | match_expr
                | if_expr
                | loop_expr
                | while_expr
                | for_expr
                | closure_expr
                | unsafe_expr
                | return_expr
                | break_expr
                | "continue"
                | "(" , expr , ")" ;

qualified_name  = ( path_ident | upper_ident ) , "." , ident ;

arg_list        = arg , { "," , arg } ;

arg             = [ ident , ":" ] , expr ;              (* optional named argument *)
```

### 3.14.9 `let` Bindings

```
let_expr        = "let" , [ "mut" ] , ident , [ ":" , type_expr ] , "=" , expr ;
```

### 3.14.10 `match` Expressions

```
match_expr      = "match" , expr , NL , INDENT
                , { match_arm , NL }
                , DEDENT ;

match_arm       = "|" , pattern , "=>" , arm_body ;

arm_body        = expr
                | NL , INDENT , expr_block , DEDENT ;

pattern         = wildcard_pattern
                | literal_pattern
                | ident_pattern
                | constructor_pattern
                | tuple_pattern
                | guarded_pattern ;

wildcard_pattern = "_" ;

literal_pattern = number_literal | string_literal | bool_literal ;

ident_pattern   = lower_ident ;

constructor_pattern = upper_ident , [ "(" , pattern_fields , ")" ] ;

pattern_fields  = pattern_field , { "," , pattern_field } ;

pattern_field   = pattern
                | ident , ":" , pattern ;         (* named field pattern *)

tuple_pattern   = "(" , pattern , "," , pattern , { "," , pattern } , ")" ;

guarded_pattern = pattern , "if" , expr ;
```

### 3.14.11 `if` / `else` Expressions

```
if_expr         = "if" , expr , NL , INDENT
                , expr_block
                , DEDENT
                , [ else_clause ] ;

else_clause     = "else" , ( if_expr | ( NL , INDENT , expr_block , DEDENT ) ) ;
```

### 3.14.12 Loop Expressions

```
loop_expr       = "loop" , NL , INDENT
                , expr_block
                , DEDENT ;

while_expr      = "while" , expr , NL , INDENT
                , expr_block
                , DEDENT ;

for_expr        = "for" , ident , "in" , expr , NL , INDENT
                , expr_block
                , DEDENT ;

break_expr      = "break" , [ expr ] ;
```

### 3.14.13 Closure Expressions

```
closure_expr    = "fn" , "(" , [ closure_params ] , ")" , "->" , closure_body ;

closure_params  = closure_param , { "," , closure_param } ;

closure_param   = ident , [ ":" , type_expr ] ;

closure_body    = expr
                | NL , INDENT , expr_block , DEDENT ;
```

### 3.14.14 `unsafe` Expressions

```
unsafe_expr     = "unsafe" , NL , INDENT
                , expr_block
                , DEDENT ;
```

### 3.14.15 `return` Expressions

```
return_expr     = "return" , expr ;
```

### 3.14.16 List and Tuple Literals

```
list_literal    = "[" , [ expr , { "," , expr } ] , "]" ;

tuple_literal   = "(" , expr , "," , expr , { "," , expr } , ")" ;
```

### 3.14.17 Struct Literal

```
struct_literal  = upper_ident , struct_body ;

struct_body     = "(" , field_init , { "," , field_init } , ")"    (* single-line *)
                | NL , INDENT , { field_init , NL } , DEDENT ;     (* multi-line *)

field_init      = ident , ":" , expr ;
```

### 3.14.18 Type Declarations

```
type_decl       = [ "pub" ] , "type" , upper_ident
                , [ "<" , type_param_list , ">" ]
                , type_body ;

type_body       = struct_type_body
                | enum_type_body
                | alias_type_body ;

struct_type_body = NL , INDENT
                 , { [ "pub" ] , ident , ":" , type_expr , NL }
                 , DEDENT ;

enum_type_body  = NL , INDENT
                , { "|" , upper_ident , [ "(" , enum_fields , ")" ] , NL }
                , DEDENT ;

enum_fields     = enum_field , { "," , enum_field } ;

enum_field      = ident , ":" , type_expr ;

alias_type_body = "=" , type_expr , NL ;
```

### 3.14.19 Trait Declarations

```
trait_decl      = [ "pub" ] , "trait" , upper_ident
                , [ "<" , type_param_list , ">" ]
                , [ ":" , type_constraint ]
                , NL , INDENT
                , { trait_item , NL }
                , DEDENT ;

trait_item       = trait_type_item
                 | trait_method_item
                 | comment ;

trait_type_item  = "type" , upper_ident ;

trait_method_item = "fn" , ident
                  , [ "<" , type_param_list , ">" ]
                  , "(" , [ param_list ] , ")" , "->" , type_expr ;
```

### 3.14.20 Trait Implementation

```
impl_decl       = "impl" , impl_target , NL , INDENT
                , { impl_fn , { NL } }
                , DEDENT ;

impl_target     = upper_ident , [ "<" , type_param_list , ">" ]
                , [ "for" , type_expr ] ;

impl_fn         = [ "pub" ] , [ "async" ] , "fn" , ident
                , [ "<" , type_param_list , ">" ]
                , "(" , [ param_list ] , ")" , "->" , type_expr
                , ( compact_body | block_body ) ;
```

### 3.14.21 Constant Declarations

```
const_decl      = [ "pub" ] , "const" , ident , ":" , type_expr , "=" , expr , NL ;
```

---

## 3.15 Semantic Constraints (Non-Syntactic Rules)

The following constraints are enforced by the compiler but are not expressible in the EBNF grammar:

1. **Indentation.** All indentation uses exactly 2 spaces per level. Tabs are a lexical error. Mixed indentation is a lexical error. The formatter (`monel fmt`) is authoritative.

2. **Expression types.** Both branches of `if`/`else` must have the same type. All arms of `match` must have the same type. The type of a block is the type of its last expression.

3. **Match exhaustiveness.** `match` expressions must be exhaustive. Every possible value of the scrutinee type must be covered by at least one arm. The compiler checks this statically for enum types and uses wildcard coverage for open types.

4. **`try` context.** The `try` keyword may only appear inside a function whose return type is `Result<T, E>`. The error type of the `try` expression must be convertible to `E`.

5. **`await` context.** The `await` keyword may only appear inside an `async fn`.

6. **`break`/`continue` context.** `break` and `continue` may only appear inside `loop`, `while`, or `for` expressions.

7. **Mutability.** Assignment (`=`) to a binding is only allowed if the binding was declared with `let mut`. Field assignment is only allowed through a `mut` reference.

8. **Effect containment.** When `effects:` is declared, the actual effects of a function body must be a subset of the declared list. When `effects:` is omitted, the compiler infers effects from the body. Effects are transitive through function calls.

9. **Forward references.** Top-level declarations may reference each other regardless of declaration order within the file (forward references are resolved during name resolution). Within a function body, names must be declared before use (no forward references within bodies).

10. **Module correspondence.** The file `src/foo/bar.mn` defines module `foo/bar`. A `module` declaration, if present, must match this path.

11. **Unique function names.** Within a module, no two functions may have the same name. Overloading is not supported. Different arities do not create distinct functions.

12. **Single-line function signatures.** The `fn NAME(PARAMS) -> TYPE` declaration must occupy a single line. If a signature is too long for one line, the formatter wraps parameters across lines but the `fn name` token remains on the first line.

13. **Operator line continuation.** When an expression spans multiple lines, binary operators must be on the same line as the left operand. The continuation (right operand) is indented one level.

14. **Contract clause ordering.** Contract clauses must appear before the function body. The compiler distinguishes contract clauses from body expressions by their keywords (`doc:`, `requires:`, `ensures:`, `effects:`, `panics:`).

15. **Compact form restrictions.** The compact form (`fn name(...) -> T = expr`) cannot include contract clauses. Use the block form for functions that need contracts.

---

## 3.16 Reserved Keywords

The following identifiers are reserved in `.mn` files and may not be used as user-defined names:

```
fn        let       mut       match     if        else
use       module    type      trait     impl      const
try       unsafe    async     await     return    loop
while     for       in        break     continue  pub
self      Self      ref       owned     true      false
and       or        not       as
```

---

## 3.17 Contract Keywords in Implementation Files

The following keywords may appear as contract clauses between a function signature and its body:

| Keyword | Purpose | Verification |
|---|---|---|
| `doc:` | Natural-language purpose description | Documentation; optional LLM review |
| `requires:` | Preconditions | SMT-verified at call sites |
| `ensures:` | Postconditions | SMT-verified at function return |
| `effects:` | Side effect declaration | Inferred from body, checked against declaration |
| `panics:` | Panic behavior (`never`) | Static proof of panic freedom |

When `effects:` is omitted, the compiler infers effects from the function body. This is the common case for private helper functions. Public API functions conventionally include an explicit `effects:` declaration for readability and documentation.

When `requires:` or `ensures:` clauses are present, the compiler uses SMT solving (Z3) to verify them. Functions without these clauses still receive type checking, effect checking, and borrow checking.
