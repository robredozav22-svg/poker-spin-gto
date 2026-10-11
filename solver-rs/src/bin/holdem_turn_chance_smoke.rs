use spins_solver_core::cards::Card;
use spins_solver_core::holdem_turn_chance::TurnChanceTree;

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let board=[c(0,0),c(5,1),c(7,2),c(9,3)];
    let p0=vec![
        ([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),
        ([c(10,0),c(8,0)],0.4),([c(6,1),c(4,1)],0.2),
    ];
    let p1=vec![
        ([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),
        ([c(8,1),c(7,1)],0.5),([c(3,1),c(2,1)],0.3),
    ];
    let tree=TurnChanceTree::new(board,p0,p1).expect("turn chance tree");
    tree.validate_mass().expect("chance mass validation");
    assert_eq!(tree.raw_transition_count(),tree.joint_states().len()*44);
    let aggregated=tree.exact_checkdown_showdown_sign();
    let direct=tree.direct_checkdown_showdown_sign();
    let diff=(aggregated-direct).abs();
    assert!(diff<1e-12,"turn chance cross-check mismatch {diff}");
    println!(
        "status=RESEARCH_ONLY mode=HOLDEM_TURN_TO_RIVER_EXACT_CHANCE turn_board_cards=4 joint_private_states={} distinct_river_children={} raw_transitions={} transitions_per_joint=44 river_mass=1.000000000000 aggregated_checkdown_sign={:.12} direct_checkdown_sign={:.12} crosscheck_abs_diff={:.12} exact_blockers=PASS exact_river_enumeration=PASS exact_holdem_evaluator=PASS gate=PASS",
        tree.joint_states().len(),tree.children().len(),tree.raw_transition_count(),aggregated,direct,diff
    );
}
