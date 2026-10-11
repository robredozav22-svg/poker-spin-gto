# Immutable Checkpoint — Accuracy / Superiority Gates

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`

## Objective

The project target is not merely to resemble GTO Wizard. The target is to produce Spin solutions with stricter measurable acceptance criteria and stronger reproducibility/provenance. However, no claim that the project is more accurate than GTO Wizard is authorized until both artifacts pass an apples-to-apples independent benchmark.

## Public external target context

GTO Wizard publicly reports modern Spin & Go solution accuracy in a range around 0.1%-0.017% of pot depending on solution and provides multiple solution families / sizing trees.

Those vendor-reported numbers are background targets only. They are not assumed to be definitionally identical to our NashConv or exploitability metrics.

## Internal acceptance target

Implemented in `solver-rs/src/accuracy_policy.rs`:

`spin-verified-exact-v1`
- normalized convergence/exploitability target <= 0.0001 of pot = 0.01% pot;
- independent validation required;
- deterministic repeat required for internal solver runs.

This is a target, not a claim that a complete Spin solution currently satisfies it.

## Exact strategy promotion gate

`StrategyRecord::VerifiedExact` now requires:
- VERIFIED_EXACT tree profile;
- Complete tree catalog;
- verified postflop continuation coverage when non-all-in edges exist;
- solver-run evidence;
- solver evidence tree profile matches strategy tree profile;
- strategy solver profile matches solver evidence profile;
- full 1,326 physical-combo strategy coverage;
- only legal actions for the exact node;
- valid action probabilities summing to 1.

Pure push/fold exact trees do not require a continuation registry when no postflop continuation edge exists.

## Solver-run provenance

Implemented in `solver-rs/src/solver_evidence.rs`.

Internal measured solve promotion evidence requires:
- acceptance policy ID;
- iterations;
- measured NashConv;
- deterministic-repeat PASS;
- exact payoff manifest provenance;
- convergence gate PASS.

External solver export evidence requires:
- accuracy evidence;
- independent validation evidence;
- matching exact tree profile;
- convergence/acceptance gate PASS.

## Strategy comparison diagnostic

Implemented in `solver-rs/src/strategy_compare.rs`.

It compares physical-combo action frequencies under identical canonical node identity.

It explicitly does NOT interpret frequency similarity as accuracy because strategically indifferent actions can mix differently at near-identical EV.

## Superiority claim gate

Implemented in `solver-rs/src/benchmark_claim.rs`.

A claim that our artifact is more accurate than an external reference is allowed only when both candidate and reference have:
- identical exact tree profile;
- identical independent evaluator profile;
- identical evaluator source;
- identical evaluated-state coverage;
- distinct immutable artifact checksums;
- at least 3 benchmark repetitions;
- a directly comparable normalized exploitability metric.

Candidate must:
- have lower measured exploitability;
- beat reference by a pre-declared absolute margin;
- pass the project's absolute accuracy ceiling if configured.

Vendor-reported accuracy alone cannot pass this gate.

Protocol: `docs/WIZARD_BENCHMARK_PROTOCOL.md`.

## Independent evaluator work

Added `solver-rs/src/independent_exact_eval.rs`.

Purpose:
- evaluate a frozen strategy independently of regret/CFR state;
- use immutable persisted exact payoffs only;
- compute current value, both best responses and NashConv;
- fail closed on missing exact payoff;
- normalize NashConv to a reference pot for same-metric benchmarking.

Added `solver-rs/src/bin/independent_exact_eval_smoke.rs`.

Required smoke:
- solve a bounded exact restricted game;
- evaluate frozen average strategy through the solver's existing evaluator;
- independently evaluate the same strategy from exact payoff lookup;
- require equality within 1e-12 for game value, both best responses, both BR gains, NashConv and joint mass.

Current heavy CI run to inspect next:
- `34132625299`
- head `cbc54aae66755958a8b1d3d4dbe623643a97d181`
- includes the new independent exact evaluator cross-check.

## Next step

1. Require run `34132625299` SUCCESS.
2. Read exact independent-evaluator smoke output.
3. If evaluator agreement fails, stop and fix before any new tree/strategy work.
4. If it passes, treat independent best-response evaluation as the basis for future apples-to-apples external benchmarks.
5. Continue obtaining/fixing one exact 15bb Spin tree profile before attempting a real multi-action strategy solve.
