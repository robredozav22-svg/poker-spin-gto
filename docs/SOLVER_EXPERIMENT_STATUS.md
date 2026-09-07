# Solver Experiment Status

Date: 2026-09-07

All values below are research diagnostics. No listed 3-max Python result is approved as chart data.

## Promotion gate

A numerical stability run may proceed to deeper validation only if:

- BTN-root action-frequency delta across independent runs <= 0.50 percentage points;
- maximum node combo-weighted action MAE <= 2.00 percentage points.

Passing this gate would prove only numerical stability inside the modeled game, not correctness of the Spin game tree.

## Experiment A — naive chance-sampled 3-max CFR

Status: REJECTED / ARCHIVED.

Observed:
- 100k vs 300k runs remained materially different;
- root BTN jam delta about 2.86 percentage points;
- internal combo-weighted node differences reached about 26.7 percentage points.

Decision: do not scale iterations blindly.

## Experiment B — stratified external-sampling MCCFR

Status: REJECTED / ARCHIVED.

Observed:
- 200 vs 600 sweeps BTN jam roughly 48.26% vs 40.12%;
- root delta about 8.15 percentage points;
- maximum internal combo-weighted difference about 28.72 percentage points;
- 300-sweep BTN jam roughly 45.46%, materially above the independent 8bb reference checkpoint.

Decision: hand-class stratification alone did not solve variance/convergence.

## Experiment C — stratified DCFR

Status: REJECTED / ARCHIVED.

Observed:
- root delta 3.609 percentage points;
- maximum node combo-weighted MAE 15.812%;
- BTN jam approximately +11.540 percentage points vs the 8bb validation anchor.

Decision: discounting/regret variant alone did not solve the sampling problem.

## Experiment D — frozen blocker-aware corpus + full opponent-action enumeration

Status: REJECTED / ARCHIVED.

Independent corpus comparison:
- BTN jam run A: 45.7849%;
- BTN jam run B: 51.8986%;
- root delta: 6.1136 percentage points;
- maximum node combo-weighted MAE: 46.7518%;
- maximum individual hand delta: 99.3518 percentage points.

Reference cross-check for the 300-sweep output:
- reference BTN jam: 34.3137%;
- solver BTN jam: 45.3499%;
- delta: +11.0362 percentage points.

Decision: FAIL. Small frozen corpora can converge to different empirical games; Python full-tree enumeration is also too expensive for the accuracy needed.

## Active direction — Rust vector/full-batch solver core

Status: FOUNDATION / RESEARCH_ONLY.

Implemented:
- flat vector regret/strategy storage;
- regret matching;
- CFR+ regret floor;
- DCFR discount primitive;
- full action-value CFR update with separate own/opponent reach weights;
- explicit 3-max public push/fold tree node identities;
- Rust unit-test CI.

Next mathematical work:
1. combo-level (1326) blocker-aware representation rather than 169-class-only sampling;
2. deterministic/vector payoff operators for terminal folds and all-in calls;
3. measured chance integration with independent holdout evaluation;
4. best-response / NashConv-style diagnostic for the restricted game;
5. only after restricted-game stability, expand the action tree;
6. non-all-in leaves require postflop continuation EV, never raw-equity substitution.

## Postflop backend status

Compatibility CI successfully:
- checked out the MIT postflop solver source;
- built its Rust CLI;
- solved a fixture from scratch;
- read measured exploitability and strategy data with our strict parser;
- passed reader regression tests.

Current blocker:
- the inspected persisted solution format does not export per-combo/per-hand EV arrays;
- therefore it is not yet continuation-value capable for our preflop fixed-point loop.

Required next backend extension:
- export per-combo EV at the relevant root/leaf state while retaining measured exploitability, config and combo labels.

## Policy

Wizard, GTO Ranges+, public charts and supplied screenshots are independent validation references only. They may reveal a game-tree mismatch or numerical failure, but must not be used as training labels to tune the self-generated solver output.
