# Project status

Current working branch: `chat-aligned-v2`.

## Recovered baseline

- `main` is intentionally untouched and remains invalid as a strategy baseline.
- Product baseline: V43 effective-stack-only.
- Chart audits: V41 RFI + V42 all-ranges sanity.
- Action-tree baseline: V37; legality/asymmetric audit: V40.
- V44 screenshot-aligned decision-tree UI implemented in the working branch.

## Chart safety

- Machine-readable source anchors added.
- Dependency-free validator added.
- GitHub Actions chart-audit gate active.
- Screenshot aggregates are cross-check references only, not production hand matrices.
- Unknown exact nodes stay blocked rather than falling back to guessed ranges.

## Independent Solver Lab

Implemented in `solver/`:

- canonical 169 starting-hand classes and all 1326 concrete combinations;
- pure-Python 5/7-card Hold'em evaluator;
- deterministic sampled equity engine;
- pot and side-pot settlement;
- canonical node identity preserving mode, payout profile, all stacks, history and sizings;
- regret-matching primitives;
- self-contained HU chance-sampled jam/fold vs call/fold solver;
- equal-stack 3-max chance-sampled push/fold solver with BTN/SB/BB private infosets and 2-way/3-way showdown settlement;
- diagnostics / convergence tooling;
- unit tests for cards, evaluator, pots, model, regret logic and solver normalization.

Current solver output status is `SOLVER_APPROX`, not full Spin GTO. Raise/limp/call branches that continue postflop are intentionally unsolved; raw preflop equity is not allowed as a fake continuation model.

## Active experiment

`3Max Solver Smoke` now runs on solver changes and PRs:

- solver unit tests;
- independent 8bb 3-max push/fold solve;
- 100,000 iterations with deterministic seed;
- JSON output uploaded as a temporary CI artifact.

## Next mathematical milestone

1. Check 8bb 3-max convergence and reproducibility across seeds.
2. Add best-response / exploitability diagnostics for the push/fold abstraction.
3. Run shallow 2/4/6/8/10bb grid.
4. Compare results to Wizard / Range+ / supplied screenshots only after solving independently.
5. Design a postflop continuation-value layer before attempting true 12-25bb raise/limp trees.
