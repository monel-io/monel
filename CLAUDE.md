# Monel Project Instructions

## Writing Style

All spec and documentation text must use a neutral, technical tone:

- No strawman-then-resolution ("This is not X. It is Y." — just state Y)
- No superlatives or unprovable claims ("no other compiler", "eliminates an entire class")
- No dramatic framing ("breaking down", "consequences are severe")
- No flowery language ("conceives an idea, translates it into code" — just "writes code")
- Hedge claims about industry trends — use "can", "may", "tends to" not definitive statements
- Don't prescribe authorship — Monel works with or without AI agents
- Don't repeat what context already says (title says "Specification" — body doesn't restate it)
- Test: would a skeptical senior engineer roll their eyes? If yes, rewrite as plain fact.
- No emdashes in spec or documentation. Use colons, periods, commas, or parentheses instead.
- No defensive or paranoid language. Don't state the absence of things (e.g., "no LLM in the pipeline", "all verification is deterministic") — compilers are deterministic by default, and calling out what's *not* in the system signals insecurity. Just describe what the system does.

## Code Style

- No comments that restate what the code does. Use descriptive method and variable names instead (function = verb, variable = noun).
- Only add comments for *why*, not *what*. If a comment explains what the code does, rename the code.
- Prefer `consume_indent()` over `self.advance(); // consume Indent`.
