# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 15:05 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest confirmed Rust CI: run `34109465365`, head `e9b0f94f59f4d466225e7f3fe7d22a0f57f4b1a4`, SUCCESS.

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

## Completed solver phases

### Phase A — exact all-in foundation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-exact-allin-phase-a.md`.

### Phase B — generic solver validation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`.

### Phase C — exact all-in payoff persistence/runtime: COMPLETE / RESEARCH_ONLY

Base checkpoint: `docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`.
Additional checkpoints:
- `docs/checkpoints/2026-09-07-payoff-integrity-c3.md`
- `docs/checkpoints/2026-09-07-hu-generation-economics-c4.md`
- `docs/checkpoints/2026-09-07-persisted-leaf-integration-c5.md`

Full HU canonical census:
- legal ordered HU pairs: 1,624,350;
- unique canonical HU keys: 93,769;
- dedup: 17.322889x.

Bounded 3-way sample only:
- 4 hero combos;
- 5,527,200 legal triples;
- 654,284 unique sample keys;
- observed dedup 8.447708x.

NEVER quote 654,284 as the full 3-way universe.

### C1 persisted lookup — complete
- versioned integer-outcome table;
- duplicate/non-canonical records rejected;
- missing key explicit `None`;
- no sampled fallback.

### C2 compute-missing build — complete
- only missing canonical keys computed;
- trusted records reused;
- canonical sorted deterministic output;
- no in-place trusted-table mutation.

### C3 integrity manifest — complete
- `SPNMAN01` sidecar;
- schema/kind/count/length/FNV/provenance/timestamp;
- corruption rejected;
- FNV is corruption detection only, not authentication.

### C4 generation economics — complete
8-key deterministic exact HU batch:
- serial: 4.477354 keys/sec, projected full HU ~5.82h;
- 2 workers: 8.972857 keys/sec, projected ~2.90h;
- 4 workers: 9.690138 keys/sec, projected ~2.69h;
- projections are batch-derived estimates, not measured full builds.

### C5 table-backed runtime leaf integration — complete

Implemented:
- `solver-rs/src/persisted_range_equity.rs`
- HU `persisted_bb_leaf_action_values(...)`
- 3-way `persisted_bb_after_btn_jam_sb_call_values(...)`
- `solver-rs/src/bin/persisted_leaf_smoke.rs`

Confirmed direct exact == persisted exact on fixtures:

HU BB=KK vs AA jammer, 8bb:
- equity 0.187445103;
- Fold EV -1.000000000;
- Call EV -5.000878349.

3-way BB=QQ / BTN=AA / SB=KK, 8bb:
- equities BB/BTN/SB = 0.146191074 / 0.665054415 / 0.188754510;
- Fold EV -1.000000000;
- Call EV -4.491414214.

Strict runtime behavior:
- missing HU payoff = REJECT;
- missing 3-way payoff = REJECT;
- sampled fallback = NONE.

Latest gate:
- run `34109465365` = SUCCESS;
- 93/93 Rust library tests pass;
- all Phase A/B/C gates remain green.

## CURRENT NEXT STEP — bounded broader-support exact solve

Do NOT jump to full 1,326 x 1,326 support.

1. Inspect current `restricted_exact.rs` and separate payoff preparation from repeated solver sweeps where needed.
2. Create deterministic broader SB/BB supports in bounded stages (for example 6x6, then 12x12 only if economics fit CI).
3. Enumerate the set of legal ordered private-hand pairs and unique canonical HU payoff keys needed by each restricted support.
4. Build those exact payoff records once, preferably controlled parallel build outside the repeated regret sweeps.
5. Reuse the in-memory/table-backed payoff matrix across many sweeps.
6. Measure separately:
   - payoff build wall time;
   - unique canonical key count;
   - encoded artifact size;
   - solver-only sweep time after payoffs exist;
   - NashConv progression;
   - strategy stability / max delta / weighted MAE.
7. Require deterministic repeated solve results from the same payoff table.
8. Compare the original 3x3 restricted benchmark with broader supports without treating either as full Spin GTO.
9. Keep CI bounded. If 12x12 payoff generation is too costly, stop at the largest measured safe stage and record the limitation.
10. Only after this scaling stage should Phase D expand the preflop action tree.

## Later phases

Phase D: expand preflop action tree branch-by-branch with exact context.

Phase E: non-all-in branches need measured postflop continuation EV; showdown equity is NOT a substitute.

Phase F: solve fully specified nodes, audit 1326->169 suit dispersion, validate externally, then consider `VERIFIED_EXACT` chart candidates.

Phase G: UI adaptation after strategy/data model is trustworthy.

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
7. Continue from **CURRENT NEXT STEP — bounded broader-support exact solve**.

Do not redo completed A/B/C work unless evaluator/schema/game assumptions deliberately change.
