# GTO Wizard Comparison Protocol

Status: RESEARCH_ONLY
Date: 2026-09-07

## Goal

The project may target accuracy stricter than publicly reported GTO Wizard Spin solution accuracy, but it must NOT claim to be more accurate than GTO Wizard from vendor-reported numbers, screenshot similarity, range overlap, or a different convergence metric.

A superiority claim is allowed only after an apples-to-apples independent benchmark under this protocol.

## Public external reference

GTO Wizard has publicly described modern Spin & Go solutions as ultra-high accuracy, with reported ranges around 0.1%-0.017% of pot depending on solution, and provides multiple solution/tree families such as General, Simple, Research/Test and different preflop sizing choices.

These public figures are treated as background targets only. They are NOT automatically metric-equivalent to our measured NashConv or any internal convergence value.

## Internal project target

`spin-verified-exact-v1` target:
- normalized convergence/exploitability upper bound <= 0.0001 of pot = 0.01% pot;
- independent validation required;
- deterministic repeat required for internal solver runs.

This threshold is an engineering acceptance target, not a current achievement and not a comparative claim.

## Required apples-to-apples comparison

Candidate and external reference must use exactly the same:
- game format;
- payout profile;
- effective stacks, including asymmetric stacks where applicable;
- complete tree profile;
- legal action set and exact sizes;
- card abstraction policy (prefer no preflop abstraction for final comparison);
- terminal payoff definition;
- postflop continuation model/tree;
- evaluated state coverage;
- independent evaluator profile;
- exploitability/Nash-distance definition and normalization.

If any of these differ, the result is descriptive only and cannot support a superiority claim.

## Three distinct benchmark layers

### Layer 1 — strategy distance diagnostic

Compare full 1,326 physical-combo strategies node-by-node:
- mean L1 frequency distance;
- maximum action-frequency delta;
- number of physical combos with >1 percentage-point action delta;
- number with >5 percentage-point action delta;
- action-mass differences.

Important: frequency similarity is NOT accuracy. Near-indifferent actions may mix differently with almost no EV loss.

### Layer 2 — independent quality benchmark

Both solutions are evaluated by the SAME independent best-response/exploitability evaluator.

Record for each artifact:
- tree profile ID;
- evaluator profile/source ID;
- strategy artifact checksum;
- evaluated state count;
- at least 3 repetitions;
- normalized exploitability fraction of pot.

No solver's own reported convergence number is accepted as the sole comparison metric.

### Layer 3 — superiority gate

A claim that our solution is more accurate is permitted only when:
- candidate and reference have identical tree/evaluator/state coverage;
- both artifacts are independently identifiable and distinct;
- both benchmarks have >=3 repetitions;
- candidate exploitability is lower;
- the improvement exceeds a pre-declared absolute margin;
- candidate also passes the project's absolute accuracy ceiling.

The gate is implemented in `solver-rs/src/benchmark_claim.rs`.

## Validation against GTO Wizard

When an exact Wizard export/reference is available:

1. Freeze one exact Wizard tree profile first.
2. Recreate or import the identical legal action tree without approximation.
3. Verify all node histories/sizes before comparing frequencies.
4. Compare full physical-combo strategies where export fidelity allows it.
5. Run both artifacts through the same independent evaluator.
6. Keep Wizard vendor accuracy as a separate metadata field only.
7. Produce a signed/versioned benchmark report with checksums.

## Forbidden shortcuts

Never claim superiority from:
- screenshot aggregates;
- 169-class visual similarity;
- one or a few hands;
- action-frequency MAE alone;
- our NashConv versus Wizard's differently defined public accuracy;
- different trees/sizings;
- different postflop models;
- interpolated stack depths;
- exploit adjustments compared against GTO baseline;
- a single benchmark run.

## Current status

No superiority claim is currently authorized.

Current Phase D infrastructure is designed so that such a claim can only become possible after exact tree, continuation, strategy, convergence and independent benchmark provenance are complete.
