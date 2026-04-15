# Monel — Progress & Roadmap

Last updated: 2026-03-23

## What Exists

### Specification (v0.2)

12 chapters + 5 worked examples + claims map. Rewritten from two-file intent/impl architecture to single-file contracts-alongside-code.

```
spec/01-overview.md        — Philosophy, architecture, 4-stage pipeline, competitive landscape
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
spec/claims.md             — Claims map: all claims with evidence status + prior art (mermaid)
spec/examples/             — hello, stack, http_handler, terminal, web_server
```

Key architecture changes (v0.1 → v0.2):
- **Single-file**: Contracts and implementation co-located in `.mn` files. No `.mn.intent` files.
- **Formal contracts**: `requires:`, `ensures:`, `invariant:`, `effects:`, `panics: never` verified by SMT solver.
- **Per-error-variant postconditions**: `err(Overflow) => self == old(self)` — similar to SPARK Ada's `Exceptional_Cases`, adapted for algebraic error types.
- **`doc:` replaces `does:`**: Optional documentation string, never compiled or verified.
- **Verification coverage**: Compiler tracks what's SMT-proven vs property-tested vs uncovered. Metric = (proven ∪ tested) / total behavior.
- **Contract-driven test generation**: `monel test --gen-contract-tests` mechanically generates property tests from contracts.

### Compiler Infrastructure (Rust)

```
crates/
  monel-core/              — AST types + structural parity checker (working, 12 tests)
  monel-cli/               — Placeholder binary (prints version)
  monel-parser/            — Placeholder (parse() stub)
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

## Evidence Score: 8 proven / 26 total claims (31%)

See [Claims Map](spec/claims.md) for the full diagram with prior art.

## What's Proven

Running code, passing tests.

| Claim | Evidence |
|-------|----------|
| Parity checker catches undeclared effects | Test + benchmark: adding Net.send without updating contracts |
| Parity checker catches undeclared error variants | Test + benchmark: adding RateLimited to AuthError |
| Parity checker catches missing implementations | Test: contract fn with no matching impl fn |
| Parity checker catches orphan implementations | Test: impl fn with no contract declaration |
| Parity checker catches return type mismatches | Test + benchmark: Option\<Session\> vs Result\<Session, AuthError\> |
| Effect implication DAG works | Test: Db.write implies Db.read, Http.send implies Net.connect+Net.send |
| Generic type matching with parameter mapping | Test: Vec\<T\> to Option\<T\> with consistent mapping |
| Rust misses 4/5 of these bug classes | Benchmark 4: Rust compiler only catches type mismatches |
| Contracts reduce AI agent tool calls ~2x | Real measurement: 29 tool calls (Rust) vs 15 (Monel) for same task |

**What this proves:** Monel's contract model catches real bug classes that Rust's type system misses, and AI agents navigate contract-annotated code more efficiently.

**What this does NOT prove:** That `.mn` files can be parsed, that SMT verification works, or that the language can compile anything. All evidence so far uses hand-constructed ASTs.

## What's Hypothetical

18 of 26 claims have no code behind them. Three are high risk (parser, lifetime inference, SMT verification). See [Claims Map](spec/claims.md) for the full breakdown with prior art and evidence status.

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

Ordered by risk reduction. Each step makes the next one possible.

```
Parser ──→ monel check ──→ Query Oracle ──→ Real Agent Benchmark
  │             │                │                    │
  │             │                └──→ Autoresearch Loop (optimize compiler for agents)
  │             │
  │             └──→ SMT Verification ──→ Contract-Driven Tests
  │             │
  │             └──→ KV Store, Shell (need effect checking)
  │
  └──→ JSON Parser (needs parsing only)
```

### Step 1: Parser ← CRITICAL PATH

**Risk:** Highest. Can we parse `.mn` files with interleaved contracts and implementation?

**What:** Build a parser that takes `stack.mn` (real file on disk) and produces ASTs. Re-run the parity checker on parsed ASTs instead of hand-constructed ones.

**Done when:** `cargo test` parses all 5 example `.mn` files and extracts contracts + implementation. Zero hand-constructed ASTs remain in tests.

**Unblocks:** Everything. Without this, all 18 hypothetical claims stay hypothetical.

**Evidence score after:** 8 → ~12 (proves F1, F3, and moves C5D from hypothetical to proven).

### Step 2: `monel check`

**Risk:** Is the contract verification pipeline usable as a real tool?

**What:** Wire up `monel check path/to/module.mn` — parse, extract contracts, run structural checks (effects, error variants, signatures), print errors with codes and locations.

**Done when:** `monel check spec/examples/stack.mn` on an intentionally broken file produces the correct verification errors.

**Unblocks:** Real developer workflow testing, query oracle, SMT verification.

### Step 3: Query Oracle

**Risk:** The 86% token savings claim and AI agent integration story.

**What:** Build `monel query context`, `monel query fn`, `monel query type`. Measure real token count against Rust baseline.

**Done when:** `monel query context fn authenticate --format json` returns structured output. Benchmark B1 uses real output instead of the hardcoded string.

**Unblocks:** Real AI agent benchmark (Step 6).

### Step 4: SMT Contract Verification

**Risk:** Core differentiator — can the compiler prove contracts hold?

**What:** Integrate Z3. Verify `requires:`/`ensures:` clauses against implementation. Start with arithmetic contracts on the stack example.

**Done when:** `monel check spec/examples/stack.mn` proves that `push` satisfies `ensures: ok => self.len == old(self.len) + 1` and that invariant `self.len <= self.capacity` is maintained.

**Unblocks:** The "compiler proves what you specified" claim. Verification coverage.

**Evidence score after:** ~12 → ~16 (proves C1A, C1B, C1C, C1F).

### Step 5: Contract-Driven Test Generation

**Risk:** "Property tests find what you forgot to specify."

**What:** Implement `monel test --gen-contract-tests`. Mechanically generate property tests from contracts.

**Done when:** Generated tests for `stack.mn` are equivalent to hand-written ones and all pass.

**Unblocks:** Verification coverage metric, the full "proven or tested" story.

### Step 6: Real AI Agent Benchmark

**Risk:** Does Monel actually help AI agents in practice, or only in theory?

**What:** A repeatable benchmark harness that gives Claude Code identical tasks in Rust and Monel, then measures everything.

**Metrics tracked per run:**

| Metric | What it measures | Baseline (Rust, 2026-03-17) |
|--------|-----------------|----------------------------|
| Tool calls | Exploration effort | 29 |
| Files read | Codebase surface touched | 9 |
| Edit-check cycles | Tries before correct solution | not yet measured |
| First-try correctness | Agent gets it right without compiler round-trip | not yet measured |
| Bugs caught at compile | Mistakes caught before runtime | 1/5 (type mismatch only) |
| Tokens consumed | Cost per task | 38,109 |
| Wall clock time | End-to-end speed | 65s |

**Benchmark tasks** (same task, both codebases):
1. "Add rate limiting to authenticate" (existing — measured once)
2. "Add a new error variant and handle it in all callers"
3. "Refactor: extract a helper function from authenticate"
4. "Add logging to all database writes" (effect-sensitive)

**Harness:** Script that runs Claude Code in headless mode with a fixed prompt, captures the transcript, extracts metrics, appends to `benchmarks/results.jsonl`. Run after each language/tooling change to detect regressions.

**Done when:** All 4 tasks run on both codebases, metrics extracted automatically, results tracked in version control.

**Evidence score after:** ~16 → ~20 (proves C3A, C3B with real data, C3C, C3D).

### Step 7: Validation Projects

**Risk:** Do real programs expose grammar/type system/ownership problems that small examples hide?

Write real programs in Monel — not toy examples, but programs complex enough to stress-test the language design. Each project targets a specific subsystem.

| Project | What it stress-tests | Depends on |
|---------|---------------------|------------|
| **Key-value store** (Redis subset: GET/SET/DEL, TTL, persistence) | Effects (Db.write, Net.read), async, concurrency (shared state), error handling | Steps 1-2 |
| **Shell** (pipes, process management, signals, job control) | Unsafe, FFI, ownership (fd lifetimes), state machines (job states) | Steps 1-2 |
| **JSON parser** | Generics, algebraic types, pattern matching, complexity bounds, pure functions | Step 1 |

**Timing:** Start the JSON parser after Step 1 (parser) — it only needs parsing, not verification. Start the kv-store and shell after Step 2 (`monel check`) — they need effect checking to be meaningful.

**Done when:** All three compile (once codegen exists) and their contracts pass verification. Until then, they serve as parser test cases and language design stress-tests.

### Step 8: Autoresearch Optimization Loop

**Risk:** Can we systematically optimize Monel's compiler output for agent efficiency?

**What:** Apply the [Autoresearch](https://github.com/karpathy/autoresearch) pattern — tight hypothesize → edit → eval → commit-or-revert loop — to iteratively improve how Monel's tooling serves AI agents. The agent modifies compiler output format, error messages, or query oracle responses, then runs `benchmarks/run.sh` as the eval metric, committing only changes that reduce tool calls or tokens.

**Why this matters:** The Autoresearch blog showed that structured search with a clear eval metric finds wins an engineer wouldn't think to try (a temperature clamp fix worth more than all architecture changes combined). Our benchmark harness is already the eval; we just need to close the loop.

**Concrete tasks:**
1. Add an iterative benchmark task (e.g., "optimize this function to pass its contract while minimizing allocations") to test agent convergence rate with contracts vs plain comments
2. Script the commit-or-revert loop around `benchmarks/run.sh` with tool_calls as the target metric
3. Run against compiler output changes: error format, `--format llm` structure, `monel context` output shape

**Depends on:** Steps 2-3 (`monel check` + query oracle must exist to optimize).

**Done when:** At least one round of automated optimization produces a measurable improvement in agent efficiency metrics.

### Later (not blocking)

- Effect-aware codegen passes
- Compiler-as-server / persistent semantic model
- `monel query blast --hypothetical`
- Lifetime inference specification (high risk, but not blocking early adoption)
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
