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
    let mut train_eval_cache=EquityCache::new();
    let mut holdout_eval_cache=EquityCache::new();

    let train_samples=10_000u64;
    let train_seed=20260907u64;
    let holdout_samples=50_000u64;
    let holdout_seed=20260917u64;
    let checkpoints=[1usize,5,10,20,50,100,200,500,1000];

    let mut prev_sb_avg=game.sb_average_strategy();
    let mut prev_bb_avg=game.bb_average_strategy();

    println!("status=RESEARCH_ONLY stack_bb=8 train_samples={train_samples} train_seed={train_seed} holdout_samples={holdout_samples} holdout_seed={holdout_seed}");
    println!("support sb=3 bb=3 sweeps=1000 checkpoints=1,5,10,20,50,100,200,500,1000");

    for sweep in 1..=1000{
        let d=game.sweep(train_samples,train_seed).expect("sweep");
        if checkpoints.contains(&sweep){
            let sb_avg=game.sb_average_strategy();
            let bb_avg=game.bb_average_strategy();
            let sb_avg_jam=marginal_action_probability(&sb_prior,&sb_avg,1).unwrap();
            let bb_avg_call=marginal_action_probability(&bb_prior,&bb_avg,1).unwrap();
            let sb_avg_max=max_action_delta_on_support(&sb_prior,&prev_sb_avg,&sb_avg).unwrap();
            let bb_avg_max=max_action_delta_on_support(&bb_prior,&prev_bb_avg,&bb_avg).unwrap();
            let sb_avg_mae=weighted_action_mae(&sb_prior,&prev_sb_avg,&sb_avg).unwrap();
            let bb_avg_mae=weighted_action_mae(&bb_prior,&prev_bb_avg,&bb_avg).unwrap();

            let train=evaluate_restricted_sampled_nashconv(
                8.0,&sb_prior,&bb_prior,&sb_avg,&bb_avg,&game.blockers,&mut train_eval_cache,train_samples,train_seed
            ).expect("train evaluation");
            let holdout=evaluate_restricted_sampled_nashconv(
                8.0,&sb_prior,&bb_prior,&sb_avg,&bb_avg,&game.blockers,&mut holdout_eval_cache,holdout_samples,holdout_seed
            ).expect("holdout evaluation");

            println!(
                "sweep={sweep} avg_sb_jam={sb_avg_jam:.6} avg_bb_call={bb_avg_call:.6} avg_sb_max={sb_avg_max:.6} avg_bb_max={bb_avg_max:.6} avg_sb_mae={sb_avg_mae:.6} avg_bb_mae={bb_avg_mae:.6} train_nashconv={:.6} holdout_nashconv={:.6} train_sb_br_gain={:.6} train_bb_br_gain={:.6} holdout_sb_br_gain={:.6} holdout_bb_br_gain={:.6} train_sb_value={:.6} holdout_sb_value={:.6} nashconv_gap={:.6} sb_value_gap={:.6} legal_pairs={} joint_z={:.6} mean_bb_jam_reach={:.6} train_cache_hits={} train_cache_misses={} holdout_cache_hits={} holdout_cache_misses={}",
                train.sampled_nashconv,
                holdout.sampled_nashconv,
                train.sb_br_gain,train.bb_br_gain,
                holdout.sb_br_gain,holdout.bb_br_gain,
                train.current_sb_value,holdout.current_sb_value,
                (holdout.sampled_nashconv-train.sampled_nashconv).abs(),
                (holdout.current_sb_value-train.current_sb_value).abs(),
                train.legal_pair_count,train.joint_normalizer,
                d.mean_bb_jam_reach,
                train_eval_cache.hits(),train_eval_cache.misses(),
                holdout_eval_cache.hits(),holdout_eval_cache.misses()
            );

            prev_sb_avg=sb_avg;
            prev_bb_avg=bb_avg;
        }
    }

    println!("NOTE: train/holdout NashConv applies only to this restricted sampled-payoff subgame; not full Spin GTO and not chart data");
}
