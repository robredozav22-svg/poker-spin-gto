# Source usage compliance — 2026

Date: 2026-09-08

## Principle

Accuracy work must respect each source's terms, access model and rate limits. A technically accessible dataset is not automatically permitted for systematic extraction.

## PreflopRanges.app

Current public terms state that the beta allows chart browsing with browser/IP quotas and prohibit scraping, bulk download, systematic extraction of the chart set, account sharing and quota/rate-limit bypass.

Project rule:

- use only limited public page/spot checks as an independent cross-check;
- do not crawl or bulk-download the 15,000+ viewer dataset;
- do not reverse-engineer or call hidden endpoints for systematic extraction;
- do not bypass browser/IP/account quotas;
- do not republish or redistribute a substantial part of the source dataset;
- keep stored references limited to facts needed for our audit (node identity, action sizes, aggregates and a small number of hand-level spot checks);
- do not use PreflopRanges data as training labels for our solver.

The public source currently does not establish enough payout/ante/solver provenance for `PASS_COMPATIBLE`; therefore it remains cross-check-only until compatibility can be established from permitted public information.

## GTO Wizard

Use normal product-supported study/export/copy functions for exact node capture. Do not reconstruct hidden data or bypass authentication/subscription controls.

A live capture must record:

- solution family;
- exact node/action path;
- effective stacks;
- payout/ante profile;
- legal sizings;
- capture timestamp;
- full node UPI range;
- UPI range for every legal action.

## Independent internal solver

Our solver can act as a promotion-quality cross-check only when it solves the same modeled game as the external primary node. Required identity includes exact tree, stacks, payout/ChipEV, ante, reach state, legal actions/sizings and terminal/continuation model.

A self-generated result cannot validate itself. Promotion-quality internal evidence needs an independent evaluator/best-response path, immutable evidence, a precommitted numerical threshold and a clear separation between solve and audit code.

## Promotion rule

`PROMOTED_2026_EXACT` may use one of two independent proof types:

1. `COMPATIBLE_EXTERNAL_SOURCE` — an independently sourced solution whose full relevant game profile is resolved and matches;
2. `INDEPENDENT_INTERNAL_SOLVER_PROOF` — our independently audited same-tree solution/evaluation evidence.

Public aggregate similarity, screenshot similarity, marketing solver tolerances or a source's newer date are not sufficient.
