# V45 UX Benchmark — poker study products

Date: 2026-09-07

Purpose: study successful poker range/solver/trainer products as UX references, not as strategy data sources. The target is a Spin & Go preflop review/training tool with minimum clicks, one effective-stack control, exact action-history routing, mixed-frequency grids, and strong data provenance.

## Products reviewed

### 1. GTO Wizard

Strong patterns to adopt:
- Dedicated Study mode with Strategy / Ranges / Breakdown / Reports views.
- Dynamic strategic info as the user moves through a line.
- Mixed-frequency hand strategy in a matrix.
- Range comparison and hand-class filtering.
- Focus mode to remove clutter.
- Undo/redo concept in Range Builder.
- Trainer that follows a hand through actions and gives immediate feedback.

Do not copy:
- General-purpose navigation complexity. Our app is narrow: Spin preflop review/training.
- Large number of tabs in the primary review flow.

Use in V45:
- Advanced details drawer, not primary screen.
- Focus mode.
- One-step back / forward history.
- Optional strategy detail panel.

### 2. GTO Ranges+

Strong patterns to adopt:
- Mobile-first preflop range browsing.
- Multiway action-sequence coverage.
- Mixed-frequency presentation.
- Exact scenario dependence: opens, calls, squeezes, reraises and jams are distinct spots.

Do not copy:
- Treat pre-solved library as our internal architecture.

Use in V45:
- Action-history-first navigation.
- Position cards with exact prior actions and sizes.
- Fast stack presets.

### 3. DTO Poker

Strong patterns to adopt:
- Training feels like playing decisions, not reading a database.
- Immediate grading after each decision.
- Custom scenarios by stack/position.
- Mobile-first interaction.
- Large solved library hidden behind a simple training experience.

Use in V45:
- Separate REVIEW and TRAIN modes.
- TRAIN presents one decision at a time.
- Immediate frequency/EV-loss feedback after answer.
- Saved weak spots / retry queue later.

### 4. Lucid Poker

Strong patterns to adopt:
- Very short setup flow.
- Shows position, stack depth and prior action directly in the drill.
- Mixed actions are treated as valid solver-approved decisions, not forced into one pure answer.
- Session review after drilling.

Use in V45:
- Mixed-action-aware grading.
- Do not mark a low-frequency correct action simply "wrong".
- Session mistake review.

### 5. ICMIZER / MTT Coach

Strong patterns to adopt:
- Training can be isolated by situation type.
- Adaptive difficulty.
- Progress metrics and weak-zone targeting.
- Favorites save repeated configuration work.
- Answer review reveals full range after the decision.

Use in V45 later:
- Favorite spots.
- Weak-spot drills.
- Difficulty based on closeness of action EVs / mixed boundary.
- Progress by node family instead of one global score.

### 6. Poker Academy

Strong patterns to adopt:
- Clear separation of Charts / Solver / Training / User simulations.
- Custom problem sets around weak spots.
- Hand replay with strategy annotations.
- Fast drill mode and deeper study mode are separate experiences.

Use in V45:
- REVIEW = fast lookup.
- TRAIN = repeated decisions.
- LAB = solver/source/audit diagnostics, hidden from normal use.

### 7. Poker Lab (AHTOOOXA/poker-charts)

Strong patterns to adopt technically:
- React + TypeScript modular architecture.
- 13x13 weighted mixed-action grid.
- Static data separation from UI.
- Zustand/local persistence pattern.
- Testable range components.

Use in V45:
- Technical donor for grid/rendering patterns only.
- Strategy data remains ours.

### 8. PokerStars Learn Spin & Go

Strong patterns to adopt conceptually:
- Effective stack is the main decision variable.
- Spin preflop ranges are taught by exact positions and stack depth.
- BTN 3-handed decision family is naturally described as raise / shove / fold.
- Short-stack push/fold is taught separately from deeper mixed trees.

Use in V45:
- Effective stack remains the only primary stack control.
- 3-max and HU separated.
- Simplified learning layer can exist separately from exact solver layer.

### 9. PreflopTrainer.com

Strong patterns to adopt:
- Instant-feedback drills.
- Progress/streak persistence in browser.
- Quick-fire mode vs full-scenario mode.

Use in V45 later:
- Quick drill mode.
- Local progress persistence.

### 10. PokerCoaching app

Strong patterns to adopt:
- Charts + quizzes coexist in one mobile app.
- Push/fold filters by stack, position, blind level, players and antes.

Use in V45:
- Filters belong in setup/advanced mode, not always visible in the main review screen.

---

# Benchmark conclusion

No single product should be copied. Best architecture is:

- **Range+/supplied screenshots:** primary navigation pattern and action-tree clarity.
- **GTO Wizard:** deep strategy inspection, mixed frequencies, focus mode and history controls.
- **DTO/Lucid:** training interaction and immediate feedback.
- **ICMIZER/Poker Academy:** adaptive drills, favorites, weak-spot review and progress.
- **Poker Lab:** reusable technical grid architecture.
- **PokerStars Learn:** effective-stack-first Spin teaching model.

The product advantage should be narrower and faster than general-purpose solvers.

# V45 UX principles

1. One primary stack control: `EFF BB`.
2. No separate position selector in the normal flow; position follows the game tree.
3. Every chart is keyed by exact action history and sizing.
4. Previous actions remain visible.
5. Only legal next actions are shown.
6. One tap on an action changes the chart immediately.
7. One-tap BACK to previous decision node.
8. Optional FORWARD after backtracking.
9. 13x13 mixed-frequency cells.
10. Aggregate action frequencies directly below the grid.
11. Exact / Simplified / Exploit are distinct strategy layers.
12. 3-max and HU are distinct modes/trees.
13. Missing chart/source is explicit; no generic fallback.
14. REVIEW, TRAIN and LAB are distinct experiences.
15. Advanced diagnostics must never clutter the fast REVIEW screen.
16. Mobile targets are first-class: large tap areas, no hover dependencies.
17. Recent spots and Favorites reduce repeat navigation.
18. Mixed-action grading in TRAIN recognizes all solver-approved actions.
19. Source/provenance is available, but one tap away rather than always occupying screen space.
20. Study-only design; no poker-client integration or live automatic recognition.
