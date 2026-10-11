use std::time::Instant;
use spins_solver_core::cards::{Card,Combo};
use spins_solver_core::continuation_registry::{ContinuationArtifactProof,ContinuationArtifactScope,ContinuationMeasurementProof};
use spins_solver_core::holdem_turn_chance::TurnChanceTree;
use spins_solver_core::holdem_turn_river_br::{evaluate_independent_br,TurnRiverBrReport};
use spins_solver_core::holdem_turn_river_cfr::HoldemTurnRiverCfr;
use spins_solver_core::range_state_fingerprint::fingerprint_range_state;
use spins_solver_core::turn_river_strategy_checksum::checksum_turn_river_strategy;

const FINAL_ITERATIONS:usize=1_600_000;
const MAX_NASHCONV_BB:f64=0.0025;
const ARTIFACT_ID:&str="turn-river-research-fixture-v1";
const RANGE_CONTEXT:&str="RESEARCH_FIXTURE_TURN_RIVER_V1";

fn c(r:u8,s:u8)->Card{r*4+s}
fn board()->[Card;4]{[c(0,0),c(5,1),c(7,2),c(9,3)]}
fn p0_range()->Vec<(Combo,f64)>{vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)]}
fn p1_range()->Vec<(Combo,f64)>{vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)]}
fn chance()->TurnChanceTree{TurnChanceTree::new(board(),p0_range(),p1_range()).expect("turn chance tree")}
fn build()->HoldemTurnRiverCfr{HoldemTurnRiverCfr::new(chance(),4.0,4.0,8.0).expect("turn river CFR")}
fn valid(br:&TurnRiverBrReport){assert!(br.p0_best_response_value+1e-9>=br.current_p0_value);assert!(br.p0_value_vs_p1_best_response-1e-9<=br.current_p0_value);assert!(br.nashconv.is_finite()&&br.nashconv>=0.0);}

fn main(){
    let range_state_id=fingerprint_range_state(RANGE_CONTEXT,&board(),&p0_range(),&p1_range()).expect("range fingerprint");

    let check=build();let checkdown=check.evaluate_strategy(&check.pure_checkdown_strategy()).unwrap().p0_value;let exact=check.exact_checkdown_value();let checkdown_diff=(checkdown-exact).abs();assert!(checkdown_diff<1e-12);

    let replay_iterations=250usize;let mut r1=build();r1.train(replay_iterations).unwrap();let rv1=r1.evaluate_average().unwrap().p0_value;let mut r2=build();r2.train(replay_iterations).unwrap();let rv2=r2.evaluate_average().unwrap().p0_value;let deterministic_diff=(rv1-rv2).abs();assert!(deterministic_diff<1e-12);

    let mut game=build();
    let start=Instant::now();game.train(FINAL_ITERATIONS).unwrap();let train_ms=start.elapsed().as_millis();
    let strategy=game.average_strategy();let value=game.evaluate_strategy(&strategy).unwrap();
    let strategy_checksum=checksum_turn_river_strategy(&strategy);
    let br=evaluate_independent_br(&chance(),4.0,4.0,8.0,&strategy,value.p0_value).unwrap();valid(&br);
    assert_eq!(value.raw_river_transitions,value.joint_private_states*44);
    assert!(br.nashconv<=MAX_NASHCONV_BB,"promotion gate failed: NashConv {:.12} bb exceeds fixed threshold {:.12} bb",br.nashconv,MAX_NASHCONV_BB);

    let measurement=ContinuationMeasurementProof::new(
        "VANILLA_CFR",FINAL_ITERATIONS as u64,br.nashconv,MAX_NASHCONV_BB,true,true,true,
    ).expect("measurement proof");
    let proof=ContinuationArtifactProof::new(
        ARTIFACT_ID,strategy_checksum.clone(),value.raw_river_transitions as u64,value.raw_river_transitions as u64,
        ContinuationArtifactScope::RangeConditioned,Some(range_state_id.clone()),Some(measurement),
    ).expect("range-conditioned artifact proof");
    assert!(proof.is_complete());assert!(proof.has_passing_measurement());assert!(proof.range_exact_ready(&range_state_id));assert!(!proof.generic_exact_ready());

    println!("status=RESEARCH_ONLY mode=HOLDEM_TURN_RIVER_RANGE_BOUND_FINAL_GATE artifact_id={} range_context={} range_state_id={} strategy_checksum={} iterations={} joint_private_states={} raw_river_transitions={} turn_public_histories=3 exact_checkdown_value={:.12} checkdown_abs_diff={:.12} deterministic_replay_abs_diff={:.12} trained_p0_value={:.12} p0_br={:.12} p0_vs_p1_br={:.12} p0_br_gain={:.12} p1_br_gain={:.12} nashconv={:.12} max_nashconv_bb={:.12} train_ms={} algorithm=VANILLA_CFR scope=RANGE_CONDITIONED generic_exact_ready=false range_exact_ready=true cfrplus_status=RESEARCH_ONLY_NOT_SELECTED showdown_cache=ENABLED exact_river_chance=PASS exact_holdem_evaluator=PASS independent_two_street_br=PASS promotion_numeric_gate=PASS persisted_repo_evidence=PENDING gate=PASS",ARTIFACT_ID,RANGE_CONTEXT,range_state_id.as_str(),strategy_checksum,FINAL_ITERATIONS,value.joint_private_states,value.raw_river_transitions,exact,checkdown_diff,deterministic_diff,value.p0_value,br.p0_best_response_value,br.p0_value_vs_p1_best_response,br.p0_br_gain,br.p1_br_gain,br.nashconv,MAX_NASHCONV_BB,train_ms);
}
