# Immutable Checkpoint — Bounded 6x6 Persisted Solver Scaling

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: run `34109902935`, head `ff676c7273282ca4374f299a4d49dd6514e7f920`, SUCCESS.

## Purpose

Test whether the exact-payoff restricted solver remains numerically and operationally practical when support expands beyond the original 3x3 sparse validation game, while ensuring board enumeration happens only during one-time payoff preparation and never inside repeated regret sweeps.

## Support

Physical exact combo support:
- SB support size: 6;
- BB support size: 6;
- legal private-hand pairs after blockers: 34;
- unique canonical HU exact payoff keys required: 34.

This is a bounded physical-combo research fixture, not a 169-class range and not a real Spin chart.

## Exact payoff preparation

- canonical exact payoff records: 34;
- encoded artifact bytes: 984;
- exact payoff build wall time: 7.707570 sec;
- lookup-backed solver constructor: 0.003508 sec.

## Solver-only runtime

Using the prepared exact lookup:
- sweeps: 10,000;
- solve #1: 0.522367 sec;
- solve #2: 0.523163 sec;
- ~19,143.639 sweeps/sec in solve #1;
- exact board enumeration during solve: ZERO.

Two independent solves from the same exact payoff table produced identical:
- SB average strategy;
- BB average strategy;
- evaluation report.

Deterministic repeat: PASS.

## Restricted equilibrium diagnostic

At 10,000 sweeps:
- average SB jam across the six physical support combos: 0.746641280;
- average BB call: 0.270040309;
- NashConv: 0.001363022bb;
- SB BR gain: 0.000765864bb;
- BB BR gain: 0.000597158bb;
- SB game value: 0.637463460bb.

Interpretation:
- the solver itself is cheap once exact payoff data is prepared;
- the dominant cost remains one-time exact matchup generation;
- lookup/table architecture successfully removes board enumeration from the iterative solve loop;
- broader support did not break deterministic convergence on this bounded fixture.

## Full gate state

Same CI run also kept green:
- 93/93 Rust tests;
- exact HU/3-way equity;
- exact leaves;
- sampled-vs-exact cross-check;
- synthetic matrix validation;
- canonical census;
- persistence/lookup/build/manifest;
- generation economics;
- persisted leaf equivalence.

## What this does NOT mean

Do not infer:
- full Spin GTO is solved;
- these six-combo averages are playable chart frequencies;
- 6x6 support represents a real range;
- non-all-in branches are solved;
- production chart promotion is allowed.

## Next bounded stage

Run a deterministic 12x12 physical-combo fixture if CI economics remain safe. Measure:
- legal pair count;
- unique canonical payoff count;
- payoff build time;
- artifact bytes;
- lookup construction time;
- solver-only time;
- deterministic repeat;
- NashConv.

Stop escalation if exact payoff build or total workflow approaches resource limits; do not jump directly to 1,326 x 1,326.
