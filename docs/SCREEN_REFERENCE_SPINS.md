# Spin screenshot reference analysis

These screenshots are a **secondary reference** for UI/tree structure and aggregate strategy cross-checks. They are not promoted to primary solver truth unless source/provenance is independently verified.

## What the screenshots prove about the desired interaction model

- 13x13 hand matrix with mixed-action cells shown as colored splits.
- Aggregate action percentages shown under the matrix.
- Decision tree rendered as position cards (BTN / SB / BB) with current stack and legal actions.
- Current decision node highlighted.
- Previous actions remain visible, so each chart is tied to an exact action history.
- Exact sizings are part of the node: e.g. `Raise 2`, `Raise 3`, `All In 15`.
- Stack presets are explicit and include 2, 4, 6, 8, 10, 12, 15, 17, 20, 23, 25 BB for 3-player classic mode.
- Heads-up states also exist after elimination; these must be modeled separately from 3-max.

## Extracted screenshot nodes

### 15 BB — BTN first-in

Visible aggregate frequencies:

- Fold: 67.22%
- Raise 2: 25.41%
- All In 15: 7.36%

This is strongly consistent with the separate 15 BB BTN first-in cross-check already recorded in the audit (minor rounding/source differences are expected and must not be silently merged).

### 15 BB — SB facing BTN Raise 2

Visible action history / node:

- BTN: Raise 2
- Hero: SB 15 BB

Visible aggregate frequencies:

- Fold: 78.31%
- Call: 2.41%
- All In 15: 19.27%

Important structural observation: there is no generic `3bet` button at this node; the available response is explicitly tied to BTN's exact open size.

### 15 BB — BB facing BTN Raise 2 + SB Call

Visible action history / node:

- BTN: Raise 2
- SB: Call
- Hero: BB 15 BB

Visible aggregate frequencies:

- Fold: 59.11%
- Call: 22.19%
- All In 15: 18.70%

This confirms a critical branch that the old GitHub `main/index.html` cannot represent at all: BB's decision after a BTN open and SB flat.

### 15 BB — BB facing an SB raise branch after BTN Fold

Visible action tree indicates:

- BTN: Fold
- SB: Raise 3
- Hero: BB 15 BB

Visible aggregate frequencies:

- Fold: 47.12%
- Call: 33.85%
- All In 15: 19.01%

This must remain a distinct chart from BB vs BTN Raise 2 and BB vs SB limp/jam branches.

### 2 BB — heads-up BTN first-in

Only two player cards are visible (BTN / SB), indicating heads-up mode after elimination rather than a 3-max node.

Visible aggregate frequencies:

- Fold: 57.16%
- Call: 0.03%
- All In 2: 42.82%

The near-zero `Call` action is visually present. Because the exact semantic context is not fully visible, this screenshot is stored only as a HU-reference checkpoint, not a production chart.

## UI implications for poker-spin-gto

The target UX should follow this model:

1. User selects only the effective-stack preset in the main flow.
2. Position/action cards dynamically expose only legal next actions.
3. Selecting an action advances the tree and immediately swaps the 13x13 chart.
4. Previous actions stay visible so there is no ambiguity about chart context.
5. The matrix supports mixed frequencies inside a cell.
6. Aggregate action frequencies are shown below the matrix.
7. 3-max and HU are distinct trees/modes.
8. Unknown / unverified nodes must display `NO VERIFIED CHART` rather than falling back to a generic range.

## Data-key implication

A chart cannot be addressed by `stack + position + action` alone.

Minimum key:

`mode -> effective_stack -> action_history -> hero_position -> available_actions -> hand -> frequencies`

Example:

`3max -> 15bb -> BTN_RAISE_2 + SB_CALL -> BB -> {fold, call, jam} -> A5s -> frequencies`

## Status

- Use screenshots as **UI/tree reference**: APPROVED.
- Use visible aggregate percentages as **cross-check anchors**: APPROVED_WITH_SOURCE_NOTE.
- Use individual hand-color boundaries as exact production data: NOT APPROVED without independent solver/source verification.
