# Immutable Checkpoint — Exact Payoff Precompute Phase C

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Latest confirmed CI: Rust Solver Core run `34103912694`, head `1153993b9d5bb1181e7af8713a74abecf8668473`, SUCCESS.

## Phase C purpose

Bound the scaling cost of exact all-in payoff preparation and introduce a persistent, versioned representation so exact board enumeration is not repeated every solver run.

## Full HU canonical census

The census iterated every legal ordered pair of exact Hold'em combos.

Measured:
- ordered legal HU combo pairs: **1,624,350**;
- unique suit-canonical HU matchup keys: **93,769**;
- deduplication factor: **17.322889x**;
- key-generation time on GitHub release runner: about **0.4576 sec**;
- raw legal pair processing throughput: about **3.55 million pairs/sec**.

Consequence:
- a complete HU exact-payoff universe is small enough to persist as a canonical table;
- solver nodes should not store duplicate suit-isomorphic payoffs;
- all exact HU leaf queries should resolve through canonical keys.

At the current measured ~0.2 sec per individual exact HU board enumeration, naively computing all 93,769 canonical keys serially would still be hours of work. Therefore generation must be treated as an explicit versioned build artifact, parallelized/batched where appropriate, not recomputed per solver execution.

## Bounded 3-way canonical census

A full ordered 1,326-combo three-player universe was deliberately NOT enumerated.

Bounded throughput sample:
- fixed hero combos sampled: 4;
- legal ordered three-hand tuples processed: **5,527,200**;
- unique canonical keys within that bounded sample: **654,284**;
- observed sample deduplication factor: **8.447708x**;
- key-generation time: about **1.753 sec**;
- throughput: about **3.15 million legal triples/sec**.

Important:
- **654,284 is NOT a full 3-way canonical-key count**;
- the observed 8.447708x dedup factor must NOT be blindly extrapolated into a claimed full-table size;
- full 3-way materialization is not approved at this point.

Decision:
- HU: complete canonical payoff-table generation is architecturally plausible;
- 3-way: generate/persist keys required by actual target supports/tree branches rather than eagerly materializing the full cubic universe.

## Persistent exact payoff format

Added:
- `solver-rs/src/payoff_table.rs`

No external Rust dependencies were added.

Binary format v1 properties:
- magic: `SPNPAY01`;
- payoff-table schema version;
- evaluator schema version;
- table kind: HU or 3-way;
- exact expected board count in header;
- exact record count;
- fixed-width canonical key;
- exact integer outcome counts, not rounded `f64` as source data.

HU records persist:
- canonical 4-card key;
- wins;
- losses;
- ties.

Expected board provenance is C(48,5)=1,712,304.

3-way records persist:
- canonical 6-card role-preserving key;
- outright wins for each seat;
- each of the three two-way tie counts;
- three-way tie count.

Expected board provenance is C(46,5)=1,370,754.

Equity is reconstructed deterministically from exact integer counts.

## Fail-closed safeguards

Decoder rejects:
- wrong magic;
- unsupported table version;
- unsupported evaluator schema version;
- wrong HU/3-way kind;
- wrong board-count provenance;
- truncated/corrupt payload length;
- records whose integer outcome totals do not equal the exact board count.

Round-trip tests preserve exact integer counts and 3-way tie provenance.

## Existing validated foundation remains intact

Still green in the same CI family:
- exact HU equity + exact HU range/leaf path;
- exact 3-way equity + exact 3-way range/leaf path;
- exact-payoff restricted coupled solver, NashConv 0.001778596bb at 10k sweeps;
- deterministic sampled-vs-exact cross-check;
- analytical synthetic matrix games including asymmetric non-50/50 equilibrium;
- direct seven-card evaluator oracle comparison.

## What this does NOT authorize

Do not infer that:
- a complete 3-way payoff table is ready;
- full 1,326-support Spin solve should start now;
- any chart is production-ready;
- non-all-in branches can use showdown equity;
- `main` may be modified.

## Next strict sequence

### C1 — persistent lookup layer
1. Add indexed in-memory lookup for decoded HU/3-way payoff tables.
2. Reject duplicate canonical keys with conflicting exact counts.
3. Permit identical duplicate records only if intentionally normalized/deduplicated, preferably reject duplicates entirely for reproducibility.
4. Add `get_exact_hu(key)` / `get_exact_threeway(key)` style accessors.
5. Add tests that a persisted exact record reproduces the direct exact evaluator result on fixed fixtures.

### C2 — compute-missing pipeline
6. Add a build-only generator path: canonical requested keys -> load existing table -> compute only missing keys -> write a new versioned artifact.
7. Never mutate a trusted table silently in-place; write a new artifact/checksum/provenance record.
8. Keep generator separate from runtime solver query path.

### C3 — provenance/integrity
9. Add deterministic payload checksum/hash to table artifact metadata or a sidecar manifest.
10. Record solver-core commit/evaluator schema/table schema/generation timestamp/key count.
11. Add sorted canonical-record ordering requirement for reproducible byte-identical builds from the same exact data.

### C4 — generation economics
12. Benchmark batch generation on a small canonical HU key set.
13. Measure exact compute throughput under controlled parallelism before generating all 93,769 HU keys.
14. Estimate CPU time and artifact size from measured generation, not only per-matchup smoke timing.
15. For 3-way, derive required key sets from concrete tree/support ranges before any large generation job.

### C5 — solver integration
16. Make exact payoff tables the preferred reusable source for all-in leaf payoffs.
17. Direct exact enumeration remains a correct fallback/build tool, not repeated inner-loop work.
18. Only after table lookup/precompute is validated, benchmark broad/full combo-support restricted games.

## Recovery rule

In a future chat, read in this order:
1. `docs/PROJECT_CHECKPOINT.md`
2. `docs/checkpoints/2026-09-07-exact-allin-phase-a.md`
3. `docs/checkpoints/2026-09-07-synthetic-matrix-phase-b.md`
4. this Phase C checkpoint
5. `docs/SOLVER_EXPERIMENT_STATUS.md`
6. `docs/PAYOFF_POLICY.md`
7. latest `Rust Solver Core` CI on `chat-aligned-v2`

Then continue from **C1 — persistent lookup layer**. Do not redo Phases A/B or the canonical census unless code/provenance changes require a deliberate revalidation.
