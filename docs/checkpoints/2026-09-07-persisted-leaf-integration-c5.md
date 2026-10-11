# Immutable Checkpoint — C5 Persisted Exact Leaf Integration

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: run `34109465365`, head `e9b0f94f59f4d466225e7f3fe7d22a0f57f4b1a4`, SUCCESS.

## Purpose

Close the runtime side of Phase C exact all-in payoff persistence: use precomputed exact payoff records in blocker-conditioned HU and genuine 3-way range integration and terminal leaf evaluation, with strict fail-closed behavior on incomplete tables.

## Added

- `solver-rs/src/persisted_range_equity.rs`
- table-backed HU API `persisted_bb_leaf_action_values(...)`
- table-backed 3-way API `persisted_bb_after_btn_jam_sb_call_values(...)`
- `solver-rs/src/bin/persisted_leaf_smoke.rs`

## Runtime policy

Persisted path:
- reads exact integer-outcome-backed payoff records only;
- uses the same blocker-conditioning / joint card-removal weighting as direct exact path;
- missing required payoff key = error;
- no direct exact enumeration occurs implicitly in persisted functions;
- no sampled equity fallback occurs implicitly;
- direct exact functions remain explicit build/research tools.

## Equivalence fixture — HU

BB = KK, jammer = AA, stack = 8bb.

Persisted result:
- equity BB = 0.187445103;
- Fold EV = -1.000000000bb;
- Call EV = -5.000878349bb.

Direct exact vs persisted:
- range equity match within 1e-12;
- Fold EV match within 1e-12;
- Call EV match within 1e-12.

Empty HU payoff lookup:
- required key missing -> REJECT.

## Equivalence fixture — genuine 3-way

Node:
`BTN jam -> SB call -> BB [Fold,Call]`

BB=QQ, BTN=AA, SB=KK, stack=8bb.

Persisted range-equity order BB/BTN/SB:
- 0.146191074 / 0.665054415 / 0.188754510.

Persisted leaf:
- Fold EV = -1.000000000bb;
- Call EV = -4.491414214bb.

Direct exact vs persisted:
- all three equities match within 1e-12;
- Fold EV match within 1e-12;
- Call EV match within 1e-12.

Empty 3-way payoff lookup:
- required key missing -> REJECT.

Smoke output explicitly records:
- `direct_vs_persisted=PASS`
- `missing_hu=REJECT`
- `missing_threeway=REJECT`
- `sampled_fallback=NONE`

## Full gate

Run `34109465365`:
- 93/93 Rust library tests passed;
- all previous exact equity / terminal / NashConv / persistence / manifest / generation economics gates stayed green;
- persisted leaf equivalence smoke passed.

## Phase C conclusion

The infrastructure path is now complete for currently modeled all-in contexts:

`canonical matchup -> exact board enumeration build tool -> versioned integer-outcome table -> integrity manifest -> read-only indexed lookup -> blocker-conditioned range integration -> terminal EV`

This does NOT mean:
- complete HU payoff artifact has been generated;
- full 3-way payoff universe exists;
- full Spin preflop tree is solved;
- any chart is production-ready.

## Next strict step

Scale the restricted exact-payoff solver using precomputed/table-backed payoffs on broader hand supports. Measure:
- number of legal combo pairs;
- number of unique canonical payoff keys needed;
- one-time exact payoff build cost;
- repeated solve runtime once payoff data is in memory;
- NashConv / strategy stability versus support size.

Do not begin full 1,326 x 1,326 support blindly. Increase support in bounded deterministic stages and preserve CI runtime limits.
