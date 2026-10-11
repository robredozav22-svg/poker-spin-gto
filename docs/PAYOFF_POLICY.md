# All-in Payoff Policy

Status: RESEARCH_ONLY
Date: 2026-09-07

## Canonical rule

For preflop all-in terminal evaluation in the Rust research solver:

- heads-up all-in payoff: EXACT board enumeration is canonical;
- genuine three-way all-in payoff: EXACT common-board enumeration is canonical;
- Monte Carlo/sample-based equity is NON-CANONICAL for terminal payoff generation.

Exact means enumerating every legal five-card board after fixed private cards:
- HU: C(48,5) = 1,712,304 boards;
- 3-way: C(46,5) = 1,370,754 boards.

## Canonical implementation paths

HU:
- `exact_equity.rs`
- `exact_range_equity.rs`
- `exact_leaf.rs`
- exact payoff matrices such as `restricted_exact.rs`

3-way:
- `exact_equity3.rs`
- `exact_range_equity3.rs`
- `exact_leaf3.rs`

## Sampled implementation policy

The following sampled paths remain in the repository for:
- regression comparisons;
- performance experiments;
- independent sanity checks;
- research on cases not yet migrated to exact calculation.

They must not be used as the authoritative payoff source when an exact all-in operator exists:
- `equity.rs::sampled_hu_equity`
- `equity3.rs::sampled_threeway_equity`
- `range_equity.rs::sampled_equity_vs_range`
- `range_equity3.rs::sampled_threeway_equity_vs_ranges`
- sampled all-in leaf helpers in `leaf_ev.rs`
- sampled SB/restricted research paths in `sb_ev.rs` / `restricted_subgame.rs`

The `sampled_` prefix is deliberate and must be retained.

## Validation rule

Sampled-vs-exact tests are one-way validation:
- exact is the reference;
- sampled may be required to fall within a documented tolerance;
- agreement does not downgrade exact to an approximation;
- disagreement must never be resolved by averaging exact and sampled results.

## Non-all-in warning

This policy applies to ALL-IN terminal payoffs only.

Non-all-in preflop branches require measured postflop continuation EV. Raw showdown equity, whether exact or sampled, is NOT a valid replacement for continuation EV.

## Promotion warning

An exact terminal payoff does not make a complete strategy `VERIFIED_EXACT`.
Tree correctness, action sizes, payout profile, convergence/exploitability, 1326->169 aggregation audit and independent solver/reference validation remain required before chart promotion.
