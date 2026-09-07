# Solver Experiment Status

Date: 2026-09-07

All values below are research diagnostics. No self-generated 3-max result is approved as production chart data.

## Promotion gate

A numerical stability run may proceed to deeper validation only if:

- relevant root action-frequency delta across independent runs <= 0.50 percentage points;
- maximum node combo-weighted action MAE <= 2.00 percentage points;
- 1326 -> 169 aggregation shows acceptably small suit-variant dispersion;
- measured exploitability / best-response diagnostics pass for the modeled game;
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
- self-contained direct 5/7-card Hold'em evaluator;
- direct seven-card evaluator validated against the 21-subset brute-force oracle on 20,000 deterministic seven-card hands;
- deterministic HU common-board equity sampling with suit-isomorphic cache;
- genuine three-way common-board equity sampling with canonical cache;
- exact HU preflop equity enumeration over all C(48,5)=1,712,304 boards;
- exact HU suit-canonical payoff cache;
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
- exact-payoff version of that restricted learner with all HU showdown payoffs precomputed once;
- exact restricted best-response / NashConv evaluator;
- BB counterfactual regret weighted by exact `P(SB Jam | BB cards)`;
- strategy stability metrics: max action delta and prior-weighted MAE;
- 1326 -> 169 aggregation with suit-variant dispersion retained as an audit signal;
- deduplicated Rust CI so one relevant push creates one Rust test run.

## Exact HU equity milestone

`exact_hu_equity()` enumerates every legal five-card board after two fixed two-card hands.

Measured CI smoke test, AA vs KK:
- boards: 1,712,304;
- AA equity: 0.812554897;
- KK equity: 0.187445103;
- AA wins: 1,388,072;
- KK wins: 317,694;
- ties: 6,538;
- zero-sum error: 0;
- runtime after release compilation: approximately 0.23 seconds for one complete matchup on the GitHub runner.

Conclusion: Monte Carlo is no longer required for HU all-in terminal payoffs. Exact HU equity is the canonical direction for those leaves.

## Exact restricted subgame milestone

The first exact coupled strategic subsystem is:

`BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`.

For the sparse deterministic validation support:
- SB: AA, A5s, 76s;
- BB: KK, AQo, 65s;
- six legal private-hand pairs after exact card removal;
- exact HU equity is precomputed once for each canonical legal matchup;
- CFR sweeps then use the fixed exact payoff matrix with no equity sampling noise.

Exact NashConv progression:
- sweep 1: 1.351545831 bb;
- sweep 50: 0.040246151 bb;
- sweep 100: 0.029302555 bb;
- sweep 200: 0.015213324 bb;
- sweep 500: 0.009025857 bb;
- sweep 1,000: 0.006427089 bb;
- sweep 2,000: 0.004349300 bb;
- sweep 5,000: 0.002590215 bb;
- sweep 10,000: 0.001778596 bb.

At 10,000 sweeps:
- average SB jam: 0.587024326;
- average BB call: 0.360426620;
- SB best-response gain: 0.001187979 bb;
- BB best-response gain: 0.000590617 bb;
- exact NashConv: 0.001778596 bb;
- SB game value: 0.060849760 bb.

Interpretation:
- NashConv continues decreasing once Monte Carlo payoff noise is removed;
- this strongly validates the regret/reach/averaging mechanics for the modeled restricted game;
- it does NOT validate the full Spin action tree;
- these sparse frequencies are diagnostics only and are not chart data.

## Sampled-vs-exact finding

With 10,000-board training equity and independent 50,000-board holdout equity at 1,000 sweeps:
- train NashConv: 0.006012 bb;
- holdout NashConv: 0.010583 bb;
- gap: 0.004571 bb.

The exact-payoff game removes that train/holdout equity sampling distinction entirely and continues below that noise floor. Conclusion: all-in payoff sampling was a material convergence bottleneck.

## Next mathematical sequence

1. benchmark exact three-way preflop equity for one fixed three-hand matchup;
2. if runtime is practical, make exact 3-way equity/cache canonical for three-way all-in terminals;
3. replace sampled 3-way terminal leaves such as `BTN jam -> SB call -> BB call` with exact payoff operators;
4. retain sampled equity only as a cross-check/research path;
5. add synthetic precomputed-payoff games to test regret/reach logic independently from poker evaluation;
6. design canonical matchup-class precomputation before any full 1326-support exact solve;
7. expand beyond push/fold only after exact all-in terminal math is stable;
8. non-all-in leaves require measured postflop continuation EV, never raw-equity substitution.

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
