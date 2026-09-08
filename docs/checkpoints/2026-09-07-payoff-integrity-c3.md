# Immutable Checkpoint — Payoff Persistence C3 Integrity

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: Rust Solver Core run `34108519175`, head `cb3a6ab65d63400201570068648c6d1b4b76303f`, SUCCESS.

## Scope

This checkpoint closes C1-C3 of exact all-in payoff persistence:
- C1 indexed read-only lookup;
- C2 build-only compute-missing generator;
- C3 artifact provenance/integrity sidecar.

It does NOT promote any poker chart or modify `main`.

## C1 — lookup

Implemented:
- `solver-rs/src/payoff_table.rs`
- `solver-rs/src/payoff_lookup.rs`
- `solver-rs/src/bin/payoff_lookup_smoke.rs`

Properties:
- exact integer outcome counts are persistence source of truth;
- duplicate canonical keys rejected;
- non-canonical stored keys rejected;
- missing key returns explicit `None`;
- no sampled fallback.

Direct exact -> persistence -> decode -> indexed lookup fixture:
- AA vs KK HU = 0.812554897;
- AA / KK / QQ 3-way = 0.665054415 / 0.188754510 / 0.146191074.

The prior 1e-15 3-way float assertion failure was traced to operation ordering of repeated `1/3` accumulation versus reconstruction from exact integer tie counts. Integer outcomes matched exactly; persisted-vs-direct floating equity uses 1e-12 tolerance only for this machine-rounding difference.

## C2 — compute missing

Implemented:
- `solver-rs/src/payoff_build.rs`
- `solver-rs/src/bin/payoff_build_smoke.rs`

Properties:
- requested and existing keys must be canonical;
- duplicate existing keys rejected;
- trusted existing records reused;
- only missing requested keys are enumerated;
- output records sorted canonically;
- generator returns a new table;
- same exact records produce byte-identical encoded payload.

Smoke:
- HU first build: compute 1;
- incremental HU: reuse 1 + compute 1;
- repeated same HU set: compute 0 and byte-identical output;
- 3-way first build: compute 1;
- repeat: reuse 1, recompute 0.

## C3 — manifest / integrity

Implemented:
- `solver-rs/src/payoff_manifest.rs`
- `solver-rs/src/bin/payoff_manifest_smoke.rs`

Manifest text magic: `SPNMAN01`.

Fields:
- table schema version;
- evaluator schema version;
- payoff kind HU/THREEWAY;
- record count;
- payload byte length;
- FNV-1a-64 checksum;
- externally supplied provenance;
- externally supplied generation timestamp.

Verification fails closed on:
- manifest schema mismatch;
- evaluator schema mismatch;
- wrong payoff kind;
- record-count mismatch;
- payload-length mismatch;
- checksum mismatch;
- invalid underlying payoff payload.

Checksum warning:
- FNV-1a-64 is only deterministic corruption/integrity detection;
- it is NOT cryptographic authentication and must never be presented as a signature/security boundary.

C3 HU smoke:
- records = 1;
- payload bytes = 60;
- checksum = `599f53c643122fcf`;
- manifest bytes = 217;
- one-byte corruption = REJECT.

## Full CI state

At this checkpoint:
- 91/91 Rust library tests passed;
- sampled sparse benchmark green;
- exact HU green;
- exact restricted NashConv green;
- exact 3-way green;
- exact HU/3-way leaf smokes green;
- sampled-vs-exact green;
- synthetic analytical matrix games green;
- canonical census green;
- persistent lookup green;
- compute-missing green;
- manifest integrity green.

## Next phase — C4 generation economics

1. Select a deterministic small batch of unique canonical HU keys.
2. Exact-enumerate it serially.
3. Exact-enumerate the same keys with controlled 2-worker and 4-worker std-thread execution.
4. Require exact integer outcomes to match the serial baseline.
5. Sort results canonically before comparing/encoding.
6. Measure wall-clock keys/sec.
7. Project the 93,769-key full HU build time from measured batch throughput and label projection as an estimate.
8. Record parallel slowdown if CI CPU scheduling makes parallelism worse.
9. Do not launch full 93,769-key generation until C4 is measured and a separate artifact build workflow is approved.
10. Do not extrapolate bounded 3-way sample into a full 3-way universe.
