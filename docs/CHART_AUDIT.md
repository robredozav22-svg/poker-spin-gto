# Spin & Go chart audit — recovered baseline

Status: active verification. The simplified `main/index.html` is **not** a strategy source and remains `INVALID_FOR_STRATEGY`.

## What was recovered from the project history

The project had already progressed far beyond the current `main` before the GitHub simplification:

- **V37** — legal preflop state machine instead of a finite list of generic spot buttons.
- **V40** — all-in/reopen legality and asymmetric-stack safety audit.
- **V41** — all ten public 3-max BTN/SB opening anchors at 8/10/15/20/25bb rebuilt and checked against exact published aggregate combo totals.
- **V42** — all-range sanity pass; fixed shorthand parsing that could create offsuit CALL + suited FOLD contradictions; rebuilt public call-vs-jam ranges; zero suited-over-offsuit violations across audited anchors.
- **V43** — final recovered UX rule: one visible `EFF BB` selector, 0.5bb steps, one-screen replay/study workflow.

See `docs/RECOVERED_PROJECT_BASELINE.md` for the preserved product/engine rules.

## Current source hierarchy

1. **GTO Wizard Spin & Go** — primary solver reference. Current GTO Wizard material documents regular Spin study depths up to 33bb, Spin+Ante to 25bb, multiple SB opening-size trees, and 600+ asymmetric-stack situations.
2. **PokerStars Learn Spin & Go** — explicit public call/Nash ranges and independent structural cross-check.
3. **Public solved Spin opening sources** — aggregate action totals and source envelopes for cross-checking BTN/SB first-in anchors.
4. **GGPoker official format material** — format/rules sanity only, not exact range source.
5. **Legacy Gold / field models** — exploit layer only, never silently labelled GTO.

## Audited 3-max RFI aggregate combo targets

The full 1326 starting combinations are distributed as follows. These are the V41 targets and all rows sum exactly to 1326.

| Stack | Position | JAM | RAISE | LIMP | FOLD |
|---:|:---:|---:|---:|---:|---:|
| 8 | BTN | 455 | 0 | 0 | 871 |
| 10 | BTN | 353 | 99 | 0 | 874 |
| 15 | BTN | 102 | 323 | 0 | 901 |
| 20 | BTN | 0 | 461 | 0 | 865 |
| 25 | BTN | 0 | 505 | 0 | 821 |
| 8 | SB | 697 | 79 | 86 | 464 |
| 10 | SB | 554 | 218 | 11 | 543 |
| 15 | SB | 258 | 352 | 212 | 504 |
| 20 | SB | 98 | 449 | 302 | 477 |
| 25 | SB | 22 | 472 | 360 | 472 |

Important: these totals do **not** magically make every per-hand mixed frequency solver-exact. V41/V42 grade these rebuilt opening matrices as source-constrained `B`: aggregate totals and source envelopes are enforced, while mixed per-hand allocation is reconstructed under those constraints.

## Explicit public call-vs-open-shove anchors

The approved public range text and combo counts are frozen in `data/source-verified-anchors.json`.

Families covered:
- 3-max BB vs BTN open-shove after SB fold: 10 / 15 / 25bb;
- 3-max BB vs SB open-shove after BTN fold: 10 / 15 / 25bb;
- 3-max SB vs BTN open-shove: 11 / 25bb;
- HU BB vs BTN open-shove: 10 / 13 / 25bb.

V42 rebuilt these from their public source definitions and enforces the dominance invariant: if the offsuit version of the same ranks is a CALL, the suited version cannot be FOLD.

## Why the current `main` ranges are rejected

The simplified `main/index.html`:
- contains only partial stack data while exposing more stack buttons;
- has many ranges that expand to nearly/all 169 hand classes;
- uses generic `3bet`/`defend` contexts instead of the prior action tree;
- cannot correctly represent limp/call/raise/jam mixed frequencies;
- lost source grades and the distinction between exact/source-constrained/model/exploit data.

This is a regression relative to V41–V43, not the baseline we should continue from.

## Required final data key

Every approved chart must be addressable by enough context to identify the actual poker node, e.g.:

`format -> payout_profile -> effective_stack -> hero_position -> previous_actions -> villain_position -> villain_size -> hero_actions -> hand -> frequencies`

A missing exact node must remain `MISSING_EXACT`; it must not be replaced by a superficially similar branch such as using `BB vs SB open jam` for `BTN raise -> SB jam -> BB cold decision`.

## Grade policy

- `A / VERIFIED_EXACT` — explicit published structured range, approved exact export, or matching Nash source.
- `B / SOURCE_CONSTRAINED` — public aggregate/envelope plus constrained mixed reconstruction; **not exact per-hand solver frequency**.
- `M / MODEL_REFERENCE` — model, asymmetric/nearest-depth proxy, or complex response without exact approved export.
- `G / EXPLOIT` — field/Gold/population adjustment.
- `MISSING_EXACT` — legal node exists, exact approved chart unavailable.

## Regression gate

Run `node scripts/validate-chart-data.mjs` whenever chart data changes. The gate checks the 169-hand/1326-combo universe, V41 aggregate anchors, exact-chart metadata/frequency sums, no silent interpolation, valid action vocabulary, and the V42 suited-over-offsuit CALL invariant.

**No guessed range is promoted into the GTO baseline.**
