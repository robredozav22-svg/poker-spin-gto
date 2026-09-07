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
    pub deterministic_repeat:bool,
    pub payoff_manifest_id:Option<String>,
    pub accuracy_evidence_id:Option<String>,
    pub independent_validation_id:Option<String>,
}

impl SolverRunEvidence{
    pub fn new(
        solver_profile_id:impl Into<String>,
        source_id:impl Into<String>,
        tree_profile_id:impl Into<String>,
        kind:SolverEvidenceKind,
        gate_status:ConvergenceGateStatus,
        acceptance_policy_id:impl Into<String>,
        iterations:Option<u64>,
        nashconv_bb:Option<f64>,
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
        if let Some(i)=iterations{if i==0{return Err("iterations must be positive when supplied".into());}}
        Ok(Self{solver_profile_id,source_id,tree_profile_id,kind,gate_status,acceptance_policy_id,iterations,nashconv_bb,deterministic_repeat,payoff_manifest_id,accuracy_evidence_id,independent_validation_id})
    }

    pub fn validate_promotion_ready(&self,expected_tree_profile_id:&str)->Result<(),String>{
        if self.tree_profile_id!=expected_tree_profile_id{return Err("solver evidence tree profile does not match strategy tree profile".into());}
        if self.gate_status!=ConvergenceGateStatus::Passed{return Err("solver convergence gate has not passed".into());}
        match self.kind{
            SolverEvidenceKind::InternalMeasured=>{
                if self.iterations.is_none(){return Err("internal solver evidence requires measured iterations".into());}
                if self.nashconv_bb.is_none(){return Err("internal solver evidence requires measured NashConv".into());}
                if !self.deterministic_repeat{return Err("internal solver evidence requires deterministic repeat PASS".into());}
                if self.payoff_manifest_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){return Err("internal solver evidence requires payoff manifest provenance".into());}
            }
            SolverEvidenceKind::ExternalSolverExport=>{
                if self.accuracy_evidence_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){return Err("external solver export requires accuracy evidence".into());}
                if self.independent_validation_id.as_deref().map(str::trim).filter(|s|!s.is_empty()).is_none(){return Err("external solver export requires independent validation evidence".into());}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn internal_promotion_requires_measured_convergence_and_manifest(){
        let e=SolverRunEvidence::new(
            "internal-cfr","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Passed,"policy-v1",
            Some(10000),Some(0.001),true,Some("payoff-manifest".into()),None,None,
        ).unwrap();
        e.validate_promotion_ready("tree").unwrap();
    }

    #[test]
    fn internal_without_nashconv_fails_closed(){
        let e=SolverRunEvidence::new(
            "internal-cfr","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Passed,"policy-v1",
            Some(10000),None,true,Some("payoff-manifest".into()),None,None,
        ).unwrap();
        assert!(e.validate_promotion_ready("tree").is_err());
    }

    #[test]
    fn external_export_requires_accuracy_and_independent_validation(){
        let e=SolverRunEvidence::new(
            "wizard-export","export","tree",SolverEvidenceKind::ExternalSolverExport,ConvergenceGateStatus::Passed,"policy-v1",
            None,None,false,None,Some("reported-accuracy".into()),Some("holdout-check".into()),
        ).unwrap();
        e.validate_promotion_ready("tree").unwrap();
    }

    #[test]
    fn external_export_without_holdout_fails_closed(){
        let e=SolverRunEvidence::new(
            "external","export","tree",SolverEvidenceKind::ExternalSolverExport,ConvergenceGateStatus::Passed,"policy-v1",
            None,None,false,None,Some("accuracy".into()),None,
        ).unwrap();
        assert!(e.validate_promotion_ready("tree").is_err());
    }

    #[test]
    fn failed_gate_never_promotes(){
        let e=SolverRunEvidence::new(
            "internal","run","tree",SolverEvidenceKind::InternalMeasured,ConvergenceGateStatus::Failed,"policy-v1",
            Some(1),Some(1.0),true,Some("manifest".into()),None,None,
        ).unwrap();
        assert!(e.validate_promotion_ready("tree").is_err());
    }
}
