#!/usr/bin/env bash
# scripts/check-aliases-schema.sh
# CI gate: the `search_aliases` field across ALL SIX help-data JSON pools must
# satisfy the Phase-61 v4.2 Help-Search schema contract:
#   1. Where `search_aliases` is present, it is an array of strings.
#   2. Every entry with `status == "implemented"` carries `search_aliases`
#      with length >= 1 (so every shipped function is searchable by alias).
# Any violation exits non-zero, printing which pool/entry failed.
#
# Phase 61 Plan 61-04 (HSQUAL-03). Mirrors scripts/check-free42-contamination.sh
# for style/shebang/strict-mode precedent.
#
# LANDMINE — never glob `docs/hp41-*-functions.json`: the sixth pool is
# `docs/hp41cv-functions.json` (NO dash before "functions"), so a glob would
# SILENTLY MISS it. All six paths are hardcoded in the POOLS list below.
set -euo pipefail

# All six pools, enumerated explicitly. DO NOT replace with a glob.
POOLS=(
    "docs/hp41-advantage-functions.json"
    "docs/hp41-math1-functions.json"
    "docs/hp41-stat1-functions.json"
    "docs/hp41-time-functions.json"
    "docs/hp41-xmem-functions.json"
    "docs/hp41cv-functions.json"
)

if ! command -v jq >/dev/null 2>&1; then
    echo "FAIL: jq is required but not installed — schema gate cannot run." >&2
    exit 2
fi

# validate_pool <file> — returns 0 if the pool satisfies the schema, 1 otherwise.
# Prints every violating entry (pool + display_name + reason) on failure.
validate_pool() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "FAIL: pool $file does not exist — schema gate cannot run." >&2
        return 2
    fi
    if ! jq -e . "$file" >/dev/null 2>&1; then
        echo "FAIL: pool $file is not valid JSON." >&2
        return 2
    fi

    # Rule 1: where present, search_aliases must be an array of strings.
    local bad_type
    bad_type="$(jq -r --arg f "$file" '
        .[]
        | select(has("search_aliases"))
        | select((.search_aliases | type) != "array"
                 or ([.search_aliases[] | select(type != "string")] | length) > 0)
        | "  \($f): \(.display_name // .op_variant // "?") — search_aliases is not an array of strings"
    ' "$file")"

    # Rule 2: every implemented entry has >= 1 alias.
    local missing
    missing="$(jq -r --arg f "$file" '
        .[]
        | select(.status == "implemented")
        | select((.search_aliases | type) != "array" or (.search_aliases | length) < 1)
        | "  \($f): \(.display_name // .op_variant // "?") — status:implemented but search_aliases missing/empty"
    ' "$file")"

    if [[ -n "$bad_type" || -n "$missing" ]]; then
        [[ -n "$bad_type" ]] && echo "$bad_type"
        [[ -n "$missing" ]] && echo "$missing"
        return 1
    fi
    return 0
}

# --self-test: prove the gate BITES. Copy the first pool to a temp file, empty
# one implemented entry's search_aliases, and assert the validator rejects it.
# Touches ONLY the temp copy — the real pools are never mutated.
if [[ "${1:-}" == "--self-test" ]]; then
    tmp="$(mktemp)"
    trap 'rm -f "$tmp"' EXIT
    src="${POOLS[0]}"
    # Empty the search_aliases of the first implemented entry.
    jq '(first(.[] | select(.status=="implemented")) | .search_aliases) = []' \
        "$src" > "$tmp"
    if validate_pool "$tmp"; then
        echo "SELF-TEST FAIL: validator accepted a pool with an empty implemented search_aliases — the gate does NOT bite." >&2
        exit 1
    fi
    echo "SELF-TEST OK: emptying an implemented entry's search_aliases is correctly rejected (gate bites)."
    exit 0
fi

failed=0
for pool in "${POOLS[@]}"; do
    if ! validate_pool "$pool"; then
        failed=1
    fi
done

if [[ "$failed" -ne 0 ]]; then
    echo "FAIL: search_aliases schema violations detected (see above)." >&2
    exit 1
fi

echo "OK: search_aliases schema valid across all ${#POOLS[@]} pools."
exit 0
