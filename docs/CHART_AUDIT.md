# Spin & Go chart audit

Status: active verification. Do not treat the ranges currently embedded in `main/index.html` as source-of-truth.

## Target format

- Game: 3-max Spin & Go / GGPoker Spin & Gold
- Primary use: preflop study / video and hand-history review
- Baseline mode: standard winner-take-all / ChipEV-like 3-max spots
- Stack key: effective stack in big blinds
- Important exception: high multipliers that pay multiple places require a separate ICM profile; they must not silently reuse the WTA baseline.

## Source hierarchy

1. **GTO Wizard Spin & Go solutions** — primary solver reference. GTO Wizard documents Spin solutions across 1-25bb, all spots, multiple sizings and asymmetric stacks; preflop solutions are free. Their published solution status reports high-accuracy Spin solutions.
2. **PokerStars Learn Spin & Go lessons** — independent structural cross-check for effective-stack logic, open sizes and response branches.
3. **PreflopRanges.app Spin & Go solved charts** — extraction/cross-check source for public text ranges and aggregate frequencies at 8/10/15/20/25bb. Do not promote a hand-level mixed strategy to VERIFIED_EXACT until matched to the selected primary solver tree.
4. **GGPoker official format pages/articles** — format sanity check only, not exact GTO range source.

## Current GitHub findings

`main/index.html` is not trustworthy as a strategy source:

- only 8bb and 15bb data blocks are actually present, while UI exposes additional stack buttons;
- many BTN/SB RFI and push/fold arrays expand to nearly/all 169 hand classes;
- many `3bet` arrays are effectively all hands;
- the data model has no previous-action context, so a generic `3bet` cannot distinguish SB vs BTN open, BB vs BTN open, BB vs SB raise, response to limp, response to shove, etc.;
- the renderer is binary raise/fold and cannot represent limp, call, jam, small 3-bet or mixed frequencies.

Therefore existing embedded ranges are marked **INVALID_FOR_STRATEGY**.

## Cross-checked first-in opening baselines

These values are recorded as audit checkpoints, not yet as final hand-level production data.

| Stack | Seat | Aggregate strategy checkpoint | Status |
|---|---|---|---|
| 8bb | BTN | jam 34.3%, fold 65.7% | CROSS_CHECKED |
| 8bb | SB | jam 52.6%, raise 6.0%, limp 6.5%, fold 35.0% | CROSS_CHECKED |
| 10bb | BTN | jam 26.6%, raise 7.5%, fold 65.9% | CROSS_CHECKED |
| 10bb | SB | jam 41.7%, raise 16.5%, limp 0.8%, fold 41.0% | CROSS_CHECKED |
| 15bb | BTN | jam 7.7%, raise 24.4%, fold 67.9% | CROSS_CHECKED |
| 15bb | SB | jam 19.5%, raise 26.5%, limp 16.0%, fold 38.0% | CROSS_CHECKED |
| 20bb | BTN | total played about 34.7%, primarily 2bb raise; exact action split pending primary-tree extraction | PARTIAL |
| 20bb | SB | jam 7.4%, raise 33.9% to 2.5bb, limp 22.8%, fold 35.9% | CROSS_CHECKED |
| 25bb | BTN | raise 38.1% to 2bb, fold 61.9% | CROSS_CHECKED |
| 25bb | SB | jam 1.7%, raise 35.6% to 2.8bb, limp 27.1%, fold 35.6% | CROSS_CHECKED |
| 12bb | BTN/SB | exact selected-tree chart not yet extracted | MISSING_EXACT |

## Public text range checkpoints

### BTN 8bb — pure jam checkpoint

`22+, A2s+, K4s+, Q8s+, J8s+, T7s+, 97s+, 86s+, 76s+, 65s+, A2o+, K9o+, QTo+, JTo+`

### BTN 15bb — pure played checkpoint (action mix still matters)

`22+, A3s+, K5s+, Q7s+, J8s+, T7s+, 97s+, 87s+, A6o+, K9o+, QTo+, JTo+`

### BTN 25bb — pure open-raise checkpoint

`44+, A2s+, K4s+, Q5s+, J6s+, T6s+, 96s+, 85s+, 75s+, 65s+, 54s+, A7o+, A5o, K9o+, Q9o+, JTo+, T9o+`

These are used to detect gross data corruption. They are not a substitute for the full mixed-frequency matrix.

## Response-tree facts independently checked

PokerStars' current Spin & Go material confirms that the correct model must branch by exact previous action:

- SB vs BTN raise is mostly 3-bet / jam / fold, with shallower stacks moving increasingly to jam/fold;
- BB vs BTN raise has very wide calls plus jam/small-3bet components depending on depth;
- BB vs BTN open shove has a separate call range;
- BB vs SB raise has its own call/jam/fold strategy and depends on SB sizing;
- BB vs SB limp has depth-dependent raise sizing and check/jam branches;
- BB vs SB open shove has a separate call range.

This means a single global `3bet` or `defend` chart is structurally wrong.

## Required chart key

Every final range must be addressable by at least:

`format -> payout_profile -> effective_stack -> hero_position -> previous_actions -> villain_position -> villain_size -> hero_actions -> hand -> frequencies`

Example:

`spin3max -> WTA -> 15bb -> BB -> BTN_RAISE_2BB_SB_FOLD -> BTN -> 2bb -> {fold,call,jam,raise} -> A5s -> frequencies`

## Verification states

- `VERIFIED_EXACT`: selected primary solver tree and exact per-hand frequencies checked.
- `CROSS_CHECKED`: aggregate/text range agrees across strong independent sources but exact mixed matrix is not yet extracted from primary tree.
- `PARTIAL`: only part of the strategy has been independently verified.
- `MISSING_EXACT`: we know the spot is required but do not yet have an exact approved chart.
- `INVALID_FOR_STRATEGY`: current data is demonstrably unsuitable and must not be served as GTO advice.

## Sources

- https://gtowizard.com/
- https://blog.gtowizard.com/status-and-info-about-our-solutions/
- https://blog.gtowizard.com/new-spins-solutions-study-plans-and-ev-comparison/
- https://www.pokerstars.com/poker/learn/strategies/spin-go-preflop-on-the-button/
- https://www.pokerstars.com/poker/learn/strategies/spin-go-preflop-in-the-small-blind/
- https://www.pokerstars.com/poker/learn/strategies/spin-go-preflop-in-the-big-blind/
- https://preflopranges.app/charts/spins/
- https://ggpoker.com/poker-games/spin-gold/
- https://ggpoker.com/blog/ggpoker-spin-gold-strategy/

## Next gate before changing strategy data

Do **not** replace `DATA` with guessed or rounded ranges. First extract and freeze the exact primary-solver matrices for the required branches, beginning with 8/10/15/20/25bb BTN/SB first-in and BB/SB responses to BTN action. Only after that should the app consume the new data files.