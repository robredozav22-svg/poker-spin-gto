# GTO Wizard 15bb Spin — Public Source Audit

Date: 2026-09-07
Status: `RESEARCH_ONLY`
Purpose: separate verified public Wizard tree facts from screenshot references and unknown/unpublished exact tree details.

## Sources checked

Official GTO Wizard material:

1. `New Spins solutions, Study Plans & EV Comparison`
   - https://blog.gtowizard.com/new-spins-solutions-study-plans-and-ev-comparison/
2. `Introducing GTO Wizard AI Heads Up Preflop Solver`
   - https://blog.gtowizard.com/introducing-gto-wizard-ai-heads-up-preflop-solver/
3. `Crush Calling Stations With These 3 Solver-Approved Strategies`
   - https://blog.gtowizard.com/crush-calling-stations-with-these-3-solver-approved-strategies/
4. Historical solution status pages are background only because trees/accuracy were later updated.

## Public facts we may treat as cross-checked

### Solution families are distinct

Wizard publicly distinguishes solution/tree families such as:
- Research;
- General;
- Simple;
- older Basic/legacy/Test/complex variants depending on generation.

Research solutions contain richer preflop sizing choices and are used to study optimal sizing. General/Simple trees are derived/simplified for study and postflop use.

Therefore:
**tree family/profile MUST be part of strategy identity.**
A generic key such as `15bb BTN first in` is insufficient.

### Multiple SB BvB opening structures exist

Wizard publicly states users can select SB opening structures/sizes including variants around:
- 2x;
- 2.25x;
- 2.5x;
- larger options in newer solution sets;
- raise-only variants;
- limp-enabled structures;
- GTO-selected sizing depending on stack depth.

Therefore there is no single universal Wizard SB 15bb tree.

### A public 15bb SB-vs-BB no-limp spot exists

Wizard publicly shows/discusses:
- 3-handed Spin;
- 15bb effective;
- BTN folds;
- SB acts versus BB;
- no-limp configuration.

This supports the existence/context of that specific node family.

It does NOT, by itself, reveal the entire preflop response tree, every permitted raise size, or all 1,326-combo strategy frequencies.

### Modern Spin solution accuracy is publicly reported as high

Wizard publicly reports modern Spin solutions solved to roughly `0.1%-0.017% of pot` depending on solution.

This is metadata/background only.
It MUST NOT be compared numerically to our NashConv unless both are re-evaluated under the same independent metric/tree/evaluator.

## What public sources DO NOT establish completely

The public pages checked do not provide a machine-readable complete 15bb symmetric 3-max tree containing every:
- BTN first-in sizing;
- SB response sizing after every BTN action;
- BB response sizing after every BTN/SB sequence;
- BTN response to SB/BB re-raises/jams;
- SB response after BB actions where action reopens;
- limp subtrees;
- exact minraise/reraise rules at every branch;
- all-in/call response continuations;
- postflop continuation trees attached to every non-all-in terminal;
- 1,326 physical-combo frequencies for every node.

Therefore the complete public 15bb profile remains:
`MISSING_EXACT` / incomplete for direct exact-tree reconstruction.

## Relationship to supplied screenshots

User-supplied screenshots expose useful 15bb action paths such as:
- BTN Fold / Raise 2 / Jam 15;
- SB response to BTN Raise 2;
- BB response after BTN Raise 2 + SB Call;
- BB response after BTN Fold + SB Raise 3.

Those are stored as `ScreenReference`.

Even where screenshot action structure is compatible with Wizard concepts, screenshots are not upgraded to `VerifiedExactTree` without exact source/tree provenance.

## Current source classification

- Wizard public family/sizing facts: `CROSS_CHECKED`
- Wizard public 15bb SB-vs-BB no-limp context: `CROSS_CHECKED`
- supplied 15bb action-path screenshots: `SCREEN_REFERENCE`
- complete Wizard 15bb 3-max tree: `MISSING_EXACT`
- exact Wizard 1,326-combo strategies for all required nodes: `MISSING_EXACT`

## Required path to a frozen exact external benchmark tree

Before using Wizard as an apples-to-apples strategy benchmark, obtain one exact identifiable tree by one of these routes:

1. exact export/API/data access that preserves every action and sizing;
2. systematic authenticated capture of the complete solution tree with reproducible identifiers;
3. another primary-source export whose tree is provably identical to the Wizard solution being compared.

Then:
- assign a stable immutable `tree_profile_id`;
- build a `Complete` `TreeCatalog`;
- checksum/version the captured tree artifact;
- verify every child edge;
- only then compare strategy frequencies and independent exploitability.

## Rule

Never fill an unpublished Wizard branch by inference, interpolation, screenshots of another tree, or a generic poker rule and then label the result `Wizard exact`.
