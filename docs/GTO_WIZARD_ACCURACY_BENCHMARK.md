# Accuracy Benchmark — Poker Spin GTO vs GTO Wizard

Status: RESEARCH_ONLY
Date: 2026-09-07

## Goal

Do not claim that this project is more accurate than GTO Wizard until the claim is supported by reproducible apples-to-apples benchmarks.

The target is to exceed public GTO Wizard quality on selected Spin & Go preflop trees in:
1. measurable equilibrium accuracy where exact/credible best-response metrics are available;
2. reproducibility;
3. provenance and auditability;
4. multiway cross-validation;
5. physical-combo fidelity;
6. deterministic tree identity and sizing fidelity.

## Public GTO Wizard reference points

Public GTO Wizard materials state that modern Spin & Go solutions were solved to approximately 0.1%–0.017% of the pot, depending on the solution. Their documentation also explains that exact Nash Distance for large multiway preflop games is generally intractable, so multiway accuracy is validated using internal benchmarks against third-party solvers and deeper/longer solves.

References:
- https://blog.gtowizard.com/introducing-gto-wizard-ai-heads-up-preflop-solver/
- https://blog.gtowizard.com/new-spins-solutions-study-plans-and-ev-comparison/
- https://blog.gtowizard.com/gto-wizard-ai-custom-multiway-solving/
- https://blog.gtowizard.com/how-solvers-work/
- https://blog.gtowizard.com/why-doesnt-my-solution-match-gto-wizard/

These figures are reference targets, not proof about any specific node we have not independently tested.

## Tier A — directly measurable equilibrium benchmark

Use only on games/subgames where exploitability / NashConv / exact best response is credibly measurable.

### Promotion target

- NashConv / exploitability <= 0.01% of starting pot.
- Deterministic repeat PASS.
- Independent recomputation PASS.
- Same exact tree, stacks, payouts and action sizes across compared solves.
- No missing payoff states.
- No approximated continuation hidden behind showdown equity.

0.01% pot is deliberately stricter than the best public GTO Wizard Spin accuracy figure of 0.017% pot.

### Gold research target

- NashConv / exploitability <= 0.005% of starting pot where computationally practical.

This is a research target, not a current achieved result.

## Tier B — full 3-max multiway preflop benchmark

Do NOT publish a fabricated exact Nash Distance for a game where it cannot be credibly computed.

A multiway solution may be promoted only after all of the following:

### 1. Tree identity lock
- same stack vector / effective-stack assumptions;
- same blind/ante structure;
- same payout profile;
- same actor order;
- same exact sizing set;
- same complete history identity;
- same solution-tree profile.

### 2. Deep-runtime self-consistency
Solve the identical tree at increasing compute budgets.

Required metrics:
- physical-combo action-frequency delta;
- aggregate action-frequency delta;
- node EV delta;
- root EV delta;
- best-response gap on any tractable restricted projections;
- change in measured regret/convergence proxies.

Promotion requires later/deeper runs to stop materially changing EV.

### 3. Independent solver holdout
At least one independent implementation or externally solved export must be compared on identical trees.

Frequency similarity is diagnostic only. EV and exploitability proxies are primary.

### 4. Third-run arbitration
If our solution and an external solver materially disagree:
- do not choose ours by default;
- run a deeper internal solve;
- compare EV of both candidate strategies against independent responses;
- isolate whether disagreement comes from tree, payouts, sizing, abstraction, convergence or implementation.

### 5. Physical combo fidelity
Promotion source is 1,326 physical Hold'em combos.

169-class output is a presentation layer only.

Before aggregation:
- inspect suit variants;
- quantify within-class frequency dispersion;
- flag any strategically meaningful suit asymmetry;
- prohibit shorthand-parser reconstruction as an exact source.

### 6. Postflop continuation coverage
Every non-all-in terminal preflop path marked `RequiresPostflopEv` must have a complete verified continuation artifact.

A showdown equity approximation is not accepted as postflop EV.

### 7. Deterministic repeat
Identical input artifact + same solver profile must reproduce acceptance metrics within declared deterministic tolerance.

## Comparison against GTO Wizard

When an exact GTO Wizard node/export is available, compare only if all input assumptions match.

For every node record:
- tree profile;
- action history;
- hero position;
- effective stack and, where relevant, full asymmetric stack vector;
- legal actions and exact sizes;
- GTO Wizard aggregate frequencies;
- our aggregate frequencies;
- physical-combo frequency deltas where source data allows;
- EV difference;
- best-response/exploitability evidence where measurable;
- source accuracy metadata;
- independent validation result.

## Strategy-frequency comparison is not an accuracy score

A large frequency difference may occur on near-indifferent actions with negligible EV difference.

Therefore:
- `mean L1 frequency delta` = diagnostic;
- `max action-frequency delta` = diagnostic;
- EV regret / exploitability / independent-response performance = accuracy evidence.

## Required superiority claim format

Forbidden:
`Our solver is more accurate than GTO Wizard.`

Allowed only after evidence:
`On benchmark set X, using identical tree/payout/sizing assumptions, our measured exploitability was Y versus reference Z, with independent validation A and deterministic-repeat PASS.`

For multiway where exact Nash Distance is unavailable:
`On benchmark set X, our solution showed lower holdout EV regret / tighter deeper-run stability than reference Y under the documented benchmark protocol.`

## Current status

Current project does NOT yet satisfy a general superiority claim over GTO Wizard.

What is already stronger by construction:
- explicit tree-profile identity;
- fail-closed action sizing;
- physical-combo strategy storage requirement for exact promotion;
- continuation artifact gate;
- solver-run provenance gate;
- deterministic and CI audit path.

These are architecture advantages, not yet proof of better poker strategy accuracy.

## Next benchmark set

Start with one complete 15bb WTA/ChipEV Spin 3-max tree.

Minimum target nodes:
- BTN first-in;
- SB first-in after BTN fold;
- SB vs BTN raise;
- SB vs BTN jam;
- BB vs BTN raise after SB fold;
- BB vs BTN raise after SB call where tree allows;
- BB vs BTN jam;
- BB vs SB raise;
- BB vs SB limp;
- BB vs SB jam;
- BTN response to SB 3bet/jam;
- BTN response to BB 3bet/jam;
- later response nodes after BB jam where applicable.

Do not call the 15bb benchmark complete until all ChildDecision descendants required by the chosen exact tree profile exist in the catalog.
