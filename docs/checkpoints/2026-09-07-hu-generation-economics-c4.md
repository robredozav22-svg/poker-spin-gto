# Immutable Checkpoint — C4 HU Exact Generation Economics

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: run `34108942617`, head `e0c993d1cc968cde1ddf8ed92c01280cb8b3d0b7`, SUCCESS.

## Purpose

Measure real exact HU payoff-generation throughput before deciding whether/how to build all 93,769 suit-canonical HU payoff keys.

## Benchmark design

Binary:
`solver-rs/src/bin/hu_generation_economics.rs`

Batch:
- 8 deterministic unique canonical HU keys;
- exact C(48,5) board enumeration for every key;
- modes: serial 1 worker, parallel 2 workers, parallel 4 workers;
- std threads only;
- result records sorted canonically after worker completion;
- exact integer wins/losses/ties must match serial baseline exactly.

Full HU canonical universe reference:
- 93,769 unique keys.

## Measured results on GitHub runner

### 1 worker
- batch time: 1.786770 sec
- throughput: 4.477354 keys/sec
- estimated full 93,769 build: 20,942.949 sec
- estimated minutes: 349.049
- estimated hours: ~5.82

### 2 workers
- batch time: 0.891578 sec
- throughput: 8.972857 keys/sec
- estimated full build: 10,450.295 sec
- estimated minutes: 174.172
- estimated hours: ~2.90
- speedup: 2.004053x

### 4 workers
- batch time: 0.825582 sec
- throughput: 9.690138 keys/sec
- estimated full build: 9,676.746 sec
- estimated minutes: 161.279
- estimated hours: ~2.69
- speedup: 2.164255x

## Correctness

- 2-worker integer outcomes == serial outcomes: PASS
- 4-worker integer outcomes == serial outcomes: PASS
- canonical sorted output: PASS

## Interpretation

2-worker scaling is essentially ideal on this runner. Four workers add only modest improvement beyond two, suggesting runner CPU capacity/contention rather than a solver correctness issue.

The projected full-table times are ESTIMATES from one deterministic 8-key CI batch. They are not measured 93,769-key build runtimes. Runner type, CPU scheduling, matchup mix, compiler/runtime variation and storage/build orchestration may change real wall time.

Decision:
- do not generate all 93,769 keys inside normal CI;
- design a separate resumable/versioned artifact generation workflow if full HU materialization is later approved;
- persistent lookup makes repeated solves economically useful once generated;
- use 2 workers as a sensible baseline on similar constrained runners; benchmark actual target hardware before a full build.

## Next phase — C5

Integrate persisted exact payoff lookup into all-in range/leaf runtime paths with strict fail-closed behavior on missing keys and no sampled fallback.
