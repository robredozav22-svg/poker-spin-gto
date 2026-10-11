# Poker Lab architecture donor notes

Repository reviewed: `AHTOOOXA/poker-charts`.

Use it as an architecture/UI donor only. Do not import its existing strategy ranges as Spin data.

## Useful concepts

### HandGrid

The project already models a 13x13 matrix and can render a hand cell as multiple horizontal action-frequency bands. This is exactly the visual primitive needed for Spin mixed strategies.

### Weighted cell model

Its data model supports a cell with action weights instead of forcing one pure action. Our Spin model should extend this concept to the vocabulary:

`FOLD / CHECK / LIMP / CALL / RAISE / JAM`

and keep explicit frequencies.

### Separation

Range data are separated from rendering. Preserve that principle: the UI must never contain the authoritative chart arrays inline as the current simplified `main/index.html` does.

### Validation stack

Poker Lab uses TypeScript/Zod/Vitest-era tooling. We can migrate toward that structure after P0 chart recovery. The first audit gate in this branch intentionally stays dependency-free (`node scripts/validate-chart-data.mjs`) so chart validation is not blocked by a frontend rewrite.

## What not to copy

- cash/other-format range packs;
- existing scenario taxonomy as-is;
- provider names as evidence of data availability;
- `gtowizard-gg-rc.ts` as a data source (it is currently an empty TODO).

## Our required key is richer

A Spin chart cannot be keyed only by hero/scenario/villain. It needs effective stack and the actual action path/sizing, and later may need payout/asymmetric-vector context.
