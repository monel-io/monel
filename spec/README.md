# Monel Language Specification

**Version:** 0.2.0-draft

Functions in Monel declare contracts (`requires:`, `ensures:`, `effects:`, `panics:`) alongside their implementation in `.mn` files. The compiler verifies all contracts deterministically. See [how verification works](06-parity.md).

- **Contracts**: `requires:`, `ensures:`, `invariant:`, `effects:`, `panics: never`
- **Implementation**: algorithms, data structures, control flow
- **Single file**: contracts and code are co-located in `.mn` files

## Chapters

| Chapter | Topic |
|---------|-------|
| [1. Overview](01-overview.md) | Model, pipeline, invariants |
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
