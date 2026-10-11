# Immutable Checkpoint — Bounded 12x12 Persisted Solver Scaling

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: run `34115851744`, head `9c0cf847e57a2714a97682c152e760d6002e66db`, SUCCESS.

## Purpose

Scale the persisted exact-payoff restricted solver beyond the 6x6 fixture while preserving bounded CI cost, exact-payoff provenance, zero board enumeration inside regret sweeps and deterministic repeatability.

## Support

Physical exact combo support:
- SB support size: 12;
- BB support size: 12;
- legal private-hand pairs after blockers: 128;
- unique canonical HU exact payoff keys required: 128.

This is a deterministic physical-combo research fixture, not a real Spin chart/range.

## Exact payoff preparation

- exact canonical payoff records: 128;
- encoded artifact bytes: 3,616;
- payoff build wall time: 25.306890 sec;
- measured build throughput: 5.057911 exact keys/sec;
- lookup-backed solver construction: 0.003664 sec.

## Solver-only runtime

Using the prepared exact lookup:
- sweeps: 10,000;
- solve #1: 0.486148 sec;
- solve #2: 0.499246 sec;
- solve #1 throughput: 20,569.871 sweeps/sec;
- exact board enumeration during solve: ZERO.

Two independent solves from the same exact payoff table produced identical:
- SB average strategy;
- BB average strategy;
- evaluation report.

Deterministic repeat: PASS.

## Restricted equilibrium diagnostic

At 10,000 sweeps:
- average SB jam across these 12 physical combos: 0.916625000;
- average BB call: 0.333364105;
- NashConv: 0.000117792bb;
- SB BR gain: 0.000052975bb;
- BB BR gain: 0.000064817bb;
- SB game value: 0.553088030bb.

## Scaling comparison

6x6 fixture:
- 34 canonical payoffs;
- payoff build ~6.55-7.71 sec depending runner pass;
- solve 10k ~0.45-0.52 sec.

12x12 fixture:
- 128 canonical payoffs;
- payoff build 25.31 sec;
- solve 10k 0.49 sec.

Conclusion:
- exact payoff generation scales roughly with required unique matchup keys and remains the dominant cost;
- once payoff data is present, restricted regret solving remains extremely cheap at these support sizes;
- solver runtime did not increase materially from 6x6 to 12x12 because current implementation/fixture size is still small relative to runner overhead and payoff-matrix scanning costs;
- persisted exact architecture is functioning as intended.

## Full gate

Same CI run retained:
- 93/93 Rust library tests passed;
- all Phase A/B/C persistence/integrity/exact-leaf gates green;
- 6x6 broad benchmark green;
- 12x12 broad benchmark green.

## What this does NOT mean

Do not infer:
- 12x12 fixture is a real Spin range;
- avg SB jam 0.916625 is chart data;
- full 1,326-combo solve is ready;
- non-all-in branches are solved;
- any chart is production-ready.

## Next strict step

Move controlled parallelism into the build-only exact HU payoff generator itself.

Requirements:
1. same canonical requested key set;
2. exact integer outcomes equal to serial build;
3. same canonical sorted records;
4. byte-identical encoded output;
5. reused existing records remain reused;
6. only missing keys are parallel-computed;
7. worker count explicit and bounded;
8. no runtime solver thread mutation or sampled fallback.

After the parallel build path is green, run a bounded 24x24 persisted fixture if projected total CI time remains safe.
