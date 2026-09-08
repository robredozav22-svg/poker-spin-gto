# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-08 13:10 +05
Active branch: `chat-aligned-v2`
PR: #1 `V46: exact-gated Spin UX + independent solver lab` — DRAFT, DO NOT MERGE
Main: untouched at `5543271f43c17ca6da045a02034a250fae22c472`
Status: `RESEARCH_ONLY` until real 2026 exact Spin nodes are admitted.

This is the FIRST file to read in a new chat.

## Product invariants

Target: best-possible off-table 3-max Spin & Go preflop study/review app, optimized for minimal-click use while watching/reviewing video/history.

Hard rules:
- effective stack is the primary stack control;
- no live-RTA framing;
- correctness before cosmetics;
- no invented frequencies, EVs or boundaries;
- no interpolation of missing stacks, especially 12/17/23bb;
- exact node identity includes format, payout profile, effective stack, hero position, full action history and exact sizes;
- no generic context-free `3bet`/`defend` chart;
- WTA/ChipEV, ante, HU and high-multiplier ICM are separate profiles;
- unsupported/missing nodes fail closed;
- screenshot/reference/aggregate-only data can never paint an exact chart;
- do not deliver a raw iPhone build until meaningful exact chart coverage exists.

## Completed mathematical foundation

### Exact all-in / payoff core

Implemented and CI-tested:
- 52 cards / 1326 physical combos;
- exact blockers and multi-player card removal;
- direct 7-card Hold'em evaluator cross-checked against brute-force oracle on 20,000 deterministic hands;
- exact HU equity over C(48,5)=1,712,304 boards;
- exact 3-way equity over C(46,5)=1,370,754 boards;
- exact HU/3-way terminal settlement and range integration;
- canonical payoff persistence and immutable lookup;
- sampled equity retained only for research cross-checks.

Important checkpoints under `docs/checkpoints/` include exact all-in Phase A, synthetic matrix Phase B and payoff persistence Phase C.

### Restricted exact preflop game

`BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`

Sparse exact-support diagnostic reaches NashConv 0.001778596bb at 10,000 sweeps. This validates solver mechanics for that restricted game only. Frequencies are NOT chart data.

### Real Hold'em river CFR

4,000,000-iteration strict gate:
- NashConv 0.000441128420bb;
- fixed threshold 0.000600000000bb;
- exact blockers/evaluator + independent BR PASS.

### Exact turn -> river chance

- exactly 44 legal river cards per legal turn private-state pair;
- exact chance normalization;
- independent state-first cross-check;
- research fixture: 16 legal private pairs / 704 transitions / zero cross-check difference.

### Two-street Hold'em CFR

Game structure:
`turn decision -> exact 44-card river chance -> river decision`

Critical bug found/fixed: river infosets must include public turn history. `check-check`, `bet-call`, `check-bet-call` are distinct states.

After fix:
- 25k vanilla CFR NashConv 0.0208214bb;
- 100k 0.00838977bb;
- 400k 0.00431132bb;
- 1.6M 0.001766543297bb.

Final threshold was fixed before run at <=0.0025bb and PASSED.

CFR+ linear averaging was tested and rejected as primary mode because it converged worse than vanilla on the same fixture.

Synthetic range-bound proof:
`data/solver-evidence/turn-river-research-fixture-v1.json`

It has `range_exact_ready=true` ONLY for that synthetic fixture and `generic_exact_ready=false`. It is forbidden as a real Spin chart.

## V46 exact-data/runtime safety

Runtime:
- `data/charts/exact-index.json`
- `ui/exact-store.js`
- exact-aware `ui/grid.js`
- `app.js` loads only indexed `VERIFIED_EXACT` nodes.

Rules:
- TRAIN unlocks only with loaded `VERIFIED_EXACT` node;
- exact action IDs preserve sizes, e.g. `RAISE_TO_2BB`, `JAM_TO_15BB`;
- zero-reach hand = `range_weight:0, strategy:null`;
- zero-reach is NEVER converted into Fold;
- screenshot/public aggregate references remain CROSS-CHECK only.

Current exact runtime index is intentionally EMPTY.

## GTO Wizard exact import path

Docs:
- `docs/GTOWIZARD_EXACT_IMPORT.md`
- template `data/charts/templates/gtowizard-upi-capture-v1.example.json`

Importer:
- `scripts/import-gtowizard-upi-node.mjs`

Required capture from one live node:
1. full node range UPI;
2. UPI range for every legal action;
3. exact solution family/node identity;
4. exact action sizes;
5. live capture timestamp.

For every physical combo:
`sum(action_range_weight) == full_node_range_weight`

Then:
`P(action|combo)=action_weight/full_range_weight`

Importer guards:
- all 1326 combos;
- preflop suit symmetry;
- 169-class conversion;
- explicit zero reach;
- no interpolation;
- no screenshot reconstruction;
- no default Fold;
- aggregate recomputation.

Dedicated CI test verifies excluded hands remain `strategy:null`.

## 2026 source policy

Files:
- `docs/SOURCE_AUDIT_2026.md`
- `data/chart-source-policy-2026.json`

Primary external reference priority for NEW exact preflop nodes:
1. current GTO Wizard Research;
2. current GTO Wizard General;
3. current GTO Wizard Simple;
4. Legacy/Basic historical only.

CI rejects:
- `VERIFIED_EXACT` without valid 2026 `captured_at`;
- GTO Wizard Legacy/Basic as a new primary exact source;
- interpolation/default fill/screenshot reconstruction;
- malformed 169-hand/reach/action data.

Never average different trees/sizings. Mismatch classes include:
- ROUNDING_ONLY;
- SOLVER_TOLERANCE;
- TREE_SIZING_MISMATCH;
- STACK_PROFILE_MISMATCH;
- PAYOUT_PROFILE_MISMATCH/UNRESOLVED;
- ANTE_FORMAT_MISMATCH/UNRESOLVED;
- SOURCE_VERSION_MISMATCH;
- UNEXPLAINED_NUMERIC_DISAGREEMENT.

## Current independent 2026 cross-check grid

Stored under `data/crosschecks/2026/` from current public PreflopRanges.app pages.

Current public roots:

8bb BTN:
- Fold 65.7%
- Jam 34.3%

8bb SB:
- Fold 35.0%
- Raise 2bb 6.0%
- Limp 6.5%
- Jam 52.6%

10bb BTN:
- Fold 65.9%
- Raise 2bb 7.5%
- Jam 26.6%

10bb SB:
- Fold 41.0%
- Raise 2bb 16.5%
- Limp 0.8%
- Jam 41.7%

15bb BTN:
- Fold 67.9%
- Raise 2bb 24.4%
- Jam 7.7%

15bb SB:
- Fold 38.0%
- Raise 2.2bb 26.5%
- Limp 16.0%
- Jam 19.5%

20bb BTN:
- Fold 65.3%
- Raise 2bb 34.7%

20bb SB:
- Fold 35.9%
- Raise 2.5bb 33.9%
- Limp 22.8%
- Jam 7.4%

25bb BTN:
- Fold 61.9%
- Raise 2bb 38.1%

25bb SB:
- Fold 35.6%
- Raise 2.8bb 35.6%
- Limp 27.1%
- Jam 1.7%

IMPORTANT: these are cross-checks only. Public payout/ante/solver provenance is unresolved, so they cannot numerically validate a real exact node until profile compatibility is resolved.

Historical 15bb screenshot BTN was 67.22 / 25.41 / 7.36. Do NOT average with current 67.9 / 24.4 / 7.7.
Historical SB screenshot tree differs from current public 2.2bb+limp tree and is classified TREE/SIZING MISMATCH.

## Cross-check engine

Script:
- `scripts/audit-2026-crosschecks.mjs`

Self-tests verify:
- ROUNDING_ONLY;
- SOLVER_TOLERANCE;
- TREE_SIZING_MISMATCH;
- unresolved payout blocking;
- UNEXPLAINED_NUMERIC_DISAGREEMENT fatal behavior.

A real unexplained numeric disagreement fails CI.

## Coverage targets

Files:
- `data/chart-coverage-targets.json`
- `scripts/report-chart-coverage.mjs`

UI stacks:
`2,4,6,8,10,12,15,17,20,23,25bb`

Current 2026 public cross-check available:
`8,10,15,20,25bb`

No current public cross-check yet:
`2,4,6,12,17,23bb`

All exact runtime nodes currently: ZERO.
Interpolation: FORBIDDEN.

P0 full response-tree stack remains 15bb:
`data/charts/15bb-node-registry.json`

## Latest confirmed CI

Chart Audit run:
- run `34203047725`
- head `2b4e89e0c82cab65d3a90ab4cc8eed283a8b0f57`
- SUCCESS

Passed steps include:
- chart-data validation;
- GTO Wizard importer self-test;
- zero-reach guard;
- live-2026 exact-node validation;
- cross-check classifier self-test;
- current 2026 cross-check audit;
- coverage report;
- exact UI-grid tests;
- router tests;
- benchmark manifests;
- solver tests.

## CURRENT NEXT STEP

P0 — first REAL exact node:

1. Capture live 2026 GTO Wizard 15bb BTN first-in from preferred current family (Research first when available; otherwise current General/Simple with exact family recorded).
2. Capture full-node UPI + EVERY action UPI + exact sizes + node identity + timestamp.
3. Import through `import-gtowizard-upi-node.mjs`.
4. Require 1326 action/full conservation, suit symmetry, 169 conversion and zero-reach guards.
5. Resolve payout/ante/tree compatibility of independent references before any numeric accuracy conclusion.
6. Run 2026 cross-check classifier.
7. If no unexplained disagreement, add node to `data/charts/exact-index.json`.
8. Only then does UI paint the first exact 15bb chart.

Then continue distinct 15bb response nodes from `data/charts/15bb-node-registry.json`:
- SB vs BTN raise;
- BB vs BTN raise after SB fold/call;
- BTN response vs SB/BB jam/3bet;
- SB/BB first-in branches;
- no generic defend/3bet shortcuts.

For non-all-in branches:
- derive real source-bound reach ranges;
- fingerprint exact reach state;
- attach measured continuation proof;
- raw equity substitution remains forbidden.

## Accuracy claim rule

We want to beat GTO Wizard in measured quality, but NEVER claim superiority because frequencies merely differ.

A `BEST_2026` or “more accurate than Wizard” claim requires apples-to-apples same-tree/profile comparison and stronger independently measured evidence. Vendor tolerance, screenshot similarity or aggregate frequency MAE alone are insufficient.

## Fresh-chat recovery

1. Read THIS file.
2. Read `docs/SOURCE_AUDIT_2026.md`.
3. Read `docs/SOLVER_EXPERIMENT_STATUS.md`.
4. Read newest `docs/checkpoints/*`.
5. Inspect latest commits on `chat-aligned-v2`.
6. Inspect latest Chart Audit + Phase E/Rust runs.
7. Continue from CURRENT NEXT STEP.

Do not redo completed all-in/payoff/solver-fixture/runtime/importer/cross-check work unless assumptions deliberately change.
