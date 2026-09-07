# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 14:50 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest confirmed Rust CI: run `34108126766`, head `f731a192c7cb2fb02a9340a80e3ec50916d078fe`, SUCCESS.

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

Checkpoint:
`docs/checkpoints/2026-09-07-exact-allin-phase-a.md`

Confirmed:
- exact HU C(48,5)=1,712,304 boards;
- AA vs KK = 0.812554897 / 0.187445103;
- exact 3-way C(46,5)=1,370,754 boards;
- AA / KK / QQ = 0.665054415 / 0.188754510 / 0.146191074;
- exact HU/3-way all-in payoff is canonical research path;
- sampled equity is cross-check/research only;
- exact restricted game NashConv = 0.001778596bb at 10k sweeps.

Policy: `docs/PAYOFF_POLICY.md`.

### Phase B — generic solver validation: COMPLETE / RESEARCH_ONLY

Checkpoint:
`docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`

Confirmed:
- Matching Pennies exact 50/50, value 0, NashConv 0;
- asymmetric `[[4,0],[-1,2]]` converges near analytical Row 3/7, Column 2/7, value 8/7;
- at 100k: row error 0.001839403, column error 0.001375967, value error 0.000017717, NashConv 0.007806707.

### Phase C — exact payoff precompute / persistence: ACTIVE

Checkpoint:
`docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`

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

Implemented:
- `solver-rs/src/payoff_table.rs`
- `solver-rs/src/payoff_lookup.rs`

Format v1:
- magic `SPNPAY01`;
- table/evaluator schema versions;
- HU/3-way kind;
- exact board-count provenance;
- canonical key;
- exact integer outcome counts.

Lookup policy:
- duplicate canonical keys = reject;
- non-canonical stored keys = reject;
- missing key = explicit `None`;
- NEVER substitute sampled equity silently.

CI persisted lookup fixture:
- HU AA/KK reload = 0.812554897;
- 3-way AA/KK/QQ reload = 0.665054415 / 0.188754510 / 0.146191074;
- integer outcomes match direct exact enumeration;
- 3-way reconstructed floating equity uses 1e-12 tolerance because direct enumeration accumulates `1/3` tie shares iteratively while persistence reconstructs from exact integer tie counts. Integer counts are source of truth.

## C2 — build-only compute-missing generator: COMPLETE / CI CONFIRMED

Implemented:
- `solver-rs/src/payoff_build.rs`
- `solver-rs/src/bin/payoff_build_smoke.rs`

Rules:
- runtime lookup remains read-only;
- input = requested canonical keys + optional existing trusted table;
- non-canonical keys rejected;
- duplicate existing keys rejected;
- existing requested keys reused;
- only missing exact keys computed;
- output records canonically sorted;
- generator returns a NEW table; no trusted artifact mutation in place;
- same exact data produces byte-identical encoded table.

CI smoke:
- first HU build computed 1;
- incremental HU build reused 1, computed 1 new key;
- idempotent HU rebuild computed 0 and produced byte-identical output;
- first 3-way build computed 1;
- repeated 3-way build reused 1, recomputed 0;
- 89/89 Rust library tests passed.

Latest full gate: run `34108126766` = SUCCESS.

## CURRENT NEXT STEP — C3 provenance / integrity

Continue here:

1. Add deterministic artifact integrity checksum/hash over encoded payoff payload.
2. Keep checksum purpose explicit: corruption/integrity detection, not authentication unless cryptographic hashing is used.
3. Add sidecar/versioned manifest containing at minimum:
   - table schema version;
   - evaluator schema version;
   - HU/3-way kind;
   - exact record count;
   - payload byte length;
   - payload checksum/hash;
   - generation provenance / solver-core revision supplied by build caller;
   - generation timestamp supplied externally, not used in deterministic payload bytes.
4. Add manifest verification that fails closed on payload mismatch.
5. Preserve canonical sorted ordering for reproducible payload bytes.
6. Do not add network or runtime mutation to solver lookup.

### After C3 — C4 generation economics

7. Benchmark exact HU batch generation with controlled parallelism.
8. Measure throughput on a representative key batch.
9. Estimate full 93,769-key HU generation time from measured batch throughput.
10. Derive 3-way required keys from concrete target branches/supports before any large generation.

### After C4 — C5 solver integration

11. Prefer persistent exact table lookup in repeated all-in leaf queries.
12. Keep direct exact enumeration as build/fallback tool, not repeated inner-loop work.
13. Only then benchmark broader/full 1,326-support restricted games.

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
7. Continue from **CURRENT NEXT STEP — C3**.

Do not redo completed A/B/C1/C2 work unless evaluator/schema/game assumptions deliberately change.
