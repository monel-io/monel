#!/usr/bin/env bash
# Live dashboard — run with: watch -n10 ./benchmarks/dashboard.sh
# Or in a dedicated terminal pane.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_FILE="$SCRIPT_DIR/results.jsonl"

# ─── Colors ───
BOLD="\033[1m"
DIM="\033[2m"
GREEN="\033[32m"
RED="\033[31m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

clear_line() { printf "\033[K"; }

# ─── Header ───
echo -e "${BOLD}MONEL — Agent Efficiency Dashboard${RESET}"
echo -e "${DIM}$(date '+%Y-%m-%d %H:%M:%S')${RESET}"
echo ""

# ─── Evidence Score ───
PROVEN=8
TOTAL=26
PCT=$((PROVEN * 100 / TOTAL))
BAR_WIDTH=26
FILLED=$((PROVEN))
EMPTY=$((TOTAL - PROVEN))
BAR="${GREEN}"
for ((i=0; i<FILLED; i++)); do BAR+="█"; done
BAR+="${RED}"
for ((i=0; i<EMPTY; i++)); do BAR+="░"; done
BAR+="${RESET}"
echo -e "${BOLD}Evidence Score${RESET}  ${BAR}  ${PROVEN}/${TOTAL} claims proven (${PCT}%)"
echo ""

# ─── Cargo Tests ───
if command -v cargo &>/dev/null; then
    TEST_OUTPUT=$(cd "$REPO_DIR" && cargo test --quiet 2>&1 || true)
    PASS_COUNT=$(echo "$TEST_OUTPUT" | grep -oE '[0-9]+ passed' | head -1 | grep -oE '[0-9]+' || echo 0)
    FAIL_COUNT=$(echo "$TEST_OUTPUT" | grep -oE '[0-9]+ failed' | head -1 | grep -oE '[0-9]+' || echo 0)
    if [[ "$FAIL_COUNT" -gt 0 ]]; then
        TEST_STATUS="${RED}${FAIL_COUNT} FAILED${RESET}"
    else
        TEST_STATUS="${GREEN}ALL PASS${RESET}"
    fi
    echo -e "${BOLD}Cargo Tests${RESET}     ${PASS_COUNT} passed  ${TEST_STATUS}"
else
    echo -e "${BOLD}Cargo Tests${RESET}     ${DIM}cargo not found${RESET}"
fi
echo ""

# ─── Parser Constructs ───
# Count checked items in parser plan
PARSER_PLAN="$REPO_DIR/spec/parser-plan.md"
if [[ -f "$PARSER_PLAN" ]]; then
    DONE=$(grep -c '\[x\]' "$PARSER_PLAN" 2>/dev/null) || DONE=0
    TODO=$(grep -c '\[ \]' "$PARSER_PLAN" 2>/dev/null) || TODO=0
    PARSER_TOTAL=$((DONE + TODO))
    if [[ "$PARSER_TOTAL" -gt 0 ]]; then
        PARSER_PCT=$((DONE * 100 / PARSER_TOTAL))
        PARSER_BAR="${GREEN}"
        PARSER_FILLED=$((DONE * 30 / PARSER_TOTAL))
        PARSER_EMPTY=$((30 - PARSER_FILLED))
        for ((i=0; i<PARSER_FILLED; i++)); do PARSER_BAR+="█"; done
        PARSER_BAR+="${DIM}"
        for ((i=0; i<PARSER_EMPTY; i++)); do PARSER_BAR+="░"; done
        PARSER_BAR+="${RESET}"
        echo -e "${BOLD}Parser Progress${RESET} ${PARSER_BAR}  ${DONE}/${PARSER_TOTAL} constructs (${PARSER_PCT}%)"
    else
        echo -e "${BOLD}Parser Progress${RESET} ${DIM}no checklist items found${RESET}"
    fi
else
    echo -e "${BOLD}Parser Progress${RESET} ${DIM}parser-plan.md not found${RESET}"
fi
echo ""

# ─── Benchmark Results ───
echo -e "${BOLD}Benchmark History${RESET}"
if [[ -f "$RESULTS_FILE" ]] && [[ -s "$RESULTS_FILE" ]]; then
    echo -e "${DIM}  Date         Task                   Codebase        Tools Reads Edits Tokens   Cost    Time${RESET}"

    tail -10 "$RESULTS_FILE" | while IFS= read -r line; do
        ts=$(echo "$line" | jq -r '.timestamp[:10]')
        task=$(echo "$line" | jq -r '.task')
        codebase=$(echo "$line" | jq -r '.codebase')
        tools=$(echo "$line" | jq -r '.metrics.tool_calls')
        reads=$(echo "$line" | jq -r '.metrics.files_read')
        edits=$(echo "$line" | jq -r '.metrics.edit_calls')
        tokens=$(echo "$line" | jq -r '.metrics.total_tokens')
        cost=$(echo "$line" | jq -r '.metrics.cost_usd')
        secs=$(echo "$line" | jq -r '.metrics.wall_seconds')
        completed=$(echo "$line" | jq -r '.config.completed')

        if [[ "$completed" == "true" ]]; then
            status=""
        else
            status=" ${RED}(incomplete)${RESET}"
        fi

        printf "  %-12s %-22s %-15s %5s %5s %5s %6s \$%5.3f %4ss%b\n" \
            "$ts" "$task" "$codebase" "$tools" "$reads" "$edits" "$tokens" "$cost" "$secs" "$status"
    done

    echo ""

    # Show latest Rust vs Monel comparison if both exist
    LATEST_TASK=$(tail -1 "$RESULTS_FILE" | jq -r '.task')
    RUST_LINE=$(grep "\"task\":\"$LATEST_TASK\"" "$RESULTS_FILE" | grep '"codebase":"rust-baseline"' | tail -1 2>/dev/null || true)
    MONEL_LINE=$(grep "\"task\":\"$LATEST_TASK\"" "$RESULTS_FILE" | grep -v '"codebase":"rust-baseline"' | tail -1 2>/dev/null || true)

    if [[ -n "$RUST_LINE" ]] && [[ -n "$MONEL_LINE" ]]; then
        echo -e "${BOLD}Latest Comparison: ${LATEST_TASK}${RESET}"
        RUST_TOOLS=$(echo "$RUST_LINE" | jq '.metrics.tool_calls')
        MONEL_TOOLS=$(echo "$MONEL_LINE" | jq '.metrics.tool_calls')
        RUST_TOKENS=$(echo "$RUST_LINE" | jq '.metrics.total_tokens')
        MONEL_TOKENS=$(echo "$MONEL_LINE" | jq '.metrics.total_tokens')
        RUST_TIME=$(echo "$RUST_LINE" | jq '.metrics.wall_seconds')
        MONEL_TIME=$(echo "$MONEL_LINE" | jq '.metrics.wall_seconds')

        delta_tools=""
        if [[ "$RUST_TOOLS" -gt 0 ]]; then
            d=$(( (MONEL_TOOLS - RUST_TOOLS) * 100 / RUST_TOOLS ))
            if [[ "$d" -le 0 ]]; then
                delta_tools="${GREEN}${d}%${RESET}"
            else
                delta_tools="${RED}+${d}%${RESET}"
            fi
        fi

        printf "  %-20s %10s %10s %10b\n" "" "Rust" "Monel" "Delta"
        printf "  %-20s %10s %10s %10b\n" "Tool calls" "$RUST_TOOLS" "$MONEL_TOOLS" "$delta_tools"
        printf "  %-20s %10s %10s\n" "Tokens" "$RUST_TOKENS" "$MONEL_TOKENS"
        printf "  %-20s %9ss %9ss\n" "Time" "$RUST_TIME" "$MONEL_TIME"
        echo ""
    fi
else
    echo -e "  ${DIM}No results yet. Run: ./benchmarks/run.sh <task> <codebase>${RESET}"
    echo ""
fi

# ─── Key risks ───
echo -e "${BOLD}Key Risks${RESET}"
echo -e "  ${RED}█${RESET} Parser         — no parser exists, all ASTs hand-constructed"
echo -e "  ${RED}█${RESET} Lifetimes      — inference rules underspecified"
echo -e "  ${RED}█${RESET} SMT            — contract verification is spec-only"
echo -e "  ${YELLOW}█${RESET} Query oracle   — 86% token savings claim is simulated"
echo -e "  ${GREEN}█${RESET} Parity checker — 5/5 bug classes caught, 12 tests pass"
echo -e "  ${GREEN}█${RESET} Agent efficiency — 2x fewer tool calls (measured)"
