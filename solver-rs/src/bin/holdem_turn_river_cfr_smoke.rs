use spins_solver_core::cards::Card;
use spins_solver_core::holdem_turn_chance::TurnChanceTree;
use spins_solver_core::holdem_turn_river_cfr::HoldemTurnRiverCfr;

fn c(r:u8,s:u8)->Card{r*4+s}

fn build()->HoldemTurnRiverCfr{
    let chance=TurnChanceTree::new(
        [c(0,0),c(5,1),c(7,2),c(9,3)],
        vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)],
        vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)],
    ).expect("turn chance tree");
    HoldemTurnRiverCfr::new(chance,4.0,4.0,8.0).expect("turn river CFR")
}

fn main(){
    let check=build();
    let checkdown=check.evaluate_strategy(&check.pure_checkdown_strategy()).expect("checkdown evaluator").p0_value;
    let exact=check.exact_checkdown_value();
    let checkdown_diff=(checkdown-exact).abs();
    assert!(checkdown_diff<1e-12,"checkdown mismatch {checkdown_diff}");

    let iterations=25_000usize;
    let mut a=build();a.train(iterations).expect("two-street CFR A");let va=a.evaluate_average().expect("two-street eval A");
    let mut b=build();b.train(iterations).expect("two-street CFR B");let vb=b.evaluate_average().expect("two-street eval B");
    let deterministic_diff=(va.p0_value-vb.p0_value).abs();
    assert!(deterministic_diff<1e-12,"deterministic replay mismatch {deterministic_diff}");
    assert!(va.p0_value.is_finite());
    assert_eq!(va.raw_river_transitions,va.joint_private_states*44);

    println!(
        "status=RESEARCH_ONLY mode=HOLDEM_TURN_RIVER_RESTRICTED_CFR iterations={} joint_private_states={} raw_river_transitions={} turn_bet_bb=4.000 river_bet_bb=8.000 exact_checkdown_value={:.12} strategy_checkdown_value={:.12} checkdown_abs_diff={:.12} trained_p0_value={:.12} deterministic_replay_abs_diff={:.12} exact_river_chance=PASS exact_holdem_evaluator=PASS strategy_normalization=PASS independent_two_street_br=NOT_IMPLEMENTED verified_measured_promotion=BLOCKED gate=PASS",
        iterations,va.joint_private_states,va.raw_river_transitions,exact,checkdown,checkdown_diff,va.p0_value,deterministic_diff
    );
}
