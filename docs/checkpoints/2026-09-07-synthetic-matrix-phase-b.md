# Immutable Checkpoint — Synthetic Solver Validation Phase B

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: Rust Solver Core run `34103322483`, head `c0efd5f4e5426f398cd2dc85305648868c113aad`, SUCCESS.

## Purpose

Validate the generic regret-matching / simultaneous-update / averaging machinery independently from:
- poker hand evaluation;
- equity calculation;
- card removal;
- terminal pot construction;
- Spin-specific game trees.

This prevents a poker payoff implementation from masking a generic solver bug.

## Added implementation

- `solver-rs/src/matrix_game.rs`
- `solver-rs/src/bin/synthetic_matrix_bench.rs`

`ZeroSumMatrixGame` supports:
- arbitrary finite row-payoff matrix;
- exact row and column action values;
- exact expected value;
- exact pure best responses;
- row/column BR gains;
- NashConv;
- simultaneous regret-matching sweep using frozen pre-update strategies.

## Test 1 — Matching Pennies

Row payoff matrix:

`[[+1,-1],[-1,+1]]`

Analytical equilibrium:
- Row: 0.5 / 0.5
- Column: 0.5 / 0.5
- value: 0
- NashConv: 0

Observed through 100,000 iterations:
- Row: exactly 0.5 / 0.5
- Column: exactly 0.5 / 0.5
- value: 0
- NashConv: 0

This case is necessary but insufficient because the initial uniform strategy is already the equilibrium.

## Test 2 — asymmetric non-50/50 game

Row payoff matrix:

`[[4,0],[-1,2]]`

For matrix `[[a,b],[c,d]]`, denominator:
`D = a-b-c+d = 7`

Analytical interior equilibrium:
- Row action 0 probability: `(d-c)/D = 3/7 = 0.4285714286`
- Column action 0 probability: `(d-b)/D = 2/7 = 0.2857142857`
- value: `(ad-bc)/D = 8/7 = 1.1428571429`

Observed at 100,000 simultaneous sweeps:
- Row: 0.430410831 / 0.569589169
- Column: 0.284338319 / 0.715661681
- Row max equilibrium error: 0.001839403
- Column max equilibrium error: 0.001375967
- value: 1.142839426
- value error: 0.000017717
- NashConv: 0.007806707

All benchmark assertions passed:
- each strategy max error < 0.01;
- value error < 0.01;
- NashConv < 0.02.

## CI gate

At this checkpoint:
- 79 Rust library tests passed, 0 failed;
- all exact HU/3-way payoff gates remain green;
- sampled-vs-exact audit remains green;
- both synthetic analytical games pass.

## Interpretation

The current generic `RegretTable` machinery successfully converges toward both:
- a symmetric 50/50 zero-sum equilibrium;
- an asymmetric interior zero-sum equilibrium with known non-50/50 mixes and non-zero value.

This is materially stronger than observing convergence only inside one poker subgame.

It still does NOT prove:
- full extensive-form CFR correctness for the entire 3-player Spin tree;
- correct reach propagation across every future branch;
- production-quality Spin charts.

## Next strict phase — Phase C

Exact payoff preparation and scaling.

1. Count unique suit-canonical HU matchup keys across broad/full legal ordered combo pairs.
2. Quantify deduplication factor versus raw legal combo pairs.
3. For 3-way, avoid naive full O(1326^3) enumeration until its computational cost is bounded.
4. First count canonical 3-way keys on representative sparse/structured supports and benchmark key-generation throughput.
5. Design persistent exact-payoff table format containing canonical key, exact equity and provenance/version metadata.
6. Benchmark table size, generation time and reuse economics.
7. Do not start a full 1,326-support coupled Spin solve until this precomputation design is validated.
