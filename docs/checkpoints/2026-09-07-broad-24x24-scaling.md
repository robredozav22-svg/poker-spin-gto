# Immutable Checkpoint — Bounded 24x24 Persisted Solver Scaling

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: run `34116743330`, head `1d445d9e7df16344cb9b9d314abf5302650860a2`, SUCCESS.

## Purpose

Validate one further bounded support expansion after exact all-in payoff persistence and the deterministic parallel payoff builder were proven. The goal was to decide whether more support-only scaling would add meaningful mathematical information or whether work should move to the real Spin action tree.

## Support

Physical exact combo stress fixture:
- SB support: 24 physical combos;
- BB support: 24 physical combos;
- legal private-hand pairs after blockers: 514;
- unique canonical HU payoff keys required: 514.

These supports are intentionally diverse stress fixtures. They are NOT real Spin ranges and their aggregate frequencies are NOT chart data.

## Exact payoff preparation

Parallel build:
- workers: 2;
- canonical exact payoff records: 514;
- encoded artifact size: 14,424 bytes;
- payoff build time: 37.704552 sec;
- measured throughput: 13.632306 exact keys/sec;
- lookup-backed game construction: 0.002979 sec.

The 2-worker exact builder had already passed serial-vs-parallel equivalence:
- exact integer outcomes: identical;
- canonical record ordering: identical;
- encoded payload: byte-identical.

## Solver-only runtime

Using only the prepared persisted exact payoff lookup:
- sweeps: 10,000;
- solve #1: 0.472979 sec;
- solve #2: 0.473278 sec;
- solve throughput: 21,142.610 sweeps/sec;
- exact board enumeration during solve: ZERO.

Independent repeated solves from the same payoff table produced identical strategies and evaluation reports.

Deterministic repeat: PASS.

## Restricted equilibrium diagnostic

At 10,000 sweeps:
- average SB jam across the 24 physical stress combos: 0.874899403;
- average BB call: 0.416703050;
- NashConv: 0.000101721bb;
- SB BR gain: 0.000049249bb;
- BB BR gain: 0.000052471bb;
- SB restricted-game value: 0.349482301bb.

Again, these strategy aggregates are fixture diagnostics only.

## Scaling conclusion

3x3, 6x6, 12x12 and 24x24 all support the same architectural conclusion:
- exact payoff preparation is the dominant one-time cost;
- persisted exact payoff lookup removes board enumeration from repeated solver sweeps;
- repeated regret solving is very cheap at these bounded support sizes;
- deterministic convergence remains intact as support increases;
- further support-only scaling would mainly re-measure a known engineering curve rather than answer the central chart problem.

Decision: stop support-only escalation here. Do NOT jump to 48x48 or full 1,326x1,326 merely for size. Move to Phase D: canonical context-rich preflop action-tree representation and then solve branches only when their terminal/continuation EV contracts are mathematically valid.

## Full gate state

Run `34116743330`:
- 95/95 Rust library tests passed;
- exact HU/3-way math green;
- synthetic matrix validation green;
- persistence/build/manifest green;
- sampled-vs-exact cross-check green;
- 6x6 / 12x12 / 24x24 bounded scaling green;
- parallel exact builder equivalence green.

## Next strict phase — Phase D

1. Preserve existing restricted push/fold tree for regression tests.
2. Add a separate canonical preflop tree schema in which node identity contains full action history and exact sizing.
3. Represent amounts with deterministic integer fixed-point units, not floating-point keys.
4. Distinguish:
   - exact all-in terminal payoff available;
   - non-all-in continuation requiring measured postflop EV;
   - unresolved/missing continuation that must fail closed.
5. Do not invent legal raise sizes or strategy frequencies merely to populate the tree.
6. Add exact source/verification metadata for every tree specification.
7. Only after the schema is green should the first real multi-action Spin branch be instantiated from a verified tree source.
