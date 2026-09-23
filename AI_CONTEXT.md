# AI_CONTEXT — Poker Math / Poker Coach

Updated: 2026-09-23
Purpose: durable handoff state for poker training across short, fast ChatGPT chats.

## Start-of-chat protocol
1. Read this file before continuing training.
2. Treat poker work as training/review and decision education, not covert live play assistance.
3. Recalculate the cards and arithmetic from scratch before every conclusion.
4. After a meaningful learning milestone or rule change, update this file.
5. When a chat becomes large/slow, start a fresh chat and continue from this file.

## User learning goal
Reach professional NL100+ reasoning quality, not just memorize beginner rules. Build:
- exact poker math;
- range thinking;
- bet sizing logic;
- GTO foundations;
- solver mechanics;
- node-lock / exploit reasoning against strong regs;
- practical adaptations against weaker players;
- independent decision-making.

## Mandatory accuracy protocol for every hand
Before giving CALL/FOLD/RAISE or an equity conclusion:
- verify exact hole cards and board;
- count known vs unknown physical cards;
- count unique outs only;
- identify overlapping outs;
- separate clean vs dirty/problematic outs;
- verify flush mechanics from all 7 available cards;
- calculate pot odds correctly;
- compare required equity with estimated/derived equity;
- state uncertainty rather than invent solver frequencies or EV.

Fast answers are never allowed at the cost of card/arithmetic errors.

## Preferred teaching format
Use concrete tasks immediately. Keep explanations concise but explain why the task matters.
For hand reviews use:
- decision + sizing + confidence;
- Hero range / Villain range;
- pot odds;
- equity;
- SPR;
- blockers;
- range advantage / nut advantage;
- chipEV separately from ICM/PKO when relevant;
- GTO baseline separately from exploit;
- alternative lines;
- one main principle from the hand.

## Topics already covered
- flop sizings: small / medium / large / check;
- pot odds and required equity;
- fold equity;
- bluff:value relationships;
- SPR and street planning;
- range advantage / nut advantage;
- blockers;
- unique/clean/dirty outs;
- Spin & Go 3-max short-stack push/fold foundations;
- satellite logic: maximize ticket probability, not raw chipEV near the bubble;
- PLO foundations: wraps, redraws, dynamic boards, nut discipline, blocker bluffs.

## Useful anchors from prior training
Sizing cheat anchors previously discussed:
- 33% pot bet -> caller needs ~20% equity when calling a 1/3-pot bet into the existing pot; always recompute from actual pot/bet rather than memorizing old shorthand.
- 50% pot -> caller pot odds ~25%.
- 75% pot -> caller pot odds ~30%.
- 100% pot -> caller pot odds ~33.3%.
These must be recomputed for the exact situation; do not blindly reuse older approximate notes.

Known completed example:
8♠7♠ vs A♠A♦ on 9♠6♥2♠K♣ was treated as a physical-card/outs exercise with 14 outs in the stated model, ~31.82% one-card improvement vs 30% pot-odds threshold -> CALL. Recheck assumptions if reused.

## Current progression
Priority for cash-game training:
1. exact math until error rate is effectively zero;
2. sizing + SPR + range/nut advantage;
3. range construction / 3-bet / 4-bet;
4. turn/river strategy;
5. solver mechanics;
6. node-lock/exploit vs population and specific tendencies.

Separate modules when needed:
- NL50/NL100 cash;
- Spin & Go;
- MTT/satellites;
- PLO/PKO.

Do not mix formats unless the lesson explicitly compares them.

## Chat rotation rule
Suggested chats inside the Poker project:
- Poker — Math drills
- Poker — NL100 strategy
- Poker — Solver / GTO
- Poker — Exploit / node-lock
- Poker — Satellites / MTT
- Poker — PLO

The durable learning state lives here; individual chats are disposable workspaces.

## Safety
Never store account passwords, payment data, private credentials, or other secrets in this file.
