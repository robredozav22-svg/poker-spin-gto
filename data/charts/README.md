# Canonical exact Spin chart nodes

This directory is reserved for machine-readable, source-bound strategy nodes.

A file may be labelled `VERIFIED_EXACT` only when it contains the full 169-hand matrix for one exact public decision node. No shorthand ranges, inferred folds, stack interpolation, screenshot recolouring, or missing-hand defaults are allowed.

## Required identity

Every exact node must identify:

- `schema_version: exact-spin-node-v1`
- format and payout profile
- exact effective stack
- hero position
- complete previous public action history
- exact legal action set and sizes
- primary source/provider and source node identity
- extraction method
- all 169 starting-hand classes
- per-hand action probabilities that sum to 1

Action IDs encode sizing, for example `RAISE_TO_2BB`, `RAISE_TO_3BB`, `JAM_TO_15BB`. A generic `RAISE` frequency without its size is not exact-node data.

## Verification rule

`VERIFIED_EXACT` is fail-closed. The importer/validator rejects a node if:

1. any of the 169 hand classes is missing or duplicated;
2. an unknown hand is present;
3. any per-hand frequency is outside `[0,1]`;
4. a hand's action frequencies do not sum to 1;
5. a frequency references an action not declared by the node;
6. an action requires a size but its target size is missing;
7. the action history omits an actor/action or required size;
8. source provider/node identity/extraction method is missing;
9. the node says it was interpolated, reconstructed from screenshot colours, or default-filled;
10. supplied aggregate action frequencies disagree with the 1326-combo-weighted matrix.

The UI must not paint mixed strategy cells from a node until this validation passes.

The existing screenshot reference data remains cross-check material only and must never be promoted by this contract.
