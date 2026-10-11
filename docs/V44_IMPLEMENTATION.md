# V44 implementation status

## Implemented

- Replaced the invalid one-file range UI in `chat-aligned-v2`.
- Added single effective-stack selector with presets 2/4/6/8/10/12/15/17/20/23/25 BB.
- Added BTN/SB/BB decision-tree cards modeled after the supplied Spin screenshots.
- Previous actions remain visible by collapsing acted seats to their chosen action.
- Exact sizing labels are part of the tree (`Raise 2`, `Raise 3`, `All In X`).
- Added separate HU toggle/tree.
- Added 13x13 matrix shell ready for mixed-frequency rendering.
- Added visible aggregate percentages for screenshot cross-check nodes only.
- Added hard `NO VERIFIED HAND FREQUENCIES` overlay so approximate individual-hand boundaries are never shown as GTO.
- Removed the old embedded invalid `DATA` from the working branch.

## Reference nodes currently wired

- 15bb BTN first-in.
- 15bb SB vs BTN Raise 2.
- 15bb BB vs BTN Raise 2 + SB Call.
- 15bb BB vs BTN Fold + SB Raise 3.
- 2bb HU BTN screenshot checkpoint.

These are UI/aggregate cross-check nodes only. They are **not** yet `VERIFIED_EXACT` hand matrices.

## Next P0

1. Import V42 explicit 169-hand call-vs-jam matrices.
2. Import V41 audited 169-hand weighted RFI matrices.
3. Convert them to the canonical node schema.
4. Run `scripts/validate-chart-data.mjs`.
5. Enable real mixed-cell painting only for nodes that pass validation and source checks.
6. Add unit tests for node routing and exact-history isolation.

## Merge policy

Do not merge into `main` until at least one complete stack (recommended: 15bb) has all required first-in and response branches loaded as verified data and the validator passes.