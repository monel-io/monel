#!/usr/bin/env bash
# Show benchmark history for a task, sorted by date.
#
# Usage:
#   ./benchmarks/history.sh [task_name]
#
# Without arguments, shows all results.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_FILE="$SCRIPT_DIR/results.jsonl"

if [[ ! -f "$RESULTS_FILE" ]]; then
    echo "No results yet. Run ./benchmarks/run.sh first."
    exit 0
fi

TASK_FILTER="${1:-}"

echo "═══════════════════════════════════════════════════════════════════════════════"
printf "  %-12s %-22s %-15s %6s %6s %6s %8s %5s\n" \
    "Date" "Task" "Codebase" "Tools" "Reads" "Edits" "Tokens" "Time"
echo "  ─────────────────────────────────────────────────────────────────────────────"

filter_cmd="cat"
if [[ -n "$TASK_FILTER" ]]; then
    filter_cmd="grep \"\\\"task\\\":\\\"$TASK_FILTER\\\"\""
fi

eval "$filter_cmd" "$RESULTS_FILE" | while IFS= read -r line; do
    ts=$(echo "$line" | jq -r '.timestamp[:10]')
    task=$(echo "$line" | jq -r '.task')
    codebase=$(echo "$line" | jq -r '.codebase')
    tools=$(echo "$line" | jq -r '.metrics.tool_calls')
    reads=$(echo "$line" | jq -r '.metrics.files_read')
    edits=$(echo "$line" | jq -r '.metrics.edit_calls')
    tokens=$(echo "$line" | jq -r '.metrics.total_tokens')
    secs=$(echo "$line" | jq -r '.metrics.wall_seconds')
    printf "  %-12s %-22s %-15s %6s %6s %6s %8s %4ss\n" \
        "$ts" "$task" "$codebase" "$tools" "$reads" "$edits" "$tokens" "$secs"
done

echo "═══════════════════════════════════════════════════════════════════════════════"
