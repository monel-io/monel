# Monel Language Specification

**Version:** 0.1.0-draft

Monel is a programming language with a compiler that enforces human intent. Every function declares its contracts (`requires:`, `ensures:`), effects, and error behavior inline alongside its implementation. The compiler verifies all contracts deterministically — no LLM in the pipeline. See [how verification works](06-parity.md).

- **Contracts** — `requires:`, `ensures:`, `invariant:`, `effects:`, `panics: never`
- **Implementation** — algorithms, data structures, control flow
- **Both live in `.mn` files** — contracts and code are co-located, verified by the compiler

## Chapters

| Chapter | Topic |
|---------|-------|
| [1. Overview](01-overview.md) | Philosophy, architecture, design principles |
| [2. Contract Syntax](02-intent-syntax.md) | EBNF grammar for contracts in `.mn` files |
| [3. Implementation Syntax](03-impl-syntax.md) | EBNF grammar for `.mn` files |
| [4. Type System](04-type-system.md) | Structural/nominal types, ownership, generics |
| [5. Effects](05-effects.md) | Effect tracking, budgets, policies |
| [6. Verification](06-parity.md) | Contract verification, coverage metrics |
| [7. Modules](07-modules.md) | File-based modules, project configuration |
| [8. Errors](08-errors.md) | Result\<T,E\>, try, panic freedom |
| [9. Concurrency](09-concurrency.md) | async/await, structured concurrency |
| [10. Systems](10-systems.md) | unsafe, FFI, raw I/O |
| [11. Tooling](11-tooling.md) | Query oracle, refactoring, dev server |
| [12. Standard Library](12-stdlib.md) | All std/* modules |
