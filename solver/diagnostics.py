from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from random import Random

from .evaluator import compare_seven
from .hands import HAND_CLASSES, combo_to_class, full_deck


@dataclass(frozen=True)
class BestResponseDiagnostics:
    samples: int
    btn_best_response_gain_bb: float
    bb_best_response_gain_bb: float
    total_unilateral_gain_bb: float
    btn_policy_ev_bb: float
    bb_policy_ev_bb: float


def estimate_hu_pushfold_best_response_gains(
    btn_strategy: dict[str, dict[str, float]],
    bb_strategy: dict[str, dict[str, float]],
    stack_bb: float,
    samples: int = 200_000,
    seed: int = 20260907,
) -> BestResponseDiagnostics:
    """Estimate unilateral best-response gains in the HU push/fold abstraction.

    The estimator samples full legal deals and runouts from our own deck and
    evaluator. It does not use Wizard, Range+, or an external equity table.

    `total_unilateral_gain_bb` is a practical convergence diagnostic inside
    this exact action abstraction. It must not be confused with exploitability
    of full no-limit Hold'em, because limp/minraise/postflop actions are absent.
    """
    if samples <= 0:
        raise ValueError("samples must be positive")

    rng = Random(seed)

    btn_fold_sum = defaultdict(float)
    btn_jam_sum = defaultdict(float)
    btn_policy_sum = defaultdict(float)
    btn_count = defaultdict(int)

    bb_fold_cf_sum = defaultdict(float)
    bb_call_cf_sum = defaultdict(float)
    bb_policy_cf_sum = defaultdict(float)

    btn_root_policy_total = 0.0
    bb_root_policy_total = 0.0

    deck = full_deck()

    for _ in range(samples):
        cards = rng.sample(deck, 9)
        btn_combo = (cards[0], cards[1])
        bb_combo = (cards[2], cards[3])
        board = tuple(cards[4:9])
        btn_hand = combo_to_class(btn_combo)
        bb_hand = combo_to_class(bb_combo)

        cmp = compare_seven(btn_combo + board, bb_combo + board)
        if cmp > 0:
            btn_showdown = stack_bb
            bb_showdown = -stack_bb
        elif cmp < 0:
            btn_showdown = -stack_bb
            bb_showdown = stack_bb
        else:
            btn_showdown = bb_showdown = 0.0

        bp = btn_strategy[btn_hand]
        cp = bb_strategy[bb_hand]

        btn_fold = -0.5
        btn_jam = (1.0 - cp["call"]) * 1.0 + cp["call"] * btn_showdown
        btn_policy = bp["fold"] * btn_fold + bp["jam"] * btn_jam

        btn_fold_sum[btn_hand] += btn_fold
        btn_jam_sum[btn_hand] += btn_jam
        btn_policy_sum[btn_hand] += btn_policy
        btn_count[btn_hand] += 1
        btn_root_policy_total += btn_policy

        # For BB, only the facing-jam branch can be changed by a BB best
        # response. Weight its counterfactual values by BTN's jam probability.
        reach = bp["jam"]
        bb_fold = -1.0
        bb_call = bb_showdown
        bb_policy = cp["fold"] * bb_fold + cp["call"] * bb_call
        bb_fold_cf_sum[bb_hand] += reach * bb_fold
        bb_call_cf_sum[bb_hand] += reach * bb_call
        bb_policy_cf_sum[bb_hand] += reach * bb_policy

        # Root BB utility also includes BTN folds, where BB wins the SB 0.5bb.
        bb_root_policy_total += bp["fold"] * 0.5 + reach * bb_policy

    btn_br_total = 0.0
    btn_current_total = 0.0
    for hand in HAND_CLASSES:
        n = btn_count[hand]
        if n <= 0:
            continue
        btn_br_total += max(btn_fold_sum[hand], btn_jam_sum[hand])
        btn_current_total += btn_policy_sum[hand]

    bb_br_cf_total = 0.0
    bb_current_cf_total = 0.0
    for hand in HAND_CLASSES:
        bb_br_cf_total += max(bb_fold_cf_sum[hand], bb_call_cf_sum[hand])
        bb_current_cf_total += bb_policy_cf_sum[hand]

    btn_gain = (btn_br_total - btn_current_total) / samples
    bb_gain = (bb_br_cf_total - bb_current_cf_total) / samples

    return BestResponseDiagnostics(
        samples=samples,
        btn_best_response_gain_bb=btn_gain,
        bb_best_response_gain_bb=bb_gain,
        total_unilateral_gain_bb=btn_gain + bb_gain,
        btn_policy_ev_bb=btn_root_policy_total / samples,
        bb_policy_ev_bb=bb_root_policy_total / samples,
    )
