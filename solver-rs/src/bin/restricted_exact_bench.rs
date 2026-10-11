use spins_solver_core::cards::{all_combos,COMBO_COUNT};
use spins_solver_core::class_aggregation::class_label;
use spins_solver_core::range::ComboRange;
use spins_solver_core::restricted_exact::ExactRestrictedSubgame;
use spins_solver_core::strategy_metrics::{marginal_action_probability,max_action_delta_on_support,weighted_action_mae};

fn range_for_labels(labels:&[&str])->ComboRange{
    let combos=all_combos();let mut w=vec![0.0;COMBO_COUNT];
    for label in labels{
        let i=combos.iter().position(|c|class_label(*c)==*label).unwrap();w[i]=1.0;
    }
    ComboRange::from_weights(w).unwrap()
}

fn main(){
    let sb_prior=range_for_labels(&["AA","A5s","76s"]);
    let bb_prior=range_for_labels(&["KK","AQo","65s"]);
    let mut game=ExactRestrictedSubgame::new(8.0,sb_prior.clone(),bb_prior.clone()).expect("exact game build");
    let checkpoints=[1usize,5,10,20,50,100,200,500,1000,2000,5000,10000];
    let mut prev_sb=game.sb_average_strategy();let mut prev_bb=game.bb_average_strategy();
    println!("status=RESEARCH_ONLY mode=EXACT_PAYOFF stack_bb=8 legal_pairs={} joint_z={:.6} exact_cache_misses={} exact_cache_hits={} sweeps=10000",game.pairs.len(),game.joint_normalizer,game.equity_cache.misses(),game.equity_cache.hits());
    for sweep in 1..=10_000{
        game.sweep().expect("exact sweep");
        if checkpoints.contains(&sweep){
            let sb=game.sb_average_strategy();let bb=game.bb_average_strategy();
            let r=game.evaluate(&sb,&bb).expect("exact evaluation");
            let sb_jam=marginal_action_probability(&sb_prior,&sb,1).unwrap();
            let bb_call=marginal_action_probability(&bb_prior,&bb,1).unwrap();
            let sb_max=max_action_delta_on_support(&sb_prior,&prev_sb,&sb).unwrap();
            let bb_max=max_action_delta_on_support(&bb_prior,&prev_bb,&bb).unwrap();
            let sb_mae=weighted_action_mae(&sb_prior,&prev_sb,&sb).unwrap();
            let bb_mae=weighted_action_mae(&bb_prior,&prev_bb,&bb).unwrap();
            println!("sweep={sweep} avg_sb_jam={sb_jam:.9} avg_bb_call={bb_call:.9} sb_max_delta={sb_max:.9} bb_max_delta={bb_max:.9} sb_weighted_mae={sb_mae:.9} bb_weighted_mae={bb_mae:.9} exact_nashconv={:.9} sb_br_gain={:.9} bb_br_gain={:.9} sb_value={:.9}",r.nashconv,r.sb_br_gain,r.bb_br_gain,r.sb_value);
            prev_sb=sb;prev_bb=bb;
        }
    }
    println!("NOTE: exact NashConv is for the exact-payoff restricted SB Fold/Jam vs BB Fold/Call game only; not full Spin GTO and not chart data");
}
