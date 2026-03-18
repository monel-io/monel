# 2. Contract Syntax

**Version:** 0.2.0-draft
**Status:** Working Draft
**Domain:** monel.io
**Date:** 2026-03-18

---

## 2.1 Purpose of This Chapter

This chapter defines the syntax and semantics of Monel contracts. Contracts are inline annotations on functions, types, state machines, and layouts in `.mn` files. They declare *what* a construct must satisfy — preconditions, postconditions, invariants, effects, panic freedom, and complexity bounds. The compiler verifies contracts against the accompanying implementation.

---

## 2.2 Design Overview

Contracts and implementation live in the same `.mn` file. There are no separate intent files. A function's contracts appear between its signature and its body, distinguished by the contract keywords (`requires:`, `ensures:`, `effects:`, etc.). The parser treats any line beginning with a contract keyword (at the appropriate indentation level) as part of the contract block.

Contracts are always verified when present. There is no opt-in annotation — the presence of a contract keyword is sufficient to activate the corresponding verification pass.

A function with no contract keywords is valid. The compiler performs standard type checking, borrow checking, and effect inference, but does not generate SMT proof obligations.

---

## 2.3 Contract Keywords

The following keywords introduce contract clauses. Each may appear at most once per declaration, except `ensures:` which may have multiple clauses (including conditional forms).

| Keyword | Applies to | Meaning |
|---|---|---|
| `requires:` | Functions | Preconditions that must hold at every call site |
| `ensures:` | Functions | Postconditions that must hold at every return point |
| `effects:` | Functions | Declared side effects; compiler infers from body and checks |
| `panics: never` | Functions | The function cannot panic on any code path |
| `complexity:` | Functions | Asymptotic bound the optimizer must respect |
| `invariant:` | Types | Predicate that holds after construction and after every mutation |
| `fails:` | Functions | Shorthand declaring which error variants the function produces |
| `doc:` | Any declaration | Optional free-text documentation; never compiled or verified |

### 2.3.1 `requires:`

Declares preconditions. Each clause is a boolean expression over the function's parameters. The compiler translates these to SMT assertions and verifies them at every call site.

```
fn binary_search(arr: &Vec<Int>, target: Int) -> Option<UInt>
  requires:
    arr.is_sorted()
    arr.len() > 0
  ...
```

If the solver cannot prove a precondition at a call site, the compiler emits an error:

```
error[S0101]: precondition `arr.is_sorted()` not proven at call site
  --> src/search.mn:10:5
   |
10 |   let idx = binary_search(data, key)
   |             ^^^^^^^^^^^^^ cannot prove `data.is_sorted()`
   |
   = help: sort the data before calling, or add a `requires:` to the caller
```

Multiple `requires:` clauses are conjoined. Writing two clauses is equivalent to writing one clause with `and`.

### 2.3.2 `ensures:`

Declares postconditions. Each clause is a boolean expression over the function's return value and parameters. The keyword `result` refers to the return value. The function `old(expr)` captures the value of `expr` at function entry.

```
fn sort(arr: &mut Vec<Int>)
  ensures:
    arr.is_sorted()
    arr.len() == old(arr.len())
    arr.is_permutation_of(old(arr))
  ...
```

#### Conditional postconditions

Postconditions can be conditional on success or on specific error variants. The `ok =>` prefix restricts the clause to the success path. The `err(Variant) =>` prefix restricts it to a specific error path.

```
fn authenticate(creds: Credentials) -> Result<Session, AuthError>
  requires:
    creds.username.len() > 0
    creds.password.len() > 0
  ensures:
    ok => result.user_id > 0
    ok => result.expires_at > Clock.now()
    err(Locked) => Db.failed_attempts(creds.username) >= 5
    err(Overflow) => self == old(self)
  effects: [Db.read, Db.write, Crypto.verify, Log.write]
  panics: never
  ...
```

The compiler verifies each conditional clause on the corresponding code paths:

- `ok => P` — at every return point that produces `Ok(v)`, `P` must hold with `result` bound to `v`.
- `err(V) => P` — at every return point that produces `Err(V(...))`, `P` must hold.
- An unconditional `ensures:` clause (no prefix) must hold at all return points.

Per-error-variant postconditions enable verification that a function preserves specific invariants on each failure path. For example, `err(Overflow) => self == old(self)` asserts rollback on overflow.

### 2.3.3 `effects:`

Declares the set of side effects the function may perform. The compiler infers the actual effects from the function body and checks that the inferred set is a subset of the declared set.

```
fn save_user(user: User) -> Result<Unit, DbError>
  effects: [Db.write, Log.write]
  ...
```

If the body performs an undeclared effect, compilation fails. If the body does not use a declared effect, the compiler emits a warning.

A function with no `effects:` declaration is pure. See Chapter 5 (Effect System) for the full specification.

### 2.3.4 `panics: never`

Declares that no reachable code path in the function can panic. The compiler proves this via static analysis. Proof obligations include:

- No explicit `panic()` calls reachable
- No array index without bounds proof
- No integer overflow (in checked mode)
- No `unwrap()` on `None` or `Err`
- No division by zero
- No assertion failures

```
fn safe_divide(a: Int, b: Int) -> Result<Int, MathError>
  requires:
    b != 0
  panics: never
  ...
```

Functions called from a `panics: never` function must themselves be `panics: never` or must be provably panic-free in context. When the compiler cannot prove panic freedom, it emits an error:

```
error[S0104]: function declared `panics: never` but may panic
  --> src/math.mn:5:3
   |
 5 |   let x = list[index]
   |           ^^^^^^^^^^^ this index operation may panic
   |
   = help: use `list.get(index)` which returns `Option<T>`
```

### 2.3.5 `complexity:`

Declares an asymptotic upper bound. The optimizer must not introduce transformations that violate this bound.

```
fn find(haystack: &Vec<T>, needle: &T) -> Option<UInt>
  complexity: O(n)
  ...
```

The compiler uses `complexity:` as an optimizer constraint: if an optimization pass would change the asymptotic behavior (for example, introducing a nested loop over the input), the pass is rejected. The compiler does not prove the implementation meets the bound — it prevents the optimizer from exceeding it.

### 2.3.6 `invariant:`

Declares a predicate on a type that must hold after every constructor call and after every method that takes `&mut self`. Public methods may assume the invariant holds at entry.

```
type BoundedQueue<T>
  items: Vec<T>
  capacity: UInt

  invariant:
    self.items.len() <= self.capacity
    self.capacity > 0
```

The compiler inserts proof obligations:

- After every constructor (including factory methods that return `Self`), the invariant must hold.
- After every method with `&mut self`, the invariant must hold at every return point.
- At the start of every public method, the invariant is assumed (the caller is responsible for ensuring it).

### 2.3.7 `fails:`

Shorthand for declaring which error variants a function can produce. This does not introduce formal postconditions; it declares the error variant set for exhaustiveness checking.

```
fn read_config(path: String) -> Result<Config, ConfigError>
  fails:
    NotFound: "file does not exist"
    ParseError: "file contents are not valid TOML"
  effects: [Fs.read]
  ...
```

The compiler verifies:

- Every variant listed in `fails:` is reachable in the implementation (warning if not).
- The implementation does not produce error variants absent from `fails:` (error if so).

The string after each variant name is a human-readable description. The compiler does not interpret it; tooling and documentation generators may use it.

`fails:` and conditional `ensures:` clauses (`err(V) => ...`) can coexist. `fails:` declares the error variant set; conditional `ensures:` clauses add postconditions per variant.

### 2.3.8 `doc:`

Optional free-text documentation. The compiler parses and preserves `doc:` text for tooling (IDE hover, `monel query`, documentation generation) but never compiles or verifies it.

```
fn authenticate(creds: Credentials) -> Result<Session, AuthError>
  doc: "Verify credentials against the user database and return a session."
  requires:
    creds.username.len() > 0
  ...
```

`doc:` can appear on functions, types, state machines, layouts, and interactions. It is always optional. When present, it must appear before contract keywords.

---

## 2.4 Function Declarations with Contracts

### 2.4.1 Full Form

A function declaration with contracts has the following structure:

```
fn name(param1: Type1, param2: Type2) -> ReturnType
  doc: "optional documentation"
  requires:
    precondition_1
    precondition_2
  ensures:
    ok => postcondition_on_success
    err(Variant) => postcondition_on_error
  effects: [Effect1, Effect2]
  panics: never
  complexity: O(n)
  fails:
    Variant1: "description"
    Variant2: "description"
  // body begins here
  let x = ...
  ...
```

Contract keywords appear between the signature and the body. They may appear in any order (though the order above is conventional). The body begins at the first line that is not a contract keyword or continuation of a contract clause.

### 2.4.2 Compact Form

For trivial functions, a one-line compact form omits the body block:

```
fn len(self: &Stack<T>) -> UInt = self.items.len()
fn is_empty(self: &Stack<T>) -> Bool = self.items.len() == 0
fn max(a: Int, b: Int) -> Int = if a >= b then a else b
```

The compact form uses `= expr` after the signature. Contract keywords may still appear before the `=`:

```
fn safe_len(self: &Stack<T>) -> UInt
  panics: never
  = self.items.len()
```

However, the compact form is intended for functions simple enough that contracts add little value. When contracts are present, the full form is generally clearer.

### 2.4.3 Generic Functions

Generic type parameters appear after the function name. Trait bounds use `:` syntax. Contract clauses can reference generic type parameters:

```
fn sort_stable<T: Ord>(arr: &mut Vec<T>)
  ensures:
    arr.is_sorted()
    arr.len() == old(arr.len())
  complexity: O(n log n)
  ...
```

### 2.4.4 Methods

Methods follow the same contract syntax. The receiver (`self`, `&self`, `&mut self`) appears as the first parameter:

```
impl BoundedQueue<T>
  fn push(self: &mut Self, item: T) -> Result<Unit, QueueError>
    requires:
      self.items.len() < self.capacity
    ensures:
      ok => self.items.len() == old(self.items.len()) + 1
      err(Full) => self == old(self)
    fails:
      Full: "queue is at capacity"
    ...
```

---

## 2.5 Type Declarations with Contracts

### 2.5.1 Struct Types

Struct types may carry an `invariant:` clause:

```
type NonEmptyVec<T>
  items: Vec<T>

  invariant:
    self.items.len() > 0
```

The `invariant:` clause appears after the field declarations, indented to the type's level. Multiple invariant clauses are conjoined.

### 2.5.2 Refinement Types

Refinement types use a `where` clause on a type alias:

```
type Port = Int where value >= 1 and value <= 65535
type NonEmptyString = String where value.len() > 0
type Percentage = Float where value >= 0.0 and value <= 100.0
```

When contracts are present on functions that accept or return refined types, the compiler uses the refinement predicate as an assumption (for parameters) or proof obligation (for return values).

### 2.5.3 Distinct Types

Distinct types prevent accidental substitution of structurally identical types:

```
distinct type UserId = Int
distinct type OrderId = Int
```

Distinct types may carry invariants via refinement:

```
distinct type Port = Int where value >= 1 and value <= 65535
```

### 2.5.4 Enum Types

Enum types may carry `invariant:` clauses on individual variants:

```
enum Shape
  Circle(radius: Float)
    invariant: radius > 0.0
  Rectangle(width: Float, height: Float)
    invariant: width > 0.0 and height > 0.0
  Triangle(a: Float, b: Float, c: Float)
    invariant: a + b > c and b + c > a and a + c > b
```

Each variant's invariant is verified at construction of that variant.

---

## 2.6 State Machine Declarations

State machines declare states and transitions. The compiler verifies that the implementation's enum types and transition functions match.

```
state_machine OrderLifecycle
  doc: "Models the lifecycle of a customer order."

  states:
    Created
    Validated
    Paid
    Shipped
    Delivered
    Cancelled

  transitions:
    Created -> Validated: validate_order
    Created -> Cancelled: cancel_order
    Validated -> Paid: charge_payment
    Validated -> Cancelled: cancel_order
    Paid -> Shipped: ship_order
    Paid -> Cancelled: cancel_and_refund
    Shipped -> Delivered: confirm_delivery

  initial: Created
  terminal: [Delivered, Cancelled]
```

The compiler verifies:

| Check | Rule |
|---|---|
| State coverage | Every declared state exists as an enum variant |
| Transition coverage | Every declared transition has a corresponding function |
| Transition correctness | Each transition function moves from the declared source state to the declared destination state |
| No illegal transitions | No code path produces a state change not declared in the state machine |
| Reachability | All states are reachable from the initial state (warning if not) |
| Terminal correctness | Terminal states have no outgoing transitions in code |
| Initial state | The initial state is the only state used in constructors |

Transition functions follow normal contract syntax:

```
fn validate_order(order: &mut Order) -> Result<Unit, OrderError>
  requires:
    order.state == OrderState.Created
  ensures:
    ok => order.state == OrderState.Validated
  effects: [Db.read]
  ...
```

---

## 2.7 Layout Declarations

Layout declarations specify UI region structure and constraints. The compiler verifies that the implementation creates the declared regions and satisfies the constraints.

```
layout MainView
  doc: "Primary application view with sidebar and content area."

  regions:
    sidebar:
      width: 20%
      min_width: 200px
    content:
      width: 80%
      min_width: 400px

  constraint: sidebar.width + content.width == 100%
```

The compiler verifies:

| Check | Rule |
|---|---|
| Region existence | Every declared region is created in the implementation |
| Percentage sum | Regions with percentage widths/heights sum to 100% |
| Min size satisfiability | Min sizes are satisfiable given the percentage constraints |
| Custom constraints | Declared constraints are satisfiable |

---

## 2.8 Interaction Declarations

Interaction declarations specify UI state machines for user flows:

```
interaction SearchFlow
  doc: "User searches for and selects a result."

  states:
    idle
    typing
    loading
    results
    selected
    error

  transitions:
    idle -> typing: on_keystroke
    typing -> typing: on_keystroke
    typing -> loading: debounce_expired
    loading -> results: search_success
    loading -> error: search_failure
    results -> typing: on_keystroke
    results -> selected: on_select
    error -> typing: on_keystroke
```

The compiler applies the same state machine verification rules as Section 2.6, adapted to event-driven transitions.

---

## 2.9 Verification Coverage

The following table summarizes what the compiler proves statically versus what requires testing.

| Contract | Verification method | Coverage |
|---|---|---|
| `requires:` | SMT solver at every call site | Proven for all inputs satisfying caller's context |
| `ensures:` (unconditional) | SMT solver at every return point | Proven for all paths |
| `ensures:` (`ok =>`) | SMT solver on success paths | Proven for all success paths |
| `ensures:` (`err(V) =>`) | SMT solver on error path `V` | Proven for all paths producing `Err(V)` |
| `effects:` | Compiler effect inference | Checked against inferred effects from body |
| `panics: never` | Static analysis of all code paths | Proven panic-free (including transitive callees) |
| `complexity:` | Optimizer constraint | Prevents optimizer violations; does not prove implementation bound |
| `invariant:` | SMT solver after constructors and `&mut self` methods | Proven at all mutation points |
| `fails:` | Reachability analysis | Variant coverage and exhaustiveness checked |
| Refinement types (`where`) | SMT solver at construction sites | Proven at every point a refined type is created |

**What requires tests:**

- Behavioral correctness beyond what contracts express (contracts specify properties, not complete behavior)
- Performance characteristics (contracts specify asymptotic bounds, not wall-clock time)
- Integration behavior across module boundaries
- Properties that exceed SMT solver capabilities (the solver may return UNKNOWN for complex predicates involving heap structures, floating-point arithmetic, or string operations)

When the SMT solver times out, the compiler reports a warning with guidance:

```
warning[S0105]: verification inconclusive for `ensures: arr.is_sorted()`
  = note: Z3 returned UNKNOWN after 10000ms
  = help: consider adding a loop invariant or writing a test
```

The solver timeout is configurable:

```toml
# monel.project
[verification]
smt_timeout_ms = 10000
smt_memory_limit_mb = 4096
```

---

## 2.10 Grammar

The following EBNF grammar specifies the contract syntax for function declarations, type declarations, state machines, layouts, and interactions.

```ebnf
(* ===== Top-level declarations ===== *)

top_level_decl  = fn_decl
                | type_decl
                | state_machine_decl
                | layout_decl
                | interaction_decl
                | use_decl ;

(* ===== Function declarations ===== *)

fn_decl         = "fn" IDENT generic_params? "(" param_list? ")" ( "->" type_expr )?
                  contract_block?
                  ( "=" expr                (* compact form *)
                  | INDENT body DEDENT      (* full form *)
                  ) ;

param_list      = param ( "," param )* ;
param           = IDENT ":" type_expr ;

generic_params  = "<" generic_param ( "," generic_param )* ">" ;
generic_param   = IDENT ( ":" trait_bound ( "+" trait_bound )* )? ;
trait_bound     = IDENT ( "<" type_arg ( "," type_arg )* ">" )? ;

(* ===== Contract block ===== *)

contract_block  = contract_clause+ ;
contract_clause = doc_clause
                | requires_clause
                | ensures_clause
                | effects_clause
                | panics_clause
                | complexity_clause
                | fails_clause ;

doc_clause      = "doc:" STRING_LITERAL ;

requires_clause = "requires:" INDENT
                    predicate_line+
                  DEDENT ;

ensures_clause  = "ensures:" INDENT
                    ensures_line+
                  DEDENT ;

ensures_line    = ( "ok" "=>" predicate_expr
                  | "err" "(" IDENT ")" "=>" predicate_expr
                  | predicate_expr
                  ) NEWLINE ;

effects_clause  = "effects:" "[" effect_list "]" ;
effect_list     = effect ( "," effect )* ;
effect          = "pure"
                | "async"
                | "unsafe"
                | IDENT "." ( IDENT | "*" ) ;

panics_clause   = "panics:" "never" ;

complexity_clause = "complexity:" COMPLEXITY_LITERAL ;
(* COMPLEXITY_LITERAL matches O(1), O(n), O(n log n), O(n^2), etc. *)

fails_clause    = "fails:" INDENT
                    fails_line+
                  DEDENT ;

fails_line      = IDENT ":" STRING_LITERAL NEWLINE ;

(* ===== Predicate expressions ===== *)

predicate_line  = predicate_expr NEWLINE ;
predicate_expr  = comparison_expr ( ( "and" | "or" ) comparison_expr )* ;
comparison_expr = additive_expr ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) additive_expr )? ;
additive_expr   = term ( ( "+" | "-" ) term )* ;
term            = factor ( ( "*" | "/" | "%" ) factor )* ;
factor          = "not" factor
                | "old" "(" expr ")"
                | atom ;
atom            = IDENT ( "." IDENT ( "(" arg_list? ")" )? )*
                | INT_LITERAL
                | FLOAT_LITERAL
                | STRING_LITERAL
                | BOOL_LITERAL
                | "(" predicate_expr ")" ;

(* ===== Type declarations ===== *)

type_decl       = struct_decl
                | enum_decl
                | alias_decl
                | distinct_decl
                | refined_decl ;

struct_decl     = "type" IDENT generic_params? INDENT
                    field_decl+
                    invariant_clause?
                  DEDENT ;

field_decl      = IDENT ":" type_expr NEWLINE ;

enum_decl       = "enum" IDENT generic_params? INDENT
                    variant_decl+
                  DEDENT ;

variant_decl    = IDENT ( "(" field_list ")" )? NEWLINE
                  ( INDENT invariant_clause DEDENT )? ;

field_list      = IDENT ":" type_expr ( "," IDENT ":" type_expr )* ;

alias_decl      = "type" IDENT "=" type_expr ;

distinct_decl   = "distinct" "type" IDENT "=" type_expr
                  ( "where" predicate_expr )?
                | "distinct" "struct" IDENT INDENT
                    field_decl+
                  DEDENT ;

refined_decl    = "type" IDENT "=" type_expr "where" predicate_expr ;

invariant_clause = "invariant:" INDENT
                     predicate_line+
                   DEDENT ;

(* ===== State machine declarations ===== *)

state_machine_decl = "state_machine" IDENT INDENT
                       doc_clause?
                       states_block
                       transitions_block
                       initial_decl
                       terminal_decl?
                     DEDENT ;

states_block    = "states:" INDENT
                    IDENT+
                  DEDENT ;

transitions_block = "transitions:" INDENT
                      transition_line+
                    DEDENT ;

transition_line = IDENT "->" IDENT ":" IDENT NEWLINE ;

initial_decl    = "initial:" IDENT ;
terminal_decl   = "terminal:" "[" IDENT ( "," IDENT )* "]" ;

(* ===== Layout declarations ===== *)

layout_decl     = "layout" IDENT INDENT
                    doc_clause?
                    regions_block
                    constraint_line*
                  DEDENT ;

regions_block   = "regions:" INDENT
                    region_decl+
                  DEDENT ;

region_decl     = IDENT ":" INDENT
                    region_prop+
                  DEDENT ;

region_prop     = IDENT ":" ( PERCENTAGE | DIMENSION ) NEWLINE ;

constraint_line = "constraint:" predicate_expr NEWLINE ;

(* ===== Interaction declarations ===== *)

interaction_decl = "interaction" IDENT INDENT
                     doc_clause?
                     states_block
                     transitions_block
                   DEDENT ;

(* ===== Terminals ===== *)

IDENT           = [a-zA-Z_] [a-zA-Z0-9_]* ;
INT_LITERAL     = [0-9]+ ;
FLOAT_LITERAL   = [0-9]+ "." [0-9]+ ;
STRING_LITERAL  = '"' [^"]* '"' ;
BOOL_LITERAL    = "true" | "false" ;
PERCENTAGE      = INT_LITERAL "%" ;
DIMENSION       = INT_LITERAL ( "px" | "em" | "rem" ) ;
COMPLEXITY_LITERAL = "O" "(" [a-zA-Z0-9 ^*+log()]+ ")" ;
INDENT          = (* increase in indentation level by 2 spaces *) ;
DEDENT          = (* decrease in indentation level *) ;
NEWLINE         = (* line terminator *) ;
```

---

## 2.11 Complete Example

The following shows a `.mn` file with contracts on a function, a type with an invariant, and a compact function.

```
use db {Db, DbError}
use crypto {Crypto}

type Session
  user_id: UInt
  token: String
  expires_at: Timestamp

  invariant:
    self.user_id > 0
    self.token.len() > 0
    self.expires_at > Timestamp.epoch()

enum AuthError
  InvalidCreds
  Locked
  Expired

fn authenticate(creds: Credentials) -> Result<Session, AuthError>
  doc: "Verify credentials against the user database and return a session."
  requires:
    creds.username.len() > 0
    creds.password.len() > 0
  ensures:
    ok => result.user_id > 0
    ok => result.expires_at > Clock.now()
    err(Locked) => Db.failed_attempts(creds.username) >= 5
  effects: [Db.read, Db.write, Crypto.verify, Log.write]
  panics: never
  fails:
    InvalidCreds: "username or password is incorrect"
    Locked: "account locked after too many failed attempts"
    Expired: "session token has expired"

  let user = Db.find_user_by_username(creds.username)
  match user
    | None => Err(AuthError.InvalidCreds)
    | Some(u) =>
      if u.failed_attempts >= 5
        Err(AuthError.Locked)
      else
        let valid = Crypto.verify(creds.password, u.password_hash)
        if not valid
          Db.increment_failed_attempts(u.id)
          Err(AuthError.InvalidCreds)
        else
          Db.reset_failed_attempts(u.id)
          let token = Crypto.generate_token()
          Ok(Session {
            user_id: u.id,
            token: token,
            expires_at: Clock.now().add_hours(24)
          })

fn is_expired(session: &Session) -> Bool = session.expires_at <= Clock.now()
```

---

## 2.12 Summary

| Aspect | Behavior |
|---|---|
| File format | Contracts are inline in `.mn` files |
| Activation | Presence of a contract keyword activates verification |
| Preconditions | `requires:` — SMT-verified at call sites |
| Postconditions | `ensures:` — SMT-verified at return points; supports `ok =>` and `err(V) =>` |
| Effects | `effects:` — compiler infers from body, checks subset |
| Panic freedom | `panics: never` — proven by static analysis |
| Complexity | `complexity:` — constrains the optimizer |
| Type invariants | `invariant:` — verified after construction and mutation |
| Error variants | `fails:` — exhaustiveness and reachability checked |
| Documentation | `doc:` — optional, never compiled |
| Compact form | `fn name(...) -> T = expr` |
| Solver timeout | Configurable; UNKNOWN results are warnings, not errors by default |
