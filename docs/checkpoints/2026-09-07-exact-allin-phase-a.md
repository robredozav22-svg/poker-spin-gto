# Immutable Checkpoint — Exact All-in Phase A

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed CI: Rust Solver Core run `34102878683`, head `44cb7a5fe16164a7ba1e911bf43a855d525f21d7`, SUCCESS.

## What is now established

### Exact HU equity
- Enumerates all C(48,5)=1,712,304 legal boards after four fixed hole cards.
- AA vs KK: 0.812554897 / 0.187445103.
- Zero-sum error: 0.
- Release runtime after compilation: about 0.21-0.23 sec for one full matchup on GitHub runner.

### Exact three-way equity
- Enumerates all C(46,5)=1,370,754 legal boards after six fixed hole cards.
- AA / KK / QQ: 0.665054415 / 0.188754510 / 0.146191074.
- Zero-sum error about 1.15e-13.
- Release runtime after compilation: about 0.25-0.28 sec for one full matchup.

### Exact HU range/leaf path
Canonical research files:
- `solver-rs/src/exact_equity.rs`
- `solver-rs/src/exact_range_equity.rs`
- `solver-rs/src/exact_leaf.rs`

Smoke BB=KK vs AA jammer at 8bb:
- exact BB equity: 0.187445103;
- `BTN fold -> SB jam -> BB fold`: -1.000000000bb;
- same node BB call: -5.000878349bb;
- `BTN jam -> SB fold -> BB fold`: -1.000000000bb;
- same node BB call: -4.907155797bb;
- same canonical matchup reused across the two leaves: 1 cache miss + 1 cache hit.

### Exact three-way range/leaf path
Canonical research files:
- `solver-rs/src/exact_equity3.rs`
- `solver-rs/src/exact_range_equity3.rs`
- `solver-rs/src/exact_leaf3.rs`

Smoke node `BTN jam -> SB call -> BB Fold/Call`, BTN=AA, SB=KK, BB=QQ:
- BB fold EV: -1.000000000bb;
- BB call EV: -4.491414214bb;
- exact equities in integrator order BB/BTN/SB: 0.146191074 / 0.665054415 / 0.188754510;
- one jointly compatible BTN/SB range pair;
- one exact cache miss on first evaluation.

### Exact restricted coupled solver
Game: `BTN folds -> SB [Fold, Jam] -> BB [Fold, Call]`.
Sparse supports: SB AA/A5s/76s; BB KK/AQo/65s.
Exact payoff matrix, six legal private-hand pairs.

Exact NashConv:
- sweep 1: 1.351545831bb
- 50: 0.040246151bb
- 100: 0.029302555bb
- 200: 0.015213324bb
- 500: 0.009025857bb
- 1,000: 0.006427089bb
- 2,000: 0.004349300bb
- 5,000: 0.002590215bb
- 10,000: 0.001778596bb

At 10,000:
- avg SB jam 0.587024326;
- avg BB call 0.360426620;
- SB BR gain 0.001187979bb;
- BB BR gain 0.000590617bb;
- SB game value 0.060849760bb.

This validates only the modeled restricted game, not full Spin GTO.

## Sampled-vs-exact independent audit

Deterministic sample corpus: 200,000 boards, seed 20260907.

HU AA vs KK:
- exact: 0.812554897;
- sampled: 0.812100000;
- absolute difference: 0.000454897.

Three-way AA/KK/QQ:
- exact: 0.665054415 / 0.188754510 / 0.146191074;
- sampled: 0.665898333 / 0.188693333 / 0.145408333;
- max absolute difference: 0.000843918.

Result: sampled implementations are useful independent cross-checks, but exact remains canonical.

Policy file: `docs/PAYOFF_POLICY.md`.

## Rust gate at this checkpoint

- 76 library tests passed, 0 failed.
- sampled sparse convergence benchmark passed.
- exact HU smoke passed.
- exact restricted NashConv benchmark passed.
- exact 3-way smoke passed.
- exact 3-way leaf smoke passed.
- exact HU leaf smoke passed.
- sampled-vs-exact cross-check passed.

## What must NOT be inferred

This checkpoint does NOT mean:
- the full 3-max Spin tree is solved;
- any self-generated chart is ready for production;
- 8/10/12/15/20/25bb ranges are verified;
- non-all-in continuation can use equity instead of postflop EV;
- external solver validation can be skipped.

No changes were promoted to `main`.

## Next strict phase

Phase B: validate solver mechanics independently of poker hand evaluation.

1. Add a synthetic zero-sum normal-form game with an analytically known equilibrium (first target: Matching Pennies, equilibrium 50/50 and value 0).
2. Drive the existing `RegretTable` update/average machinery with exact matrix action values.
3. Measure average-strategy error, best-response gaps and NashConv.
4. Add at least one asymmetric analytically known 2x2 game after Matching Pennies, so a hard-coded tendency toward 50/50 cannot pass accidentally.
5. Only after synthetic games pass should we design canonical exact-payoff precomputation for broad/full 1,326-combo supports.

Recovery rule: if a future chat starts after this checkpoint, read `docs/PROJECT_CHECKPOINT.md`, this file, `docs/SOLVER_EXPERIMENT_STATUS.md`, and the latest Rust CI before continuing.
