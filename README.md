# Monel

A programming language where contracts are co-located with implementation and verified by the compiler.

```
fn push(self: mut Stack<T>, value: T)
  requires: self.len < self.capacity
  ensures: self.len == old(self.len) + 1
  effects: [Mem.alloc]
  panics: never

  if self.len == self.buf.len
    self.grow()
  self.buf[self.len] = value
  self.len += 1
```

Every function declares what it requires, guarantees, and which side effects it performs. The compiler checks all of it deterministically — no LLM in the compilation pipeline.

## Status: Early Research (31% of claims proven)

This project is in early development. Most of the language exists only as a specification.

**What works today:**
- 12-chapter [language specification](https://monel-io.github.io/monel/) with 5 worked examples
- Structural parity checker (12 tests) — catches undeclared effects, missing implementations, error variant drift, signature mismatches
- Effect implication DAG — `Db.write` implies `Db.read`
- [Benchmark](crates/monel-bench/) showing Monel catches 5/5 bug classes vs Rust's 1/5

**What does NOT work yet:**
- No parser — all ASTs are hand-constructed in tests
- No `monel check` command
- No code generation
- No SMT verification of contracts
- No query oracle (`monel query context`)

See [PROGRESS.md](PROGRESS.md) for the full roadmap and [claims.md](spec/claims.md) for every claim mapped to its evidence status.

## Building

```bash
cargo build
cargo test
cargo run --package monel-bench   # run benchmarks
```

## Specification

The spec is built with [mdBook](https://rust-lang.github.io/mdBook/):

```bash
cargo xtask serve   # local dev server at localhost:3000
```

Or read it at [monel-io.github.io/monel](https://monel-io.github.io/monel/).

## Project Structure

```
crates/
  monel-core/          — AST types, structural parity checker (12 tests)
  monel-cli/           — Placeholder binary
  monel-parser/        — Placeholder (parse() stub)
  monel-bench/         — Benchmark: Monel vs Rust for AI agent efficiency
  xtask/               — Build tasks (mdbook)
spec/                  — Language specification (mdBook source)
benchmarks/            — AI agent efficiency benchmark harness
```

## License

Apache-2.0
