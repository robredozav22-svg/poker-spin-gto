# Solver experiment log

## 2026-09-06 — 3-max 8bb chance-sampled push/fold V1

Model: `THREE_MAX_CHANCE_SAMPLED_PUSH_FOLD_V1`

Purpose: verify whether the first independent 3-max solver is numerically stable enough to be considered as a chart source.

### Run A

- iterations: 100,000
- seed: 20260910
- BTN root jam: 40.5438%

### Run B

- iterations: 300,000
- seed: 20260911
- BTN root jam: 37.6850%

### Stability result

- BTN root difference: 2.8589 percentage points
- worst combo-weighted node/action MAE: 26.7135 percentage points
- worst single-hand delta: 99.7863 percentage points

### Reference context

The recovered 8bb BTN external audit anchor is 455/1326 jam combos = 34.31%. This reference is validation-only and was not used by the solver.

### Decision

**FAIL — NOT PROMOTABLE.**

Reason: independent runs are far too unstable at the hand and downstream-node level. The output remains `SOLVER_APPROX` research data only and must not be rendered in the production chart UI.

### Action

Instead of merely increasing naive chance-sampled iterations, build and benchmark a stratified external-sampling variant so that every traversing player's 169 hand classes receive deliberate sampling coverage. Add a formal stability gate before any solver output may advance to external-reference validation.

## Promotion gate (research stage)

A solver abstraction may advance from convergence testing only when independent runs satisfy at least:

- BTN root aggregate delta <= 0.50 percentage points;
- maximum combo-weighted node/action MAE <= 2.00 percentage points.

Passing these thresholds is **not** proof of full Spin GTO correctness. The next stages are action-tree parity, external cross-checks, exploitability / unilateral-deviation diagnostics where applicable, and postflop-continuation validation.
