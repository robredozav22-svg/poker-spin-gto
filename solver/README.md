# Independent Spin Solver

Goal: generate our own strategy data from poker mathematics, then use GTO Wizard / Range+ / supplied screenshots only as external validation references.

## Non-negotiable rule

The solver must never contain guessed ranges. A chart may enter the app only when its provenance and precision are explicit.

Statuses:

- `SOLVER_EXACT`: exact game abstraction + converged solver + exact/approved equity inputs.
- `SOLVER_APPROX`: self-generated result with a declared approximation (sampling / abstraction / limited tree).
- `REFERENCE_CROSSCHECK`: external screenshot/site used only to compare outputs.
- `NOT_SOLVED`: no output may be shown as GTO.

## Implemented foundation

- canonical 169 starting-hand classes and all 1326 combinations;
- self-contained 5/7-card Hold'em evaluator;
- exhaustive concrete-combo equity oracle;
- deterministic sampled equity for development;
- canonical Spin node key preserving mode, payout profile, all stacks, action history and sizings;
- regret-matching core;
- HU BTN/SB jam-or-fold vs BB call-or-fold equilibrium abstraction that refuses to run without a complete equity table;
- unit tests for card model, evaluator, node identity, regret matching and no-guess behavior.

## Why 15-25bb is not solved yet

A correct 15bb+ first-in chart cannot be derived from preflop all-in equity alone. Raise/call/limp branches reach postflop, so their EV depends on future realization, position, SPR and downstream strategy. Therefore the first solver milestone is shallow/push-fold validation; the second milestone adds a postflop continuation-value model or full postflop abstraction.

## Milestones

### S1 — exact cards + shallow HU

1. Build our own class-vs-class equity table.
2. Solve HU 2/4/6bb push-fold abstractions.
3. Compare aggregate and boundary-hand behavior to external references.
4. Measure exploitability inside the same abstraction.

### S2 — 3-max all-in trees

Add sequential 3-player shove/call/fold trees with blockers, side-pot correctness and WTA utilities.

### S3 — preflop raise/limp trees

Add exact legal action trees and sizings for BTN/SB/BB. Non-all-in terminal nodes must reference a continuation-value model; no heuristic raw-equity shortcut is allowed in production.

### S4 — postflop continuation

Either:

- solve a controlled postflop abstraction; or
- train/cache continuation values from solved postflop subgames.

Only after this stage do 15/17/20/23/25bb self-generated ranges become candidates for `SOLVER_EXACT`/high-quality `SOLVER_APPROX`.

## Validation targets from supplied screenshots

These are **targets, not training labels**:

- 15bb BTN first-in: Fold 67.22 / Raise 2 25.41 / Jam 7.36.
- 15bb SB vs BTN Raise 2: Fold 78.31 / Call 2.41 / Jam 19.27.
- 15bb BB vs BTN Raise 2 + SB Call: Fold 59.11 / Call 22.19 / Jam 18.70.
- 15bb BB vs BTN Fold + SB Raise 3: Fold 47.12 / Call 33.85 / Jam 19.01.
- HU 2bb screenshot checkpoint: Fold 57.16 / Call 0.03 / Jam 42.82 (semantic context still requires confirmation).

We compare against them only after solving independently.
