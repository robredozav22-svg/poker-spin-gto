# V45 UX Specification

## Product modes

### REVIEW — default
Fast chart lookup for off-table video/hand-history review.

Primary flow:
1. choose `EFF BB`;
2. tap actions in the BTN/SB/BB tree;
3. chart updates instantly;
4. use BACK/RECENT/FAVORITE when moving among nearby nodes.

No setup wizard. No generic position dropdown. No hidden chart fallback.

### TRAIN
One decision at a time against an approved strategy solution.

After action:
- show whether action is solver-approved;
- show its frequency;
- show primary alternative(s);
- show EV loss only when the underlying model actually provides reliable EV;
- allow immediate jump to full chart;
- save mistakes/close decisions for retry.

### LAB
Research and audit layer for solver/source diagnostics.

Contains:
- node ID;
- source/provenance;
- solver assumptions;
- convergence/exploitability status;
- chart validation state;
- aggregate deltas vs independent reference anchors.

LAB is not part of normal fast review.

## Main REVIEW screen hierarchy

1. Compact header: SPINS + mode switch.
2. Effective stack strip.
3. 13x13 matrix — largest visual object.
4. Aggregate action bar/frequencies.
5. BTN / SB / BB action-tree cards.
6. Bottom action row for current actor on small screens.
7. Back / recent / favorite controls.
8. Source status as small badge; tap for details.

## Interaction rules

- Stack change resets action history unless a compatible node exists and is explicitly restored.
- Tapping an action appends one immutable history event.
- Current actor is derived from the tree, not selected manually.
- Previous seats collapse to their selected action but remain visible.
- Current seat is visually emphasized.
- Illegal actions are not displayed rather than displayed disabled when possible.
- `BACK` removes exactly one action.
- `FORWARD` restores a previously undone action until a new branch is chosen.
- Long action histories may scroll horizontally, but the active decision and chart must remain visible.

## Stack control

Default quick presets:
`2 / 4 / 6 / 8 / 10 / 12 / 15 / 17 / 20 / 23 / 25`

Rules:
- effective stack only in REVIEW;
- step 0.5 BB advanced control can be added without cluttering presets;
- no silent interpolation if a solution does not exist for the exact stack;
- asymmetric-stack research belongs in LAB/advanced mode until we have validated solution support.

## Chart matrix

- conventional 13x13 Hold'em ordering;
- pairs diagonal;
- suited top-right;
- offsuit bottom-left;
- cell can show multiple action slices;
- hand label always readable above slices;
- tap/press cell opens exact hand frequencies;
- optionally show EVs only when verified;
- no approximate visual boundaries under a verified/GTO label.

## Actions and colors

Action semantics are data-driven, not hardcoded to one global set.

Examples:
- Fold
- Check
- Limp
- Call
- Raise 2
- Raise 2.5
- Raise 3
- small 3bet
- Jam

Color mapping must remain consistent across a session, but exact palette is a UI concern and not part of solver data.

## Tree model

Canonical route key:
`mode -> payout_profile -> effective_stack -> action_history -> hero_position`

Example:
`3max/WTA/15/BTN_R2/SB_CALL/BB`

The tree definition determines legal actions and exact sizes. Charts cannot be keyed by only stack + position.

## Source states

Visible compact badges:
- `EXACT`
- `CROSS-CHECK`
- `APPROX`
- `MISSING`

Detailed mapping remains:
- VERIFIED_EXACT
- CROSS_CHECKED
- SOLVER_APPROX
- MISSING_EXACT
- INVALID_FOR_STRATEGY

No `APPROX` or screenshot-only chart may masquerade as `EXACT`.

## Recent spots

Keep a small MRU list, e.g. last 5–10 nodes.

Example:
- 15 / BTN R2 / SB Call / BB
- 15 / BTN R2 / SB Fold / BB
- 15 / BTN Fold / SB R3 / BB

One tap restores the entire route.

## Favorites

Favorite stores the canonical node ID, not screen coordinates or labels.
This makes favorites stable across UI redesigns.

## TRAIN grading

A decision can have these outcomes:
- PRIMARY — high-frequency solver action;
- VALID_MIX — solver-approved lower-frequency action;
- LOW_FREQ — legal but very low solver frequency;
- ERROR — action frequency effectively zero under the approved strategy.

Thresholds must be configurable and should not rewrite the underlying frequencies.

## Simplified strategy layer

`SIMPLIFIED` is generated/stored separately from `EXACT`.
It may convert mixed strategies into easier study rules but must never overwrite solver output.

Example display toggle:
`EXACT | SIMPLE | EXPLOIT`

## Exploit layer

Future profiles:
- GTO baseline
- REG
- FISH
- population/custom node-lock

Exploit strategy must carry its own provenance/model version and never be visually confused with baseline GTO.

## Mobile requirements

- primary action targets >= 44 CSS px high;
- no critical hover-only actions;
- chart must fit width without horizontal scrolling;
- stack presets horizontally scroll if needed;
- action buttons remain thumb-accessible;
- mode/source details use drawers/sheets, not permanent sidebars;
- minimize keyboard/text input.

## Desktop requirements

- same information hierarchy as mobile;
- extra width may expose history/recent/source panes without changing routing behavior;
- keyboard shortcuts allowed: Back, Reset, stack +/- and action number keys.

## Non-goals for V45

- poker-client integration;
- screen/card recognition during active play;
- automatic live assistance;
- generic cash/MTT UI complexity;
- production use of self-generated solver charts before promotion gates pass.

## V45 implementation order

P0:
1. immutable history + BACK/FORWARD;
2. canonical node router;
3. correct mixed 13x13 renderer;
4. source-state badge and MISSING behavior;
5. recent nodes;
6. mobile layout.

P1:
7. favorites;
8. REVIEW/TRAIN switch;
9. mixed-action trainer grading;
10. full-chart jump from trainer;
11. local progress/error queue.

P2:
12. SIMPLE layer;
13. exploit profiles;
14. adaptive weak-spot drills;
15. LAB diagnostics UI.
