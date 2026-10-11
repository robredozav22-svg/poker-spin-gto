use crate::benchmark_claim::IndependentAccuracyMeasurement;
use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::payoff_lookup::HuPayoffLookup;
use crate::range::ComboRange;
use crate::StrategySnapshot;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct IndependentExactRestrictedReport{
    pub current_sb_value_bb:f64,
    pub sb_best_response_value_bb:f64,
    pub sb_value_vs_bb_best_response_bb:f64,
    pub sb_br_gain_bb:f64,
    pub bb_br_gain_bb:f64,
    pub nashconv_bb:f64,
    pub reference_pot_bb:f64,
    pub normalized_nashconv_fraction_of_pot:f64,
    pub legal_pairs:usize,
    pub joint_normalizer:f64,
}

fn validate_strategy(s:&StrategySnapshot)->Result<(),String>{
    if s.infosets!=COMBO_COUNT||s.actions!=2||s.probabilities.len()!=COMBO_COUNT*2{
        return Err("restricted strategy must be 1326 x [Fold,Aggressive]".into());
    }
    for i in 0..COMBO_COUNT{
        let r=s.row(i);
        if r.iter().any(|p|!p.is_finite()||*p<0.0||*p>1.0){return Err(format!("invalid probability at combo {i}"));}
        if (r[0]+r[1]-1.0).abs()>1e-9{return Err(format!("strategy row {i} does not sum to 1"));}
    }
    Ok(())
}

/// Independent exact evaluator for the validated restricted node:
/// BTN folds -> SB [Fold,Jam] -> BB [Fold,Call].
///
/// It does not read solver regrets, average-strategy accumulators, or any solver
/// evaluation cache. All showdown values come from an immutable exact payoff
/// lookup. Missing payoff records fail closed.
pub fn evaluate_btn_fold_sb_jam_bb_call_exact(
    stack_bb:f64,
    sb_prior:&ComboRange,
    bb_prior:&ComboRange,
    sb_strategy:&StrategySnapshot,
    bb_strategy:&StrategySnapshot,
    blockers:&BlockerMatrix,
    lookup:&HuPayoffLookup,
)->Result<IndependentExactRestrictedReport,String>{
    if stack_bb<1.0{return Err("stack must be at least 1bb".into());}
    validate_strategy(sb_strategy)?;validate_strategy(bb_strategy)?;
    let combos=all_combos();
    let sb_support:Vec<usize>=sb_prior.weights().iter().enumerate().filter_map(|(i,w)|(*w>0.0).then_some(i)).collect();
    let bb_support:Vec<usize>=bb_prior.weights().iter().enumerate().filter_map(|(i,w)|(*w>0.0).then_some(i)).collect();

    let mut z=0.0;let mut legal_pairs=0usize;
    for &i in &sb_support{for &j in &bb_support{if blockers.compatible(i,j){z+=sb_prior.weights()[i]*bb_prior.weights()[j];legal_pairs+=1;}}}
    if z<=f64::EPSILON{return Err("no legal private-hand pairs".into());}

    let sb_fold_ev=-0.5f64;
    let sb_jam_bb_fold_ev=1.0f64;

    let mut current=0.0;
    let mut sb_marginal=vec![0.0;COMBO_COUNT];
    let mut sb_jam_value_mass=vec![0.0;COMBO_COUNT];
    let mut fixed_sb_fold_mass=vec![0.0;COMBO_COUNT];
    let mut sb_if_bb_fold_mass=vec![0.0;COMBO_COUNT];
    let mut sb_if_bb_call_mass=vec![0.0;COMBO_COUNT];

    for &i in &sb_support{
        let jam=sb_strategy.row(i)[1];
        for &j in &bb_support{
            if !blockers.compatible(i,j){continue;}
            let p=sb_prior.weights()[i]*bb_prior.weights()[j]/z;
            let call=bb_strategy.row(j)[1];
            let eq=lookup.get_equity(combos[i],combos[j])?
                .ok_or_else(||format!("missing exact HU payoff for independent evaluator sb={i} bb={j}"))?;
            let showdown=eq.hero*(2.0*stack_bb)-stack_bb;
            let jam_value=(1.0-call)*sb_jam_bb_fold_ev+call*showdown;
            current+=p*((1.0-jam)*sb_fold_ev+jam*jam_value);

            sb_marginal[i]+=p;
            sb_jam_value_mass[i]+=p*jam_value;
            fixed_sb_fold_mass[j]+=p*(1.0-jam)*sb_fold_ev;
            sb_if_bb_fold_mass[j]+=p*jam*sb_jam_bb_fold_ev;
            sb_if_bb_call_mass[j]+=p*jam*showdown;
        }
    }

    let mut sb_br=0.0;
    for &i in &sb_support{
        if sb_marginal[i]<=f64::EPSILON{continue;}
        sb_br+=(sb_marginal[i]*sb_fold_ev).max(sb_jam_value_mass[i]);
    }

    let mut sb_vs_bb_br=0.0;
    for &j in &bb_support{
        sb_vs_bb_br+=fixed_sb_fold_mass[j]+sb_if_bb_fold_mass[j].min(sb_if_bb_call_mass[j]);
    }

    let sb_gain=(sb_br-current).max(0.0);
    let bb_gain=(current-sb_vs_bb_br).max(0.0);
    let nashconv=sb_gain+bb_gain;
    let reference_pot_bb=1.5;
    Ok(IndependentExactRestrictedReport{
        current_sb_value_bb:current,
        sb_best_response_value_bb:sb_br,
        sb_value_vs_bb_best_response_bb:sb_vs_bb_br,
        sb_br_gain_bb:sb_gain,
        bb_br_gain_bb:bb_gain,
        nashconv_bb:nashconv,
        reference_pot_bb,
        normalized_nashconv_fraction_of_pot:nashconv/reference_pot_bb,
        legal_pairs,
        joint_normalizer:z,
    })
}

pub fn measure_btn_fold_sb_jam_bb_call_exact_repeated(
    stack_bb:f64,
    sb_prior:&ComboRange,
    bb_prior:&ComboRange,
    sb_strategy:&StrategySnapshot,
    bb_strategy:&StrategySnapshot,
    blockers:&BlockerMatrix,
    lookup:&HuPayoffLookup,
    repetitions:u32,
    subject_id:impl Into<String>,
    tree_profile_id:impl Into<String>,
    artifact_checksum:impl Into<String>,
)->Result<IndependentAccuracyMeasurement,String>{
    if repetitions<3{return Err("independent benchmark requires at least 3 exact repetitions".into());}
    let first=evaluate_btn_fold_sb_jam_bb_call_exact(stack_bb,sb_prior,bb_prior,sb_strategy,bb_strategy,blockers,lookup)?;
    for _ in 1..repetitions{
        let next=evaluate_btn_fold_sb_jam_bb_call_exact(stack_bb,sb_prior,bb_prior,sb_strategy,bb_strategy,blockers,lookup)?;
        if first!=next{return Err("exact independent evaluator is not deterministic across repetitions".into());}
    }
    let m=IndependentAccuracyMeasurement{
        subject_id:subject_id.into(),
        tree_profile_id:tree_profile_id.into(),
        evaluator_profile_id:"independent-exact-btnfold-sbjam-bbcall-v1".into(),
        evaluator_source_id:"solver-rs/src/independent_exact_eval.rs".into(),
        artifact_checksum:artifact_checksum.into(),
        evaluated_states:first.legal_pairs as u64,
        repetitions,
        normalized_exploitability_fraction_of_pot:first.normalized_nashconv_fraction_of_pot,
    };
    m.validate()?;
    Ok(m)
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;
    use crate::equity::canonical_hu_matchup;
    use crate::exact_equity::exact_hu_equity;
    use crate::payoff_table::{HuPayoffRecord,HuPayoffTable};

    fn c(r:u8,s:u8)->Card{r*4+s}
    fn idx(target:[Card;2])->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
    fn singleton(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}
    fn strategy(aggressive:f64)->StrategySnapshot{let mut p=Vec::with_capacity(COMBO_COUNT*2);for _ in 0..COMBO_COUNT{p.push(1.0-aggressive);p.push(aggressive);}StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities:p}}

    fn fixture()->([Card;2],[Card;2],HuPayoffLookup,BlockerMatrix){
        let aa=[c(12,0),c(12,1)];let kk=[c(11,2),c(11,3)];
        let e=exact_hu_equity(aa,kk).unwrap();
        let table=HuPayoffTable{records:vec![HuPayoffRecord{key:canonical_hu_matchup(aa,kk).unwrap(),wins:e.wins,losses:e.losses,ties:e.ties}]};
        (aa,kk,HuPayoffLookup::from_table(table).unwrap(),BlockerMatrix::build())
    }

    #[test]
    fn exact_single_pair_report_is_self_consistent(){
        let (aa,kk,lookup,blockers)=fixture();
        let r=evaluate_btn_fold_sb_jam_bb_call_exact(8.0,&singleton(idx(aa)),&singleton(idx(kk)),&strategy(0.5),&strategy(0.5),&blockers,&lookup).unwrap();
        assert_eq!(r.legal_pairs,1);assert!((r.joint_normalizer-1.0).abs()<1e-12);
        assert!(r.sb_br_gain_bb>=0.0&&r.bb_br_gain_bb>=0.0);
        assert!((r.nashconv_bb-r.sb_br_gain_bb-r.bb_br_gain_bb).abs()<1e-12);
        assert!((r.normalized_nashconv_fraction_of_pot-r.nashconv_bb/1.5).abs()<1e-15);
    }

    #[test]
    fn repeated_exact_measurement_is_valid_and_deterministic(){
        let (aa,kk,lookup,blockers)=fixture();
        let m=measure_btn_fold_sb_jam_bb_call_exact_repeated(
            8.0,&singleton(idx(aa)),&singleton(idx(kk)),&strategy(0.5),&strategy(0.5),&blockers,&lookup,
            3,"candidate","fixture-tree","artifact-a",
        ).unwrap();
        assert_eq!(m.repetitions,3);assert_eq!(m.evaluated_states,1);
        assert!(m.normalized_exploitability_fraction_of_pot>=0.0);
    }

    #[test]
    fn repeated_measurement_rejects_too_few_runs(){
        let (aa,kk,lookup,blockers)=fixture();
        assert!(measure_btn_fold_sb_jam_bb_call_exact_repeated(
            8.0,&singleton(idx(aa)),&singleton(idx(kk)),&strategy(0.5),&strategy(0.5),&blockers,&lookup,
            2,"candidate","fixture-tree","artifact-a",
        ).is_err());
    }

    #[test]
    fn missing_exact_payoff_fails_closed(){
        let blockers=BlockerMatrix::build();let sb=0usize;let bb=(0..COMBO_COUNT).find(|j|blockers.compatible(sb,*j)).unwrap();
        let lookup=HuPayoffLookup::from_table(HuPayoffTable{records:vec![]}).unwrap();
        assert!(evaluate_btn_fold_sb_jam_bb_call_exact(8.0,&singleton(sb),&singleton(bb),&strategy(0.5),&strategy(0.5),&blockers,&lookup).is_err());
    }
}
