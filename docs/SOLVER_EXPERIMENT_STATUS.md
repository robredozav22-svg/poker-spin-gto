# Solver Experiment Status

Date: 2026-09-07

All values below are research diagnostics. No self-generated 3-max result is approved as production chart data.

## Promotion gate

A numerical stability run may proceed to deeper validation only if:

- relevant root action-frequency delta across independent runs <= 0.50 percentage points;
- maximum node combo-weighted action MAE <= 2.00 percentage points;
- 1326 -> 169 aggregation shows acceptably small suit-variant dispersion;
- model/tree assumptions match the target Spin node;
- external references are used only as holdout validation, never as solver training labels.

Passing these gates proves only numerical stability inside the modeled game. It does not by itself prove full Spin GTO correctness.

## Rejected Python 3-max experiments

### A — naive chance-sampled CFR
Status: REJECTED / ARCHIVED.
- root BTN jam delta about 2.86pp across runs;
- internal combo-weighted differences reached about 26.7pp.

### B — stratified external-sampling MCCFR
Status: REJECTED / ARCHIVED.
- BTN jam roughly 48.26% vs 40.12%;
- root delta about 8.15pp;
- maximum internal combo-weighted difference about 28.72pp.

### C — stratified DCFR
Status: REJECTED / ARCHIVED.
- root delta 3.609pp;
- maximum node combo-weighted MAE 15.812%;
- BTN jam about +11.540pp vs the 8bb validation anchor.

### D — frozen blocker-aware corpus + full opponent-action enumeration
Status: REJECTED / ARCHIVED.
- BTN jam run A 45.7849%;
- BTN jam run B 51.8986%;
- root delta 6.1136pp;
- maximum node combo-weighted MAE 46.7518%;
- maximum individual hand delta 99.3518pp;
- 300-sweep BTN jam 45.3499% vs reference 34.3137% (+11.0362pp).

Decision: do not scale these Python approaches. Archived benchmark workflows are manual-only.

## Active direction — Rust combo-level solver core

Status: RESEARCH_ONLY.

Implemented:
- exact 52-card / 1326-combo representation;
- blocker compatibility bitsets;
- exact joint three-player card removal;
- normalized combo ranges and blocker-conditioned ranges;
- self-contained 5/7-card Hold'em evaluator;
- deterministic HU common-board equity sampling with suit-isomorphic cache;
- genuine three-way common-board equity sampling with canonical cache;
- HU range-equity integration;
- joint three-way range-equity integration with mutually compatible opponent hands only;
- zero-sum terminal settlement, dead blinds and unmatched-jam returns;
- HU and three-way equity -> chip-EV settlement;
- vector regret matching;
- CFR+ floor;
- DCFR discount primitive;
- action-value regret updates with separate own/opponent reach weights;
- Bayesian P(hand | action) range updates;
- hero-card-conditioned P(action | hero cards) and conditional action ranges;
- zero-probability action handling without inventing a range;
- BB Fold/Call EV vs SB jam and BTN jam;
- genuine three-way BB Fold/Call leaf after BTN jam + SB call;
- SB Fold/Jam EV against the blocker-conditioned current BB response strategy;
- coupled restricted learner for `BTN folds -> SB Fold/Jam -> BB Fold/Call`;
- BB counterfactual regret weighted by exact `P(SB Jam | BB cards)`;
- strategy stability metrics: max action delta and prior-weighted MAE;
- 1326 -> 169 aggregation with suit-variant dispersion retained as an audit signal;
- deduplicated Rust CI so one relevant push creates one Rust test run.

## Current restricted subgame milestone

The first coupled strategic subsystem is:

`BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`.

This is a real mutually dependent strategy loop, but it is still only a restricted push/fold subgame and must not be labelled full Spin GTO.

Current sweep logic:
1. freeze SB and BB current strategy snapshots;
2. for every BB private combo, compute `P(SB Jam | BB cards)` after card removal;
3. condition the SB jam range on the exact BB cards;
4. compute BB Fold/Call action values;
5. scale BB regret by the exact counterfactual jam reach;
6. for every SB private combo, condition BB Fold/Call probabilities and call range on the exact SB cards;
7. compute SB Fold/Jam action values;
8. update both regret tables;
9. retain current and average strategies separately.

Next validation sequence:
1. finish `cargo test --all` on the current Rust head;
2. deterministic sparse coupled sweeps;
3. compare successive snapshots using max action delta / weighted MAE;
4. independent-seed and holdout runs;
5. optimize equity/range operators before full 1326-support solves;
6. add a best-response / exploitability diagnostic appropriate to the restricted game;
7. only then expand the action tree.

## 1326 -> 169 chart policy

A 169-cell display is derived from the 1326-combo strategy only after aggregation.

For every hand class we retain:
- action-frequency mean;
- exact combo count (pair 6, suited 4, offsuit 12);
- maximum deviation of any suit variant from the class mean.

Large suit dispersion blocks promotion. The UI must not hide noisy combo strategies behind a clean-looking 169-cell average.

## Postflop backend status

Compatibility CI already successfully:
- checked out the MIT postflop solver source;
- built its Rust CLI;
- solved a fixture from scratch;
- read measured exploitability and strategy data with the strict parser;
- passed reader regression tests.

Current blocker:
- the persisted solution format inspected so far does not export the per-combo EV arrays needed for preflop continuation values.

Required extension:
- export per-combo EV at relevant root/leaf states while retaining measured exploitability, config and combo labels.

Non-all-in preflop leaves must use measured continuation EV. Raw equity is not an acceptable replacement.

## Policy

Wizard, GTO Ranges+, public charts and supplied screenshots are independent validation references only. They can reveal game-tree mismatch or numerical failure, but are not solver training labels.
