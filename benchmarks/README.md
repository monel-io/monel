# AI Agent Benchmark Harness

Measures how efficiently Claude Code works with Monel vs Rust on identical tasks.

## Quick Start

```bash
# Run a single task on a codebase
./benchmarks/run.sh add-rate-limiting .

# Compare Rust vs Monel on the same task
./benchmarks/compare.sh add-rate-limiting

# View history
./benchmarks/history.sh
./benchmarks/history.sh add-rate-limiting  # filter by task
```

## Metrics

| Metric | What it measures |
|--------|-----------------|
| `tool_calls` | Total tool invocations (Read, Edit, Bash, Grep, etc.) |
| `files_read` | Number of Read tool calls |
| `edit_calls` | Number of Edit tool calls |
| `search_calls` | Grep + Glob calls (exploration effort) |
| `total_tokens` | Input + output tokens consumed |
| `wall_seconds` | End-to-end wall clock time |

## Tasks

Each task is a markdown file in `tasks/` with a fixed prompt:

| Task | What it tests |
|------|--------------|
| `add-rate-limiting` | Adding new behavior + error variant + effects |
| `add-error-variant` | Adding a variant and updating all callers |
| `extract-helper` | Refactoring with contract/effect splitting |
| `add-audit-logging` | Effect-sensitive change (Db.write → must add Log.write) |

## How It Works

1. `run.sh` invokes `claude -p` in headless mode with `--output-format stream-json`
2. Raw stream-json output is saved to `raw/<run_id>.jsonl`
3. Metrics are extracted by parsing the stream for tool_use events and usage data
4. Results are appended to `results.jsonl` (one JSON object per line)

## Files

```
benchmarks/
  run.sh              — Run one task on one codebase
  compare.sh          — Run same task on Rust + Monel, show side-by-side
  history.sh          — Show all past results
  tasks/              — Task prompts (one .md per task)
  results.jsonl       — Accumulated results (git-tracked)
  raw/                — Raw stream-json output (git-ignored)
```

## Requirements

- `claude` CLI (Claude Code)
- `jq` for JSON parsing
