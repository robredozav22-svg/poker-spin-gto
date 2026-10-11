use crate::accuracy_policy::AccuracyPolicy;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum SolverEvidenceKind{
    InternalMeasured,
    ExternalSolverExport,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum ConvergenceGateStatus{
    Passed,
    Failed,
    NotEvaluated,
}

#[derive(Debug,Clone,PartialEq)]
pub struct SolverRunEvidence{
    pub solver_profile_id:String,
    pub source_id:String,
    pub tree_profile_id:String,
    pub kind:SolverEvidenceKind,
    pub gate_status:ConvergenceGateStatus,
    pub acceptance_policy_id:String,
    pub iterations:Option<u64>,
    pub nashconv_bb:Option<f64>,
    pub reference_pot_bb:Option<f64>,
    pub normalized_nashconv_fraction_of_pot:Option<f64>,
    pub deterministic_repeat:bool,
    pub payoff_manifest_id:Option<String>,
    pub accuracy_evidence_id:Option<String>,
    pub independent_validation_id:Option<String>,
}

impl SolverRunEvidence{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        solver_profile_id:impl Into<String>,
        source_id:impl Into<String>,
        tree_profile_id:impl Into<String>,
        kind:SolverEvidenceKind,
        gate_status:ConvergenceGateStatus,
        acceptance_policy_id:impl Into<String>,
        iterations:Option<u64>,
        nashconv_bb:Option<f64>,
        reference_pot_bb:Option<f64>,
        normalized_nashconv_fraction_of_pot:Option<f64>,
        deterministic_repeat:bool,
        payoff_manifest_id:Option<String>,
        accuracy_evidence_id:Option<String>,
        independent_validation_id:Option<String>,
    )->Result<Self,String>{
        let solver_profile_id=solver_profile_id.into();
        let source_id=source_id.into();
        let tree_profile_id=tree_profile_id.into();
        let acceptance_policy_id=acceptance_policy_id.into();
        if solver_profile_id.trim().is_empty(){return Err("solver_profile_id must not be empty".into());}
        if source_id.trim().is_empty(){return Err("solver evidence source_id must not be empty".into());}
        if tree_profile_id.trim().is_empty(){return Err("solver evidence tree_profile_id must not be empty".into());}
        if acceptance_policy_id.trim().is_empty(){return Err("acceptance_policy_id must not be empty".into());}
        if let Some(v)=nashconv_bb{if !v.is_finite()||v<0.0{return Err("nashconv_bb must be finite and nonnegative".into());}}
        if let Some(v)=reference_pot_bb{if !v.is_finite()||v<=0.0{return Err("reference_pot_bb must be finite and positive".into());}}
        if let Some(v)=normalized_nashconv_fraction_of_pot{if !v.is_finite()||v<0.0{return Err("normalized NashConv must be finite and nonnegative".into());}}
        if let Some(i)=iterations{if i==0{return Err("iterations must be positive when supplied".into());}}
        Ok(Self{
            solver_profile_id,source_id,tree_profile_id,kind,gate_status,acceptance_policy_id,
            iterations,nashconv_bb,reference_pot_bb,normalized_nashconv_fraction_of_pot,
            deterministic_repeat,payoff_manifest_id,accuracy_evidence_id,independent_validation_id,
        })
    }

    pub fn validate_promotion_ready(&self,expected_tree_profile_id:&str,policy:&AccuracyPolicy)->Result<(),String>{
        if self.tree_profile_id!=expected_tree_profile_id{return Err("solver evidence tree profile does not match strategy tree profile".into());}
        if self.acceptance_policy_id!=policy.id{return Err("solver evidence acceptance policy does not match supplied AccuracyPolicy".into());}
        if self.gate_status!=ConvergenceGateStatus::Passed{return Err("solver convergence gate has not passed".into());}

        let normalized=self.normalized_nashconv_fraction_of_pot
            .ok_or_else(||"promotion evidence requires normalized convergence/exploitability fraction of pot".to_string())?;
        if normalized>policy.max_fraction_of_pot{
            return Err(format!("normalized convergence {:.12} exceeds policy maximum {:.12}",normalized,policy.max_fraction_of_pot));
        }

        if policy.require_independent_validation{
            if self.independent_validation_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){
                return Err("accuracy policy requires independent validation evidence".into());
            }
        }

        match self.kind{
            SolverEvidenceKind::InternalMeasured=>{
                if self.iterations.is_none(){return Err("internal solver evidence requires measured iterations".into());}
                let nashconv=self.nashconv_bb.ok_or_else(||"internal solver evidence requires measured NashConv".to_string())?;
                let pot=self.reference_pot_bb.ok_or_else(||"internal solver evidence requires reference pot".to_string())?;
                let recomputed=nashconv/pot;
                if (recomputed-normalized).abs()>1e-12{
                    return Err(format!("normalized NashConv mismatch: supplied {:.12}, recomputed {:.12}",normalized,recomputed));
                }
                if policy.require_deterministic_repeat_for_internal && !self.deterministic_repeat{
                    return Err("accuracy policy requires deterministic repeat PASS for internal solver".into());
                }
                if self.payoff_manifest_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){return Err("internal solver evidence requires payoff manifest provenance".into());}
            }
            SolverEvidenceKind::ExternalSolverExport=>{
                if self.accuracy_evidence_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){return Err("external solver export requires accuracy evidence".into());}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::accuracy_policy::AccuracyPolicy;

    fn policy()->AccuracyPolicy{AccuracyPolicy::new("policy-v1",0.0001,true,true,"fixture").unwrap()}

    fn good_internal()->SolverRunEvidence{
        SolverRunEvidence::new(
            "internal-cfr","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Passed,"policy-v1",
            Some(10_000),Some(0.00012),Some(1.5),Some(0.00008),true,Some("payoff-manifest".into()),None,Some("independent-br".into()),
        ).unwrap()
    }

    #[test]
    fn internal_promotion_requires_measured_convergence_and_manifest(){good_internal().validate_promotion_ready("tree",&policy()).unwrap();}

    #[test]
    fn internal_without_nashconv_fails_closed(){
        let e=SolverRunEvidence::new(
            "internal-cfr","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Passed,"policy-v1",
            Some(10_000),None,Some(1.5),Some(0.00008),true,Some("payoff-manifest".into()),None,Some("independent".into()),
        ).unwrap();
        assert!(e.validate_promotion_ready("tree",&policy()).is_err());
    }

    #[test]
    fn normalized_metric_must_match_nashconv_divided_by_pot(){
        let mut e=good_internal();e.normalized_nashconv_fraction_of_pot=Some(0.00007);
        let err=e.validate_promotion_ready("tree",&policy()).unwrap_err();
        assert!(err.contains("normalized NashConv mismatch"));
    }

    #[test]
    fn threshold_is_enforced_even_when_gate_status_says_passed(){
        let e=SolverRunEvidence::new(
            "internal","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Passed,"policy-v1",
            Some(10_000),Some(0.0003),Some(1.5),Some(0.0002),true,Some("manifest".into()),None,Some("independent".into()),
        ).unwrap();
        let err=e.validate_promotion_ready("tree",&policy()).unwrap_err();
        assert!(err.contains("exceeds policy maximum"));
    }

    #[test]
    fn wrong_policy_id_fails_closed(){
        let mut e=good_internal();e.acceptance_policy_id="other".into();
        assert!(e.validate_promotion_ready("tree",&policy()).is_err());
    }

    #[test]
    fn independent_validation_is_required_by_policy(){
        let mut e=good_internal();e.independent_validation_id=None;
        assert!(e.validate_promotion_ready("tree",&policy()).is_err());
    }

    #[test]
    fn external_export_requires_accuracy_and_independent_validation(){
        let e=SolverRunEvidence::new(
            "wizard-export","export","tree",SolverEvidenceKind::ExternalSolverExport,ConvergenceGateStatus::Passed,"policy-v1",
            None,None,None,Some(0.00009),false,None,Some("independent-accuracy".into()),Some("holdout-check".into()),
        ).unwrap();
        e.validate_promotion_ready("tree",&policy()).unwrap();
    }

    #[test]
    fn external_export_without_holdout_fails_closed(){
        let e=SolverRunEvidence::new(
            "external","export","tree",SolverEvidenceKind::ExternalSolverExport,ConvergenceGateStatus::Passed,"policy-v1",
            None,None,None,Some(0.00009),false,None,Some("accuracy".into()),None,
        ).unwrap();
        assert!(e.validate_promotion_ready("tree",&policy()).is_err());
    }

    #[test]
    fn failed_gate_never_promotes(){
        let e=SolverRunEvidence::new(
            "internal","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Failed,"policy-v1",
            Some(1),Some(0.00012),Some(1.5),Some(0.00008),true,Some("manifest".into()),None,Some("independent".into()),
        ).unwrap();
        assert!(e.validate_promotion_ready("tree",&policy()).is_err());
    }
}
