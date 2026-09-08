# Immutable Checkpoint — Phase D Tree + Provenance Foundation

Date: 2026-09-07
Branch: `chat-aligned-v2`
Status: `RESEARCH_ONLY`
Confirmed full CI: run `34122294749`, head `ff0a4ee9aba12843654e1ca296cc6a94a95a1287`, SUCCESS.

## Purpose

Replace context-free/generic preflop action labels with a canonical, fail-closed Spin preflop tree identity before importing any strategy charts. Preserve the validated push/fold solver subsystem while creating a separate architecture for real multi-action Spin nodes.

## Canonical node identity

`PreflopNodeKey` now contains:
- format;
- payout profile;
- tree profile ID;
- effective stack in fixed-point hundredths of a BB;
- current actor;
- complete ordered action history including actor and exact action size.

Consequences:
- 2.00bb and 2.50bb opens are different nodes;
- BTN raise -> SB call -> BB is different from BTN fold -> SB raise -> BB;
- General/Research/Simple/custom solution trees cannot collide merely because stack/history look similar;
- `RaiseTo(stack)` is rejected; all-in action must be canonical `JamTo(stack)`.

## Action / continuation model

Supported preflop actions:
- Fold;
- Check;
- LimpTo;
- CallTo;
- RaiseTo;
- JamTo.

Continuation contracts:
- `ChildDecision`;
- `ExactFoldSettlement`;
- `ExactAllInShowdown`;
- `RequiresPostflopEv`;
- `Unresolved`.

`ChildDecision` carries an explicit `next_actor`; child keys are deterministically generated as:
`parent history + parent actor/action + next actor`.

Important multiway semantics now covered:
- BTN Fold in 3-max may pass action to SB instead of ending the hand;
- BB Fold after BTN raise + SB call may leave BTN/SB going postflop -> `RequiresPostflopEv`;
- an all-in CallTo(stack) may create exact showdown or pass action to a third player;
- non-all-in raise cannot be valued as exact showdown;
- jam cannot be substituted with postflop continuation EV.

## Tree profiles

`TreeProfileSpec` records:
- stable profile ID;
- family;
- format;
- payout profile;
- verification state;
- source ID;
- declared sizing families;
- notes.

A node cannot claim `VerifiedExactTree` under a non-exact profile.

Current registered profile:
`screen-reference-spins-15bb-v1`
- family: ScreenReference;
- verification: ScreenReference;
- source: supplied screenshot reference documentation;
- declared visible sizing families: Raise2x, Raise3x, Jam.

This profile is explicitly NOT an exact solver tree.

## 15bb screenshot reference tree

Machine-readable reference-only nodes now include:
1. BTN first-in: Fold / RaiseTo 2.00 / JamTo 15.00 -> next actor SB;
2. SB vs BTN RaiseTo 2.00: Fold / CallTo 2.00 / JamTo 15.00 -> next actor BB;
3. BB vs BTN RaiseTo 2.00 + SB CallTo 2.00:
   - Fold -> postflop BTN/SB continuation;
   - Call -> postflop 3-way continuation;
   - Jam -> BTN response decision;
4. BB vs BTN Fold + SB RaiseTo 3.00:
   - Fold -> exact fold settlement;
   - Call -> heads-up postflop continuation;
   - Jam -> SB response decision.

No per-hand frequencies are stored in this reference tree.

## Tree catalog

`TreeCatalog` enforces:
- unique node keys;
- profile/node consistency;
- Complete vs PartialReference state;
- a `VerifiedExactTree` profile cannot use a partial catalog;
- Complete catalogs must contain every derived `ChildDecision` node.

## Strategy provenance

`StrategyRecord` is keyed to a catalog node and stores physical-combo action frequencies.

For `VerifiedExact` promotion it requires:
- `VerifiedExactTree` tree profile;
- Complete tree catalog;
- continuation registry supplied;
- all required postflop continuation edges have exact-coverage approval;
- all 1,326 physical Hold'em combos present;
- each row sums to 1 within strict tolerance;
- no duplicate combo/action rows;
- every action used by a strategy row is legal at that exact node.

This intentionally prevents direct promotion of a 169-class shorthand chart without an explicit physical-combo/suit-symmetry path.

## Postflop continuation guard

`ContinuationRegistry` exists specifically to stop non-all-in paths from silently using showdown equity.

A `RequiresPostflopEv` edge is treated as missing unless an explicit continuation source/model entry is registered at the required verification level.

Current project has NO measured postflop continuation artifact, so non-all-in exact promotion remains blocked by design.

## Recovery of old V41/V42

Search/recovery context did NOT yield the actual prior V41/V42 169-hand tables, only summaries that they had existed/audits had been performed.

Known old assets such as GoldCharts are exploit/fish-oriented, and prior V35 contained FIELD50 approximations. They are not accepted as GTO baseline and were not imported.

Do not reconstruct V41/V42 by memory or screenshot inference.

## Full regression gate

CI run `34122294749` passed together:
- all current Rust unit/schema tests;
- deterministic sparse benchmark;
- exact HU equity;
- exact restricted solver;
- exact 3-way equity and terminal leaves;
- sampled-vs-exact cross-check;
- matrix-game validation;
- canonical payoff census;
- payoff lookup/build/manifest tests;
- generation economics;
- persisted leaf equivalence;
- 6x6, 12x12, 24x24 persisted exact restricted benchmarks;
- parallel payoff builder equivalence.

## Next strict work

1. Split fast Phase D schema CI from expensive full mathematical regression so tree iteration does not recompute 24x24 every commit.
2. Strengthen continuation evidence with actual artifact completeness/integrity proof before it can ever unlock exact promotion.
3. Add explicit solver-run/convergence provenance to strategy promotion.
4. Obtain one exact primary-source 15bb Spin tree profile including all legal sizes/branches.
5. Build a Complete tree catalog for that exact profile.
6. Only then import/solve physical-combo strategies.
7. Aggregate 1326 -> 169 only after suit-dispersion audit.
8. Do not render colored playable charts until the exact-data gate for that node passes.
