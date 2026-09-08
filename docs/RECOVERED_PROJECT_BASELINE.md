# Recovered project baseline — V43

This file records the latest recovered work from the project history so the GitHub rewrite does not restart from the simplified `main/index.html`.

## Baseline

**Application baseline:** `GG_PokerOK_Spin_PREFLOP_V43_EFFECTIVE_STACK_ONLY.html`  
**Data audits to preserve:** V41 RFI re-audit + V42 all-ranges sanity audit  
**Tree/engine guarantees to preserve:** V37 legal preflop state machine + V40 all-in/asymmetric legality fixes  
**Use case:** replay / study of recorded hands, not live assistance.

## V43 product rule

The visible workflow uses one selector: `EFF BB`.

Recovered V43 checks:
- single effective-stack selector;
- separate BTN/SB/BB stack controls removed;
- 0.5bb increments supported;
- one-screen layout;
- hero-turn guard;
- action-tree and chart lookup use the selected effective stack.

Tradeoff: one effective stack cannot reconstruct a strongly asymmetric three-player solver state exactly. If an exact asymmetric vector is later imported, it must be handled as a separate expert/data mode, not silently inferred from `EFF BB`.

## V41 source repair that must not regress

V40 incorrectly removed the solver limp component from some SB first-in spots while still presenting the result as source-backed. V41 rebuilt all ten public 3-max opening anchors (BTN/SB at 8/10/15/20/25bb) under hard source constraints:
- published aggregate action-combo totals must match;
- pure-open source hands are played 100%;
- 50%+ source-envelope hands are played at least 50%;
- outside-envelope hands remain below 50%;
- mixed cells remain mixed and display frequencies.

Exact aggregate combo targets are stored in `data/source-verified-anchors.json`.

## V42 parser/sanity repair that must not regress

V42 found a systematic shorthand parsing issue in call-vs-jam ranges: naive handling of forms such as `A2o+`, `K6o+`, `Q9o+` could produce an offsuit CALL while the same suited hand was FOLD.

Required invariant:

`if XY offsuit is CALL, XY suited cannot be FOLD`

V42 rebuilt the public call-vs-open-shove families and reported zero suited-over-offsuit violations for all audited anchors.

V42 also retained the V41 RFI constraints and explicitly warned that unusual non-monotonic source bands (for example SB 15bb Axs bands) must not be “fixed” by intuition. Source evidence wins; mixed source bands must not be converted into unjustified pure actions.

## Tree guarantees recovered from V37/V40

The UI is not a collection of generic `RFI / 3bet / defend` buttons. It is a legal preflop state machine.

Required actions and transitions:
- Fold / Limp / Call / Check;
- non-all-in Raise;
- All-in;
- returns after raises, squeezes, 3-bets, 4-bets and further legal re-raises;
- short-all-in and action-reopen rules;
- chart is shown only when `actor == Hero`;
- otherwise the UI shows the opponent whose action is next;
- BACK can restore any previous decision point.

Known critical nodes include, among others:
- `BTN raise -> SB jam -> BB decision`;
- `BTN limp -> SB jam -> BB decision`;
- BTN response after SB/BB aggression;
- BB responses to BTN/SB raise, limp and shove.

A technically valid branch may exist without an approved exact chart. In that case the product must say that the branch exists but the exact chart source is missing. It must not substitute a different node.

## Data grades

- `A / VERIFIED_EXACT`: explicit published structured range / exact approved export / matching Nash source.
- `B / SOURCE_CONSTRAINED`: public source-backed aggregate/envelope with reconstructed mixed per-hand allocation under explicit constraints. This is **not** solver-exact per-hand frequency.
- `M / MODEL_REFERENCE`: model, asymmetric proxy, nearest-depth proxy, or complex response without exact approved per-hand export.
- `G / EXPLOIT`: exploit / legacy Gold / population adjustment. Never presented as the GTO baseline.
- `MISSING_EXACT`: legal node exists, exact approved chart not available.

## Architecture donor: Poker Lab

`AHTOOOXA/poker-charts` is useful as a UI/data-architecture donor, not as a Spin data source.

Useful ideas to reuse:
- 13x13 HandGrid;
- a cell with action-frequency weights;
- multi-action color bands inside one hand cell;
- range data separated from rendering;
- TypeScript/Zod/Vitest style validation.

Do **not** import its existing cash/other-format ranges as Spin ranges. Its `gtowizard-gg-rc.ts` provider is currently an empty TODO.

## Primary verification hierarchy going forward

1. Approved exact export from the selected Spin solver tree (GTO Wizard Spin solution is the current primary reference).
2. PokerStars Learn current Spin course for explicit public call/Nash ranges and structural cross-checks.
3. Public solved Spin opening sources for aggregate/envelope cross-checks.
4. GGPoker official pages for format/rules sanity only.
5. Exploit/legacy charts only as a visibly separate layer.

GTO Wizard currently documents regular Spin study depths up to 33bb, Spin+Ante up to 25bb, multiple SB opening-size trees, and 600+ asymmetric-stack situations. This expands available verification coverage but does not authorize us to invent or copy an unexported exact tree.

## Non-negotiable rule

**No guessed chart enters the GTO baseline.**

A beautiful wrong matrix is still wrong — just with better typography.
