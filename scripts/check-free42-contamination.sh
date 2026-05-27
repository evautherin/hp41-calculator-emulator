#!/usr/bin/env bash
# scripts/check-free42-contamination.sh
# CI gate: hp41-core/src/ops/math1/ AND hp41-core/src/ops/stat1/ AND
# hp41-core/src/ops/time/ AND hp41-core/src/ops/advantage/ must contain
# no distinctive Free42 identifiers (Intel BID library, decNumber, Free42
# internals, copyright markers, stats-domain function-name conventions)
# outside the allowlisted disclaim header on each math1/ + stat1/ + time/ + advantage/ file.
#
# Phase 32 Plan 32-03: D-32.7 (12-symbol math1-scoped policy) + D-32.8
# (dual just-ci / ci.yml invocation).
# Phase 33 Plan 33-00 (this file): D-33.8 reassignment of STAT-QUAL-09 to
# Phase 33 — extended PATTERN by 6 stats-domain prefix tokens AND added
# STAT1_DIR scanned directory. The 6 stats-domain tokens are prefix patterns
# sourced from github.com/thomasokken/free42/blob/master/common/core_math2.cc
# function-name conventions consulted on 2026-05-22 (Plan 33-00 RESEARCH.md
# §"Pitfall 5" candidate list; per-prefix regex chosen over enumerated full
# names to catch every Free42 statistical function in one match-arm without
# brittle exact-name maintenance).
# Pitfall 19: Free42 GPL contamination via copy-paste from the Free42 reference impl.
# Pitfall 27: Free42 stats-domain copy-paste from core_math2.cc must be caught
# BEFORE the first stat1/*.rs algorithm file lands (Plan 33-02 onward).
# Phase 43: ADV_DIR added — Advantage Pac (advantage/) carries the same
# Free42 disclaim header on every file and must be scanned with the same guard.
set -euo pipefail

MATH1_DIR="hp41-core/src/ops/math1"
STAT1_DIR="hp41-core/src/ops/stat1"
TIME_DIR="hp41-core/src/ops/time"
ADV_DIR="hp41-core/src/ops/advantage"
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# WR-01: explicit directory existence check — if any scanned directory is
# missing (refactor moved files, script invoked from wrong cwd, future
# module split), the grep pipeline would silently exit 0 ("no contamination")
# because a missing-path grep returns non-zero, causing the pipeline failure
# to evaluate as "no matches". That would neutralise the Pitfall 19 / D-32.7
# license guard. Exit 2 on missing directory is unambiguous.
for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR" "$ADV_DIR"; do
    if [[ ! -d "$dir" ]]; then
        echo "FAIL: $dir does not exist — license guard cannot run." >&2
        exit 2
    fi
done

# D-32.7 (12 math1-era tokens) + D-33.8 (6 stat1-era prefix tokens) = 18 total.
# The 12 math1-era tokens are verified zero false-positives against the
# Phase 32 v3.0 source tree. The 6 stat1-era prefix tokens
# (math_normal_, math_chi2_, math_t_dist_, math_F_dist_, math_gamma_,
# math_beta_inc) are prefix patterns drawn from Free42 core_math2.cc
# function-name conventions; prefix matching catches every Free42
# statistical function (e.g. math_normal_cdf, math_normal_pdf,
# math_normal_inv, math_chi2_cdf, math_chi2_pdf, math_chi2_inv,
# math_t_dist_cdf, math_t_dist_pdf, math_t_dist_inv, math_F_dist_cdf,
# math_F_dist_pdf, math_F_dist_inv, math_gamma_lower, math_gamma_upper,
# math_beta_inc, math_beta_inc_cf, ...) without enumeration brittleness.
# The bare string "Free42" is deliberately NOT in this pattern — 122
# legitimate "Free42 v3.0.5: <value>" cross-check references exist across
# the codebase (per Phase 32 RESEARCH.md). The 18 tokens below are tight
# enough to never match those.
# D-32.7 (12 math1-era tokens) + D-33.8 (6 stat1-era prefix tokens) + Phase 38 (3 time-era tokens) = 21 total.
# Phase 38 adds 3 time-module identifiers drawn from Free42 core_commands7.cc
# function-name conventions: `core_commands7` (Free42 time-module source file),
# `date2j` (Free42 internal Julian-Day conversion function name),
# `j2date` (Free42 internal Julian-Day-to-date function name).
# Phase 43: advantage/ directory added to the scan (same 21-token PATTERN applies).
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License|math_normal_|math_chi2_|math_t_dist_|math_F_dist_|math_gamma_|math_beta_inc|core_commands7|date2j|j2date'

for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR" "$ADV_DIR"; do
    if matches=$(grep -rn -E "$PATTERN" "$dir" | grep -v "$DISCLAIM_LINE"); then
        echo "FAIL: Free42 contamination detected in $dir:"
        echo "$matches"
        exit 1
    fi
done

echo "OK: no Free42 contamination detected in $MATH1_DIR/ or $STAT1_DIR/ or $TIME_DIR/ or $ADV_DIR/"
exit 0
