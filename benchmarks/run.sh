#!/usr/bin/env bash
# Monel AI Agent Benchmark Harness
#
# Runs a fixed task in Claude Code headless mode against a codebase,
# captures stream-json output, extracts metrics, appends to results.jsonl.
#
# Usage:
#   ./benchmarks/run.sh <task_name> <codebase_dir> [--max-turns N]
#
# Example:
#   ./benchmarks/run.sh add-rate-limiting ./crates/monel-bench/rust-baseline
#   ./benchmarks/run.sh add-rate-limiting .
#
# Results are appended to benchmarks/results.jsonl

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_FILE="$SCRIPT_DIR/results.jsonl"
TASKS_DIR="$SCRIPT_DIR/tasks"

# ─── Args ───

TASK_NAME="${1:?Usage: run.sh <task_name> <codebase_dir> [--max-turns N]}"
CODEBASE_DIR="${2:?Usage: run.sh <task_name> <codebase_dir> [--max-turns N]}"
MAX_TURNS=20

shift 2 || true
while [[ $# -gt 0 ]]; do
    case "$1" in
        --max-turns) MAX_TURNS="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

TASK_FILE="$TASKS_DIR/$TASK_NAME.md"
if [[ ! -f "$TASK_FILE" ]]; then
    echo "Error: task file not found: $TASK_FILE"
    echo "Available tasks:"
    ls "$TASKS_DIR"/*.md 2>/dev/null | xargs -I{} basename {} .md
    exit 1
fi

PROMPT="$(cat "$TASK_FILE")"
CODEBASE_DIR="$(cd "$CODEBASE_DIR" && pwd)"
CODEBASE_NAME="$(basename "$CODEBASE_DIR")"
RUN_ID="$(date +%Y%m%dT%H%M%S)-$$"
RAW_FILE="$SCRIPT_DIR/raw/$RUN_ID.jsonl"

mkdir -p "$SCRIPT_DIR/raw"

echo "Benchmark: $TASK_NAME"
echo "Codebase:  $CODEBASE_NAME ($CODEBASE_DIR)"
echo "Max turns: $MAX_TURNS"
echo "Run ID:    $RUN_ID"
echo "Raw output: $RAW_FILE"
echo ""

# ─── Run Claude Code ───

START_TIME=$(date +%s)

cd "$CODEBASE_DIR"
claude -p "$PROMPT" \
    --output-format stream-json \
    --verbose \
    --max-turns "$MAX_TURNS" \
    > "$RAW_FILE" 2>&1 || true

END_TIME=$(date +%s)
WALL_SECONDS=$((END_TIME - START_TIME))

# ─── Extract metrics from stream-json ───
#
# Stream-json format (one JSON object per line):
#   {"type":"system","subtype":"init", ...}                    — session start
#   {"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read",...}],...}} — tool call
#   {"type":"user","message":{"content":[{"type":"tool_result",...}],...}}                 — tool result
#   {"type":"result","usage":{...},"total_cost_usd":...,"num_turns":...}                  — final summary

# Tool calls by name: count tool_use content blocks
count_tool() {
    local count
    count=$(grep -c "\"name\":\"$1\"" "$RAW_FILE" 2>/dev/null) || count=0
    echo "$count"
}

TOOL_CALLS=$(grep -c '"type":"tool_use"' "$RAW_FILE" 2>/dev/null) || TOOL_CALLS=0
FILES_READ=$(count_tool Read)
EDIT_CALLS=$(count_tool Edit)
BASH_CALLS=$(count_tool Bash)
GREP_CALLS=$(count_tool Grep)
GLOB_CALLS=$(count_tool Glob)
SEARCH_CALLS=$((GREP_CALLS + GLOB_CALLS))
AGENT_CALLS=$(count_tool Agent)
WRITE_CALLS=$(count_tool Write)

# Extract from the final "type":"result" line (has all aggregate data)
RESULT_LINE=$(grep '"type":"result"' "$RAW_FILE" 2>/dev/null | tail -1)

if [[ -n "$RESULT_LINE" ]]; then
    INPUT_TOKENS=$(echo "$RESULT_LINE" | jq '.usage.input_tokens // 0')
    OUTPUT_TOKENS=$(echo "$RESULT_LINE" | jq '.usage.output_tokens // 0')
    CACHE_READ=$(echo "$RESULT_LINE" | jq '.usage.cache_read_input_tokens // 0')
    CACHE_CREATION=$(echo "$RESULT_LINE" | jq '.usage.cache_creation_input_tokens // 0')
    COST_USD=$(echo "$RESULT_LINE" | jq '.total_cost_usd // 0')
    NUM_TURNS=$(echo "$RESULT_LINE" | jq '.num_turns // 0')
    DURATION_MS=$(echo "$RESULT_LINE" | jq '.duration_ms // 0')
    DURATION_API_MS=$(echo "$RESULT_LINE" | jq '.duration_api_ms // 0')
    STOP_REASON=$(echo "$RESULT_LINE" | jq -r '.stop_reason // "unknown"')
    SESSION_ID=$(echo "$RESULT_LINE" | jq -r '.session_id // "unknown"')
else
    INPUT_TOKENS=0
    OUTPUT_TOKENS=0
    CACHE_READ=0
    CACHE_CREATION=0
    COST_USD=0
    NUM_TURNS=0
    DURATION_MS=0
    DURATION_API_MS=0
    STOP_REASON="no_result"
    SESSION_ID="unknown"
fi

COMPLETED="true"
if [[ "$STOP_REASON" == "max_turns" ]]; then
    COMPLETED="false"
fi

# ─── Write result ───

RESULT=$(jq -n \
    --arg run_id "$RUN_ID" \
    --arg task "$TASK_NAME" \
    --arg codebase "$CODEBASE_NAME" \
    --arg codebase_dir "$CODEBASE_DIR" \
    --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg session_id "$SESSION_ID" \
    --arg stop_reason "$STOP_REASON" \
    --argjson tool_calls "$TOOL_CALLS" \
    --argjson files_read "$FILES_READ" \
    --argjson edit_calls "$EDIT_CALLS" \
    --argjson write_calls "$WRITE_CALLS" \
    --argjson bash_calls "$BASH_CALLS" \
    --argjson search_calls "$SEARCH_CALLS" \
    --argjson agent_calls "$AGENT_CALLS" \
    --argjson input_tokens "$INPUT_TOKENS" \
    --argjson output_tokens "$OUTPUT_TOKENS" \
    --argjson cache_read "$CACHE_READ" \
    --argjson cache_creation "$CACHE_CREATION" \
    --argjson cost_usd "$COST_USD" \
    --argjson num_turns "$NUM_TURNS" \
    --argjson duration_ms "$DURATION_MS" \
    --argjson duration_api_ms "$DURATION_API_MS" \
    --argjson wall_seconds "$WALL_SECONDS" \
    --argjson max_turns "$MAX_TURNS" \
    --argjson completed "$COMPLETED" \
    '{
        run_id: $run_id,
        timestamp: $timestamp,
        session_id: $session_id,
        task: $task,
        codebase: $codebase,
        codebase_dir: $codebase_dir,
        metrics: {
            tool_calls: $tool_calls,
            files_read: $files_read,
            edit_calls: $edit_calls,
            write_calls: $write_calls,
            bash_calls: $bash_calls,
            search_calls: $search_calls,
            agent_calls: $agent_calls,
            input_tokens: $input_tokens,
            output_tokens: $output_tokens,
            cache_read_tokens: $cache_read,
            cache_creation_tokens: $cache_creation,
            total_tokens: ($input_tokens + $output_tokens),
            cost_usd: $cost_usd,
            num_turns: $num_turns,
            duration_ms: $duration_ms,
            duration_api_ms: $duration_api_ms,
            wall_seconds: $wall_seconds
        },
        config: {
            max_turns: $max_turns,
            completed: $completed,
            stop_reason: $stop_reason
        }
    }')

echo "$RESULT" | jq -c . >> "$RESULTS_FILE"

# ─── Print summary ───

echo ""
echo "═══════════════════════════════════════════════"
echo "  Results: $TASK_NAME @ $CODEBASE_NAME"
echo "═══════════════════════════════════════════════"
echo "$RESULT" | jq '{
    tool_calls: .metrics.tool_calls,
    files_read: .metrics.files_read,
    edits: .metrics.edit_calls,
    searches: .metrics.search_calls,
    turns: .metrics.num_turns,
    tokens: .metrics.total_tokens,
    cost: .metrics.cost_usd,
    wall_time: "\(.metrics.wall_seconds)s",
    completed: .config.completed,
    stop_reason: .config.stop_reason
}'
echo ""
echo "Appended to: $RESULTS_FILE"
echo "Raw transcript: $RAW_FILE"
