# Monel Language Specification

**Version:** 0.1.0-draft

Monel is a programming language with a compiler that enforces human intent. Every function's contracts, effects, and error behavior are declared in intent files and verified against the implementation at compile time. See [how parity checking works](06-parity.md).

- **Intent** (`.mn.intent`) — specifies *what*: contracts, effects, errors, invariants
- **Implementation** (`.mn`) — specifies *how*: algorithms, data structures, control flow
- **Parity Compiler** (`monelc`) — rejects code that doesn't match its intent

## Chapters

| Chapter | Topic |
|---------|-------|
| [1. Overview](01-overview.md) | Philosophy, architecture, design principles |
| [2. Intent Syntax](02-intent-syntax.md) | EBNF grammar for `.mn.intent` files |
| [3. Implementation Syntax](03-impl-syntax.md) | EBNF grammar for `.mn` files |
| [4. Type System](04-type-system.md) | Structural/nominal types, ownership, generics |
| [5. Effects](05-effects.md) | Effect tracking, budgets, policies |
| [6. Parity](06-parity.md) | Structural + semantic verification |
| [7. Modules](07-modules.md) | File-based modules, project configuration |
| [8. Errors](08-errors.md) | Result\<T,E\>, try, panic freedom |
| [9. Concurrency](09-concurrency.md) | async/await, structured concurrency |
| [10. Systems](10-systems.md) | unsafe, FFI, raw I/O |
| [11. Tooling](11-tooling.md) | Query oracle, refactoring, dev server |
| [12. Standard Library](12-stdlib.md) | All std/* modules |
