use std::collections::HashSet;

use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,Card,Combo,COMBO_COUNT};
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::independent_exact_eval::evaluate_btn_fold_sb_jam_bb_call_exact;
use spins_solver_core::payoff_build::build_hu_payoff_table_parallel;
use spins_solver_core::payoff_lookup::HuPayoffLookup;
use spins_solver_core::range::ComboRange;
use spins_solver_core::restricted_exact::ExactRestrictedSubgame;

fn c(r:u8,s:u8)->Card{r*4+s}
fn idx(target:Combo)->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
fn range(indices:&[usize])->ComboRange{
    let mut w=vec![0.0;COMBO_COUNT];for &i in indices{w[i]=1.0;}ComboRange::from_weights(w).unwrap()
}

fn main(){
    let sb=[
        idx([c(12,0),c(12,1)]), // AA
        idx([c(12,2),c(3,2)]),  // A5s
        idx([c(5,0),c(4,0)]),   // 76s
    ];
    let bb=[
        idx([c(11,2),c(11,3)]), // KK
        idx([c(10,0),c(10,1)]), // QQ
        idx([c(7,2),c(6,2)]),   // 98s
    ];
    let sb_prior=range(&sb);let bb_prior=range(&bb);
    let blockers=BlockerMatrix::build();let combos=all_combos();
    let mut keys=HashSet::new();
    for &i in &sb{for &j in &bb{if blockers.compatible(i,j){keys.insert(canonical_hu_matchup(combos[i],combos[j]).unwrap());}}}
    let mut requested:Vec<_>=keys.into_iter().collect();requested.sort_by_key(|k|k.0);
    let (table,stats)=build_hu_payoff_table_parallel(&requested,None,2).expect("exact payoff fixture build");
    assert_eq!(stats.computed_missing,requested.len());
    let lookup=HuPayoffLookup::from_table(table).unwrap();

    let mut game=ExactRestrictedSubgame::new_from_payoff_lookup(8.0,sb_prior.clone(),bb_prior.clone(),&lookup).unwrap();
    for _ in 0..10_000{game.sweep().unwrap();}
    let sb_strategy=game.sb_average_strategy();let bb_strategy=game.bb_average_strategy();
    let internal=game.evaluate(&sb_strategy,&bb_strategy).unwrap();
    let independent=evaluate_btn_fold_sb_jam_bb_call_exact(
        8.0,&sb_prior,&bb_prior,&sb_strategy,&bb_strategy,&blockers,&lookup
    ).unwrap();

    let tol=1e-12;
    assert!((internal.sb_value-independent.current_sb_value_bb).abs()<tol);
    assert!((internal.sb_best_response_value-independent.sb_best_response_value_bb).abs()<tol);
    assert!((internal.sb_value_vs_bb_best_response-independent.sb_value_vs_bb_best_response_bb).abs()<tol);
    assert!((internal.sb_br_gain-independent.sb_br_gain_bb).abs()<tol);
    assert!((internal.bb_br_gain-independent.bb_br_gain_bb).abs()<tol);
    assert!((internal.nashconv-independent.nashconv_bb).abs()<tol);
    assert_eq!(internal.legal_pairs,independent.legal_pairs);
    assert!((internal.joint_normalizer-independent.joint_normalizer).abs()<tol);

    println!(
        "status=RESEARCH_ONLY mode=INDEPENDENT_EXACT_EVAL_CROSSCHECK support=3x3 canonical_payoffs={} legal_pairs={} sweeps=10000 sb_value={:.12} sb_br={:.12} sb_vs_bb_br={:.12} sb_gain={:.12} bb_gain={:.12} nashconv_bb={:.12} normalized_nashconv_fraction_of_pot={:.12} evaluator_agreement=PASS tolerance=1e-12",
        requested.len(),independent.legal_pairs,independent.current_sb_value_bb,independent.sb_best_response_value_bb,
        independent.sb_value_vs_bb_best_response_bb,independent.sb_br_gain_bb,independent.bb_br_gain_bb,
        independent.nashconv_bb,independent.normalized_nashconv_fraction_of_pot
    );
}
