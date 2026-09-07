# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 19:25 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.

This is the FIRST file to read in a new chat.

## Product / strategy invariants

Target: trustworthy off-the-table 3-max Spin & Go preflop study/review app for GGPokerOK-style play.

Hard rules:
- effective stack is the primary stack control;
- one-screen/minimal-click UX only after chart data is trustworthy;
- no live-RTA framing;
- strategy correctness before UI polish;
- no invented frequencies/EV/ranges;
- no silent interpolation of missing stacks;
- exact action history, position, sizing and tree profile in node identity;
- separate WTA/ChipEV, ICM, simplified and exploit layers;
- unsupported/missing nodes fail closed;
- no chart promotion from screenshot/reference-only data;
- no raw iPhone build until meaningful verified chart coverage exists.

## Completed mathematical foundation

### Phase A — exact all-in foundation
Checkpoint: `docs/checkpoints/2026-09-07-exact-allin-phase-a.md`.

### Phase B — generic solver validation
Checkpoint: `docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`.

### Phase C — exact payoff persistence/runtime
Checkpoints:
- `docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`
- `docs/checkpoints/2026-09-07-payoff-integrity-c3.md`
- `docs/checkpoints/2026-09-07-hu-generation-economics-c4.md`
- `docs/checkpoints/2026-09-07-persisted-leaf-integration-c5.md`

Confirmed:
- full HU canonical census: 1,624,350 legal ordered pairs -> 93,769 canonical keys;
- exact integer-outcome payoff tables;
- deterministic compute-missing builds;
- manifest/integrity layer;
- read-only fail-closed exact lookup;
- persisted HU and genuine 3-way exact terminal leaves;
- missing exact payoff = REJECT;
- sampled fallback = NONE.

## Bounded exact support scaling — complete for current purpose

6x6 checkpoint: `docs/checkpoints/2026-09-07-broad-6x6-scaling.md`.
12x12 checkpoint: `docs/checkpoints/2026-09-07-broad-12x12-scaling.md`.
24x24 checkpoint: `docs/checkpoints/2026-09-07-broad-24x24-scaling.md`.

24x24 confirmed:
- 514 legal pairs / 514 canonical exact payoff keys;
- 2-worker payoff build ~37.70s;
- 10,000 solver sweeps ~0.47s;
- NashConv 0.000101721bb;
- deterministic repeat PASS;
- exact board enumeration during solve ZERO.

Conclusion: exact payoff preparation is the bottleneck; repeated solver sweeps are cheap. Stop support-only escalation for its own sake.

## Phase D — canonical real Spin tree: ACTIVE

Checkpoint: `docs/checkpoints/2026-09-07-phase-d-tree-provenance-foundation.md`.

Implemented:
- fixed-point BB sizing (`Bb100`) in identity;
- full actor/action history;
- explicit tree profile ID;
- explicit next actor on ChildDecision;
- Fold / Check / LimpTo / CallTo / RaiseTo / JamTo;
- exact fold settlement / exact all-in showdown / child decision / postflop EV required / unresolved continuation contracts;
- multiway fold semantics;
- all-in call semantics;
- `RaiseTo(stack)` rejected in favor of canonical `JamTo(stack)`.

Tree profile/catalog guards:
- exact strategy cannot exist under a non-exact tree profile;
- exact tree profile requires Complete catalog;
- Complete catalog requires all child nodes;
- duplicate node keys rejected.

Current 15bb screenshot tree remains `ScreenReference`, never exact.

## Strategy provenance / promotion guards

Implemented:
- `strategy_record.rs`
- `strategy_compare.rs`
- `solver_evidence.rs`
- `continuation_registry.rs`
- `accuracy_policy.rs`
- `benchmark_claim.rs`

`VERIFIED_EXACT` strategy requires:
- VERIFIED_EXACT tree profile;
- Complete tree catalog;
- verified measured postflop continuation artifact coverage when non-all-in edges exist;
- solver-run convergence evidence;
- matching tree/solver profile IDs;
- all 1,326 physical combos;
- only actions legal at the exact node;
- valid action probabilities.

## Accuracy target and GTO Wizard comparison policy

Checkpoint: `docs/checkpoints/2026-09-07-accuracy-superiority-gates.md`.
Protocol: `docs/WIZARD_BENCHMARK_PROTOCOL.md`.

Public Wizard Spin references report modern solution accuracy around 0.1%-0.017% of pot depending on solution/tree family. This is external background only.

Internal project target:
- `spin-verified-exact-v1` <= 0.0001 fraction of pot = 0.01% pot;
- independent validation required;
- deterministic repeat required for internal solver runs.

IMPORTANT:
- this 0.01% target is not a claim that the project already achieves it on full Spin;
- Wizard vendor accuracy is NOT assumed metric-equivalent to our NashConv;
- frequency similarity is NOT accuracy;
- no claim "more accurate than Wizard" is allowed from screenshots, 169-class overlap, action-frequency MAE, different trees, different continuation models, or different evaluators.

Superiority claim is allowed only by `benchmark_claim.rs` when BOTH candidate and reference are measured with:
- identical exact tree profile;
- identical independent evaluator profile/source;
- identical evaluated-state coverage;
- distinct immutable artifact checksums;
- at least 3 repetitions;
- directly comparable normalized exploitability metric;
- pre-declared minimum improvement margin;
- candidate passing absolute project accuracy ceiling.

## Independent exact evaluator — CURRENT CORE GATE

Implemented:
- `solver-rs/src/independent_exact_eval.rs`
- `solver-rs/src/bin/independent_exact_eval_smoke.rs`

Purpose:
- evaluate frozen strategies without reading regret/CFR state;
- use persisted exact payoff only;
- recompute current value, both best responses and NashConv independently;
- normalize NashConv to reference pot;
- fail closed on missing payoff.

Cross-check design:
- solve bounded exact 3x3 restricted game;
- freeze average SB/BB strategies;
- evaluator A = existing `ExactRestrictedSubgame.evaluate`;
- evaluator B = new independent exact evaluator;
- require equality within 1e-12 for current value, both best responses, BR gains, NashConv and joint mass.

Current heavy CI:
- run `34132625299`;
- head `cbc54aae66755958a8b1d3d4dbe623643a97d181`;
- independent exact evaluator cross-check step has already completed SUCCESS;
- full run is still finishing later 12x12/24x24 regression steps at this checkpoint.

Latest fast Phase D CI:
- run `34132200858`;
- SUCCESS.

## External comparison status

Actual old V41/V42 explicit matrices were not recovered from current GitHub history. Do not reconstruct them from memory.
Old GoldCharts/FIELD50 approximations are not accepted as GTO baseline.

GTO Wizard is used as:
1. tree/source reference;
2. external strategy holdout when exact export fidelity is available;
3. independent comparison target under same-tree benchmark.

Do not copy approximate screenshot hand boundaries and call them solver data.

## CURRENT NEXT STEP

1. Require full run `34132625299` SUCCESS and save exact independent-evaluator output.
2. If evaluator A/B disagree at any tolerance, stop and fix before tree/strategy expansion.
3. If green, promote independent exact best-response evaluation as benchmark evaluator foundation.
4. Obtain/freeze one exact 15bb Spin tree profile with complete legal actions/sizes. Do not assume one universal Wizard 15bb tree; General/Research/Simple/custom trees differ.
5. Build a Complete catalog for that one profile.
6. For all-in leaves use persisted exact payoff.
7. For all non-all-in leaves require measured postflop continuation artifacts; raw showdown equity remains forbidden.
8. Solve physical 1,326 combo strategies only after continuation coverage is complete.
9. Audit 1326 -> 169 suit dispersion before any chart rendering.
10. Compare our frozen artifact against an exact external reference using `docs/WIZARD_BENCHMARK_PROTOCOL.md`.
11. Only if apples-to-apples exploitability is materially lower may the project claim superior accuracy.

## Fresh-chat recovery protocol

1. Read THIS file.
2. Read newest files under `docs/checkpoints/`.
3. Read `docs/WIZARD_BENCHMARK_PROTOCOL.md`.
4. Read `docs/PAYOFF_POLICY.md`.
5. Inspect latest commits on `chat-aligned-v2`.
6. Inspect newest `Phase D Tree Schema` and `Rust Solver Core` runs.
7. Continue from `CURRENT NEXT STEP`.

Do not redo completed A/B/C/6x6/12x12/24x24/Phase-D foundation work unless assumptions deliberately change.
