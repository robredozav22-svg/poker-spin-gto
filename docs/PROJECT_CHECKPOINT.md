# Poker Spin GTO — Live Project Recovery Index

Last updated: 2026-09-08 13:50 +05
Active branch: `chat-aligned-v2`
Latest consolidated head: `d1ccb6219001fb117265363414000c6b50f1e562`
PR: #1 — DRAFT, DO NOT MERGE
Main: untouched at `5543271f43c17ca6da045a02034a250fae22c472`
Status: `RESEARCH_ONLY` until real live-2026 exact Spin nodes are promoted.

READ THIS FILE FIRST IN A NEW CHAT.

## Non-negotiable product rules

- Off-table REVIEW/TRAIN tool; no live-RTA framing.
- Effective stack is the primary stack control.
- Correct charts/tree context before cosmetics.
- No invented frequencies, EVs, actions or boundaries.
- No interpolation for missing stacks, especially 12/17/23bb.
- Exact identity = format + payout/ChipEV + ante + effective stack + position + complete action history + exact sizings.
- No generic context-free `3bet`/`defend` chart.
- WTA/ChipEV, Spin+Ante, HU and high-multiplier ICM are separate profiles.
- Zero reach = `range_weight:0, strategy:null`, NEVER default Fold.
- Screenshot/aggregate/reference/research data cannot paint exact runtime charts.
- Do not deliver a raw iPhone build until meaningful exact chart coverage exists.

## Mathematical foundation already completed

- 52 cards / 1326 physical combos, exact blockers and joint removal.
- Direct 7-card evaluator cross-checked against brute-force 21-subset oracle on 20,000 deterministic hands.
- Exact HU equity over C(48,5)=1,712,304 boards.
- Exact 3-way equity over C(46,5)=1,370,754 boards.
- Exact HU/3-way range integration and terminal chip-EV settlement.
- Persisted canonical exact payoff tables/lookups.
- Restricted exact preflop diagnostic reaches NashConv 0.001778596bb at 10k sweeps; NOT chart data.
- Real Hold'em river CFR: 4,000,000 iterations, NashConv 0.000441128420bb, precommitted threshold 0.000600000000bb PASS.
- Exact turn→river chance: exactly 44 legal river cards per legal private pair.
- Two-street Hold'em CFR bug fixed: river infosets include public turn history (`check-check`, `bet-call`, `check-bet-call`).
- Vanilla two-street convergence: 25k 0.0208214; 100k 0.00838977; 400k 0.00431132; 1.6M 0.001766543297bb. Precommitted <=0.0025 PASS.
- CFR+ tested and rejected as primary mode because it converged worse.
- Research fixture evidence: `data/solver-evidence/turn-river-research-fixture-v1.json`; range-specific only, NEVER real Spin chart.

## Live-2026 exact chart ingestion

Importer: `scripts/import-gtowizard-upi-node.mjs`
Docs/template: `docs/GTOWIZARD_EXACT_IMPORT.md`, `data/charts/templates/gtowizard-upi-capture-v1.example.json`

Required live GTO Wizard capture:
1. current solution family (Research preferred, else current General/Simple);
2. exact node identity/action path;
3. payout profile;
4. ante profile;
5. exact stack/action sizings;
6. 2026 timestamp;
7. full node UPI;
8. UPI for EVERY legal action.

Importer:
- accepts physical combos or exact 169 classes;
- rejects shorthand `22+`, `A2s+`, etc.;
- expands to all 1326 physical combos;
- requires combo-by-combo action/full conservation;
- checks preflop suit symmetry;
- derives 169 classes;
- preserves explicit zero reach;
- recomputes aggregates;
- stores canonical source-capture SHA-256 in `import_audit`;
- carries payout + ante through to exact node.

Independent exact validator: `scripts/validate-exact-spin-nodes.mjs` repeats the critical schema/provenance/169/zero-reach checks and requires live-2026 capture. Legacy/Basic GTO Wizard cannot be a new exact primary.

## Canonical runtime identity / immutable admission

Runtime route now includes ante explicitly, e.g.:
`3MAX/WTA_CHIPEV_BASELINE/NONE/15/ROOT`

Files:
- `data/charts/exact-index.json` — currently EMPTY.
- `data/charts/exact-promotion-manifest.json` — currently EMPTY.
- `ui/exact-store.js`
- `scripts/validate-exact-index.cjs`
- `scripts/build-exact-index-entry.cjs`

Every runtime exact node must match:
- canonical route derived from its own JSON;
- `artifact_id`;
- live capture timestamp;
- exact file SHA-256;
- `VERIFIED_EXACT`.

Runtime verifies SHA-256 again in browser with WebCrypto. Exact files cannot be orphaned from the index.

## Promotion gate — valid file is NOT enough

Validator: `scripts/validate-exact-promotion.cjs`
Helper: `scripts/build-promotion-candidate.cjs`
End-to-end test: `scripts/test-exact-promotion-pipeline.mjs`

Candidate builder computes immutable route/artifact/evidence hashes but outputs `PROMOTION_CANDIDATE_NOT_ADMITTED`. It cannot auto-promote.

`PROMOTED_2026_EXACT` requires technical PASS plus independent compatible evidence. Two accepted proof types:

1. `COMPATIBLE_EXTERNAL_SOURCE`
   - full profile/tree compatibility resolved;
   - same tree/sizings;
   - source-usage compliant;
   - no unexplained numeric disagreement.

2. `INDEPENDENT_INTERNAL_SOLVER_PROOF`
   - exact same tree/profile/reach state;
   - independent evaluator;
   - independent best response;
   - precommitted threshold PASS;
   - immutable evidence file under `data/solver-evidence/`;
   - CI recomputes evidence SHA-256 and rejects tampering.

Runtime `ui/exact-store.js` enforces the same proof types as CI.

## 2026 source/compliance policy

Files:
- `data/chart-source-policy-2026.json` schema v2
- `docs/SOURCE_AUDIT_2026.md`
- `docs/SOURCE_USAGE_COMPLIANCE_2026.md`
- `scripts/validate-source-policy-2026.mjs`

Primary external reference order for new exact preflop:
1. current GTO Wizard Research;
2. current General;
3. current Simple;
4. Legacy/Basic historical only.

Hard source rules:
- no interpolation/default fill/screenshot reconstruction;
- never average different trees/sizings;
- source terms/rate limits must be respected;
- PreflopRanges public terms prohibit scraping/bulk/systematic extraction/quota bypass, therefore ONLY limited public spot checks are used;
- PreflopRanges payout/ante/solver provenance remains unresolved, so it is cross-check-only and cannot directly promote exact data.

## Independent 2026 public cross-check grid

Stored in `data/crosschecks/2026/` for 8/10/15/20/25bb BTN+SB roots. Missing public control at 2/4/6/12/17/23bb; NEVER interpolate.

Aggregates:
- 8 BTN: Fold 65.7 / Jam 34.3
- 8 SB: Fold 35.0 / Raise2 6.0 / Limp 6.5 / Jam 52.6
- 10 BTN: Fold 65.9 / Raise2 7.5 / Jam 26.6
- 10 SB: Fold 41.0 / Raise2 16.5 / Limp 0.8 / Jam 41.7
- 15 BTN: Fold 67.9 / Raise2 24.4 / Jam 7.7
- 15 SB: Fold 38.0 / Raise2.2 26.5 / Limp 16.0 / Jam 19.5
- 20 BTN: Fold 65.3 / Raise2 34.7
- 20 SB: Fold 35.9 / Raise2.5 33.9 / Limp 22.8 / Jam 7.4
- 25 BTN: Fold 61.9 / Raise2 38.1
- 25 SB: Fold 35.6 / Raise2.8 35.6 / Limp 27.1 / Jam 1.7

Limited public 15bb SB hand spot:
- AKo: Fold 0 / Limp 42 / Raise2.2 30 / Jam 28 (integer-percent display precision).

Cross-check tooling:
- `scripts/validate-crosscheck-profiles.mjs` — source/provenance/rounding/usage-policy validation.
- `scripts/audit-2026-crosschecks.mjs` — aggregate compatibility/disagreement classifier.
- `scripts/audit-2026-hand-spotchecks.mjs` — per-hand diagnostic; integer-display tolerance 0.51pp; unresolved payout/ante stays diagnostic-only.

Fatal only after profile compatibility is resolved. Different sizing/tree = mismatch classification, never averaging.

## Coverage

`data/chart-coverage-targets.json`:
- format SPIN_3MAX
- payout WTA_CHIPEV_BASELINE
- ante NONE
- UI stacks 2/4/6/8/10/12/15/17/20/23/25
- interpolation forbidden
- exact runtime nodes: ZERO
- P0 full response tree: 15bb, registry `data/charts/15bb-node-registry.json`.

## Latest confirmed CI

Chart Audit run `34206468697`
Head `d1ccb6219001fb117265363414000c6b50f1e562`
Conclusion: SUCCESS

24 major validation steps passed, including:
- chart data;
- 2026 source policy invariants;
- cross-check source compliance;
- GTO Wizard importer;
- zero-reach guard;
- end-to-end promotion pipeline;
- exact node schema;
- exact index route/SHA;
- runtime integrity;
- promotion self-test/manifest;
- aggregate + hand-level 2026 cross-check audits;
- coverage;
- UI/router;
- solver tests.

## CURRENT P0

First REAL 2026 exact node:
1. capture live GTO Wizard 15bb BTN first-in from preferred current family;
2. full node UPI + every action UPI + payout + ante + exact sizing + node identity + timestamp;
3. import -> 1326 conservation -> 169 matrix;
4. run independent exact validator;
5. build promotion candidate (NOT auto-promoted);
6. resolve independent same-tree evidence: compatible external source OR our independently measured same-tree solver proof;
7. classify every disagreement;
8. only after promotion proof PASS add immutable exact-index entry;
9. UI then paints first real exact chart / TRAIN unlocks.

Then repeat distinct 15bb response nodes: SB vs BTN raise; BB vs BTN raise after SB fold/call; BTN responses vs SB/BB jam/3bet; SB/BB first-in. Never collapse them into generic defend/3bet.

## Accuracy claim rule

Goal is measured quality better than existing tools, but NEVER claim “more accurate than GTO Wizard” because frequencies merely differ. `BEST_2026` requires apples-to-apples same profile/tree and stronger independently measured evidence.

## Fresh-chat recovery

1. Read THIS file.
2. Read `docs/SOURCE_AUDIT_2026.md` and `docs/SOURCE_USAGE_COMPLIANCE_2026.md`.
3. Read `docs/SOLVER_EXPERIMENT_STATUS.md` + newest `docs/checkpoints/*`.
4. Fetch current `chat-aligned-v2` head.
5. Fetch latest Chart Audit/Phase E/Rust CI.
6. Continue from CURRENT P0; do not redo completed foundations unless assumptions intentionally change.
