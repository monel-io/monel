#!/usr/bin/env bash
# Run the same task on Rust baseline and Monel, then compare metrics.
#
# Usage:
#   ./benchmarks/compare.sh <task_name>
#
# Example:
#   ./benchmarks/compare.sh add-rate-limiting

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TASK_NAME="${1:?Usage: compare.sh <task_name>}"

echo "═══════════════════════════════════════════════"
echo "  Comparison: $TASK_NAME"
echo "  Rust baseline vs Monel"
echo "═══════════════════════════════════════════════"
echo ""

# Run on Rust baseline (uses a git worktree to avoid polluting the repo)
RUST_BASELINE="$REPO_DIR/crates/monel-bench/rust-baseline"
if [[ ! -d "$RUST_BASELINE" ]]; then
    echo "Error: Rust baseline not found at $RUST_BASELINE"
    exit 1
fi

echo "▸ Running on Rust baseline..."
"$SCRIPT_DIR/run.sh" "$TASK_NAME" "$RUST_BASELINE"

echo ""
echo "▸ Running on Monel codebase..."
"$SCRIPT_DIR/run.sh" "$TASK_NAME" "$REPO_DIR"

echo ""
echo "═══════════════════════════════════════════════"
echo "  Side-by-side comparison"
echo "═══════════════════════════════════════════════"
echo ""

# Get the last two results for this task
RESULTS_FILE="$SCRIPT_DIR/results.jsonl"
RUST_RESULT=$(grep "\"task\":\"$TASK_NAME\"" "$RESULTS_FILE" | grep '"codebase":"rust-baseline"' | tail -1)
MONEL_RESULT=$(grep "\"task\":\"$TASK_NAME\"" "$RESULTS_FILE" | grep -v '"codebase":"rust-baseline"' | tail -1)

if [[ -z "$RUST_RESULT" ]] || [[ -z "$MONEL_RESULT" ]]; then
    echo "Could not find both results. Check $RESULTS_FILE"
    exit 1
fi

printf "  %-25s %10s %10s %8s\n" "Metric" "Rust" "Monel" "Delta"
printf "  %-25s %10s %10s %8s\n" "─────────────────────────" "──────────" "──────────" "────────"

for metric in tool_calls files_read edit_calls search_calls total_tokens wall_seconds; do
    rust_val=$(echo "$RUST_RESULT" | jq -r ".metrics.$metric")
    monel_val=$(echo "$MONEL_RESULT" | jq -r ".metrics.$metric")
    if [[ "$rust_val" -gt 0 ]] 2>/dev/null; then
        delta_pct=$(( (monel_val - rust_val) * 100 / rust_val ))
        if [[ "$delta_pct" -le 0 ]]; then
            delta="${delta_pct}%"
        else
            delta="+${delta_pct}%"
        fi
    else
        delta="n/a"
    fi
    printf "  %-25s %10s %10s %8s\n" "$metric" "$rust_val" "$monel_val" "$delta"
done

echo ""
