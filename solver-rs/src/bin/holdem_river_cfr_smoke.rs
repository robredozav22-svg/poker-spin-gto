use spins_solver_core::cards::Card;
use spins_solver_core::holdem_river_cfr::RiverRangeGame;

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let board=[c(0,0),c(5,1),c(7,2),c(9,3),c(1,0)];
    let p0=vec![
        ([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],1.0),
        ([c(10,0),c(8,0)],1.0),([c(6,1),c(4,1)],1.0),
    ];
    let p1=vec![
        ([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],1.0),
        ([c(8,1),c(7,1)],1.0),([c(3,1),c(2,1)],1.0),
    ];
    let mut game=RiverRangeGame::new(board,p0,p1,4.0,4.0).expect("river range game");
    let iterations=100_000usize;
    game.train(iterations).expect("river CFR training");
    let strategy=game.average_strategy();
    let report=game.evaluate_independent(&strategy).expect("independent river BR evaluation");
    assert!((report.joint_normalizer-1.0).abs()<1e-12);
    assert!(report.nashconv<0.003,"river NashConv too large: {}",report.nashconv);
    println!(
        "status=RESEARCH_ONLY mode=HOLDEM_RIVER_RANGE_CFR iterations={} board_cards=5 p0_support=4 p1_support=4 legal_joint_states={} pot_bb={:.3} bet_bb={:.3} current_p0_value={:.12} p0_br={:.12} p0_vs_p1_br={:.12} p0_br_gain={:.12} p1_br_gain={:.12} nashconv={:.12} joint_normalizer={:.12} holdem_evaluator=EXACT_SEVEN_CARD blockers=EXACT independent_bruteforce_br=PASS gate=PASS",
        iterations,report.legal_joint_states,game.pot(),game.bet(),report.current_p0_value,
        report.p0_best_response_value,report.p0_value_vs_p1_best_response,
        report.p0_br_gain,report.p1_br_gain,report.nashconv,report.joint_normalizer
    );
}
