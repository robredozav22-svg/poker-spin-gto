# Solver Experiment Status

Date: 2026-09-08

All self-generated values remain research diagnostics unless explicitly tied to a real source-bound Spin node and promoted through the exact-data gates.

## Global promotion principle

Passing a numerical solver gate proves only the modeled game. A user-facing exact Spin chart additionally requires:

- live 2026 source/node identity;
- exact stack/payout/ante/action-tree/sizing profile;
- complete 1326-combo reach and action conservation;
- 169-class aggregation with explicit zero-reach hands;
- independent cross-check on a compatible tree;
- discrepancy classification;
- runtime exact-index admission.

External sources are validation references, never solver training labels.

## Rejected Python 3-max experiments

A naive sampled CFR, stratified external-sampling MCCFR, DCFR variant and frozen blocker-aware corpus all failed stability/accuracy diagnostics by large margins. They remain archived and cannot produce chart data.

Representative failures included root action deltas above 3–8pp, combo-weighted node MAE above 15pp, and fixed-corpus 300-sweep BTN jam 45.35% vs an independent 8bb checkpoint near 34.31%.

## Rust exact all-in foundation

Implemented and CI-tested:

- exact 52-card / 1326-combo representation;
- blocker compatibility and exact multi-player card removal;
- direct 7-card Hold'em evaluator, cross-checked against brute-force 21-subset oracle on 20,000 deterministic hands;
- exact HU preflop equity over C(48,5)=1,712,304 boards;
- exact 3-way preflop equity over C(46,5)=1,370,754 boards;
- exact HU and 3-way range integration;
- exact all-in chip-EV terminal settlement;
- canonical exact caches;
- sampled equity retained only for research/cross-check paths.

Exact HU AA vs KK smoke:
- AA equity 0.812554897;
- KK equity 0.187445103;
- 1,712,304 boards;
- zero-sum error 0.

Exact 3-way AA/KK/QQ smoke:
- equities 0.665054415 / 0.188754510 / 0.146191074;
- 1,370,754 boards;
- three-way ties 5,448;
- zero-sum error about 1.15e-13.

## Exact restricted preflop strategic milestone

Restricted diagnostic game:

`BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`

Sparse exact support reaches NashConv 0.001778596 bb at 10,000 sweeps. This validates regret/reach/averaging mechanics for that restricted exact-payoff game only. Its frequencies are not Spin charts.

## Real Hold'em river CFR milestone

A fixed-range river game now uses:

- real Hold'em private cards;
- exact blockers;
- exact 7-card evaluator;
- independent brute-force best response;
- cached showdown outcomes.

Strict 4,000,000-iteration gate:
- NashConv 0.000441128420 bb;
- fixed threshold 0.000600000000 bb;
- PASS.

## Exact turn -> river chance layer

For every legal fixed private-hand pair on the turn:

- exactly 44 unseen river cards are enumerated;
- chance mass is exactly normalized;
- aggregated river-child evaluation is cross-checked against independent state-first enumeration.

Research fixture:
- 16 legal private-state pairs;
- 704 exact transitions;
- 48 distinct river children;
- checkdown cross-check difference 0.

## Two-street Hold'em CFR milestone

The restricted two-street game models:

turn decision -> exact 44-card river chance -> river decision

A critical modeling error was found and fixed: river infosets must include the public turn action history. `check-check`, `bet-call` and `check-bet-call` are separate public states because they imply different reach ranges and pot sizes.

Before the fix, independent NashConv was about 2.2603 bb. After separating the three public histories it fell to about 0.02082 bb at only 25k iterations.

Showdown outcomes are precomputed once, removing repeated 7-card evaluation from the CFR hot loop.

Vanilla CFR convergence on the same exact fixture:
- 25k: 0.0208214 bb;
- 100k: 0.00838977 bb;
- 400k: 0.00431132 bb;
- 1.6M: 0.001766543297 bb.

The final 1.6M gate threshold was fixed in advance at <= 0.0025 bb and passed.

CFR+ with linear averaging was tested but not selected:
- 100k CFR+: about 0.01661569 bb;
- 400k CFR+: about 0.00829956 bb;
- vanilla was materially better at equal iteration counts on this fixture.

Therefore the current selected research engine for this layer is vanilla CFR. CFR+ remains research-only.

## Range-bound continuation proof

A numerical PASS alone cannot create `VERIFIED_MEASURED` continuation data.

The proof object now requires:

- immutable SHA-256 `range_state_id`;
- strategy SHA-256 checksum;
- algorithm and iteration count;
- measured NashConv and precommitted threshold;
- exact river-chance flag;
- exact Hold'em evaluator flag;
- independent best-response flag;
- complete state coverage.

Research fixture evidence is persisted in:

`data/solver-evidence/turn-river-research-fixture-v1.json`

It has `range_exact_ready=true` for that synthetic fixture but `generic_exact_ready=false` and is explicitly forbidden as a real Spin chart.

## 2026 external chart policy

Current primary external baseline priority:

1. GTO Wizard Research for preflop sizing/tree discovery when available;
2. current GTO Wizard General;
3. current GTO Wizard Simple;
4. Legacy/Basic only as historical references.

A new exact chart must be captured from a live 2026 node. CI rejects non-2026 exact captures and rejects GTO Wizard Legacy/Basic as new primary exact sources.

Current independent references include PreflopRanges.app and GTOCharts.com. They are cross-checks only unless full solver/node provenance and lossless exact frequencies become available.

Observed 15bb public mismatch demonstrates why trees must never be averaged:

- current independent BTN first-in reference: Fold 67.9%, Raise 2bb 24.4%, Jam 7.7%;
- historical project screenshot: Fold 67.22%, Raise 2bb 25.41%, Jam 7.36%;
- current independent SB after BTN fold: Fold 38%, Raise 2.2bb 26.5%, Limp 16%, Jam 19.5%.

The SB profile clearly differs from historical screenshot tree assumptions. This is classified as TREE/SIZING MISMATCH, not averaged into a synthetic strategy.

See:
- `docs/SOURCE_AUDIT_2026.md`
- `data/chart-source-policy-2026.json`

## Exact GTO Wizard import path

GTO Wizard allows copying the full node range and ranges for individual actions in standard UPI/Pio/GTO+ text format.

The importer:

`scripts/import-gtowizard-upi-node.mjs`

requires lossless conservation:

`sum(action range weight for combo) == full node range weight for combo`

and derives:

`P(action | combo) = action_weight(combo) / full_range_weight(combo)`.

It checks all 1326 physical combos, preflop suit symmetry and converts to 169 classes.

Zero-reach hands are preserved as:

`range_weight: 0, strategy: null`

They are never defaulted to Fold. A dedicated CI test protects this rule.

## Runtime chart safety

V46 runtime uses `data/charts/exact-index.json` plus `ui/exact-store.js`.

Only indexed `VERIFIED_EXACT` nodes can paint the 13x13 grid or unlock TRAIN. Screenshot aggregates, public anchors and solver research evidence cannot enter the exact renderer.

The runtime index is intentionally empty until the first live 2026 exact node passes all gates.

## Immediate next sequence

1. capture live 2026 GTO Wizard 15bb BTN first-in from preferred current solution family;
2. capture full node UPI plus every action UPI and exact node/sizing metadata;
3. import -> 1326 conservation -> 169 exact matrix;
4. compare aggregates and hand boundaries against compatible independent current references;
5. classify any mismatch before promotion;
6. admit the first node to `exact-index.json` only after PASS;
7. repeat for SB/BB response nodes without generic defend/3bet shortcuts;
8. derive real reach states for non-all-in branches and run validated continuation solver proofs where necessary;
9. only after sufficient exact-node coverage consider a `BEST_2026` claim.

## Non-negotiable rule

We do not claim to be more accurate than GTO Wizard merely because our frequencies differ. We claim improvement only when the same game tree/profile is compared and stronger measured evidence supports it.
