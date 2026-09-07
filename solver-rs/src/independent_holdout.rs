use crate::benchmark_claim::IndependentAccuracyMeasurement;
use crate::restricted_exact::{ExactRestrictedReport,ExactRestrictedSubgame};
use crate::StrategySnapshot;

#[derive(Debug,Clone,PartialEq)]
pub struct ExactHoldoutRun{
    pub report:ExactRestrictedReport,
    pub normalized_exploitability_fraction_of_pot:f64,
}

pub fn evaluate_exact_holdout_repeated(
    subgame:&ExactRestrictedSubgame,
    sb:&StrategySnapshot,
    bb:&StrategySnapshot,
    reference_pot_bb:f64,
    repetitions:u32,
    subject_id:impl Into<String>,
    tree_profile_id:impl Into<String>,
    evaluator_profile_id:impl Into<String>,
    evaluator_source_id:impl Into<String>,
    artifact_checksum:impl Into<String>,
)->Result<(ExactHoldoutRun,IndependentAccuracyMeasurement),String>{
    if !reference_pot_bb.is_finite()||reference_pot_bb<=0.0{return Err("reference_pot_bb must be finite and positive".into());}
    if repetitions<3{return Err("independent exact holdout requires at least 3 repetitions".into());}

    let first=subgame.evaluate(sb,bb)?;
    let normalized=first.nashconv/reference_pot_bb;
    if !normalized.is_finite()||normalized<0.0{return Err("normalized exploitability is invalid".into());}

    for _ in 1..repetitions{
        let next=subgame.evaluate(sb,bb)?;
        assert_repeat_equivalent(&first,&next)?;
    }

    let measurement=IndependentAccuracyMeasurement{
        subject_id:subject_id.into(),
        tree_profile_id:tree_profile_id.into(),
        evaluator_profile_id:evaluator_profile_id.into(),
        evaluator_source_id:evaluator_source_id.into(),
        artifact_checksum:artifact_checksum.into(),
        evaluated_states:first.legal_pairs as u64,
        repetitions,
        normalized_exploitability_fraction_of_pot:normalized,
    };
    measurement.validate()?;
    Ok((ExactHoldoutRun{report:first,normalized_exploitability_fraction_of_pot:normalized},measurement))
}

fn assert_repeat_equivalent(a:&ExactRestrictedReport,b:&ExactRestrictedReport)->Result<(),String>{
    if a.legal_pairs!=b.legal_pairs{return Err("exact holdout repeat changed legal pair count".into());}
    if (a.joint_normalizer-b.joint_normalizer).abs()>1e-15{return Err("exact holdout repeat changed joint normalizer".into());}
    for (name,x,y) in [
        ("sb_value",a.sb_value,b.sb_value),
        ("sb_best_response_value",a.sb_best_response_value,b.sb_best_response_value),
        ("sb_value_vs_bb_best_response",a.sb_value_vs_bb_best_response,b.sb_value_vs_bb_best_response),
        ("sb_br_gain",a.sb_br_gain,b.sb_br_gain),
        ("bb_br_gain",a.bb_br_gain,b.bb_br_gain),
        ("nashconv",a.nashconv,b.nashconv),
    ]{
        if (x-y).abs()>1e-15{return Err(format!("exact holdout repeat mismatch in {name}: {x} vs {y}"));}
    }
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::COMBO_COUNT;
    use crate::range::ComboRange;

    fn uniform_strategy(fold:f64)->StrategySnapshot{
        let mut probabilities=Vec::with_capacity(COMBO_COUNT*2);
        for _ in 0..COMBO_COUNT{probabilities.push(fold);probabilities.push(1.0-fold);}
        StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities}
    }

    fn one_combo_range(i:usize)->ComboRange{
        let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()
    }

    #[test]
    fn repeated_exact_holdout_emits_valid_measurement(){
        let probe=ExactRestrictedSubgame::new(8.0,one_combo_range(0),one_combo_range(10)).or_else(|_|{
            let blockers=crate::blockers::BlockerMatrix::build();
            let j=(0..COMBO_COUNT).find(|j|blockers.compatible(0,*j)).unwrap();
            ExactRestrictedSubgame::new(8.0,one_combo_range(0),one_combo_range(j))
        }).unwrap();
        let s=uniform_strategy(0.5);
        let (run,m)=evaluate_exact_holdout_repeated(
            &probe,&s,&s,1.5,3,"candidate","fixture-tree","exact-restricted-br-v1","independent-holdout","checksum-a",
        ).unwrap();
        assert_eq!(m.repetitions,3);
        assert_eq!(m.evaluated_states,run.report.legal_pairs as u64);
        assert!((m.normalized_exploitability_fraction_of_pot-run.report.nashconv/1.5).abs()<1e-15);
    }

    #[test]
    fn fewer_than_three_repetitions_is_rejected(){
        let blockers=crate::blockers::BlockerMatrix::build();
        let j=(0..COMBO_COUNT).find(|j|blockers.compatible(0,*j)).unwrap();
        let sub=ExactRestrictedSubgame::new(8.0,one_combo_range(0),one_combo_range(j)).unwrap();
        let s=uniform_strategy(0.5);
        assert!(evaluate_exact_holdout_repeated(&sub,&s,&s,1.5,2,"x","t","e","s","c").is_err());
    }

    #[test]
    fn invalid_reference_pot_is_rejected(){
        let blockers=crate::blockers::BlockerMatrix::build();
        let j=(0..COMBO_COUNT).find(|j|blockers.compatible(0,*j)).unwrap();
        let sub=ExactRestrictedSubgame::new(8.0,one_combo_range(0),one_combo_range(j)).unwrap();
        let s=uniform_strategy(0.5);
        assert!(evaluate_exact_holdout_repeated(&sub,&s,&s,0.0,3,"x","t","e","s","c").is_err());
    }
}
