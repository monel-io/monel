# 4. Type System

This chapter specifies the Monel type system, including primitive and composite types, structural and nominal typing, refinement types, generics, traits, ownership and borrowing, type inference, and the relationship between contracts and implementation.

---

## 4.1 Design Principles

The Monel type system is designed around three goals:

1. **Expressiveness** — types describe *what* data means, not just what shape it has. Refinement types, algebraic constraints, and contracts let authors specify invariants that the compiler enforces.

2. **Writability** — the type system uses standard, predictable syntax. Type inference within function bodies reduces boilerplate. Explicit signatures at function boundaries provide unambiguous context for code generation.

3. **Zero-cost safety** — ownership and borrowing eliminate garbage collection. Refinement types are verified at compile time where possible, with optional runtime checks. The type system catches errors that would otherwise require tests.

---

## 4.2 Primitive Types

Monel provides the following built-in primitive types:

| Type     | Description                        | Size      | Default Value |
|----------|------------------------------------|-----------|---------------|
| `Int`    | Signed integer, platform-width     | 64-bit    | `0`           |
| `Int8`   | Signed 8-bit integer               | 8-bit     | `0`           |
| `Int16`  | Signed 16-bit integer              | 16-bit    | `0`           |
| `Int32`  | Signed 32-bit integer              | 32-bit    | `0`           |
| `Int64`  | Signed 64-bit integer              | 64-bit    | `0`           |
| `UInt`   | Unsigned integer, platform-width   | 64-bit    | `0`           |
| `UInt8`  | Unsigned 8-bit integer             | 8-bit     | `0`           |
| `UInt16` | Unsigned 16-bit integer            | 16-bit    | `0`           |
| `UInt32` | Unsigned 32-bit integer            | 32-bit    | `0`           |
| `UInt64` | Unsigned 64-bit integer            | 64-bit    | `0`           |
| `Float`  | IEEE 754 double precision          | 64-bit    | `0.0`         |
| `Float32`| IEEE 754 single precision          | 32-bit    | `0.0`         |
| `Bool`   | Boolean                            | 1 byte    | `false`       |
| `Byte`   | Unsigned 8-bit value               | 8-bit     | `0`           |
| `Char`   | Unicode scalar value (U+0000..U+10FFFF, excluding surrogates) | 32-bit | `'\0'` |
| `String` | UTF-8 encoded, owned string        | 3 words   | `""`          |
| `Unit`   | Zero-sized type, single value `()` | 0-bit     | `()`          |
| `Never`  | Uninhabited type, no values exist  | 0-bit     | N/A           |

### 4.2.1 `Int` and `UInt`

`Int` is the default integer type. Integer literals without a suffix are inferred as `Int`. Suffixes select specific widths:

```
let x = 42          // Int
let y = 42i32       // Int32
let z = 42u8        // UInt8
```

Arithmetic on fixed-width integers checks for overflow in debug builds and wraps in release builds. The `wrapping_*`, `checked_*`, and `saturating_*` method families provide explicit control.

### 4.2.2 `Float` and `Float32`

`Float` is the default floating-point type. Numeric literals containing a decimal point are inferred as `Float`. The suffix `f32` selects `Float32`:

```
let x = 3.14        // Float
let y = 3.14f32     // Float32
```

### 4.2.3 `String`

`String` is an owned, growable, UTF-8 encoded string. String slices are represented by `&String` (a borrowed reference to a `String`). Raw string literals use `r"..."` or `r#"..."#` syntax for strings containing quotes.

```
let s = "hello"
let raw = r#"she said "hi""#
```

### 4.2.4 `Unit` and `Never`

`Unit` is the type of expressions that produce no meaningful value. Functions that return nothing implicitly return `Unit`. The single value of `Unit` is written `()`.

`Never` is the bottom type. It has no values. Functions that never return (e.g., infinite loops, process exit) have return type `Never`. `Never` coerces to any type, making it compatible with any expression context:

```
fn exit(code: Int) -> Never
  effects: [Process.exit]
  os.exit(code)

let x: Int = if condition then 42 else exit(1)  // exit returns Never, coerces to Int
```

---

## 4.3 Composite Types

### 4.3.1 Tuples

Tuples are fixed-size, heterogeneous ordered collections:

```
let point = (1, 2)            // (Int, Int)
let record = ("Alice", 30)    // (String, Int)
let (name, age) = record      // destructuring
```

Tuples support indexing with `.0`, `.1`, etc. The unit type `Unit` is the zero-element tuple `()`.

### 4.3.2 Arrays

`Array<T>` is a fixed-size, stack-allocated sequence of elements of type `T`. The size is part of the type when used with const generics:

```
let a: Array<Int> = [1, 2, 3, 4, 5]
let b: Array<Int, 5> = [1, 2, 3, 4, 5]   // with const generic size
```

### 4.3.3 Vec

`Vec<T>` is a growable, heap-allocated sequence:

```
let v: Vec<Int> = [1, 2, 3]
v.push(4)
```

### 4.3.4 Map and Set

`Map<K, V>` is an ordered hash map. `Set<T>` is an ordered hash set. Keys must implement the `Hash + Eq` traits:

```
let users: Map<String, User> = {}
let tags: Set<String> = {"alpha", "beta"}
```

### 4.3.5 Option and Result

`Option<T>` represents a value that may be absent:

```
enum Option<T>
  Some(T)
  None
```

`Result<T, E>` represents a computation that may fail:

```
enum Result<T, E>
  Ok(T)
  Err(E)
```

Both are fundamental to error handling (see Chapter 7). The `?` operator propagates `None` or `Err` to the caller.

---

## 4.4 Pointer Types

### 4.4.1 References

References are the primary way to borrow data without taking ownership:

| Syntax     | Description                    |
|------------|--------------------------------|
| `&T`       | Immutable (shared) reference   |
| `&mut T`   | Mutable (exclusive) reference  |

References are always valid — they cannot be null and cannot dangle. The borrow checker (Section 4.10) enforces these guarantees.

### 4.4.2 Raw Pointers

`Ptr<T>` and `MutPtr<T>` are raw pointer types for unsafe, low-level operations:

| Type        | Description                  |
|-------------|------------------------------|
| `Ptr<T>`    | Immutable raw pointer to `T` |
| `MutPtr<T>` | Mutable raw pointer to `T`   |

Raw pointers:
- May be null.
- May dangle (point to deallocated memory).
- May be unaligned.
- Do not enforce aliasing rules.
- Can only be dereferenced inside `unsafe` blocks.
- Require the `unsafe` effect on the enclosing function.

```
fn read_hardware_register(addr: UInt64) -> UInt32
  effects: [unsafe]
  unsafe
    let ptr = addr as MutPtr<UInt32>
    *ptr
```

---

## 4.5 Structural Typing

Monel uses structural typing by default. Two types are compatible if they have the same structure — the same fields with the same types, in the same order.

### 4.5.1 Struct Declarations

```
struct Point
  x: Float
  y: Float

struct Coordinate
  x: Float
  y: Float
```

`Point` and `Coordinate` are structurally equivalent. A value of type `Point` can be used wherever `Coordinate` is expected, and vice versa.

### 4.5.2 Structural Subtyping

A struct `A` is a structural subtype of struct `B` if `A` has all the fields of `B` (with matching types), possibly with additional fields. Subtyping applies when passing arguments or assigning to variables with a wider type:

```
struct Point3D
  x: Float
  y: Float
  z: Float

fn distance_2d(p: Point) -> Float
  (p.x * p.x + p.y * p.y).sqrt()

let p = Point3D { x: 1.0, y: 2.0, z: 3.0 }
distance_2d(p)  // OK: Point3D is a structural subtype of Point
```

### 4.5.3 Structural Subtyping Rules

The subtyping relation `A <: B` holds when:

1. **Width subtyping**: `A` has every field that `B` has, with identical types for each, and may have additional fields.
2. **Depth subtyping is NOT supported**: Field types must match exactly. If `B` has field `x: T`, then `A` must also have `x: T`, not `x: U` even if `U <: T`. This restriction prevents unsoundness with mutable references.

Function types are contravariant in parameter types and covariant in return types:

```
// If A <: B, then:
//   (B) -> A  <:  (A) -> B    // contravariant params, covariant return
```

### 4.5.4 When Structural Typing Applies

Structural compatibility is checked:
- At function call sites (argument passing)
- At variable assignment with explicit type annotations
- At return statements
- In generic type bound satisfaction

Structural typing does NOT apply:
- To `distinct type` declarations (Section 4.6)
- To enum variants (variants are always nominal)
- When a trait bound is required (traits are nominal)

---

## 4.6 Nominal Typing with `distinct type`

When accidental substitution of structurally-identical types would be a bug, use `distinct type` to create a nominal wrapper:

```
distinct type UserId = Int
distinct type OrderId = Int

fn get_user(id: UserId) -> User
  // ...

let uid: UserId = UserId(42)
let oid: OrderId = OrderId(42)

get_user(uid)   // OK
get_user(oid)   // COMPILE ERROR: OrderId is not UserId
get_user(42)    // COMPILE ERROR: Int is not UserId
```

### 4.6.1 Properties of Distinct Types

- A `distinct type` creates a new nominal type that is NOT structurally equivalent to its underlying type.
- Explicit conversion is required: `UserId(42)` to wrap, `uid.value` to unwrap.
- Distinct types can implement traits independently of their underlying type.
- Distinct types can opt in to inheriting operations from their underlying type with `deriving`:

```
distinct type Celsius = Float
  deriving: [Add, Sub, Ord, Eq, Display]
```

Without `deriving`, no operations from the underlying type are available.

### 4.6.2 Distinct Struct Types

Structs can also be declared as distinct:

```
distinct struct Velocity
  x: Float
  y: Float

struct Point
  x: Float
  y: Float

// Velocity and Point are NOT interchangeable despite identical fields
```

### 4.6.3 Rationale

Domain-specific types like `Port`, `UserId`, `Email` should not be accidentally mixed with raw `Int` or `String` values. `distinct type` enforces this. The compiler verifies that types declared as semantically distinct are implemented as `distinct type`.

---

## 4.7 Type Aliases

A type alias introduces a new name for an existing type without creating a distinct type. Aliases are fully interchangeable with their target:

```
type Seconds = Float
type UserMap = Map<UserId, User>
type Callback = fn(Event) -> Result<Unit, Error>
```

Aliases are expanded during compilation. They exist purely for readability and do not create new types.

---

## 4.8 Algebraic Data Types

### 4.8.1 Enums

Enums define a type that is one of several variants:

```
enum Shape
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
  Triangle(a: Float, b: Float, c: Float)
```

Variants can hold data (as above), or be unit variants:

```
enum Direction
  North
  South
  East
  West
```

Pattern matching on enums must be exhaustive:

```
fn area(s: Shape) -> Float
  match s
    Circle(r) => Float.pi * r * r
    Rectangle(w, h) => w * h
    Triangle(a, b, c) =>
      let s = (a + b + c) / 2.0
      (s * (s - a) * (s - b) * (s - c)).sqrt()
```

### 4.8.2 Enums with Constraints

Enum variants can carry contracts that are enforced by the compiler:

```
type Shape
  doc: "Represents geometric shapes with valid dimensions"
  variants:
    Circle:
      radius: Float where radius > 0
    Rectangle:
      width: Float where width > 0
      height: Float where height > 0
    Triangle:
      a: Float, b: Float, c: Float
      where: a + b > c and b + c > a and a + c > b
```

These constraints become compile-time proof obligations when preconditions are statically provable. Otherwise, they generate runtime validation in constructors.

### 4.8.3 Recursive Enums

Enums may be recursive. Recursive variants must be behind a pointer (typically `Box<T>`):

```
enum Expr
  Literal(Int)
  Add(Box<Expr>, Box<Expr>)
  Mul(Box<Expr>, Box<Expr>)
```

---

## 4.9 Refinement Types

Refinement types allow constraining a type's values with a predicate. They are declared as contracts and enforced according to the verification tier.

### 4.9.1 Syntax

```
type Port = Int where value >= 1 and value <= 65535
type NonEmptyString = String where value.len() > 0
type Percentage = Float where value >= 0.0 and value <= 100.0
type EvenInt = Int where value % 2 == 0
type SortedVec<T: Ord> = Vec<T> where value.is_sorted()
```

The keyword `value` refers to the value being constrained. The `where` clause contains a boolean expression over `value` and its methods.

### 4.9.2 Compound Refinements

Refinement predicates support `and`, `or`, `not`, comparisons, arithmetic, method calls on the value, and quantifiers:

```
type ValidEmail = String
  where value.contains("@")
    and value.len() >= 3
    and value.len() <= 254

type Matrix<T, const R: UInt, const C: UInt> = Array<Array<T, C>, R>
  where R > 0 and C > 0
```

### 4.9.3 Verification Tiers

Refinement types are checked differently depending on the verification tier:

| Tier | Behavior |
|------|----------|
| Lightweight (default) | Refinement predicates are checked at construction time via runtime assertions. The compiler inserts validation code at every point where a refined type is constructed. |
| With `requires`/`ensures` contracts | Refinement predicates are discharged as SMT proof obligations. The compiler generates Z3 queries to prove that the predicate holds for all possible inputs at the construction site. If the solver cannot prove it, compilation fails with a diagnostic. |
| With `--smt-timeout` | Same as above, but the solver is given a time budget. Unresolved obligations are reported as warnings, not errors. |

### 4.9.4 Refinement Propagation

When a function accepts a refined type, the refinement is assumed to hold within the function body. When a function returns a refined type, the compiler must verify the refinement at the return site:

```
fn next_port(p: Port) -> Port
  // Here, p >= 1 and p <= 65535 is known
  let next = p + 1
  if next > 65535
    1        // wraps around; satisfies Port refinement
  else
    next     // satisfies Port refinement because next <= 65535

fn bad_next_port(p: Port) -> Port
  p + 1      // ERROR: cannot prove p + 1 <= 65535
```

### 4.9.5 Refinement Subtyping

A refined type `T where P` is a subtype of `T`. A value of type `Port` can be used wherever `Int` is expected. The reverse is not true — assigning an `Int` to a `Port` requires a runtime check or a proof.

A refined type `T where P1 and P2` is a subtype of `T where P1`. More restrictive refinements are subtypes of less restrictive ones.

### 4.9.6 Refinement Types as Contracts

Refinement types express domain constraints directly in the type definition:

```
fn connect(host: String, port: Port) -> Result<Connection, ConnectionError>
  doc: "Establish TCP connection to host on given port"
  effects: [Net.connect]
  requires: host.len() > 0
  // ...
```

The compiler verifies that the parameter type for `port` is `Port` (or a type with an equivalent or stronger refinement).

---

## 4.10 Ownership and Borrowing

Monel uses an ownership and borrowing system inspired by Rust. The key difference is that **lifetime annotations are always inferred** — programmers never write explicit lifetime parameters.

### 4.10.1 Ownership Rules

1. Every value has exactly one owner.
2. When the owner goes out of scope, the value is dropped (its destructor runs, and its memory is freed).
3. Ownership can be transferred (moved) to another variable or function.

```
let s1 = String.from("hello")
let s2 = s1                    // s1 is moved to s2; s1 is no longer valid
// println(s1)                 // COMPILE ERROR: use of moved value
println(s2)                    // OK
```

### 4.10.2 Copy Types

Types that implement the `Copy` trait are copied instead of moved. All primitive types (`Int`, `Float`, `Bool`, `Byte`, `Char`, `Unit`) are `Copy`. Structs and enums composed entirely of `Copy` fields can derive `Copy`:

```
struct Point
  x: Float
  y: Float
  deriving: [Copy]
```

### 4.10.3 Borrowing Rules

References allow borrowing a value without taking ownership:

1. At any given time, you can have EITHER:
   - Any number of immutable references `&T`, OR
   - Exactly one mutable reference `&mut T`
2. References must always be valid (no dangling references).

```
fn calculate(data: &Vec<Int>) -> Int
  // data is borrowed immutably; caller retains ownership
  data.iter().sum()

fn append(data: &mut Vec<Int>, value: Int)
  // data is borrowed mutably; exclusive access
  data.push(value)
```

### 4.10.4 Lifetime Inference

Unlike Rust, Monel does not expose lifetime parameters in function signatures. The compiler infers lifetimes using the following rules:

**Rule 1 — Input lifetimes**: Each reference parameter gets its own inferred lifetime.

**Rule 2 — Single input reference**: If there is exactly one input reference parameter, its lifetime is assigned to all output references.

**Rule 3 — Method receiver**: If the function is a method with `&self` or `&mut self`, the lifetime of `self` is assigned to all output references.

**Rule 4 — Elision failure**: If the compiler cannot determine output lifetimes from rules 1-3, it performs whole-program lifetime analysis within the crate. If that also fails, it reports an error with a suggestion.

```
// The compiler infers that the returned reference lives as long as `items`
fn first(items: &Vec<Int>) -> &Int
  &items[0]

// The compiler infers that the returned reference lives as long as `self`
impl Cache
  fn get(key: &String) -> &Option<Value>
    self.store.get(key)
```

For cases where the compiler's inference is ambiguous, the programmer can add a clarifying annotation using named borrows:

```
fn longest(a: &String as 'x, b: &String as 'y) -> &String as 'x | 'y
  if a.len() >= b.len() then a else b
```

This syntax is intentionally rare and only needed when automatic inference fails. The `as 'name` annotation is a hint, not a lifetime parameter in the Rust sense — it guides the inference algorithm.

### 4.10.5 Move Semantics

Non-`Copy` types are moved by default:

```
fn consume(s: String)
  println(s)

let greeting = String.from("hello")
consume(greeting)
// greeting is no longer valid here
```

To explicitly clone:

```
let greeting = String.from("hello")
consume(greeting.clone())
// greeting is still valid here
```

### 4.10.6 Drop and Destructors

Types can implement the `Drop` trait to run cleanup code when they go out of scope:

```
impl Drop for Connection
  fn drop(&mut self)
    self.close()
```

Drop order is deterministic: fields are dropped in declaration order, local variables in reverse declaration order.

---

## 4.11 Generics

### 4.11.1 Generic Functions

Functions can be parameterized over types:

```
fn identity<T>(x: T) -> T
  x

fn swap<A, B>(pair: (A, B)) -> (B, A)
  (pair.1, pair.0)
```

### 4.11.2 Generic Structs and Enums

```
struct Pair<A, B>
  first: A
  second: B

enum Tree<T>
  Leaf(T)
  Branch(Box<Tree<T>>, Box<Tree<T>>)
```

### 4.11.3 Trait Bounds

Generic type parameters can be constrained with trait bounds:

```
fn max<T: Ord>(a: T, b: T) -> T
  if a >= b then a else b

fn print_all<T: Display>(items: &Vec<T>)
  for item in items
    println("{}", item)
```

Multiple bounds use `+`:

```
fn search<T: Eq + Hash + Display>(collection: &Set<T>, target: &T) -> Bool
  collection.contains(target)
```

`where` clauses provide an alternative syntax for complex bounds:

```
fn merge<K, V>(a: Map<K, V>, b: Map<K, V>) -> Map<K, V>
  where K: Eq + Hash + Ord
        V: Clone
  // ...
```

### 4.11.4 Const Generics

Type parameters can be compile-time constant values:

```
struct FixedBuffer<T, const N: UInt>
  data: Array<T, N>
  len: UInt

impl<T, const N: UInt> FixedBuffer<T, N>
  fn capacity() -> UInt
    N

  fn is_full(&self) -> Bool
    self.len == N
```

Const generic parameters may appear in refinement predicates:

```
struct Matrix<T, const R: UInt, const C: UInt>
  data: Array<Array<T, C>, R>

fn multiply<T: Mul + Add + Default, const M: UInt, const N: UInt, const P: UInt>(
  a: &Matrix<T, M, N>,
  b: &Matrix<T, N, P>
) -> Matrix<T, M, P>
  // N must match — enforced by the type system
  // ...
```

### 4.11.5 Monomorphization

Generics are monomorphized at compile time. Each concrete instantiation of a generic function or type produces specialized code. There is no runtime dispatch cost for generics.

---

## 4.12 Traits

### 4.12.1 Trait Declaration

A trait defines a set of methods that a type must implement:

```
trait Printable
  fn to_string(&self) -> String

trait Serializable
  fn serialize(&self) -> Vec<Byte>
  fn deserialize(data: &Vec<Byte>) -> Result<Self, DeserializeError>
```

### 4.12.2 Default Methods

Traits can provide default implementations:

```
trait Describable
  fn name(&self) -> String

  fn description(&self) -> String
    "A " + self.name()
```

### 4.12.3 Trait Implementation

```
impl Printable for Point
  fn to_string(&self) -> String
    "({self.x}, {self.y})"

impl Printable for Circle
  fn to_string(&self) -> String
    "Circle(r={self.radius})"
```

### 4.12.4 Trait Inheritance

Traits can extend other traits:

```
trait Ordered: Eq
  fn compare(self: Self, other: Self) -> Ordering
```

A type implementing `Ordered` must also implement `Eq`.

### 4.12.5 Associated Types

Traits can declare associated types:

```
trait Iterator
  type Item

  fn next(&mut self) -> Option<Self.Item>

impl Iterator for Counter
  type Item = Int

  fn next(&mut self) -> Option<Int>
    self.current += 1
    if self.current <= self.max
      Some(self.current)
    else
      None
```

### 4.12.6 Orphan Rule

A trait can be implemented for a type only if either the trait or the type is defined in the current module. This prevents conflicting implementations from different crates.

### 4.12.7 Dynamic Dispatch

When the concrete type is not known at compile time, trait objects provide dynamic dispatch:

```
fn print_all(items: &Vec<&dyn Printable>)
  for item in items
    println(item.to_string())
```

Trait objects use `&dyn Trait` or `Box<dyn Trait>`. Only object-safe traits can be used as trait objects (no `Self` return types, no generic methods).

---

## 4.13 Type Inference

### 4.13.1 Inference Scope

Monel requires explicit type annotations at module boundaries and infers types within function bodies:

**Must be annotated explicitly:**
- Function parameter types
- Function return types
- Struct field types
- Enum variant field types
- Trait method signatures
- Public constant types
- `distinct type` declarations

**Inferred automatically:**
- Local variable types (`let x = 42` infers `Int`)
- Closure parameter and return types (when unambiguous)
- Generic type arguments at call sites
- Intermediate expression types

### 4.13.2 Inference Algorithm

The compiler uses bidirectional type checking with local constraint solving:

1. **Check mode**: When the expected type is known (e.g., from a type annotation or function return type), the expression is checked against that type.
2. **Synth mode**: When no expected type is available, the type is synthesized bottom-up from the expression structure.
3. **Unification**: Type variables introduced by generics are unified with concrete types as constraints are gathered.

Type inference never crosses function boundaries. Each function is checked independently using only the signatures of called functions, not their bodies.

### 4.13.3 Inference Failures

When inference fails, the compiler reports an error with a suggestion to add an annotation:

```
let items = []   // ERROR: cannot infer element type for empty collection
                 // SUGGESTION: let items: Vec<Int> = []
```

### 4.13.4 Numeric Literal Inference

Unsuffixed integer literals default to `Int`. Unsuffixed float literals default to `Float`. When the expected type is known, literals adopt that type:

```
let x = 42                // Int
let y: UInt8 = 42         // UInt8 (literal checked against expected type)
let z: Float32 = 3.14     // Float32
```

---

## 4.14 Type Coercions and Conversions

### 4.14.1 Implicit Coercions

Monel performs a small set of implicit coercions:

| From | To | Condition |
|------|----|-----------|
| `mut T` | `T` | Always (mutable ref coerces to immutable) |
| `T` | borrowed `T` | When a borrow is expected and the value is an lvalue |
| `Never` | any type | Always (bottom type) |
| `T where P` | `T` | Always (refinement subtyping) |
| `Array<T, N>` | `Array<T>` | When a dynamically-sized array is expected |

No other implicit coercions exist. In particular:
- No implicit numeric widening (`Int8` does not coerce to `Int`).
- No implicit `String` conversions.
- No implicit truth-value coercions (`Int` does not coerce to `Bool`).

### 4.14.2 Explicit Conversions

The `as` keyword performs explicit conversions between numeric types:

```
let x: Int = 256
let y: UInt8 = x as UInt8   // truncates: y == 0
let z: Float = x as Float   // lossless: z == 256.0
```

Conversions that may lose information (truncation, float-to-int) are allowed with `as` but produce a warning if the source value is not a compile-time constant. Use `try_into()` for checked conversions:

```
let x: Int = 256
let y: Result<UInt8, OverflowError> = x.try_into()
```

### 4.14.3 The `From` and `Into` Traits

Infallible conversions implement `From<T>`:

```
impl From<Int32> for Int
  fn from(x: Int32) -> Int
    // lossless widening
```

The `Into<T>` trait is automatically derived from `From<T>`. User-defined types should implement `From`, and callers use `Into`:

```
fn process(id: impl Into<UserId>)
  let id = id.into()
  // ...
```

---

## 4.15 Const Evaluation

Expressions marked `const` are evaluated at compile time:

```
const MAX_CONNECTIONS: Int = 1024
const BUFFER_SIZE: UInt = 4 * 1024 * 1024

const fn factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1)

const FACT_10: Int = factorial(10)
```

### 4.15.1 What Can Be `const`

- Arithmetic and logical operations on primitives
- `const fn` calls
- Struct and enum construction with const fields
- Array and tuple construction with const elements
- Pattern matching on const values
- `if`/`else` with const conditions

### 4.15.2 What Cannot Be `const`

- Heap allocation
- Any effectful operation
- Trait method calls (unless the concrete type is known)
- Loops (use recursion in `const fn` instead)

---

## 4.16 Type Contracts and Verification

Types can carry contracts (refinements, field declarations) that the compiler verifies against the implementation.

### 4.16.1 Type Declarations with Contracts

In `.mn` files, types are declared with optional documentation and constraints:

```
type Port = Int where value >= 1 and value <= 65535
  doc: "A valid TCP/UDP port number"

struct UserProfile
  doc: "Complete user profile for display"
  id: UserId
  name: NonEmptyString
  email: ValidEmail
  created_at: Timestamp
```

### 4.16.2 Compiler Verification Rules

The compiler verifies:

1. **Base type match**: The underlying type agrees (e.g., both based on `Int`).
2. **Field match**: Struct fields match in name, type, and order.
3. **Refinement enforcement**: The `where` clause is enforced at construction sites — either via runtime checks or SMT proof obligations when `requires`/`ensures` contracts are present.
4. **Nominal agreement**: Types declared as semantically distinct (e.g., `UserId` vs. `OrderId`) must use `distinct type`.
5. **Variant match**: For enum types, all declared variants must exist with matching field types.

---

## 4.17 Summary of Typing Rules

| Rule | Description |
|------|-------------|
| Structural equivalence | Two non-distinct types with the same shape are interchangeable |
| Nominal distinctness | `distinct type` prevents structural equivalence |
| Refinement subtyping | `T where P` is a subtype of `T` |
| Ownership uniqueness | Every value has exactly one owner |
| Borrow exclusivity | `&mut T` is exclusive; `&T` is shared |
| Lifetime inference | All lifetimes are compiler-inferred |
| Monomorphization | Generics are specialized at compile time |
| Explicit boundaries | Function signatures require type annotations |
| Local inference | Types within function bodies are inferred |
| Effect awareness | Pointer dereference requires `unsafe` effect |
| Contract enforcement | Type contracts are verified by the compiler |

---

## 4.18 Grammar (Informative)

The following grammar fragments describe type syntax. See Appendix A for the complete grammar.

```
type_expr     = primitive_type
              | tuple_type
              | array_type
              | ref_type
              | ptr_type
              | fn_type
              | generic_type
              | path_type
              | refined_type
              | "(" type_expr ")"

primitive_type = "Int" | "Int8" | "Int16" | "Int32" | "Int64"
               | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
               | "Float" | "Float32"
               | "Bool" | "Byte" | "Char" | "String"
               | "Unit" | "Never"

tuple_type    = "(" type_expr ("," type_expr)* ")"
array_type    = "Array" "<" type_expr ("," const_expr)? ">"
ref_type      = "&" "mut"? type_expr
ptr_type      = ("Ptr" | "MutPtr") "<" type_expr ">"
fn_type       = "fn" "(" (type_expr ("," type_expr)*)? ")" "->" type_expr
generic_type  = IDENT "<" type_arg ("," type_arg)* ">"
type_arg      = type_expr | const_expr
path_type     = IDENT ("::" IDENT)*
refined_type  = type_expr "where" predicate_expr

distinct_decl = "distinct" "type" IDENT "=" type_expr
              | "distinct" "struct" IDENT struct_body
type_alias    = "type" IDENT "=" type_expr
```
