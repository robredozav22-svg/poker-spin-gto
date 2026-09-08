# V43 migration checklist

Do not replace `main/index.html` until the following gates are met.

## P0 — charts and source safety

- [x] Recover V41 RFI aggregate targets.
- [x] Recover V42 call-vs-jam parser/sanity invariants.
- [x] Recover V43 effective-stack-only UX rule.
- [x] Freeze audited aggregate anchors in machine-readable JSON.
- [x] Add chart validator.
- [x] Add CI audit workflow.
- [ ] Import V42 explicit 169-hand public call-vs-jam tables as data files.
- [ ] Import V41 source-constrained 169-hand RFI matrices for 8/10/15/20/25bb BTN/SB.
- [ ] Verify every imported matrix against V41/V42 audit totals/invariants.
- [ ] Build a coverage report for every legal Hero response family: A/B/M/MISSING_EXACT.

## P1 — architecture

- [ ] Split the monolithic HTML into strategy data, state machine, HandGrid renderer and UI shell.
- [ ] Reuse the useful Poker Lab concepts: 169-hand matrix + action-frequency cell model.
- [ ] Keep the recovered V37 legal action tree instead of generic RFI/3bet buttons.
- [ ] Preserve one visible `EFF BB` selector with 0.5bb steps.
- [ ] Preserve one-screen replay workflow and Hero-turn guard.
- [ ] Display source grade and exact/proxy status on every chart.

## P2 — later layers

- [ ] Add `SIMPLIFIED` training strategy separately from source-backed baseline.
- [ ] Add REG/FISH/population exploit profiles separately (`G`/exploit layer).
- [ ] Add optional expert asymmetric-stack source mode only when exact vector data are available.
- [ ] Deploy only after chart and branch regression tests pass.

## Merge gate

`main` stays unchanged until P0 chart imports have passed automated validation and sampled hand-by-hand comparison against recovered V41/V42 data.
