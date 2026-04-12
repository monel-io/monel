# Parser Plan

## Goal

Parse `.mn` files into ASTs. Replace all hand-constructed test ASTs with parsed ones. This is the critical path — every other roadmap item is blocked on this.

## Success Metrics

| Metric | Target | How to measure |
|--------|--------|----------------|
| **Example files parsed** | 5/5 (hello, stack, http_handler, terminal, web_server) | `cargo test` — one integration test per file that parses without error |
| **Construct coverage** | All constructs used in examples parse correctly | Checklist below; tracked as `#[test]` per construct |
| **Contract clause coverage** | 8/8 clause types parsed | `requires:`, `ensures:`, `effects:`, `panics:`, `complexity:`, `invariant:`, `fails:`, `doc:` |
| **Parity tests migrated** | 12/12 hand-constructed ASTs replaced | All tests in `parity/tests.rs` parse from `.mn` text instead of building ASTs by hand |
| **Roundtrip fidelity** | parse → pretty-print → parse = same AST | Automated test per example file |
| **Error quality** | Errors include line/column and expected token | Manual review of 5 intentionally broken inputs |

Track progress by running `cargo test -p monel-parser` — each passing test proves one more construct works.

## Architecture Decision: Unified AST

The existing code has two AST types from the old two-file model:
- `IntentFile` (contracts only — from `.mn.intent`)
- `ImplFile` (code only — from `.mn`)

The new single-file model needs a **unified AST** where contracts and body coexist:

```rust
// New: single SourceFile replaces both IntentFile and ImplFile
struct SourceFile {
    module: Option<String>,
    imports: Vec<Import>,
    declarations: Vec<Decl>,
}

enum Decl {
    Fn(FnDecl),
    Type(TypeDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Const(ConstDecl),
    StateMachine(StateMachineDecl),
}

// Contracts and body together
struct FnDecl {
    name: String,
    visibility: Visibility,
    is_async: bool,
    type_params: Vec<TypeParam>,
    params: Vec<Param>,
    return_type: Type,
    // Contracts (all optional)
    doc: Option<String>,
    requires: Vec<Expr>,
    ensures: Vec<EnsuresClause>,
    effects: Vec<Effect>,
    panics: PanicsDecl,           // Never | Conditions(Vec<Expr>) | Unspecified
    complexity: Option<Complexity>,
    fails: Vec<FailsEntry>,
    // Body (None for trait method signatures)
    body: Option<Expr>,
}
```

The parity checker currently compares `IntentFile` vs `ImplFile`. After the AST change, it compares contracts vs body within the same `SourceFile`. This is a refactor of the checker, not just the parser.

**Migration path:**
1. Define new unified AST in `monel-core`
2. Build parser producing unified AST
3. Refactor parity checker to work on unified AST
4. Delete old `IntentFile`/`ImplFile` types
5. Migrate tests

## Parser Framework: Hand-written Recursive Descent

Indentation-sensitive grammars are hard to express in PEG/parser combinator frameworks. Python, Haskell, and Rust (rustc) all use hand-written recursive descent for this reason.

**Approach:**
1. **Lexer** (with logos for tokenization) emits `Indent`, `Dedent`, `Newline` tokens
2. **Parser** is recursive descent consuming the token stream
3. **Error recovery** via synchronization points (next top-level `fn`, `type`, `use`)

**Crate:** `monel-parser` (rename from `monel-intent-parser`)

## Construct Checklist

Ordered by what the examples need, not by spec chapter.

### Phase 1: Minimal — parse `hello.mn`

This is the smallest example and exercises the most constructs.

- [ ] Lexer: identifiers, integers, strings, operators, comments (`#`)
- [ ] Lexer: indentation tracking (Indent/Dedent emission)
- [ ] `use` imports: `use std/io {println}`
- [ ] `type` alias: `type Name = distinct String`
- [ ] `type` struct with fields: `type Greeter` + indented fields
- [ ] `type` enum with variants: `type GreetError | EmptyMessage | TooLong(Int)`
- [ ] `invariant:` on types (multi-line predicates)
- [ ] `fn` declarations with params, return type
- [ ] `fn` compact form: `fn name(self: T) -> R = expr`
- [ ] Contract clauses: `requires:`, `ensures:`, `effects:`, `panics:`, `complexity:`
- [ ] `ensures:` with `ok =>`, `err(Variant) =>`, `result is Some(s) =>`
- [ ] `panics: never` vs `panics: condition`
- [ ] `complexity: O(n) where n = expr`
- [ ] Expressions: if/return, let, assignment, method calls, field access, string interpolation
- [ ] `match` with `| pattern => expr`
- [ ] `for` loops with `invariant:`
- [ ] `try expr`
- [ ] Struct construction (indented, no braces)

**Done when:** `cargo test parse_hello` passes — parsed AST matches expected structure.

### Phase 2: Core — parse `stack.mn`

Adds generics, traits, impl blocks, visibility, constants, owned self.

- [ ] `pub` visibility: `pub fn`, `pub type`, `pub const`, `pub trait`
- [ ] `const` declarations: `pub const DEFAULT_CAPACITY: Int = 16`
- [ ] Inherent `impl` blocks: `impl Stack<T>` with indented methods
- [ ] `trait` definitions with associated types: `type Item`
- [ ] Default trait methods (method with body inside trait)
- [ ] `impl Trait for Type`: `impl Display for Stack<T>`
- [ ] `where` clauses on impl: `where T: Display`
- [ ] `self: owned T` (ownership transfer)
- [ ] Generic type params: `<T>`, `<T: Ord + Clone>`
- [ ] `old()` in ensures clauses
- [ ] Closures: `fn() -> Option<T>`
- [ ] Integer literals with underscores: `1_000_000`

### Phase 3: Full — parse `http_handler.mn`, `terminal.mn`, `web_server.mn`

Adds async, unsafe, state machines, layouts, interactions.

- [ ] `async fn` / `await`
- [ ] `unsafe` blocks
- [ ] `state_machine` declarations (states, transitions `--event-->`)
- [ ] `layout` declarations
- [ ] `interaction` declarations
- [ ] Struct update syntax: `Post { ..p, title: new }`
- [ ] Named arguments: `configure(port: 8080)`
- [ ] `while` / `loop` / `break` / `continue`

### Phase 4: Parity Migration

- [ ] Refactor parity checker to use unified AST
- [ ] Replace all 12 hand-constructed test ASTs
- [ ] Roundtrip tests for all 5 example files

## File Structure

```
crates/
  monel-parser/               # renamed from monel-intent-parser
    src/
      lib.rs                  # pub fn parse(input: &str) -> Result<SourceFile, ParseError>
      lexer.rs                # Token types, Indent/Dedent emission
      parser.rs               # Recursive descent
      error.rs                # ParseError with line/col/expected
    tests/
      hello.rs                # parse hello.mn
      stack.rs                # parse stack.mn
      http_handler.rs         # parse http_handler.mn
      terminal.rs             # parse terminal.mn
      web_server.rs           # parse web_server.mn
  monel-core/
    src/
      ast.rs                  # Unified AST (replaces intent_ast.rs + impl_ast.rs)
      parity/                 # Refactored to work on unified AST
```

## Non-Goals (for this step)

- Type checking / type inference
- Borrow checking / lifetime analysis
- Effect inference from function bodies
- SMT translation of contract expressions
- Code generation
- Semantic analysis beyond parsing

The parser produces a syntactic AST. Semantic analysis (are types valid? do effects match?) is Step 2 (`monel check`).

## Estimated Scope

| Component | Rough size |
|-----------|-----------|
| Lexer + indentation | ~400 lines |
| Type expression parser | ~150 lines |
| Declaration parser (fn, type, trait, impl, use) | ~500 lines |
| Contract clause parser | ~200 lines |
| Expression parser (precedence climbing) | ~400 lines |
| Pattern parser | ~150 lines |
| Error types + recovery | ~150 lines |
| Unified AST definition | ~200 lines |
| Parity checker refactor | ~300 lines |
| Tests | ~500 lines |
| **Total** | **~3000 lines** |

## Risk

**Indentation ambiguity.** The hardest part is distinguishing contract clauses from body code — both are indented under `fn`. The rule: lines starting with a contract keyword (`requires:`, `ensures:`, `effects:`, `panics:`, `complexity:`, `fails:`, `doc:`) at the first indentation level below `fn` are contracts. The first line that doesn't match a contract keyword starts the body. This is unambiguous if we commit to it, but it means a local variable named `effects` would be a parse error inside a function body at contract indentation level. Acceptable — these are reserved keywords.

**Expression complexity.** The examples use string interpolation (`"{self.name}"`), method chaining, closures, and pattern matching. A production-quality expression parser is significant work. For Phase 1, we can parse expressions as opaque token sequences (skip body parsing) and still prove the grammar works for declarations and contracts.

## Open Questions

1. **Body parsing depth.** Do we parse function bodies into a full expression AST, or treat them as opaque text for now? Full AST is needed for `monel check` (Step 2) but not for proving the grammar works. Recommend: parse bodies into AST from the start — we'll need it soon and retrofitting is harder.

2. **Expression AST for contracts.** Contract predicates like `self.len == old(self.len) + 1` need to be parsed into expression ASTs (for SMT translation later). Same expression parser as bodies, but with `old()` and `result` as special forms.

3. **Parser combinator vs fully hand-written.** Logos for lexing + hand-written recursive descent for parsing is the recommendation. Alternative: use `chumsky` which handles error recovery well. Decision needed before starting.
