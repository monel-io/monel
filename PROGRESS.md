# Monel — Progress & Roadmap

Last updated: 2026-03-17

## What Exists

### Specification (v0.1)

12 chapters + 3 worked examples (~14,000 lines). 9-persona review applied, all P0 contradictions resolved.

```
spec/01-overview.md        — Philosophy, architecture, SDD positioning, AI-native codegen
spec/02-intent-syntax.md   — EBNF grammar for .mn.intent
spec/03-impl-syntax.md     — EBNF grammar for .mn
spec/04-type-system.md     — Structural/nominal types, ownership, refinement
spec/05-effects.md         — Effect categories, implication DAG, budgets, policies
spec/06-parity.md          — Structural + semantic verification, caching
spec/07-modules.md         — File-based modules, monel.project manifest
spec/08-errors.md          — Result<T,E>, try, panics:never
spec/09-concurrency.md     — async/await, structured concurrency
spec/10-systems.md         — unsafe, FFI, PTY, GPU
spec/11-tooling.md         — Query oracle, monel context, hot-swap
spec/12-stdlib.md          — All std/* modules with intent signatures
spec/examples/             — stack, http_handler, terminal (intent + impl pairs)
```

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

The critical feasibility proof. Verifies structural correspondence between intent and implementation ASTs.

What it checks:
- Function existence (intent fn must have impl fn, and vice versa)
- Signature matching (params, return types, generics with type parameter mapping)
- Effect subset checking with implication expansion (Db.write implies Db.read)
- Error variant exhaustiveness (declared vs produced)
- Type parity (struct fields, enum variants, payloads)
- @no_intent annotation handling

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
| Parity checker catches undeclared effects | Test + benchmark scenario: adding Net.send without updating intent |
| Parity checker catches undeclared error variants | Test + benchmark scenario: adding RateLimited to AuthError |
| Parity checker catches missing implementations | Test: intent fn with no matching impl fn |
| Parity checker catches orphan implementations | Test: impl fn with @intent tag but no intent declaration |
| Parity checker catches return type mismatches | Test + benchmark: Option\<Session\> vs Result\<Session, AuthError\> |
| Effect implication DAG works | Test: Db.write implies Db.read, Http.send implies Net.connect+Net.send |
| Generic type matching with parameter mapping | Test: Vec\<T\> to Option\<T\> with consistent mapping |
| Rust misses 4/5 of these bug classes | Benchmark 4: Rust compiler only catches type mismatches |
| Intent files reduce AI agent tool calls ~2x | Real agent measurement: 29 tool calls (Rust) vs 15 (Monel) for same task |
| Intent front-loads information agents can't get from Rust | Agent got effects, contracts, panic guarantees from intent before reading impl |

## What's Hypothetical

These are claimed in the spec or benchmark but have no code behind them.

### High Risk (could kill the project if wrong)

| Claim | Status | Why it's risky |
|-------|--------|----------------|
| `.mn.intent` and `.mn` files can be parsed into ASTs | No parser exists. All ASTs are hand-constructed in tests. | If the grammar is ambiguous or the indentation-based syntax is unparseable, everything downstream breaks. The parity checker is proven but useless without parsed input. |
| Lifetime inference with 3 elision rules + whole-program analysis | Spec only (§4.10.4), acknowledged as underspecified | Rust spent years on lifetime inference and still has explicit lifetimes. The spec's "Rule 4: whole-program analysis" is a wish, not a specification. |

### Medium Risk

| Claim | Status | Why |
|-------|--------|-----|
| `monel query context` gives 86% token savings | Benchmark has a hardcoded simulated output string. No command exists. | Output format is straightforward once parser exists, but real savings depend on codebase size. The 86% number is from a tiny example. |
| Effect-aware codegen (pure memoization, panics:never elimination) | Spec only (§1.7 Stage 5) | Optimization passes need the full pipeline (parser → type checker → codegen). Cool but far from provable today. |
| Compiler-as-persistent-server with speculative analysis | Spec only (§1.7.1) | Architecture decision more than feasibility question. Needs parser + semantic model first. |

### Core Differentiator (High Importance, Uncertain Feasibility)

| Claim | Status | Why it matters |
|-------|--------|----------------|
| Semantic parity via LLM (Stage 4, `does:` verification) | No code | This is the "compiler enforces human intent" claim. Without it, Monel verifies interfaces (signatures, effects, errors) but not intent (does this code actually authenticate a user?). Structural checks alone are valuable but closer to "a thorough linter" than the full vision. The challenge: LLM verification is non-deterministic and can hallucinate. Building the core differentiator on probabilistic judgment needs careful design — caching, confidence thresholds, committee voting, and clear communication that this is advisory, not proof. |

### Low Risk

| Claim | Status | Why |
|-------|--------|-----|
| Policy-level forbidden effect combinations | No code | Straightforward rule check once effects are tracked. Low engineering risk. |
| Effect budget instrumentation | No code | Compiler-inserted counters. Proven concept in other systems. |

### Benchmark Honesty

| Benchmark | Verdict |
|-----------|---------|
| B1: Token efficiency — Rust row | **Real** — measured from actual baseline files |
| B1: Token efficiency — Monel full files | **Real** — measured from actual example files |
| B1: Token efficiency — Monel query context | **Fake** — hardcoded simulated output, command doesn't exist |
| B2: Drift detection | **Real** — parity checker runs on mutated ASTs |
| B3: Spec density — line counts | **Hand-counted** — reasonable but not computed from files |
| B3: Verifiability table — "parity S3" and "parity S4" | **Partially fake** — S3 (contracts) and S4 (semantic) don't exist |
| B4: Bug detection | **Real** — parity checker runs all 5 mutation scenarios |
| B2: "Forbidden effect combination" claim | **Fake** — no policy checker exists |

---

## Roadmap: Turning Hypotheticals Into Proof

Ordered by risk reduction — each step makes the next one possible.

### Step 1: Intent Parser

**Risk addressed:** Highest risk hypothetical — can we parse `.mn.intent` files?

Build a parser that takes `http_handler.mn.intent` (real file on disk) and produces the same `IntentFile` AST the parity checker already consumes. Then re-run the parity checker on parsed ASTs instead of hand-constructed ones.

**Done when:** `cargo test` parses all 3 example `.mn.intent` files and parity-checks them against hand-constructed impl ASTs. Zero hand-constructed intent ASTs remain in tests.

**Unblocks:** Everything. Impl parser, query commands, real benchmarks.

### Step 2: Impl Parser

**Risk addressed:** Same as Step 1 but for `.mn` files.

Parse `http_handler.mn` into `ImplFile`. Combined with Step 1, this means parity checking works on real files end-to-end.

**Done when:** `cargo test` parses all 3 example `.mn` + `.mn.intent` pairs and parity-checks them. Zero hand-constructed ASTs remain anywhere.

**Unblocks:** Real benchmarks, `monel check` command.

### Step 3: `monel check` CLI Command

**Risk addressed:** Is the parity checker usable as a real tool?

Wire up: `monel check path/to/module` parses intent + impl, runs parity, prints errors with codes and locations.

**Done when:** Running `monel check spec/examples/http_handler` on an intentionally broken file produces the correct parity errors.

**Unblocks:** Real developer workflow testing, `monel query context`.

### Step 4: `monel query context`

**Risk addressed:** The 86% token savings claim.

Build the actual command. Takes a function name, returns its intent + impl + dependency types + callers. Measure real token count against the Rust baseline.

**Done when:** `monel query context fn authenticate --format json` returns structured output. Benchmark B1 uses this real output instead of the hardcoded string.

**Unblocks:** Real AI agent benchmarks, integration with Claude Code via Bash tool calls.

### Step 5: Real AI Agent Benchmark

**Risk addressed:** Does Monel actually help AI agents in practice, or only in theory?

Give Claude Code a task in both the Rust baseline and the Monel codebase (with `monel query context` available as a tool). Measure tool calls, tokens, correctness, and drift detection.

**Done when:** Benchmark runs with real agents, real tool calls, real token counts. No simulated numbers.

### Step 6: Semantic Parity Proof of Concept

**Risk addressed:** Core differentiator — can an LLM reliably verify that code matches its `does:` description?

Without this, Monel verifies interfaces (signatures, effects, errors) but not intent. The `does:` string is where human intent lives. This is the difference between "a thorough linter" and "a compiler that enforces human intent."

Build a minimal Stage 4 prototype: take a `does:` string and its implementation, send to an LLM, return pass/fail with confidence score. Test against the 3 example files with intentionally wrong implementations.

**Challenges:** Non-deterministic output, model version coupling, hallucination risk. The prototype must address: caching (same input = same result), confidence thresholds (what score means "pass"?), and clear communication that this is advisory verification, not proof.

**Done when:** Running `monel check --semantic spec/examples/http_handler` on a correct implementation passes, and on an intentionally wrong implementation (e.g., `does: "authenticate user"` but code just returns `Ok(Session::default())`) fails with a meaningful explanation.

**Unblocks:** The full vision. Without this, Monel is a structural verification tool. With it, Monel verifies intent.

### Later (not blocking)

- Effect-aware codegen passes
- Compiler-as-server / persistent semantic model
- `monel query blast --hypothetical`
- Lifetime inference specification

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
