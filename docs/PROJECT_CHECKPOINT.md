# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 16:30 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest confirmed Rust CI: run `34116743330`, head `1d445d9e7df16344cb9b9d314abf5302650860a2`, SUCCESS.

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

## Completed mathematical foundation

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

Confirmed:
- full HU canonical census: 1,624,350 legal ordered pairs -> 93,769 canonical keys;
- exact integer-outcome tables;
- read-only fail-closed lookup;
- compute-missing deterministic builds;
- manifest/integrity layer;
- persisted HU and genuine 3-way exact leaves;
- missing exact payoff = REJECT;
- sampled fallback = NONE.

## Bounded support scaling — COMPLETE for current purpose

### 6x6
Checkpoint: `docs/checkpoints/2026-09-07-broad-6x6-scaling.md`.
- legal pairs 34;
- 34 canonical payoffs;
- NashConv 0.001363022bb;
- 10k solve ~0.3-0.5 sec after payoff preparation;
- deterministic repeat PASS.

### 12x12
Checkpoint: `docs/checkpoints/2026-09-07-broad-12x12-scaling.md`.
- legal pairs 128;
- 128 canonical payoffs;
- NashConv 0.000117792bb;
- 10k solve ~0.36-0.58 sec after payoff preparation;
- deterministic repeat PASS.

### 24x24
Checkpoint: `docs/checkpoints/2026-09-07-broad-24x24-scaling.md`.
Confirmed on run `34116743330`:
- support SB/BB 24 x 24 physical combos;
- legal pairs 514;
- unique canonical payoffs 514;
- artifact 14,424 bytes;
- 2-worker exact payoff build 37.704552 sec;
- payoff generation throughput 13.632306 keys/sec;
- lookup construction 0.002979 sec;
- 10,000 solver sweeps 0.472979 sec;
- repeat 0.473278 sec;
- NashConv 0.000101721bb;
- SB BR gain 0.000049249bb;
- BB BR gain 0.000052471bb;
- exact enumeration during solve ZERO;
- deterministic repeat PASS.

### Parallel payoff builder
Confirmed in same gate:
- 95/95 Rust tests passed;
- 2-key serial vs 2-worker parallel exact outcomes identical;
- encoded payload byte-identical;
- canonical sorted output PASS.

Decision: STOP support-only escalation here. Do not do 48x48/full 1326 merely for size. The bottleneck and scaling behavior are already established: one-time payoff preparation dominates; solver sweeps are cheap once exact data exists.

## CURRENT PHASE — D: canonical context-rich preflop tree

Current legacy restricted tree lives in `solver-rs/src/tree.rs` and intentionally models only Fold/Jam/Call regression nodes.

Do NOT destroy it; it protects the validated push/fold subsystem.

### Immediate Phase D work

1. Add a separate canonical preflop tree schema for real Spin branches.
2. Node identity must include full previous action history and exact sizing.
3. Use integer fixed-point BB amounts in keys; do not use `f64` as identity/hash data.
4. Represent distinct actions at minimum:
   - Fold
   - Check where legal
   - Limp/Call
   - Call
   - RaiseTo with exact size
   - Jam with exact effective-stack amount/context
5. Preserve actor BTN/SB/BB at every event.
6. Distinguish continuation contracts:
   - all-in terminal -> exact payoff infrastructure available;
   - non-all-in terminal/continuation -> requires measured postflop continuation EV;
   - unresolved continuation -> fail closed.
7. Do not invent legal raise sizes to populate a branch. Tree specifications need source/verification metadata.
8. Add tests proving histories with different sizing/action order are different nodes.
9. Add tests proving a non-all-in leaf cannot be marked exact merely because showdown equity is available.
10. Only after schema is green instantiate the first real multi-action Spin branch from a verified tree/reference.

### Then

11. Match exact tree specs against GTO Wizard / other trusted Spin solver references.
12. Start with one stack/node family, recommended 15bb because existing screenshot/reference structure is richest.
13. For all-in continuations use persisted exact payoff.
14. For raise/call/limp branches, block solving until measured postflop continuation EV is available; raw equity is forbidden as substitution.
15. Expand branch-by-branch, not by creating one generic `3bet` chart.

## Later phases

Phase E: measured postflop continuation EV for non-all-in leaves.

Phase F: fully specified node solves -> 1326 strategy -> 169 aggregation with suit dispersion -> independent Wizard/reference validation -> only then `VERIFIED_EXACT` chart candidates.

Phase G: finished mobile/iPhone web app only when meaningful chart coverage is ready. User explicitly does NOT want a raw iPhone build.

## Promotion gates

No chart becomes production because CI is green.

Before chart promotion:
- exact tree/sizing/payout assumptions documented;
- convergence / best-response evidence measured;
- exact payoff/continuation provenance established;
- 1326->169 suit dispersion audited;
- independent external holdout comparison performed;
- no missing branch substituted;
- user explicitly approves promotion to `main`.

## Fresh-chat recovery protocol

1. Read THIS file.
2. Read newest `docs/checkpoints/` files, especially the 24x24 checkpoint.
3. Read `docs/SOLVER_EXPERIMENT_STATUS.md`.
4. Read `docs/PAYOFF_POLICY.md`.
5. Inspect latest commits on `chat-aligned-v2`.
6. Inspect newest `Rust Solver Core` CI.
7. Continue from **CURRENT PHASE — D**.

Do not redo completed A/B/C/6x6/12x12/24x24 work unless assumptions deliberately change.
