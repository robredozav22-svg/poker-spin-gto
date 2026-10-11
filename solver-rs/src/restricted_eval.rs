use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::equity_cache::EquityCache;
use crate::range::ComboRange;
use crate::terminal::{Seat,TerminalId,TerminalPot};
use crate::terminal_ev::expected_two_active_payoff;
use crate::StrategySnapshot;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct RestrictedExploitabilityReport{
    pub current_sb_value:f64,
    pub sb_best_response_value:f64,
    pub sb_value_vs_bb_best_response:f64,
    pub sb_br_gain:f64,
    pub bb_br_gain:f64,
    pub sampled_nashconv:f64,
    pub legal_pair_count:usize,
    pub joint_normalizer:f64,
    pub samples_per_matchup:u64,
    pub seed:u64,
}

fn validate_strategy(s:&StrategySnapshot)->Result<(),String>{
    if s.infosets!=COMBO_COUNT{return Err(format!("expected {COMBO_COUNT} combo infosets"));}
    if s.actions!=2{return Err("restricted strategy must have exactly [Fold,Aggressive] actions".into());}
    if s.probabilities.len()!=COMBO_COUNT*2{return Err("strategy shape mismatch".into());}
    for i in 0..COMBO_COUNT{
        let row=s.row(i);
        if row.iter().any(|p|!p.is_finite()||*p<0.0||*p>1.0){return Err(format!("invalid probability at combo {i}"));}
        if (row.iter().sum::<f64>()-1.0).abs()>1e-9{return Err(format!("strategy row {i} does not sum to 1"));}
    }
    Ok(())
}

/// Best-response diagnostic for the restricted two-player subgame after BTN
/// folds: SB [Fold,Jam] -> BB [Fold,Call].
///
/// Private-hand probability is the normalized legal joint distribution
///   P(sb,bb) ∝ sb_prior(sb) * bb_prior(bb) * I(disjoint cards).
/// Showdown values come from one deterministic sampled equity game identified
/// by (samples_per_matchup, seed). Therefore `sampled_nashconv` is a NashConv
/// diagnostic only for this restricted sampled-payoff game, not full Spin GTO.
pub fn evaluate_restricted_sampled_nashconv(
    stack_bb:f64,
    sb_prior:&ComboRange,
    bb_prior:&ComboRange,
    sb_strategy:&StrategySnapshot,
    bb_strategy:&StrategySnapshot,
    blockers:&BlockerMatrix,
    cache:&mut EquityCache,
    samples_per_matchup:u64,
    seed:u64,
)->Result<RestrictedExploitabilityReport,String>{
    if stack_bb<1.0{return Err("stack must be at least 1bb".into());}
    if samples_per_matchup==0{return Err("samples_per_matchup must be positive".into());}
    validate_strategy(sb_strategy)?;validate_strategy(bb_strategy)?;

    let sb_support:Vec<usize>=sb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();
    let bb_support:Vec<usize>=bb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();
    let mut z=0.0;let mut legal_pair_count=0usize;
    for &i in &sb_support{for &j in &bb_support{if blockers.compatible(i,j){z+=sb_prior.weights()[i]*bb_prior.weights()[j];legal_pair_count+=1;}}}
    if z<=f64::EPSILON{return Err("no legal SB/BB private-hand pairs".into());}

    let fold_pot=TerminalPot::for_terminal(TerminalId::BtnFoldSbFold,stack_bb);
    let fold_ev_sb=fold_pot.settle(&[Seat::Bb])[Seat::Sb as usize];
    let jam_fold_pot=TerminalPot::for_terminal(TerminalId::BtnFoldSbJamBbFold,stack_bb);
    let jam_when_bb_folds=jam_fold_pot.settle(&[Seat::Sb])[Seat::Sb as usize];
    let call_pot=TerminalPot::for_terminal(TerminalId::BtnFoldSbJamBbCall,stack_bb);
    let combos=all_combos();

    let mut current=0.0;
    let mut sb_marginal=vec![0.0;COMBO_COUNT];
    let mut sb_jam_value_mass=vec![0.0;COMBO_COUNT];
    let mut fixed_sb_fold_mass=vec![0.0;COMBO_COUNT];
    let mut bb_if_fold_mass=vec![0.0;COMBO_COUNT];
    let mut bb_if_call_mass=vec![0.0;COMBO_COUNT];

    for &i in &sb_support{
        let s_jam=sb_strategy.row(i)[1];
        for &j in &bb_support{
            if !blockers.compatible(i,j){continue;}
            let p=sb_prior.weights()[i]*bb_prior.weights()[j]/z;
            let c_call=bb_strategy.row(j)[1];
            let eq=cache.get_or_compute(combos[i],combos[j],samples_per_matchup,seed)?;
            let showdown=expected_two_active_payoff(&call_pot,Seat::Sb,Seat::Bb,eq.hero)[Seat::Sb as usize];
            let jam_vs_bb=(1.0-c_call)*jam_when_bb_folds+c_call*showdown;
            let pair_value=(1.0-s_jam)*fold_ev_sb+s_jam*jam_vs_bb;
            current+=p*pair_value;

            sb_marginal[i]+=p;
            sb_jam_value_mass[i]+=p*jam_vs_bb;

            fixed_sb_fold_mass[j]+=p*(1.0-s_jam)*fold_ev_sb;
            bb_if_fold_mass[j]+=p*s_jam*jam_when_bb_folds;
            bb_if_call_mass[j]+=p*s_jam*showdown;
        }
    }

    let mut sb_br=0.0;
    for &i in &sb_support{
        if sb_marginal[i]<=f64::EPSILON{continue;}
        let fold_mass=sb_marginal[i]*fold_ev_sb;
        sb_br+=fold_mass.max(sb_jam_value_mass[i]);
    }

    let mut sb_vs_bb_br=0.0;
    for &j in &bb_support{
        sb_vs_bb_br+=fixed_sb_fold_mass[j]+bb_if_fold_mass[j].min(bb_if_call_mass[j]);
    }

    let sb_br_gain=(sb_br-current).max(0.0);
    let bb_br_gain=(current-sb_vs_bb_br).max(0.0);
    Ok(RestrictedExploitabilityReport{
        current_sb_value:current,
        sb_best_response_value:sb_br,
        sb_value_vs_bb_best_response:sb_vs_bb_br,
        sb_br_gain,
        bb_br_gain,
        sampled_nashconv:sb_br_gain+bb_br_gain,
        legal_pair_count,
        joint_normalizer:z,
        samples_per_matchup,
        seed,
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    fn one_combo_range(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}
    fn strategy(fold:f64)->StrategySnapshot{let mut p=Vec::with_capacity(COMBO_COUNT*2);for _ in 0..COMBO_COUNT{p.push(fold);p.push(1.0-fold);}StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities:p}}

    #[test]
    fn one_legal_pair_has_nonnegative_br_gaps_and_exact_joint_mass(){
        let blockers=BlockerMatrix::build();let sb=0usize;let bb=(0..COMBO_COUNT).find(|j|blockers.compatible(sb,*j)).unwrap();let mut cache=EquityCache::new();
        let r=evaluate_restricted_sampled_nashconv(8.0,&one_combo_range(sb),&one_combo_range(bb),&strategy(0.5),&strategy(0.5),&blockers,&mut cache,100,7).unwrap();
        assert_eq!(r.legal_pair_count,1);assert!((r.joint_normalizer-1.0).abs()<1e-12);assert!(r.sb_br_gain>=0.0);assert!(r.bb_br_gain>=0.0);assert!((r.sampled_nashconv-r.sb_br_gain-r.bb_br_gain).abs()<1e-12);
    }

    #[test]
    fn illegal_joint_support_fails_closed(){
        let blockers=BlockerMatrix::build();let r=one_combo_range(0);let mut cache=EquityCache::new();
        assert!(evaluate_restricted_sampled_nashconv(8.0,&r,&r,&strategy(0.5),&strategy(0.5),&blockers,&mut cache,10,1).is_err());
    }
}
