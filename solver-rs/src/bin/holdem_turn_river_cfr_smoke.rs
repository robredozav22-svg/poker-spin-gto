use std::time::Instant;
use spins_solver_core::cards::Card;
use spins_solver_core::holdem_turn_chance::TurnChanceTree;
use spins_solver_core::holdem_turn_river_br::{evaluate_independent_br,TurnRiverBrReport};
use spins_solver_core::holdem_turn_river_cfr::HoldemTurnRiverCfr;

fn c(r:u8,s:u8)->Card{r*4+s}
fn chance()->TurnChanceTree{TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)]).expect("turn chance tree")}
fn build()->HoldemTurnRiverCfr{HoldemTurnRiverCfr::new(chance(),4.0,4.0,8.0).expect("turn river CFR")}
fn measure(game:&HoldemTurnRiverCfr)->TurnRiverBrReport{let s=game.average_strategy();let v=game.evaluate_strategy(&s).unwrap().p0_value;evaluate_independent_br(&chance(),4.0,4.0,8.0,&s,v).unwrap()}
fn valid(br:&TurnRiverBrReport){assert!(br.p0_best_response_value+1e-9>=br.current_p0_value);assert!(br.p0_value_vs_p1_best_response-1e-9<=br.current_p0_value);assert!(br.nashconv.is_finite()&&br.nashconv>=0.0);}

fn main(){
    let check=build();let checkdown=check.evaluate_strategy(&check.pure_checkdown_strategy()).unwrap().p0_value;let exact=check.exact_checkdown_value();let checkdown_diff=(checkdown-exact).abs();assert!(checkdown_diff<1e-12);

    let replay_iterations=250usize;let mut r1=build();r1.train_cfr_plus(replay_iterations).unwrap();let rv1=r1.evaluate_average().unwrap().p0_value;let mut r2=build();r2.train_cfr_plus(replay_iterations).unwrap();let rv2=r2.evaluate_average().unwrap().p0_value;let deterministic_diff=(rv1-rv2).abs();assert!(deterministic_diff<1e-12);

    let mut vanilla=build();let start=Instant::now();vanilla.train(100_000).unwrap();let vanilla_ms=start.elapsed().as_millis();let br_v100=measure(&vanilla);valid(&br_v100);

    let mut plus=build();
    let start=Instant::now();plus.train_cfr_plus(25_000).unwrap();let plus_25_ms=start.elapsed().as_millis();let br_p25=measure(&plus);valid(&br_p25);
    let start=Instant::now();plus.train_cfr_plus(75_000).unwrap();let plus_100_add_ms=start.elapsed().as_millis();let br_p100=measure(&plus);valid(&br_p100);
    let start=Instant::now();plus.train_cfr_plus(300_000).unwrap();let plus_400_add_ms=start.elapsed().as_millis();let br_p400=measure(&plus);valid(&br_p400);
    let value=plus.evaluate_average().unwrap();

    assert!(br_p400.nashconv<br_p25.nashconv,"CFR+ 400k must improve over 25k");
    assert_eq!(value.raw_river_transitions,value.joint_private_states*44);

    println!("status=RESEARCH_ONLY mode=HOLDEM_TURN_RIVER_CFR_PLUS_COMPARISON replay_iterations={} joint_private_states={} raw_river_transitions={} turn_public_histories=3 exact_checkdown_value={:.12} checkdown_abs_diff={:.12} deterministic_replay_abs_diff={:.12} vanilla_nashconv_100k={:.12} vanilla_100k_ms={} cfrplus_nashconv_25k={:.12} cfrplus_25k_ms={} cfrplus_nashconv_100k={:.12} cfrplus_additional_75k_ms={} cfrplus_nashconv_400k={:.12} cfrplus_additional_300k_ms={} cfrplus_value_400k={:.12} showdown_cache=ENABLED exact_river_chance=PASS exact_holdem_evaluator=PASS independent_two_street_br=PASS promotion_gate=NOT_YET_SET gate=PASS",replay_iterations,value.joint_private_states,value.raw_river_transitions,exact,checkdown_diff,deterministic_diff,br_v100.nashconv,vanilla_ms,br_p25.nashconv,plus_25_ms,br_p100.nashconv,plus_100_add_ms,br_p400.nashconv,plus_400_add_ms,value.p0_value);
}
