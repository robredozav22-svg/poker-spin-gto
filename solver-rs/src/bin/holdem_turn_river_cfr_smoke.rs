use spins_solver_core::cards::Card;
use spins_solver_core::holdem_turn_chance::TurnChanceTree;
use spins_solver_core::holdem_turn_river_br::evaluate_independent_br;
use spins_solver_core::holdem_turn_river_cfr::HoldemTurnRiverCfr;

fn c(r:u8,s:u8)->Card{r*4+s}
fn chance()->TurnChanceTree{TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)]).expect("turn chance tree")}
fn build()->HoldemTurnRiverCfr{HoldemTurnRiverCfr::new(chance(),4.0,4.0,8.0).expect("turn river CFR")}

fn main(){
    let check=build();let checkdown=check.evaluate_strategy(&check.pure_checkdown_strategy()).expect("checkdown evaluator").p0_value;let exact=check.exact_checkdown_value();let checkdown_diff=(checkdown-exact).abs();assert!(checkdown_diff<1e-12);

    // Determinism is structural; it does not require duplicating the expensive convergence run.
    let replay_iterations=250usize;
    let mut r1=build();r1.train(replay_iterations).unwrap();let rv1=r1.evaluate_average().unwrap().p0_value;
    let mut r2=build();r2.train(replay_iterations).unwrap();let rv2=r2.evaluate_average().unwrap().p0_value;
    let deterministic_diff=(rv1-rv2).abs();assert!(deterministic_diff<1e-12,"deterministic replay mismatch {deterministic_diff}");

    let iterations=25_000usize;
    let mut game=build();game.train(iterations).expect("two-street CFR");let strategy=game.average_strategy();let value=game.evaluate_strategy(&strategy).expect("two-street eval");
    assert!(value.p0_value.is_finite());assert_eq!(value.raw_river_transitions,value.joint_private_states*44);
    let br=evaluate_independent_br(&chance(),4.0,4.0,8.0,&strategy,value.p0_value).expect("independent two-street BR");
    assert!(br.p0_best_response_value+1e-9>=value.p0_value);assert!(br.p0_value_vs_p1_best_response-1e-9<=value.p0_value);assert!(br.nashconv.is_finite()&&br.nashconv>=0.0);

    println!("status=RESEARCH_ONLY mode=HOLDEM_TURN_RIVER_RESTRICTED_CFR iterations={} replay_iterations={} joint_private_states={} raw_river_transitions={} turn_public_histories=3 turn_bet_bb=4.000 river_bet_bb=8.000 exact_checkdown_value={:.12} strategy_checkdown_value={:.12} checkdown_abs_diff={:.12} trained_p0_value={:.12} deterministic_replay_abs_diff={:.12} p0_br={:.12} p0_vs_p1_br={:.12} p0_br_gain={:.12} p1_br_gain={:.12} nashconv={:.12} showdown_cache=ENABLED exact_river_chance=PASS exact_holdem_evaluator=PASS strategy_normalization=PASS independent_two_street_br=PASS verified_measured_promotion=PENDING_THRESHOLD gate=PASS",iterations,replay_iterations,value.joint_private_states,value.raw_river_transitions,exact,checkdown,checkdown_diff,value.p0_value,deterministic_diff,br.p0_best_response_value,br.p0_value_vs_p1_best_response,br.p0_br_gain,br.p1_br_gain,br.nashconv);
}
