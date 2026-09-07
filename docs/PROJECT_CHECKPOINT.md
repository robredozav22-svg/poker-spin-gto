# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 15:00 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest confirmed Rust CI: run `34108942617`, head `e0c993d1cc968cde1ddf8ed92c01280cb8b3d0b7`, SUCCESS.

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

## Completed phases

### Phase A — exact all-in payoff foundation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-exact-allin-phase-a.md`.

### Phase B — generic solver validation: COMPLETE / RESEARCH_ONLY
Checkpoint: `docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`.

### Phase C — exact payoff precompute / persistence: ACTIVE
Base checkpoint: `docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`.

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

## C1 — persistent lookup: COMPLETE / CI CONFIRMED

Implemented `payoff_table.rs` + `payoff_lookup.rs`.
- duplicate/non-canonical keys rejected;
- missing exact key = explicit `None`;
- no silent sampled fallback;
- exact integer outcomes are persistence source of truth.

## C2 — build-only compute-missing: COMPLETE / CI CONFIRMED

Implemented `payoff_build.rs`.
- trusted existing records reused;
- only missing canonical exact keys computed;
- output sorted canonically;
- new table returned; no in-place mutation;
- idempotent same-data build gives byte-identical payload.

## C3 — provenance / integrity: COMPLETE / CI CONFIRMED

Checkpoint: `docs/checkpoints/2026-09-07-payoff-integrity-c3.md`.

Implemented `payoff_manifest.rs`.
- manifest magic `SPNMAN01`;
- schema/evaluator/kind/count/length/FNV-1a-64/provenance/timestamp;
- one-byte payload corruption rejected;
- FNV is integrity/corruption detection only, not authentication/signature.

## C4 — HU generation economics: COMPLETE / CI CONFIRMED

Implemented:
- `solver-rs/src/bin/hu_generation_economics.rs`

Deterministic batch:
- 8 unique canonical HU matchup keys;
- serial, 2-worker, 4-worker exact board enumeration;
- all exact integer wins/losses/ties identical across modes;
- final output sorted canonically.

Measured on GitHub runner in run `34108942617`:

SERIAL 1 worker:
- 8 keys = 1.786770 sec;
- 4.477354 keys/sec;
- projected 93,769-key full HU build = 20,942.949 sec = 349.049 min = about 5.82 h.

PARALLEL 2 workers:
- 8 keys = 0.891578 sec;
- 8.972857 keys/sec;
- projected full HU build = 10,450.295 sec = 174.172 min = about 2.90 h;
- speedup vs serial = 2.004053x.

PARALLEL 4 workers:
- 8 keys = 0.825582 sec;
- 9.690138 keys/sec;
- projected full HU build = 9,676.746 sec = 161.279 min = about 2.69 h;
- speedup vs serial = 2.164255x.

Interpretation:
- 2-worker scaling is essentially ideal on this runner;
- 4 workers bring only modest gain beyond 2, consistent with runner CPU contention/capacity;
- projected full-table times are estimates from this 8-key batch, NOT measured 93,769-key runtimes;
- do not start full generation inside normal CI.

Latest gate:
- run `34108942617` = SUCCESS;
- 91/91 library tests pass;
- all prior exact/persistence/manifest gates remain green.

## CURRENT NEXT STEP — C5 solver integration

Continue here:

1. Add table-backed exact HU range-equity integration using `HuPayoffLookup`.
2. Add table-backed exact 3-way joint-range integration using `ThreeWayPayoffLookup`.
3. Missing required canonical payoff key must return an explicit error/fail closed.
4. Never fall back automatically to sampled equity.
5. Add table-backed versions of supported all-in terminal leaf operators:
   - HU BB Fold/Call leaves;
   - `BTN jam -> SB call -> BB Fold/Call` genuine 3-way leaf.
6. Keep current direct exact enumeration functions as build/research fallback tools, not silent runtime fallback.
7. Prove table-backed leaf values equal direct exact values on AA/KK and AA/KK/QQ fixtures.
8. Add CI smoke for complete-table hit and deliberate missing-key failure.
9. After C5 is green, benchmark broader combo-support restricted games using precomputed table-backed payoffs rather than board enumeration inside repeated solve runs.

## Later phases — do not jump ahead

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
7. Continue from **CURRENT NEXT STEP — C5**.

Do not redo completed A/B/C1/C2/C3/C4 work unless evaluator/schema/game assumptions deliberately change.
