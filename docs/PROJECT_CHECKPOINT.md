# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-07 14:05 +05
Active branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Main policy: DO NOT modify `main` until validation gates pass and the user explicitly approves promotion.
Latest confirmed Rust CI: run `34103912694`, head `1153993b9d5bb1181e7af8713a74abecf8668473`, SUCCESS.

This is the FIRST file to read in a new chat. It intentionally stays concise. Detailed, immutable mathematical milestones live under `docs/checkpoints/`.

## Current project objective

Build a trustworthy off-the-table 3-max Spin & Go preflop study/review app for GGPokerOK-style play.

Hard UX rules retained:
- effective stack is the primary stack control;
- fast one-screen workflow;
- minimal clicks / large controls;
- preflop current scope;
- no live-RTA framing;
- strategy correctness and branch completeness come before UI polish.

Hard strategy rules retained:
- never invent frequencies/EV/ranges;
- never interpolate missing stacks and call them solved;
- preserve action history, positions and sizes;
- keep GTO/WTA, simplified, exploit and multi-place ICM layers separate;
- unsupported/missing nodes fail closed.

Canonical future strategy key:
`format -> payout_profile -> effective_stack -> hero_position -> previous_actions -> villain_position -> villain_size -> hero_actions -> hand -> frequencies`

Verification states:
`VERIFIED_EXACT`, `CROSS_CHECKED`, `PARTIAL`, `MISSING_EXACT`, `INVALID_FOR_STRATEGY`.

## Existing chart status

The original app chart data remains `INVALID_FOR_STRATEGY` and must not be revived.

Read:
- `docs/CHART_AUDIT.md`
- `data/chart-manifest.json`

External sources such as GTO Wizard / PokerStars / public solved charts are independent holdout/reference evidence, not training labels for the self-built solver.

## Completed solver phases

### Phase A — exact all-in payoff foundation: COMPLETE / RESEARCH_ONLY

Read immutable checkpoint:
`docs/checkpoints/2026-09-07-exact-allin-phase-a.md`

Key confirmed facts:
- exact HU enumerates C(48,5)=1,712,304 boards;
- AA vs KK = 0.812554897 / 0.187445103;
- exact 3-way enumerates C(46,5)=1,370,754 boards;
- AA / KK / QQ = 0.665054415 / 0.188754510 / 0.146191074;
- exact HU and exact 3-way all-in payoff operators are canonical research paths;
- sampled all-in equity remains cross-check/research only;
- sampled-vs-exact 200k-board audit passed with <0.001 max absolute equity error on the fixtures;
- exact restricted `BTN fold -> SB Fold/Jam -> BB Fold/Call` game reached NashConv 0.001778596bb at 10,000 sweeps.

Policy:
`docs/PAYOFF_POLICY.md`

### Phase B — synthetic generic solver validation: COMPLETE / RESEARCH_ONLY

Read immutable checkpoint:
`docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`

Confirmed independently of poker evaluation:
- Matching Pennies equilibrium exactly 50/50, value 0, NashConv 0;
- asymmetric matrix `[[4,0],[-1,2]]` converges near analytical equilibrium Row 3/7, Column 2/7, value 8/7;
- at 100k iterations: row error 0.001839403, column error 0.001375967, value error 0.000017717, NashConv 0.007806707.

Conclusion: generic `RegretTable`, frozen simultaneous action-value updates and averaging work on known zero-sum games, not only on poker payoffs.

### Phase C — exact payoff precompute/scaling: ACTIVE

Read current immutable checkpoint:
`docs/checkpoints/2026-09-07-payoff-precompute-phase-c.md`

Confirmed full HU canonical census:
- ordered legal HU pairs: 1,624,350;
- unique suit-canonical HU keys: 93,769;
- dedup factor: 17.322889x;
- census key generation ~0.458 sec.

Bounded 3-way sample only — NOT full census:
- 4 fixed hero combos;
- 5,527,200 legal ordered triples;
- 654,284 unique keys inside that sample;
- observed dedup 8.447708x;
- ~1.753 sec key generation.

Never quote 654,284 as the full 3-way key universe.

## Persistent payoff-table foundation

Implemented:
`solver-rs/src/payoff_table.rs`

Format v1:
- magic `SPNPAY01`;
- table schema version;
- evaluator schema version;
- HU/3-way kind;
- exact board-count provenance;
- record count;
- exact canonical key;
- exact integer outcome counts.

No external Rust dependency added.

Decoder fails closed on wrong magic/version/schema/kind/board count/length/outcome totals.

HU and 3-way round-trip tests preserve exact integer outcomes; equity is reconstructed from counts rather than stored rounded floats.

## Latest CI gate

Rust workflow:
`.github/workflows/solver-rust-core.yml`

Latest confirmed run: `34103912694` = SUCCESS.

The gate includes:
- all Rust unit tests, including payoff-table roundtrip/corruption tests;
- sampled sparse convergence benchmark;
- exact HU equity smoke;
- exact restricted NashConv benchmark;
- exact 3-way equity smoke;
- exact 3-way terminal leaf smoke;
- exact HU terminal leaf smoke;
- sampled-vs-exact audit;
- synthetic analytical matrix benchmark;
- canonical payoff census benchmark.

A documentation-only commit may be newer than this CI head; check whether any newer `solver-rs/**` commit exists before assuming the run covers the current solver code.

## CURRENT NEXT STEP — continue here

### C1 — indexed persistent lookup layer

1. Build indexed in-memory HU and 3-way lookup objects from decoded payoff tables.
2. Require canonical-key uniqueness; duplicates must fail closed for deterministic provenance.
3. Add exact accessors returning reconstructed exact equity/outcome data.
4. Add fixture tests proving persisted AA/KK and AA/KK/QQ records reproduce direct exact evaluator results.
5. Add table-query miss behavior explicitly: lookup returns missing, never substitutes sampled equity silently.

### C2 — build-only compute-missing generator

6. Separate runtime lookup from artifact generation.
7. Generator input = requested canonical keys + optional existing trusted table.
8. Compute only missing exact keys.
9. Sort records canonically for deterministic byte output.
10. Write a NEW artifact; never mutate a trusted table silently in place.

### C3 — provenance/integrity

11. Add deterministic payload checksum/hash or sidecar manifest.
12. Record table/evaluator schema, solver-core revision/provenance, record count and generation metadata.
13. Require stable sorted ordering and reject conflicting duplicate keys.

### C4 — generation economics

14. Benchmark exact HU batch generation under controlled parallelism.
15. Estimate full 93,769-key HU generation from measured batch throughput, not only one-matchup smoke timing.
16. For 3-way, derive exact required key sets from concrete target branches/supports before large generation.

### C5 — solver integration

17. Prefer persistent exact table lookups for repeated all-in leaf queries.
18. Direct exact enumeration stays a correct fallback/build tool, not an inner-loop repeated operation.
19. Only after C1-C5 are green benchmark broad/full 1,326-support restricted games.

## Later phases — do not jump ahead

Phase D: expand exact preflop action tree branch-by-branch with full context.

Phase E: non-all-in leaves require measured postflop continuation EV; exact showdown equity is NOT a substitute.

Phase F: solve fully specified nodes, audit 1326->169 suit dispersion, validate externally, then consider `VERIFIED_EXACT` candidates.

Phase G: UI adaptation after strategy/data model is trustworthy; effective stack remains primary control.

## Promotion gates remain mandatory

No chart becomes production merely because code/CI is green.

Before chart promotion:
- exact tree and sizing assumptions documented;
- payout profile documented;
- convergence / best-response evidence measured;
- exact/stable payoff provenance established;
- 1326->169 suit-variant dispersion audited;
- independent external holdout comparison performed;
- no missing branch substituted;
- user explicitly approves promotion to `main`.

## Recovery protocol for a fresh chat

1. Read THIS file.
2. Read the newest files in `docs/checkpoints/`, especially:
   - `2026-09-07-exact-allin-phase-a.md`
   - `2026-09-07-synthetic-matrix-phase-b.md`
   - `2026-09-07-payoff-precompute-phase-c.md`
3. Read `docs/SOLVER_EXPERIMENT_STATUS.md`.
4. Read `docs/PAYOFF_POLICY.md`.
5. Read `docs/CHART_AUDIT.md` + `data/chart-manifest.json` if chart/tree work is relevant.
6. Inspect latest commits on `chat-aligned-v2`.
7. Inspect newest `Rust Solver Core` CI.
8. Continue from **CURRENT NEXT STEP**, not from old chat text.

Do not redo completed Phase A/B work or the HU census unless evaluator/schema/game assumptions changed and deliberate revalidation is required.
