# Chart data policy

This directory is strategy data, not UI decoration.

## Never ingest source shorthand directly into production

V42 found that naive parsing of source strings such as `A2o+`, `K6o+`, `Q9o+` could generate logically broken call charts. When an explicit 169-hand source table is available, store/import that table rather than repeatedly parsing shorthand text.

`range_text` values in `source-verified-anchors.json` are audit labels for humans, **not parser input**.

## Ingestion order

1. Identify the exact poker node: format, effective stack, hero, full previous action history and sizing.
2. Record source URL/export identifier and source depth/vector.
3. Import explicit 169-hand action/frequency data when available.
4. Assign source grade.
5. Run `node scripts/validate-chart-data.mjs`.
6. Only approved grades may be exposed by the GTO baseline UI.

## Missing data

A missing exact response node stays missing. Do not borrow a chart from a different branch simply because the actors or stack look similar.

## Mixed frequencies

Do not collapse source-constrained mixed cells to a pure action for cosmetic simplicity. A separate `SIMPLIFIED` strategy may be created later, but it must be labelled separately from the GTO/source-backed baseline.
