use spins_solver_core::cards::{all_combos,COMBO_COUNT};
use spins_solver_core::class_aggregation::class_label;
use spins_solver_core::equity_cache::EquityCache;
use spins_solver_core::range::ComboRange;
use spins_solver_core::restricted_eval::evaluate_restricted_sampled_nashconv;
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
    let mut eval_cache=EquityCache::new();

    let samples_per_matchup=500u64;
    let seed=20260907u64;
    let checkpoints=[1usize,5,10,20,50,100];

    let mut prev_checkpoint_sb=game.sb_current_strategy();
    let mut prev_checkpoint_bb=game.bb_current_strategy();
    let mut prev_checkpoint_sb_avg=game.sb_average_strategy();
    let mut prev_checkpoint_bb_avg=game.bb_average_strategy();

    println!("status=RESEARCH_ONLY stack_bb=8 samples_per_matchup={samples_per_matchup} seed={seed}");
    println!("support sb=3 bb=3 sweeps=100 checkpoints=1,5,10,20,50,100");

    for sweep in 1..=100{
        let d=game.sweep(samples_per_matchup,seed).expect("sweep");
        if checkpoints.contains(&sweep){
            let sb=game.sb_current_strategy();
            let bb=game.bb_current_strategy();
            let sb_avg=game.sb_average_strategy();
            let bb_avg=game.bb_average_strategy();

            let sb_jam=marginal_action_probability(&sb_prior,&sb,1).unwrap();
            let bb_call=marginal_action_probability(&bb_prior,&bb,1).unwrap();
            let sb_avg_jam=marginal_action_probability(&sb_prior,&sb_avg,1).unwrap();
            let bb_avg_call=marginal_action_probability(&bb_prior,&bb_avg,1).unwrap();

            let sb_current_max=max_action_delta_on_support(&sb_prior,&prev_checkpoint_sb,&sb).unwrap();
            let bb_current_max=max_action_delta_on_support(&bb_prior,&prev_checkpoint_bb,&bb).unwrap();
            let sb_current_mae=weighted_action_mae(&sb_prior,&prev_checkpoint_sb,&sb).unwrap();
            let bb_current_mae=weighted_action_mae(&bb_prior,&prev_checkpoint_bb,&bb).unwrap();

            let sb_avg_max=max_action_delta_on_support(&sb_prior,&prev_checkpoint_sb_avg,&sb_avg).unwrap();
            let bb_avg_max=max_action_delta_on_support(&bb_prior,&prev_checkpoint_bb_avg,&bb_avg).unwrap();
            let sb_avg_mae=weighted_action_mae(&sb_prior,&prev_checkpoint_sb_avg,&sb_avg).unwrap();
            let bb_avg_mae=weighted_action_mae(&bb_prior,&prev_checkpoint_bb_avg,&bb_avg).unwrap();

            let current_eval=evaluate_restricted_sampled_nashconv(
                8.0,&sb_prior,&bb_prior,&sb,&bb,&game.blockers,&mut eval_cache,samples_per_matchup,seed
            ).expect("current strategy evaluation");
            let average_eval=evaluate_restricted_sampled_nashconv(
                8.0,&sb_prior,&bb_prior,&sb_avg,&bb_avg,&game.blockers,&mut eval_cache,samples_per_matchup,seed
            ).expect("average strategy evaluation");

            println!(
                "sweep={sweep} current_sb_jam={sb_jam:.6} current_bb_call={bb_call:.6} avg_sb_jam={sb_avg_jam:.6} avg_bb_call={bb_avg_call:.6} current_sb_max={sb_current_max:.6} current_bb_max={bb_current_max:.6} current_sb_mae={sb_current_mae:.6} current_bb_mae={bb_current_mae:.6} avg_sb_max={sb_avg_max:.6} avg_bb_max={bb_avg_max:.6} avg_sb_mae={sb_avg_mae:.6} avg_bb_mae={bb_avg_mae:.6} current_nashconv={:.6} avg_nashconv={:.6} current_sb_br_gain={:.6} current_bb_br_gain={:.6} avg_sb_br_gain={:.6} avg_bb_br_gain={:.6} current_sb_value={:.6} avg_sb_value={:.6} legal_pairs={} joint_z={:.6} mean_bb_jam_reach={:.6} train_cache_hits={} train_cache_misses={} eval_cache_hits={} eval_cache_misses={}",
                current_eval.sampled_nashconv,
                average_eval.sampled_nashconv,
                current_eval.sb_br_gain,
                current_eval.bb_br_gain,
                average_eval.sb_br_gain,
                average_eval.bb_br_gain,
                current_eval.current_sb_value,
                average_eval.current_sb_value,
                current_eval.legal_pair_count,
                current_eval.joint_normalizer,
                d.mean_bb_jam_reach,
                d.hu_cache_hits,d.hu_cache_misses,
                eval_cache.hits(),eval_cache.misses()
            );

            prev_checkpoint_sb=sb;
            prev_checkpoint_bb=bb;
            prev_checkpoint_sb_avg=sb_avg;
            prev_checkpoint_bb_avg=bb_avg;
        }
    }

    println!("NOTE: sampled NashConv is only for this restricted fixed-payoff game; not full Spin GTO and not chart data");
}
