# Exact GTO Wizard node import

## Why this path

Do not reconstruct solver frequencies from screenshots.

GTO Wizard Study Mode supports copying:

1. the full player range at a selected node;
2. the range assigned to each individual action at that node.

The copied text is standard weighted Pio/GTO+/UPI range text. For a private combo `c`:

`strategy(action | c) = action_range_weight(c) / full_node_range_weight(c)`

Therefore the complete set of action-specific copied ranges can reconstruct the exact conditional strategy for every reachable combo while preserving mixed frequencies.

## Required capture bundle

For one exact preflop node store:

- format / payout profile / effective stack;
- hero position;
- complete previous public action history with exact sizes;
- GTO Wizard solution family and exact node identity;
- declared legal actions with exact sizes;
- `full_range_upi` copied at the node;
- one `action_range_upi` for every legal action.

The importer must reject the bundle unless, combo by combo:

- full range weight is valid;
- every action weight is valid and does not exceed full range weight;
- the sum of action weights equals the full range weight within tolerance;
- every reachable combo receives a complete conditional strategy;
- preflop suit-symmetric combos aggregate consistently into 169 classes;
- the resulting 169 cells each sum to 1;
- aggregate action frequencies can be recomputed from the source ranges.

## Source limitations

A public GTO Wizard article that visually shows a strategy matrix is useful as a node/context cross-check only. It is not sufficient for `VERIFIED_EXACT` because screenshot colours do not provide a lossless source for exact mixed frequencies.

The importer output must pass `scripts/validate-exact-spin-nodes.mjs` before the app can display it as exact strategy data.
