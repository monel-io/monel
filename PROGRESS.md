# Monel — Progress & Roadmap

Last updated: 2026-03-18

## What Exists

### Specification (v0.2)

12 chapters + 4 worked examples. Rewritten from two-file intent/impl architecture to single-file contracts-alongside-code.

```
spec/01-overview.md        — Philosophy, architecture, 4-stage pipeline
spec/02-intent-syntax.md   — Contract syntax (requires:, ensures:, invariant:, effects:)
spec/03-impl-syntax.md     — EBNF grammar for .mn files (contracts + implementation)
spec/04-type-system.md     — Structural/nominal types, ownership, generics
spec/05-effects.md         — Effect categories, implication DAG, budgets, policies
spec/06-parity.md          — Verification: SMT contracts, effect checking, coverage metrics
spec/07-modules.md         — File-based modules, monel.project manifest
spec/08-errors.md          — Result<T,E>, try, panics:never, per-error-variant postconditions
spec/09-concurrency.md     — async/await, structured concurrency
spec/10-systems.md         — unsafe, FFI, PTY, GPU
spec/11-tooling.md         — Query oracle, monel context, dev server
spec/12-stdlib.md          — All std/* modules with contract signatures
spec/examples/             — stack, http_handler, terminal, web_server (single .mn files)
```

Key architecture changes (v0.1 → v0.2):
- **Single-file**: Contracts and implementation co-located in `.mn` files. No `.mn.intent` files.
- **No LLM in compiler**: All verification is deterministic (type checking, borrow checking, effect checking, SMT contract verification). LLM is an authoring tool only.
- **Formal contracts**: `requires:`, `ensures:`, `invariant:`, `effects:`, `panics: never` — verified by SMT solver, not LLM.
- **Per-error-variant postconditions**: `err(Overflow) => self == old(self)` — unique to Monel.
- **`doc:` replaces `does:`**: Optional documentation string, never compiled or verified.
- **Verification coverage**: Compiler tracks what's SMT-proven vs property-tested vs uncovered. Metric = (proven ∪ tested) / total behavior.
- **Contract-driven test generation**: `monel test --gen-contract-tests` mechanically generates property tests from contracts.

### Compiler Infrastructure (Rust)

```
crates/
  monel-core/              — AST types + structural parity checker (working, 12 tests)
  monel-cli/               — Placeholder binary (prints version)
  monel-intent-parser/     — Placeholder (parse() stub)
  monel-bench/             — Benchmark: Monel vs Rust for AI agent efficiency
  xtask/                   — Build tasks (mdbook install/build/serve)
```

### Parity Checker (monel-core)

The structural parity checker was built against the v0.1 two-file architecture. It verifies correspondence between intent and implementation ASTs. Still passes all tests but needs updating for the single-file model (contracts and code parsed from the same `.mn` file).

What it checks:
- Function existence (declared contract must have implementation, and vice versa)
- Signature matching (params, return types, generics with type parameter mapping)
- Effect subset checking with implication expansion (Db.write implies Db.read)
- Error variant exhaustiveness (declared vs produced)
- Type parity (struct fields, enum variants, payloads)

Error codes: P0001-P0002 (existence), P0101-P0107 (signatures), P0201-P0202 (effects), P0301-P0302 (errors), P0401-P0413 (types).

All 12 tests pass. Tested against stack, http_handler, and terminal example scenarios.

### Benchmark (monel-bench)

Compares AI agent efficiency when modifying code in Rust vs Monel. Uses the HTTP handler "add rate limiting" scenario.

See [Benchmark Results](#benchmark-results) below for what's real vs hypothetical.

---

## What's Proven

These claims have running code and passing tests behind them.

| Claim | Evidence |
|-------|----------|
| Parity checker catches undeclared effects | Test + benchmark scenario: adding Net.send without updating contracts |
| Parity checker catches undeclared error variants | Test + benchmark scenario: adding RateLimited to AuthError |
| Parity checker catches missing implementations | Test: contract fn with no matching impl fn |
| Parity checker catches orphan implementations | Test: impl fn with no contract declaration |
| Parity checker catches return type mismatches | Test + benchmark: Option\<Session\> vs Result\<Session, AuthError\> |
| Effect implication DAG works | Test: Db.write implies Db.read, Http.send implies Net.connect+Net.send |
| Generic type matching with parameter mapping | Test: Vec\<T\> to Option\<T\> with consistent mapping |
| Rust misses 4/5 of these bug classes | Benchmark 4: Rust compiler only catches type mismatches |
| Contract files reduce AI agent tool calls ~2x | Real agent measurement: 29 tool calls (Rust) vs 15 (Monel) for same task |

## What's Hypothetical

These are claimed in the spec or benchmark but have no code behind them.

### High Risk (could kill the project if wrong)

| Claim | Status | Why it's risky |
|-------|--------|----------------|
| `.mn` files (contracts + impl) can be parsed into ASTs | No parser exists. All ASTs are hand-constructed in tests. | If the grammar is ambiguous or the indentation-based syntax is unparseable, everything downstream breaks. The parity checker is proven but useless without parsed input. |
| Lifetime inference with 3 elision rules + whole-program analysis | Spec only (§4.10.4), acknowledged as underspecified | Rust spent years on lifetime inference and still has explicit lifetimes. The spec's "Rule 4: whole-program analysis" is a wish, not a specification. |
| SMT verification of `requires:`/`ensures:` clauses | Spec only (§6) | Z3 integration is well-understood engineering, but contract expressions need precise semantics. Bounded quantifiers and `old()` references add complexity. |

### Medium Risk

| Claim | Status | Why |
|-------|--------|-----|
| `monel query context` gives 86% token savings | Benchmark has a hardcoded simulated output string. No command exists. | Output format is straightforward once parser exists, but real savings depend on codebase size. The 86% number is from a tiny example. |
| Effect-aware codegen (pure memoization, panics:never elimination) | Spec only (§1.7) | Optimization passes need the full pipeline (parser → type checker → codegen). |
| Contract-driven property test generation | Spec only (§12.19.4) | Mechanically translating `requires:`/`ensures:` into property tests is well-defined, but generator quality matters. |
| Verification coverage metric | Spec only (§6) | Tracking (proven ∪ tested) / total behavior requires defining "total behavior" precisely. |

### Low Risk

| Claim | Status | Why |
|-------|--------|-----|
| Policy-level forbidden effect combinations | No code | Straightforward rule check once effects are tracked. |
| Effect budget instrumentation | No code | Compiler-inserted counters. Proven concept in other systems. |

### Benchmark Honesty

| Benchmark | Verdict |
|-----------|---------|
| B1: Token efficiency — Rust row | **Real** — measured from actual baseline files |
| B1: Token efficiency — Monel full files | **Real** — measured from actual example files |
| B1: Token efficiency — Monel query context | **Fake** — hardcoded simulated output, command doesn't exist |
| B2: Drift detection | **Real** — parity checker runs on mutated ASTs |
| B3: Spec density — line counts | **Hand-counted** — reasonable but not computed from files |
| B4: Bug detection | **Real** — parity checker runs all 5 mutation scenarios |
| B2: "Forbidden effect combination" claim | **Fake** — no policy checker exists |

---

## Roadmap: Turning Hypotheticals Into Proof

Ordered by risk reduction — each step makes the next one possible.

### Step 1: Parser

**Risk addressed:** Highest risk hypothetical — can we parse `.mn` files with interleaved contracts and implementation?

Build a parser that takes `stack.mn` (real file on disk) and produces both contract AST and implementation AST from the same file. Then re-run the parity checker on parsed ASTs instead of hand-constructed ones.

**Done when:** `cargo test` parses all 4 example `.mn` files and extracts contracts + implementation. Zero hand-constructed ASTs remain in tests.

**Unblocks:** Everything. Contract verification, query commands, real benchmarks.

### Step 2: `monel check` CLI Command

**Risk addressed:** Is the contract verification pipeline usable as a real tool?

Wire up: `monel check path/to/module.mn` parses the file, extracts contracts and implementation, runs structural checks (effects, error variants, signatures), prints errors with codes and locations.

**Done when:** Running `monel check spec/examples/stack.mn` on an intentionally broken file produces the correct verification errors.

**Unblocks:** Real developer workflow testing, `monel query context`.

### Step 3: Compiler Query Oracle

**Risk addressed:** The 86% token savings claim and AI agent integration story.

Build `monel query context`, `monel query fn`, `monel query type`. Takes a symbol name, returns its contracts + implementation + dependency types + callers. Measure real token count against the Rust baseline.

**Done when:** `monel query context fn authenticate --format json` returns structured output. Benchmark B1 uses this real output instead of the hardcoded string.

**Unblocks:** Real AI agent benchmarks, integration with Claude Code via Bash tool calls.

### Step 4: SMT Contract Verification

**Risk addressed:** Core differentiator — can the compiler prove contracts hold?

Integrate Z3 (or similar SMT solver). Verify `requires:`/`ensures:` clauses against implementation. Start with arithmetic contracts on the stack example, then expand to the web server example.

**Done when:** `monel check spec/examples/stack.mn` proves that `push` satisfies `ensures: ok => self.len == old(self.len) + 1` and that the type invariant `self.len <= self.capacity` is maintained.

**Unblocks:** The "compiler proves what you specified" claim. Verification coverage metrics.

### Step 5: Contract-Driven Test Generation

**Risk addressed:** "Property tests find what you forgot to specify."

Implement `monel test --gen-contract-tests`. Mechanically generate property tests from `requires:`/`ensures:` clauses. Run with a property testing framework.

**Done when:** Running `monel test --gen-contract-tests spec/examples/stack.mn` produces tests equivalent to the hand-written `stack.mn.test` and all tests pass.

**Unblocks:** Verification coverage metric, the full "proven or tested" story.

### Step 6: Real AI Agent Benchmark

**Risk addressed:** Does Monel actually help AI agents in practice, or only in theory?

Give Claude Code a task in both the Rust baseline and the Monel codebase (with `monel query context` available as a tool). Measure tool calls, tokens, correctness, and drift detection.

**Done when:** Benchmark runs with real agents, real tool calls, real token counts. No simulated numbers.

### Later (not blocking)

- Effect-aware codegen passes
- Compiler-as-server / persistent semantic model
- `monel query blast --hypothetical`
- Lifetime inference specification
- LLM-assisted test generation (`monel test --gen-llm-tests`)
- Verification coverage reporting

---

## Benchmark Results

Run with `cargo run --package monel-bench`. Current output:

```
Benchmark 2: Drift Detection — REAL (parity checker runs live)

  Rust compiler:  PASS (compiles, no drift detection)
  Monel parity:   3 errors caught
    - error[P0202]: undeclared effect Net.send
    - error[P0302]: undeclared error variant RateLimited
    - error[P0402]: type AuthError has variant RateLimited not declared in intent

Benchmark 4: Bug Detection — REAL (parity checker runs live)

  Bug Scenario                              Rust    Monel
  AI adds undeclared database write         MISS    CATCH
  AI adds error variant not in spec         MISS    CATCH
  AI forgets to implement a declared fn     MISS    CATCH
  AI adds helper without @no_intent         MISS    CATCH
  AI changes return type subtly             CATCH   CATCH

  Rust: 1/5    Monel: 5/5

AI Agent Exploration — REAL (actual Claude agents, measured 2026-03-17)

  Metric          Rust Baseline    Monel
  Tool calls      29               15
  Files read      9                5
  Total tokens    38,109           41,543
  Time            65s              58s

  Note: Token counts are similar because Monel agent also read spec chapters.
  The win is tool calls (2x fewer) and information quality (effects, contracts,
  panic guarantees available before reading implementation).
```
