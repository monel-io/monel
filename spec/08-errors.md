# 8. Error Handling

## 8.1 Overview

Monel treats errors as values. There are no exceptions, no unwinding, and no hidden control flow. Every operation that can fail returns a `Result<T, E>`, and the compiler enforces exhaustive handling of all error variants. Error types are declared in intent files, making the failure modes of every function visible to both humans and LLMs without reading the implementation.

## 8.2 Core Principles

1. **Errors are values.** A function that can fail returns `Result<T, E>`. There is no `throw`, no exception hierarchy, no stack unwinding.
2. **Error variants are declared in intent.** The intent file lists every variant an error type can have. The compiler verifies the implementation matches.
3. **Exhaustive handling is required.** Every `match` on a `Result` or error type must handle all variants. The compiler rejects incomplete matches.
4. **Panics are for invariant violations only.** A `panic` indicates a bug in the program, not a recoverable error. In `@strict` intent, `panics: never` enables compile-time proof that no panics can occur.

## 8.3 The Result Type

`Result<T, E>` is the fundamental error handling type:

```
type Result<T, E>
  Ok(T)
  Err(E)
```

It is a sum type with exactly two variants. `T` is the success value, `E` is the error value. There is no null, no nil, no undefined -- absence is modeled with `Option<T>`, failure with `Result<T, E>`.

### 8.3.1 Result Usage

```
fn find_user(name: String) -> Result<User, DbError>
  let row = try Db.query("SELECT * FROM users WHERE name = ?", [name])
  match row
    Some(r) -> Ok(User.from_row(r))
    None -> Err(DbError.NotFound("user not found: {name}"))
```

## 8.4 Error Type Declarations

### 8.4.1 In Lightweight Intent

In lightweight (default) intent, error variants are declared with the `fails:` keyword:

```
intent fn authenticate(username: String, password: String) -> Result<Session, AuthError>
  does: "authenticates a user and returns a session"
  fails:
    InvalidCreds: "username or password is incorrect"
    Locked: "account has been locked after too many attempts"
    Expired: "session token has expired"
  effects: [Db.read, Crypto.hash]
```

The `fails:` block lists each variant with a human-readable description. The description serves two purposes:
1. Documentation for humans reading the intent.
2. Default error message if the implementation does not provide a custom one.

### 8.4.2 In Strict Intent

In `@strict` intent, error variants are declared with the `errors:` keyword, which supports additional metadata:

```
@strict
intent fn authenticate(username: String, password: String) -> Result<Session, AuthError>
  does: "authenticates a user and returns a session"
  requires:
    username.len() > 0
    password.len() >= 8
  ensures:
    result.is_ok() implies result.unwrap().user_id != 0
  errors:
    InvalidCreds:
      description: "username or password is incorrect"
      recoverable: true
      http_status: 401
    Locked:
      description: "account has been locked"
      recoverable: false
      http_status: 423
      retry_after: "30 minutes"
    Expired:
      description: "session token has expired"
      recoverable: true
      http_status: 401
  effects: [Db.read, Crypto.hash]
```

The additional metadata fields (`http_status`, `recoverable`, `retry_after`, etc.) are freeform key-value pairs. The compiler does not interpret them, but tooling (`monel generate`, HTTP framework integrations) can use them.

### 8.4.3 Standalone Error Type Declarations

Error types can be declared as standalone types in intent, separate from any function:

```
intent type AuthError
  variants:
    InvalidCreds: "invalid username or password"
    Locked: "account is locked"
    Expired: "session has expired"
```

This is useful when multiple functions share the same error type. Functions reference the type by name:

```
intent fn authenticate(username: String, password: String) -> Result<Session, AuthError>
  does: "authenticates a user and returns a session"
  effects: [Db.read, Crypto.hash]

intent fn refresh_session(token: String) -> Result<Session, AuthError>
  does: "refreshes an expired session"
  effects: [Db.read]
```

### 8.4.4 Error Type with Fields

Error variants can carry data:

```
intent type DbError
  variants:
    NotFound:
      description: "record not found"
      fields: [table: String, id: String]
    ConnectionFailed:
      description: "could not connect to database"
      fields: [host: String, port: UInt16, cause: String]
    QueryFailed:
      description: "query execution failed"
      fields: [query: String, cause: String]
    Timeout:
      description: "query timed out"
      fields: [query: String, duration_ms: UInt64]
```

In the implementation, these variants are constructed with their fields:

```
Err(DbError.NotFound(table: "users", id: user_id))
Err(DbError.ConnectionFailed(host: db_host, port: 5432, cause: e.message()))
```

## 8.5 The `try` Keyword

The `try` keyword propagates errors. It is equivalent to Rust's `?` operator but uses an explicit keyword for readability.

### 8.5.1 Basic Usage

```
fn load_config(path: String) -> Result<Config, ConfigError>
  let content = try read_file(path)
  let parsed = try parse_toml(content)
  let config = try Config.from_toml(parsed)
  Ok(config)
```

`try expr` evaluates `expr`. If the result is `Ok(v)`, the expression evaluates to `v`. If the result is `Err(e)`, the function immediately returns `Err(e)`.

### 8.5.2 Type Requirements

`try` can only be used in a function that returns `Result<T, E>`. The error type of the `try` expression must be convertible to the function's error type `E`. If the types do not match, the compiler emits an error:

```
error: incompatible error types in `try` expression
  --> src/config/mod.mn:12:18
   |
12 |   let content = try read_file(path)
   |                 ^^^ this returns Result<String, IoError>
   |
   = note: function returns Result<Config, ConfigError>
   = help: add an error conversion: `impl From<IoError> for ConfigError`
   = help: or map the error: `try read_file(path).map_err(ConfigError.from_io)`
```

### 8.5.3 `try` With Error Mapping

When error types do not align, `try` can be combined with `.map_err()`:

```
fn load_config(path: String) -> Result<Config, ConfigError>
  let content = try read_file(path).map_err(|e| ConfigError.IoError(path, e.message()))
  let parsed = try parse_toml(content).map_err(|e| ConfigError.ParseError(path, e.line, e.message()))
  Ok(Config.from_toml(parsed))
```

### 8.5.4 `try` in Blocks

`try` can be used in block expressions to scope error handling:

```
fn process_data(input: String) -> Result<Output, ProcessError>
  let validated = try
    let parsed = try parse(input)
    let checked = try validate(parsed)
    Ok(checked)

  let result = try transform(validated)
  Ok(result)
```

## 8.6 Compiler Verification

The compiler enforces several invariants about error types and their usage.

### 8.6.1 Variant Coverage in Implementation

Every error variant declared in intent must appear in the implementation. If a variant is declared but never constructed, the compiler emits a warning:

```
warning: unused error variant
  --> src/auth/mod.mn.intent:8:5
   |
 8 |     Expired: "session has expired"
   |     ^^^^^^^ variant `Expired` is declared but never constructed
   |
   = help: remove the variant from intent, or add code that produces it
```

### 8.6.2 No Undeclared Variants

If the implementation constructs an error variant that is not declared in the intent, the compiler emits an error:

```
error: undeclared error variant
  --> src/auth/mod.mn:23:10
   |
23 |     Err(AuthError.RateLimited("too many attempts"))
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ variant `RateLimited` is not declared
   |
   = help: add to intent:
   |     RateLimited: "too many login attempts"
```

### 8.6.3 Exhaustive Matching

Every `match` on a `Result` or error type must handle all variants:

```
match result
  Ok(session) -> use_session(session)
  Err(AuthError.InvalidCreds) -> show_login_error()
  Err(AuthError.Locked) -> show_locked_message()
  # compiler error: non-exhaustive match, missing Err(AuthError.Expired)
```

The compiler suggests adding the missing arms:

```
error: non-exhaustive match
  --> src/login/mod.mn:15:1
   |
15 | match result
   | ^^^^^ missing variants:
   |   - Err(AuthError.Expired)
   |
   = help: add the missing arm, or use a wildcard:
   |   Err(AuthError.Expired) -> handle_expired()
   |   Err(_) -> handle_other_errors()
```

A wildcard `Err(_)` arm can be used to handle all remaining error variants, but the compiler emits a lint warning because wildcard error handling is often a mistake. The lint can be suppressed with `#[allow(wildcard_error)]`.

### 8.6.4 `try` Error Type Compatibility

The compiler verifies that every `try` expression's error type is compatible with the enclosing function's return error type. Compatibility means one of:
1. The types are identical.
2. A `From<SourceError>` conversion exists for the target error type.
3. An explicit `.map_err()` transforms the error.

### 8.6.5 Dead Error Paths

In `@strict` mode, the compiler performs reachability analysis on error paths. If a precondition makes an error variant unreachable, the compiler notes this:

```
@strict
intent fn divide(a: Int64, b: Int64) -> Result<Int64, MathError>
  requires: b != 0
  errors:
    DivisionByZero: "divisor is zero"
```

```
info: error variant `DivisionByZero` is unreachable given precondition `b != 0`
  --> src/math/mod.mn.intent:5:5
   |
   = note: consider removing the variant, or relaxing the precondition
```

## 8.7 Panic

`panic` is reserved for invariant violations -- situations that indicate a bug in the program, not a runtime condition that should be handled.

### 8.7.1 Usage

```
fn get_element(list: Vec<T>, index: UInt) -> T
  if index >= list.len()
    panic("index {index} out of bounds for list of length {list.len()}")
  list[index]
```

### 8.7.2 Panic Behavior

When a `panic` is reached:
1. The current task is terminated.
2. A stack trace is printed to stderr.
3. In structured concurrency, sibling tasks are cancelled.
4. The process exits with a non-zero status code.

Panics do not unwind. There is no `catch` for panics. Resources are cleaned up via the runtime's structured concurrency guarantees (parent scopes outlive child scopes, so resource owners are always valid).

### 8.7.3 `panics: never` in Strict Intent

In `@strict` intent, a function can declare `panics: never`:

```
@strict
intent fn safe_divide(a: Float64, b: Float64) -> Result<Float64, MathError>
  does: "divides a by b"
  requires: true  # no preconditions
  ensures: result.is_ok() implies result.unwrap() == a / b
  panics: never
  errors:
    DivisionByZero: "divisor is zero"
```

When `panics: never` is declared, the compiler statically verifies that no code path in the function (or any function it calls) can reach a `panic`. This includes:
- Explicit `panic()` calls.
- Array/slice index operations without bounds checks.
- Integer overflow in debug mode.
- Unwrap on `Option::None` or `Result::Err`.

If the compiler cannot prove panic-freedom, it emits an error:

```
error: function declared `panics: never` but may panic
  --> src/math/mod.mn:5:3
   |
 5 |   let x = list[index]
   |           ^^^^^^^^^^^ this index operation may panic
   |
   = help: use `list.get(index)` which returns `Option<T>`
```

### 8.7.4 Debug Assertions

`assert` is syntactic sugar for a conditional panic:

```
assert(x > 0, "x must be positive")
# equivalent to:
if x <= 0
  panic("assertion failed: x must be positive")
```

Debug assertions (`debug_assert`) are compiled only in debug mode. In release mode, they are no-ops.

A function that uses `assert` or `debug_assert` cannot declare `panics: never` unless the compiler can prove the assertion condition always holds.

## 8.8 Error Conversion

### 8.8.1 The `From` Trait

Error types can implement `From` conversions to enable automatic conversion in `try` expressions:

```
# In intent
intent impl From<IoError> for ConfigError
  does: "converts an I/O error to a config error"

# In implementation
impl From<IoError> for ConfigError
  fn from(e: IoError) -> ConfigError
    ConfigError.IoError(e.path(), e.message())
```

When a `From` conversion exists, `try` automatically applies it:

```
fn load_config(path: String) -> Result<Config, ConfigError>
  # IoError is auto-converted to ConfigError via From
  let content = try read_file(path)
  Ok(parse(content))
```

### 8.8.2 Conversion Rules

1. At most one `From` conversion may exist for any pair of error types.
2. Conversion chains are not followed. If `From<A> for B` and `From<B> for C` exist, that does NOT imply `From<A> for C`.
3. The compiler suggests adding `From` implementations when a `try` expression fails due to type mismatch.

## 8.9 Error Context and Wrapping

### 8.9.1 The `context` Method

Errors can be wrapped with additional context using the `.context()` method:

```
fn load_user_config() -> Result<Config, AppError>
  let path = try find_config_path().context("could not locate config file")
  let config = try load_config(path).context("failed to load config from {path}")
  Ok(config)
```

`.context()` wraps the original error in a context layer. The original error is preserved and accessible via `.source()`.

### 8.9.2 Context Chains

Contexts can be nested arbitrarily deep:

```
AppError: failed to start application
  caused by: failed to load config from /home/user/.config/app.toml
    caused by: could not parse TOML
      caused by: expected '=' at line 12, column 5
```

The `Display` implementation for context-wrapped errors formats the full chain.

### 8.9.3 The `with_context` Method

For expensive context construction, use `.with_context()` which takes a closure:

```
let data = try fetch(url).with_context(|| "failed to fetch {url} (attempt {attempt})")
```

The closure is only called if the result is `Err`.

## 8.10 Custom Error Types

### 8.10.1 Simple Enum Errors

The most common error type is an enum with descriptive variants:

```
intent type ParseError
  variants:
    UnexpectedToken:
      description: "unexpected token in input"
      fields: [token: String, line: UInt32, column: UInt32]
    UnexpectedEof:
      description: "unexpected end of input"
      fields: [expected: String]
    InvalidSyntax:
      description: "invalid syntax"
      fields: [message: String, line: UInt32, column: UInt32]
```

### 8.10.2 Struct Errors

For errors with a single representation, a struct error type can be used:

```
intent type HttpError
  fields:
    status: UInt16
    message: String
    url: Url
    headers: Map<String, String>
```

Struct errors are used when there is only one "variant" and the error is distinguished by its field values rather than its variant name.

### 8.10.3 Composite Error Types

Error types can compose other error types:

```
intent type AppError
  variants:
    Config(ConfigError): "configuration error"
    Db(DbError): "database error"
    Auth(AuthError): "authentication error"
    Internal: "internal error"
      fields: [message: String]
```

When a variant wraps another error type, `From` conversions are automatically generated:

```
# These are auto-generated by the compiler:
impl From<ConfigError> for AppError
  fn from(e: ConfigError) -> AppError = AppError.Config(e)

impl From<DbError> for AppError
  fn from(e: DbError) -> AppError = AppError.Db(e)

impl From<AuthError> for AppError
  fn from(e: AuthError) -> AppError = AppError.Auth(e)
```

This allows `try` to automatically lift inner errors to the composite type.

## 8.11 Relationship to Effects

Errors and effects are orthogonal but interact in important ways.

### 8.11.1 Effect-ful Operations Produce Errors

Functions with effects typically return `Result` because the effectful operation can fail:

```
intent fn read_file(path: String) -> Result<String, IoError>
  does: "reads the entire contents of a file"
  effects: [Fs.read]
  fails:
    NotFound: "file does not exist"
    PermissionDenied: "insufficient permissions"
    IoError: "generic I/O error"
```

The compiler does not enforce a strict relationship between effects and errors, but `monel.policy` can:

```toml
[effects]
# Functions with Db.write must return Result (not infallible)
require_result_for = ["Db.write", "Http.send", "Fs.write"]
```

### 8.11.2 Pure Functions and Errors

Functions with no effects can still return `Result`:

```
intent fn parse_json(input: String) -> Result<JsonValue, ParseError>
  does: "parses a JSON string into a value"
  effects: []
  fails:
    InvalidSyntax: "input is not valid JSON"
    MaxDepthExceeded: "nesting depth exceeds limit"
```

This is valid -- parsing is pure but fallible.

### 8.11.3 Infallible Effect-ful Functions

Some effect-ful functions cannot fail:

```
intent fn log_message(level: LogLevel, message: String) -> ()
  does: "writes a log message"
  effects: [Log.write]
```

This is also valid -- the function has effects but no error return. The compiler permits this without warning.

## 8.12 Error Reporting in Compiler Output

The Monel compiler emits errors in a format designed for both human readability and machine consumption.

### 8.12.1 Human-Readable Format

```
error[E0412]: non-exhaustive match on Result<Session, AuthError>
  --> src/login/mod.mn:15:1
   |
13 | fn handle_login(username: String, password: String) -> Result<(), LoginError>
14 |   let session = try authenticate(username, password)
15 |   match validate_session(session)
   |   ^^^^^ missing variants:
   |     - Err(AuthError.Expired)
   |
   = note: AuthError.Expired was added in src/auth/mod.mn.intent:8
   = help: add the missing arm:
   |     Err(AuthError.Expired) -> try refresh_session(session.token)
```

### 8.12.2 Edit-Compatible JSON Format

For integration with editors and AI coding tools, the compiler can emit errors as JSON with edit suggestions:

```json
{
  "errors": [
    {
      "code": "E0412",
      "severity": "error",
      "message": "non-exhaustive match on Result<Session, AuthError>",
      "file": "src/login/mod.mn",
      "line": 15,
      "column": 1,
      "end_line": 18,
      "end_column": 1,
      "related": [
        {
          "file": "src/auth/mod.mn.intent",
          "line": 8,
          "message": "AuthError.Expired declared here"
        }
      ],
      "suggestions": [
        {
          "message": "add the missing match arm",
          "edits": [
            {
              "file": "src/login/mod.mn",
              "action": "insert_after",
              "line": 17,
              "text": "    Err(AuthError.Expired) -> try refresh_session(session.token)"
            }
          ]
        }
      ]
    }
  ]
}
```

This JSON format is used by `monel` when invoked with `--output=json`. AI coding tools parse these suggestions and apply them as edits.

### 8.12.3 Error Codes

All error-handling-related diagnostics have stable error codes:

| Code   | Description                                    |
|--------|------------------------------------------------|
| E0401  | `try` used in function that does not return `Result` |
| E0402  | `try` error type incompatible with function return type |
| E0403  | No `From` conversion for error type in `try`   |
| E0410  | Undeclared error variant in implementation     |
| E0411  | Unused error variant (declared but never produced) |
| E0412  | Non-exhaustive match on error type             |
| E0413  | Wildcard match on error type (lint)            |
| E0420  | `panics: never` violated                       |
| E0421  | `panic` in function without `panics:` declaration |
| E0430  | Error variant field mismatch (intent vs impl)  |
| E0440  | Error type conversion cycle detected           |

## 8.13 The `Option` Type

While not strictly an error type, `Option<T>` is the companion to `Result<T, E>` for representing absence:

```
type Option<T>
  Some(T)
  None
```

### 8.13.1 Option vs Result

- Use `Option<T>` when absence is expected and not an error (e.g., looking up a key that may not exist).
- Use `Result<T, E>` when absence is an error condition with a specific cause.

```
# Option: absence is expected
intent fn get_env(name: String) -> Option<String>
  does: "returns the value of an environment variable, if set"

# Result: absence is an error
intent fn require_env(name: String) -> Result<String, ConfigError>
  does: "returns the value of a required environment variable"
  fails:
    Missing: "required environment variable is not set"
```

### 8.13.2 Converting Option to Result

`Option` can be converted to `Result` via `.ok_or()` and `.ok_or_else()`:

```
let port = try get_env("PORT")
  .ok_or(ConfigError.Missing("PORT"))
  .and_then(|s| s.parse::<UInt16>().map_err(|_| ConfigError.Invalid("PORT", s)))
```

## 8.14 Error Handling Patterns

### 8.14.1 The Collect Pattern

When processing a collection where each element can fail, use the collect pattern:

```
fn parse_all(inputs: Vec<String>) -> Result<Vec<Parsed>, ParseError>
  inputs.map(|s| parse(s)).collect()
```

`collect()` on an iterator of `Result<T, E>` returns `Result<Vec<T>, E>`. It short-circuits on the first error.

### 8.14.2 The Accumulate Pattern

When you want to collect all errors (not just the first), use `accumulate`:

```
fn validate_all(fields: Vec<Field>) -> Result<Vec<ValidField>, Vec<ValidationError>>
  let (successes, errors) = fields
    .map(|f| validate(f))
    .partition_results()

  if errors.is_empty()
    Ok(successes)
  else
    Err(errors)
```

### 8.14.3 The Fallback Pattern

Try multiple strategies, falling back on error:

```
fn find_config() -> Result<Config, ConfigError>
  load_config("./monel.project")
    .or_else(|_| load_config("~/.config/monel/config.toml"))
    .or_else(|_| Ok(Config.default()))
```

### 8.14.4 The Retry Pattern

For transient errors, use retry with backoff:

```
fn fetch_with_retry(url: Url, max_attempts: UInt32) -> Result<Response, HttpError>
  let mut attempt = 0
  loop
    attempt += 1
    match fetch(url)
      Ok(response) -> return Ok(response)
      Err(e) if e.is_transient() and attempt < max_attempts ->
        sleep(Duration.from_millis(100 * 2.pow(attempt)))
        continue
      Err(e) -> return Err(e)
```
