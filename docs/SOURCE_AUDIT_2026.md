# 2026 Spin chart source audit

Date: 2026-09-08

## Objective

The application must not merely reproduce one provider. The target is the most accurate, current and auditable off-table Spin preflop chart set we can produce for the exact game profile.

## Current primary source policy

### GTO Wizard

GTO Wizard remains the preferred primary external solver reference because:

- its current product still exposes Spin & Go solutions;
- its overhauled Spin solution families replaced old General with higher-accuracy modern solutions;
- the published Spin overhaul describes new General/Simple solutions solved to roughly 0.1%–0.017% of pot and optimized preflop sizing trees;
- Research solutions are specifically intended for studying optimal preflop sizings;
- ranges from a selected node and ranges of individual actions can be copied in standard UPI/Pio/GTO+ text format.

For new exact preflop charts, source-family preference is:

1. Research — sizing/tree discovery and exact preflop strategy when available;
2. General — current high-accuracy full-game baseline;
3. Simple — implementation-oriented cross-check;
4. Legacy/Basic — historical comparison only; never a new primary exact source.

A live 2026 node capture is required. The historical publication year of the solver family is not enough by itself.

## Independent 2026 references

### PreflopRanges.app

Current public pages expose solved 3-max Spin charts and aggregate frequencies for 8–25bb, with a much larger viewer advertised for response nodes. This is useful as an independent solver-derived cross-check but not as the source of `VERIFIED_EXACT` data unless its exact source/tree identity can be established and full per-hand frequencies are captured losslessly.

Observed current public 15bb examples:

- BTN first in: Fold 67.9%, Raise to 2bb 24.4%, Jam 7.7%.
- SB after BTN fold: Fold 38.0%, Raise to 2.2bb 26.5%, Limp 16.0%, Jam 19.5%.

Historical screenshot checkpoints in this project differ:

- BTN first in screenshot: Fold 67.22%, Raise to 2bb 25.41%, Jam 7.36%.
- SB historical screenshot reference does not match the same public 2.2bb+limp tree.

These differences must **not** be averaged. The SB difference is clearly a tree/sizing-profile difference. BTN differences may be rounding, solver version or tree/profile variation and require exact node identity before being treated as an accuracy disagreement.

### GTOCharts.com

A 2026 site currently advertises solver-computed Spin & Go charts. It is retained as a freshness/coverage and gross-anomaly reference, not an exact promotion source, because the publicly indexed material does not expose enough solver configuration and node provenance for our exact gate.

## Required comparison key

Two charts are numerically comparable only when all fields match:

- format: Spin regular vs Spin+Ante;
- player count;
- payout/chip-EV profile;
- effective stacks for all players, not just hero;
- hero position;
- complete action history;
- every legal action and exact size;
- blind/ante structure;
- solution family/tree profile.

If any field differs, classify the result as a profile mismatch rather than an accuracy disagreement.

## Promotion ladder

A chart reaches `VERIFIED_EXACT` only after:

1. live 2026 primary-source capture;
2. exact full-node UPI + every action UPI;
3. 1326-combo conservation check;
4. 169-class conversion with explicit zero-reach hands;
5. suit-symmetry check for preflop classes;
6. aggregate recomputation from the matrix;
7. exact tree/sizing metadata validation;
8. independent current-source cross-check;
9. discrepancy classification;
10. runtime exact-index admission.

`BEST_2026` is a higher claim and must additionally have compatible-source agreement or a documented, technically justified reason for disagreement plus stronger numerical evidence from our own validated solver.

## Solver relationship

External charts are validation/reference data, not training labels. Our own solver is allowed to outperform an external provider only if measured evidence supports that claim. We never infer superiority from a different frequency, a newer date, or a lower-looking solver tolerance alone.
