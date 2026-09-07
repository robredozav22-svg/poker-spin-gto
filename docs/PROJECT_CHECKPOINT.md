# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 16:20 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest fully confirmed broad-scaling CI: run `34115851744`, head `9c0cf847e57a2714a97682c152e760d6002e66db`, SUCCESS.

This is the FIRST file to read in a new chat. Detailed immutable milestones live under `docs/checkpoints/`.

## Product / strategy invariants

Target: trustworthy off-the-table 3-max Spin & Go preflop study/review app for GGPokerOK-style play.

Keep:
- effective stack as primary stack control;
- one-screen/minimal-click UX;
- preflop current scope;
- no live-RTA framing;
- strategy correctness before UI polish;
- no invented frequencies/EV/ranges;
- no interpolation of missing stacks as solved data;
- exact action-history/position/sizing context;
- separate GTO/WTA, simplified, exploit and multi-place ICM layers;
- missing/unsupported nodes fail closed.

Canonical future strategy key:
`format -> payout_profile -> effective_stack -> hero_position -> previous_actions -> villain_position -> villain_size -> hero_actions -> hand -> frequencies`

Verification states:
`VERIFIED_EXACT`, `CROSS_CHECKED`, `PARTIAL`, `MISSING_EXACT`, `INVALID_FOR_STRATEGY`.

Original app charts remain `INVALID_FOR_STRATEGY`. Read `docs/CHART_AUDIT.md` + `data/chart-manifest.json` before chart work.

## Completed foundation

### Phase A — exact all-in foundation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-exact-allin-phase-a.md`.

### Phase B — generic solver validation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`.

### Phase C — exact all-in payoff persistence/runtime: COMPLETE / RESEARCH_ONLY

Key checkpoints:
- `docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`
- `docs/checkpoints/2026-09-07-payoff-integrity-c3.md`
- `docs/checkpoints/2026-09-07-hu-generation-economics-c4.md`
- `docs/checkpoints/2026-09-07-persisted-leaf-integration-c5.md`

Confirmed infrastructure:
- full HU canonical census: 1,624,350 legal ordered pairs -> 93,769 canonical keys;
- exact integer-outcome payoff tables;
- read-only fail-closed lookup;
- compute-missing build path;
- deterministic sorted byte-identical output;
- integrity manifest;
- persisted HU and genuine 3-way terminal leaves;
- missing exact payoff = REJECT;
- sampled fallback = NONE.

## Broader-support scaling — confirmed

### 6x6 physical-combo fixture
Checkpoint: `docs/checkpoints/2026-09-07-broad-6x6-scaling.md`.

Confirmed:
- legal pairs: 34;
- unique canonical payoffs: 34;
- artifact: 984 bytes;
- payoff build ~6.55-7.71 sec across runner passes;
- 10,000 solver sweeps ~0.45-0.52 sec;
- NashConv: 0.001363022bb;
- exact enumeration during solve: ZERO;
- deterministic repeated solve: PASS.

### 12x12 physical-combo fixture
Checkpoint: `docs/checkpoints/2026-09-07-broad-12x12-scaling.md`.

Confirmed on run `34115851744`:
- legal pairs: 128;
- unique canonical payoffs: 128;
- artifact: 3,616 bytes;
- exact payoff build: 25.306890 sec;
- lookup construction: 0.003664 sec;
- 10,000 solver sweeps: 0.486148 sec;
- repeat solve: 0.499246 sec;
- solver throughput: 20,569.871 sweeps/sec;
- NashConv: 0.000117792bb;
- exact enumeration during solve: ZERO;
- deterministic repeat: PASS.

Interpretation:
- one-time exact payoff generation is clearly the bottleneck;
- regret solving itself remains cheap after payoffs exist;
- 12x12 remains only a bounded research fixture, not chart data.

## Parallel exact-payoff builder — CURRENT GATE

Implemented in `solver-rs/src/payoff_build.rs`:
- `build_hu_payoff_table_parallel(...)`;
- explicit bounded worker count;
- only missing requested keys dispatched;
- existing trusted records reused;
- worker completion order discarded;
- final records canonical-key sorted;
- intended byte-identical output versus serial builder;
- workers=0 rejected;
- non-canonical and duplicate keys rejected.

Smoke prepared:
`solver-rs/src/bin/payoff_parallel_build_smoke.rs`.

It requires serial and 2-worker parallel exact computation of two canonical keys to have:
- identical integer outcomes;
- identical canonical records;
- byte-identical encoded payload.

Current CI to inspect first:
- run `34116339869` on head `c7dde30f32ee3c54391e3ca7e2393728acae26c6`.
- It compiles the prepared 24x24 benchmark and executes the parallel-build equivalence smoke.
- Do NOT enable 24x24 execution until this run is SUCCESS.

## Prepared but NOT YET EXECUTED — 24x24

File:
`solver-rs/src/bin/restricted_persisted_24x24_bench.rs`.

Design:
- SB support: 24 physical combos;
- BB support: 24 physical combos;
- 2-worker parallel exact payoff build;
- persisted lookup only during solve;
- two independent 10,000-sweep solves;
- exact integer payoff table reused;
- repeated strategies/reports must be identical;
- exact enumeration during solve must remain ZERO.

This benchmark is intentionally not yet wired into workflow.

## CURRENT NEXT STEP

1. Check run `34116339869`.
2. If parallel-build equivalence smoke is green, add `restricted_persisted_24x24_bench` to `.github/workflows/solver-rust-core.yml`.
3. Run 24x24 and measure:
   - legal pairs;
   - unique canonical payoff keys;
   - artifact bytes;
   - parallel payoff build time / keys per second;
   - lookup construction time;
   - solver-only time;
   - NashConv;
   - deterministic repeat.
4. Save an immutable 24x24 checkpoint if green.
5. Do NOT jump directly to full 1,326 x 1,326.
6. After 24x24, decide whether the next best step is 48x48 scaling or Phase D tree expansion based on measured payoff-build economics and mathematical value.

## Later phases

Phase D: expand preflop action tree branch-by-branch with exact context.

Phase E: non-all-in branches need measured postflop continuation EV; showdown equity is NOT a substitute.

Phase F: solve fully specified nodes, audit 1326->169 suit dispersion, validate externally, then consider `VERIFIED_EXACT` chart candidates.

Phase G: UI adaptation only after strategy/data model is trustworthy. The user explicitly does NOT want a raw iPhone build.

## Promotion gates

No chart becomes production because CI is green.

Before chart promotion:
- tree/sizing/payout assumptions documented;
- convergence / best-response evidence measured;
- exact payoff provenance established;
- 1326->169 suit dispersion audited;
- independent external holdout comparison performed;
- no missing branch substituted;
- user explicitly approves promotion to `main`.

## Fresh-chat recovery protocol

1. Read THIS file.
2. Read newest `docs/checkpoints/` files.
3. Read `docs/SOLVER_EXPERIMENT_STATUS.md`.
4. Read `docs/PAYOFF_POLICY.md`.
5. Inspect latest commits on `chat-aligned-v2`.
6. Inspect newest `Rust Solver Core` CI.
7. Continue from **CURRENT NEXT STEP**.

Do not redo completed A/B/C/6x6/12x12 work unless evaluator/schema/game assumptions deliberately change.
