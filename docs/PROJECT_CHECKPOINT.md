# Poker Spin GTO — Project Recovery Checkpoint

Last updated: 2026-09-07
Branch: `chat-aligned-v2`
Main branch policy: DO NOT modify `main` until research/validation gates are passed and user explicitly approves promotion.
Project status: `RESEARCH_ONLY`

This file is the persistent recovery point for future chats. Read this file first before continuing solver or chart work.

## 1. Product target

Build an off-the-table Spin & Go preflop study/review application for 3-max GGPokerOK-style play.

Primary UX constraints:
- fast one-screen workflow;
- minimal clicks;
- large controls;
- effective stack is the primary stack input;
- no hand typing in the final study workflow if avoidable;
- preflop only for the current scope;
- designed for video/hand-history review, not live RTA.

Strategy target must remain separate from UI implementation.

## 2. Hard strategy rules

Never invent solver frequencies, EVs, ranges, missing stack charts or action branches.

Never interpolate a missing 12bb chart from 10bb/15bb and call it solved.

Never collapse context-dependent decisions into one generic `3bet` or `defend` chart.

The canonical strategy lookup must eventually distinguish:

`format -> payout_profile -> effective_stack -> hero_position -> previous_actions -> villain_position -> villain_size -> hero_actions -> hand -> frequencies`

Required verification states:
- `VERIFIED_EXACT`
- `CROSS_CHECKED`
- `PARTIAL`
- `MISSING_EXACT`
- `INVALID_FOR_STRATEGY`

GTO/WTA baseline, simplified strategy and exploit overlays must remain separate layers.

High-multiplier multi-place ICM must not silently reuse WTA/ChipEV ranges.

## 3. Existing chart audit conclusion

Original app chart data is not trustworthy for production strategy use.

Known failures include:
- UI had 8/10/15/20/25bb but old data effectively supported only 8 and 15;
- unsupported stack choices could collapse to all-fold output;
- old 15bb BTN RFI was near all 169 hand classes;
- old `3bet` arrays lacked action-history and sizing context;
- old renderer supported only binary raise/fold actions;
- mixed limp/call/jam/small-raise strategies could not be represented;
- non-standard range notation/comments existed;
- hand normalization had a likely uppercase/lowercase suited/offsuit bug.

Relevant audit files:
- `docs/CHART_AUDIT.md`
- `data/chart-manifest.json`

Do not reintroduce the rejected old range model.

## 4. External chart/reference policy

Reference hierarchy currently used for independent validation:
1. GTO Wizard Spin & Go — preferred exact solver reference when the exact tree can be matched;
2. PokerStars Learn Spin & Go — structural cross-check;
3. PreflopRanges.app Spin & Go solved charts — public aggregate/text cross-check;
4. GGPoker official Spin & Gold material — format/payout sanity, not exact solver range truth.

External charts are HOLDOUT VALIDATION REFERENCES. They are not training labels for the self-built solver.

Public chart values must not be labelled `VERIFIED_EXACT` unless the exact game tree, stack model, sizing and payout assumptions match.

## 5. Rust solver architecture — implemented

Location: `solver-rs/`

Implemented foundations:
- exact 52-card deck representation;
- 1,326 exact private two-card combos;
- 169-class aggregation with suit-variant dispersion audit;
- blocker compatibility matrix/bitsets;
- exact joint three-player card removal;
- normalized and blocker-conditioned ranges;
- direct Hold'em evaluator;
- direct seven-card evaluator validated against brute-force 21-five-card-subset oracle on 20,000 deterministic hands;
- sampled HU common-board equity path;
- sampled genuine 3-way common-board equity path;
- suit-canonical equity caches;
- exact HU preflop board enumeration;
- exact 3-way preflop board enumeration;
- terminal pot settlement with blinds/dead money/unmatched returns;
- chip-EV bridges;
- regret matching / CFR+ primitives;
- action-value regret updates with reach weights;
- Bayesian P(hand | action) range updates;
- exact restricted best-response/NashConv diagnostic;
- strategy stability metrics;
- exact-payoff restricted coupled solver;
- exact 3-way range integration foundation;
- exact BB leaf foundation for `BTN jam -> SB call -> BB Fold/Call`.

Important source files:
- `solver-rs/src/evaluator.rs`
- `solver-rs/src/exact_equity.rs`
- `solver-rs/src/exact_equity3.rs`
- `solver-rs/src/exact_range_equity3.rs`
- `solver-rs/src/exact_leaf3.rs`
- `solver-rs/src/restricted_exact.rs`
- `solver-rs/src/restricted_eval.rs`
- `solver-rs/src/regret.rs`
- `solver-rs/src/terminal.rs`
- `solver-rs/src/terminal_ev.rs`

## 6. Confirmed numerical milestones

### Exact HU

Complete enumeration after four hole cards:
- boards: C(48,5) = 1,712,304;
- smoke matchup: AA vs KK;
- AA equity: 0.812554897;
- KK equity: 0.187445103;
- wins/losses/ties: 1,388,072 / 317,694 / 6,538;
- zero-sum error: 0;
- measured release runtime on GitHub runner after compilation: about 0.22-0.23 sec/matchup.

Decision: HU all-in terminal payoffs should use exact equity, not Monte Carlo.

### Exact 3-way

Complete enumeration after six hole cards:
- boards: C(46,5) = 1,370,754;
- smoke matchup: AA vs KK vs QQ;
- equities: 0.665054415 / 0.188754510 / 0.146191074;
- outright wins: 909,810 / 256,920 / 198,576;
- three-way ties: 5,448;
- zero-sum error approximately 1.15e-13;
- measured release runtime after compilation: about 0.27 sec/matchup.

Decision: 3-way all-in terminal payoffs should also use exact equity. Sampled 3-way remains only a research/cross-check path.

### Exact restricted coupled game

Validation game:
`BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`

Sparse support:
- SB: AA, A5s, 76s;
- BB: KK, AQo, 65s;
- six legal private-hand pairs.

Exact NashConv progression:
- 1 sweep: 1.351545831 bb
- 50: 0.040246151 bb
- 100: 0.029302555 bb
- 200: 0.015213324 bb
- 500: 0.009025857 bb
- 1,000: 0.006427089 bb
- 2,000: 0.004349300 bb
- 5,000: 0.002590215 bb
- 10,000: 0.001778596 bb

At 10,000 sweeps:
- average SB jam: 0.587024326;
- average BB call: 0.360426620;
- SB BR gain: 0.001187979 bb;
- BB BR gain: 0.000590617 bb;
- game value to SB: 0.060849760 bb.

Interpretation: removal of board-sampling noise allowed NashConv to continue downward. This validates the restricted regret/reach/averaging mechanics much more strongly than the earlier sampled experiments.

This does NOT prove full Spin GTO.

## 7. Rejected solver paths

Do not restart these unless there is a specifically justified new mathematical reason:
- naive Python chance-sampled CFR;
- stratified external-sampling MCCFR;
- stratified DCFR experiments;
- frozen blocker-aware Python corpus approach.

They failed stability and/or reference checks. Detailed numbers are retained in `docs/SOLVER_EXPERIMENT_STATUS.md`.

QuickGTO/`sol5000/gto` is not an authoritative Spin preflop solver. It may only be used for experiments/sanity checks.

## 8. Current exact all-in migration state

Completed:
- exact HU enumerator;
- exact HU cache;
- exact 3-way enumerator;
- exact 3-way cache;
- exact 3-way range integrator;
- `exact_leaf3.rs` implementing the exact BB decision after `BTN jam -> SB call`.

Immediate current gate:
- compile and release-smoke `exact_leaf3.rs` through CI;
- verify fold EV, exact call EV, joint compatible pair count and cache behavior;
- only after green CI mark this leaf canonical.

## 9. Next action plan — strict order

### Phase A — finish exact all-in terminal layer
1. Pass CI for `exact_leaf3`.
2. Add exact HU range-equity helper and convert remaining HU all-in leaves to exact cached payoff operators.
3. Convert `BTN jam -> SB call -> BB call` and other genuine 3-way all-in branches to exact cached payoff operators.
4. Add equivalence/cross-check tests between sampled and exact operators on fixed fixtures, allowing sampling tolerance only on sampled side.
5. Mark sampled all-in paths deprecated-for-canonical-payoff but keep them for audit/research.

### Phase B — solver correctness independent of poker evaluator
6. Add small synthetic precomputed-payoff games with analytically known equilibria.
7. Verify regret matching, reach weighting, average strategy and NashConv converge to those known equilibria.
8. Add deterministic promotion thresholds to CI diagnostics, but do not automatically promote chart data.

### Phase C — scale exact payoff preparation
9. Measure number of unique suit-canonical HU and 3-way matchup classes needed by target sparse/full ranges.
10. Design persistent/precomputed exact payoff tables so repeated solver runs do not recompute enumerations.
11. Benchmark memory/runtime before full 1,326-support coupled solves.

### Phase D — expand preflop tree
12. Expand beyond the current push/fold subgame only after Phases A-C are stable.
13. Preserve action-history context: BTN/SB/BB position, exact previous actions and sizes.
14. Implement target response families one branch at a time with explicit tests.

### Phase E — non-all-in continuation
15. Do NOT replace postflop continuation EV with raw equity.
16. Extend/integrate the postflop solver so relevant non-all-in leaf states expose measured per-combo continuation EV.
17. Retain exploitability/config/combo labels with those continuation values.

### Phase F — chart production
18. Solve only fully specified nodes.
19. Aggregate 1,326 combo strategy to 169 classes while retaining suit-variant dispersion.
20. Compare to independent solver/reference holdouts.
21. Only passing nodes may become `VERIFIED_EXACT` candidates.
22. Missing/failed nodes remain fail-closed, never filled by interpolation.

### Phase G — UI
23. Only after strategy/data schema is reliable, adapt the best Poker Lab UI concepts.
24. Effective stack remains the primary stack control.
25. UI must support mixed multi-action cells and exact branch context.

## 10. Promotion / safety gates

No self-generated chart goes to production because CI is green.

Before any solver output can be considered for chart promotion:
- exact tree assumptions documented;
- payoff profile documented;
- numerical convergence measured;
- best-response/exploitability metric measured where applicable;
- independent-run or exact-payoff stability established;
- 1326 -> 169 suit dispersion audited;
- external holdout comparison performed;
- no missing branch silently substituted;
- user explicitly approves promotion to `main`.

## 11. CI

Workflow:
`.github/workflows/solver-rust-core.yml`

Current intended checks include:
- `cargo test --all`;
- deterministic sampled sparse convergence benchmark;
- exact HU smoke;
- exact-payoff restricted convergence benchmark;
- exact 3-way smoke;
- exact 3-way terminal leaf smoke.

When continuing in a new chat, first check the newest CI run on `chat-aligned-v2` before editing solver code.

## 12. Repository continuity protocol

After every material milestone:
1. commit only to `chat-aligned-v2` unless explicitly promoting;
2. ensure CI status is known;
3. update `docs/SOLVER_EXPERIMENT_STATUS.md` with mathematical findings;
4. update THIS file with current completed state + immediate next step;
5. add a concise PR #1 checkpoint comment if the milestone changes architecture or validation status;
6. never rely on chat history alone for project state.

If chat context is lost, recovery sequence is:
1. open `docs/PROJECT_CHECKPOINT.md`;
2. open `docs/SOLVER_EXPERIMENT_STATUS.md`;
3. inspect `docs/CHART_AUDIT.md` and `data/chart-manifest.json`;
4. inspect latest commits on `chat-aligned-v2`;
5. inspect latest `Rust Solver Core` CI result;
6. continue from section 9 in this file.

## 13. Immediate next step at this checkpoint

Wait for/check CI on the commit that added `exact_leaf3_smoke`. If green:
- record the exact BB call/fold smoke output;
- mark exact 3-way BB leaf canonical for research solver use;
- update this checkpoint and solver status;
- then implement exact HU range-equity migration for remaining HU all-in leaves.

If CI fails:
- do not bypass it;
- inspect the failing compile/math assertion;
- fix the exact leaf before expanding further.
