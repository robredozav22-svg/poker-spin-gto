use std::collections::HashSet;
use std::time::Instant;

use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,Card,Combo,COMBO_COUNT};
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::payoff_build::build_hu_payoff_table_parallel;
use spins_solver_core::payoff_lookup::HuPayoffLookup;
use spins_solver_core::range::ComboRange;
use spins_solver_core::restricted_exact::ExactRestrictedSubgame;

fn c(r:u8,s:u8)->Card{r*4+s}
fn idx(target:Combo)->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
fn range(indices:&[usize])->ComboRange{
    let mut w=vec![0.0;COMBO_COUNT];
    for &i in indices{w[i]=1.0;}
    ComboRange::from_weights(w).unwrap()
}

fn main(){
    // Deterministic bounded physical-combo support. Diversity is intentional for
    // stress testing only; these are NOT real Spin ranges or chart boundaries.
    let sb=[
        idx([c(12,0),c(12,1)]), // AA
        idx([c(11,0),c(11,1)]), // KK
        idx([c(10,0),c(10,1)]), // QQ
        idx([c(9,0),c(9,1)]),   // JJ
        idx([c(8,0),c(8,1)]),   // TT
        idx([c(7,0),c(7,1)]),   // 99
        idx([c(6,0),c(6,1)]),   // 88
        idx([c(5,0),c(5,1)]),   // 77
        idx([c(12,2),c(11,2)]), // AKs
        idx([c(12,3),c(10,3)]), // AQs
        idx([c(12,2),c(9,2)]),  // AJs
        idx([c(12,3),c(8,3)]),  // ATs
        idx([c(12,2),c(3,2)]),  // A5s
        idx([c(11,2),c(10,2)]), // KQs
        idx([c(11,3),c(9,3)]),  // KJs
        idx([c(11,2),c(8,2)]),  // KTs
        idx([c(10,3),c(9,3)]),  // QJs
        idx([c(10,2),c(8,2)]),  // QTs
        idx([c(9,2),c(8,2)]),   // JTs
        idx([c(8,3),c(7,3)]),   // T9s
        idx([c(7,2),c(6,2)]),   // 98s
        idx([c(6,3),c(5,3)]),   // 87s
        idx([c(5,2),c(4,2)]),   // 76s
        idx([c(4,3),c(3,3)]),   // 65s
    ];

    let bb=[
        idx([c(12,0),c(11,3)]), // AKo
        idx([c(12,1),c(10,2)]), // AQo
        idx([c(12,0),c(9,3)]),  // AJo
        idx([c(12,1),c(8,2)]),  // ATo
        idx([c(11,0),c(10,3)]), // KQo
        idx([c(11,1),c(9,2)]),  // KJo
        idx([c(11,0),c(8,3)]),  // KTo
        idx([c(10,1),c(9,2)]),  // QJo
        idx([c(10,0),c(8,3)]),  // QTo
        idx([c(9,1),c(8,2)]),   // JTo
        idx([c(10,2),c(10,3)]), // QQ variant
        idx([c(9,2),c(9,3)]),   // JJ variant
        idx([c(8,2),c(8,3)]),   // TT variant
        idx([c(7,2),c(7,3)]),   // 99 variant
        idx([c(6,2),c(6,3)]),   // 88 variant
        idx([c(12,0),c(7,0)]),  // A9s
        idx([c(11,1),c(7,1)]),  // K9s
        idx([c(10,0),c(7,0)]),  // Q9s
        idx([c(9,1),c(7,1)]),   // J9s
        idx([c(8,0),c(6,0)]),   // T8s
        idx([c(7,1),c(5,1)]),   // 97s
        idx([c(6,0),c(4,0)]),   // 86s
        idx([c(5,1),c(3,1)]),   // 75s
        idx([c(3,0),c(2,0)]),   // 54s
    ];

    let sb_prior=range(&sb);let bb_prior=range(&bb);
    let blockers=BlockerMatrix::build();let combos=all_combos();
    let mut keys=HashSet::new();let mut legal_pairs=0usize;
    for &i in &sb{
        for &j in &bb{
            if !blockers.compatible(i,j){continue;}
            legal_pairs+=1;
            keys.insert(canonical_hu_matchup(combos[i],combos[j]).unwrap());
        }
    }
    let mut requested:Vec<_>=keys.into_iter().collect();
    requested.sort_by_key(|k|k.0);

    let workers=2usize;
    let build_start=Instant::now();
    let (table,stats)=build_hu_payoff_table_parallel(&requested,None,workers).expect("24x24 parallel exact payoff build");
    let build_seconds=build_start.elapsed().as_secs_f64();
    assert_eq!(stats.computed_missing,requested.len());
    let artifact_bytes=table.encode().unwrap().len();
    let lookup=HuPayoffLookup::from_table(table).unwrap();

    let construct_start=Instant::now();
    let mut game1=ExactRestrictedSubgame::new_from_payoff_lookup(8.0,sb_prior.clone(),bb_prior.clone(),&lookup).expect("persisted 24x24 game 1");
    let construct_seconds=construct_start.elapsed().as_secs_f64();
    assert_eq!(game1.pairs.len(),legal_pairs);
    assert_eq!(game1.equity_cache.misses(),0);

    let sweeps=10_000usize;
    let solve1_start=Instant::now();
    for _ in 0..sweeps{game1.sweep().unwrap();}
    let solve1_seconds=solve1_start.elapsed().as_secs_f64();
    let sb1=game1.sb_average_strategy();let bb1=game1.bb_average_strategy();
    let report1=game1.evaluate(&sb1,&bb1).unwrap();

    let mut game2=ExactRestrictedSubgame::new_from_payoff_lookup(8.0,sb_prior,bb_prior,&lookup).expect("persisted 24x24 game 2");
    let solve2_start=Instant::now();
    for _ in 0..sweeps{game2.sweep().unwrap();}
    let solve2_seconds=solve2_start.elapsed().as_secs_f64();
    let sb2=game2.sb_average_strategy();let bb2=game2.bb_average_strategy();
    let report2=game2.evaluate(&sb2,&bb2).unwrap();

    assert_eq!(sb1,sb2,"repeated 24x24 persisted solve SB strategy differs");
    assert_eq!(bb1,bb2,"repeated 24x24 persisted solve BB strategy differs");
    assert_eq!(report1,report2,"repeated 24x24 persisted solve report differs");
    assert_eq!(game2.equity_cache.misses(),0);

    let avg_sb_jam=sb.iter().map(|i|sb1.row(*i)[1]).sum::<f64>()/sb.len() as f64;
    let avg_bb_call=bb.iter().map(|i|bb1.row(*i)[1]).sum::<f64>()/bb.len() as f64;

    println!(
        "status=RESEARCH_ONLY mode=PERSISTED_RESTRICTED_24X24 support_sb={} support_bb={} legal_pairs={} unique_canonical_payoffs={} artifact_bytes={} payoff_build_workers={} payoff_build_seconds={:.6} payoff_keys_per_second={:.6} lookup_construct_seconds={:.6} sweeps={} solve1_seconds={:.6} solve2_seconds={:.6} solve_sweeps_per_second={:.3} avg_sb_jam={:.9} avg_bb_call={:.9} nashconv={:.9} sb_br_gain={:.9} bb_br_gain={:.9} sb_value={:.9} exact_enumeration_during_solve=ZERO deterministic_repeat=PASS",
        sb.len(),bb.len(),legal_pairs,requested.len(),artifact_bytes,workers,build_seconds,requested.len() as f64/build_seconds,construct_seconds,
        sweeps,solve1_seconds,solve2_seconds,sweeps as f64/solve1_seconds,avg_sb_jam,avg_bb_call,report1.nashconv,report1.sb_br_gain,report1.bb_br_gain,report1.sb_value
    );
    println!("NOTE: bounded 24x24 physical-combo restricted game only; not full Spin GTO and not chart data.");
}
