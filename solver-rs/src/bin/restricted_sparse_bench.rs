use spins_solver_core::cards::{all_combos,COMBO_COUNT};
use spins_solver_core::class_aggregation::class_label;
use spins_solver_core::range::ComboRange;
use spins_solver_core::restricted_subgame::SbJamBbResponseSubgame;
use spins_solver_core::strategy_metrics::{marginal_action_probability,max_action_delta_on_support,weighted_action_mae};

fn range_for_labels(labels:&[&str])->ComboRange{
    let combos=all_combos();
    let mut weights=vec![0.0;COMBO_COUNT];
    for label in labels{
        let index=combos.iter().position(|c|class_label(*c)==*label)
            .unwrap_or_else(||panic!("missing class {label}"));
        weights[index]=1.0;
    }
    ComboRange::from_weights(weights).expect("valid sparse range")
}

fn main(){
    let sb_prior=range_for_labels(&["AA","A5s","76s"]);
    let bb_prior=range_for_labels(&["KK","AQo","65s"]);
    let mut game=SbJamBbResponseSubgame::new(8.0,sb_prior.clone(),bb_prior.clone())
        .expect("subgame construction");

    let mut prev_sb=game.sb_current_strategy();
    let mut prev_bb=game.bb_current_strategy();
    let samples_per_matchup=500u64;
    let seed=20260907u64;

    println!("status=RESEARCH_ONLY stack_bb=8 samples_per_matchup={samples_per_matchup} seed={seed}");
    println!("support sb=3 bb=3 sweeps=20");

    for sweep in 1..=20{
        let d=game.sweep(samples_per_matchup,seed).expect("sweep");
        let sb=game.sb_current_strategy();
        let bb=game.bb_current_strategy();
        if sweep==1 || sweep%5==0{
            let sb_jam=marginal_action_probability(&sb_prior,&sb,1).unwrap();
            let bb_call=marginal_action_probability(&bb_prior,&bb,1).unwrap();
            let sb_max=max_action_delta_on_support(&sb_prior,&prev_sb,&sb).unwrap();
            let bb_max=max_action_delta_on_support(&bb_prior,&prev_bb,&bb).unwrap();
            let sb_mae=weighted_action_mae(&sb_prior,&prev_sb,&sb).unwrap();
            let bb_mae=weighted_action_mae(&bb_prior,&prev_bb,&bb).unwrap();
            println!(
                "sweep={sweep} sb_jam={sb_jam:.6} bb_call={bb_call:.6} sb_max_delta={sb_max:.6} bb_max_delta={bb_max:.6} sb_weighted_mae={sb_mae:.6} bb_weighted_mae={bb_mae:.6} mean_bb_jam_reach={:.6} cache_hits={} cache_misses={}",
                d.mean_bb_jam_reach,d.hu_cache_hits,d.hu_cache_misses
            );
        }
        prev_sb=sb;
        prev_bb=bb;
    }

    let sb_avg=game.sb_average_strategy();
    let bb_avg=game.bb_average_strategy();
    let sb_avg_jam=marginal_action_probability(&sb_prior,&sb_avg,1).unwrap();
    let bb_avg_call=marginal_action_probability(&bb_prior,&bb_avg,1).unwrap();
    println!("average_strategy sb_jam={sb_avg_jam:.6} bb_call={bb_avg_call:.6}");
    println!("NOTE: stability diagnostics only; not NashConv and not chart data");
}
