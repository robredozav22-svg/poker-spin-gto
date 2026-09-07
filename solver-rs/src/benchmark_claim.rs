#[derive(Debug,Clone,PartialEq)]
pub struct IndependentAccuracyMeasurement{
    pub subject_id:String,
    pub tree_profile_id:String,
    pub evaluator_profile_id:String,
    pub evaluator_source_id:String,
    pub artifact_checksum:String,
    pub evaluated_states:u64,
    pub repetitions:u32,
    /// Exploitability / Nash-distance-like loss normalized to pot under the
    /// SAME evaluator definition for every compared subject.
    pub normalized_exploitability_fraction_of_pot:f64,
}

impl IndependentAccuracyMeasurement{
    pub fn validate(&self)->Result<(),String>{
        for (name,value) in [
            ("subject_id",self.subject_id.as_str()),
            ("tree_profile_id",self.tree_profile_id.as_str()),
            ("evaluator_profile_id",self.evaluator_profile_id.as_str()),
            ("evaluator_source_id",self.evaluator_source_id.as_str()),
            ("artifact_checksum",self.artifact_checksum.as_str()),
        ]{
            if value.trim().is_empty(){return Err(format!("{name} must not be empty"));}
        }
        if self.evaluated_states==0{return Err("evaluated_states must be positive".into());}
        if self.repetitions<3{return Err("accuracy benchmark requires at least 3 repetitions".into());}
        let x=self.normalized_exploitability_fraction_of_pot;
        if !x.is_finite()||x<0.0{return Err("normalized exploitability must be finite and nonnegative".into());}
        Ok(())
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct SuperiorityPolicy{
    /// Candidate must beat reference by at least this absolute fraction of pot.
    pub minimum_absolute_margin_fraction_of_pot:f64,
    /// Optional hard candidate ceiling. Example: 0.0001 = 0.01% pot.
    pub candidate_max_fraction_of_pot:Option<f64>,
}

impl SuperiorityPolicy{
    pub fn validate(&self)->Result<(),String>{
        if !self.minimum_absolute_margin_fraction_of_pot.is_finite()||self.minimum_absolute_margin_fraction_of_pot<0.0{
            return Err("minimum superiority margin must be finite and nonnegative".into());
        }
        if let Some(x)=self.candidate_max_fraction_of_pot{
            if !x.is_finite()||x<0.0{return Err("candidate max fraction must be finite and nonnegative".into());}
        }
        Ok(())
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct SuperiorityReport{
    pub candidate_fraction_of_pot:f64,
    pub reference_fraction_of_pot:f64,
    pub absolute_improvement_fraction_of_pot:f64,
    pub relative_improvement:f64,
    pub claim_allowed:bool,
}

/// This gate intentionally ignores vendor-reported accuracy numbers. A claim of
/// superiority is allowed only from our own apples-to-apples independent
/// measurements under one evaluator and one exact tree profile.
pub fn evaluate_superiority(
    candidate:&IndependentAccuracyMeasurement,
    reference:&IndependentAccuracyMeasurement,
    policy:&SuperiorityPolicy,
)->Result<SuperiorityReport,String>{
    candidate.validate()?;reference.validate()?;policy.validate()?;
    if candidate.tree_profile_id!=reference.tree_profile_id{return Err("superiority comparison requires identical tree_profile_id".into());}
    if candidate.evaluator_profile_id!=reference.evaluator_profile_id{return Err("superiority comparison requires identical evaluator_profile_id".into());}
    if candidate.evaluator_source_id!=reference.evaluator_source_id{return Err("superiority comparison requires identical evaluator source".into());}
    if candidate.evaluated_states!=reference.evaluated_states{return Err("superiority comparison requires identical evaluated-state coverage".into());}
    if candidate.artifact_checksum==reference.artifact_checksum{return Err("candidate and reference artifacts must be distinct".into());}

    let c=candidate.normalized_exploitability_fraction_of_pot;
    let r=reference.normalized_exploitability_fraction_of_pot;
    let absolute=r-c;
    let relative=if r<=f64::EPSILON{if c<=f64::EPSILON{0.0}else{f64::NEG_INFINITY}}else{absolute/r};
    let margin_pass=absolute>=policy.minimum_absolute_margin_fraction_of_pot;
    let ceiling_pass=policy.candidate_max_fraction_of_pot.map(|m|c<=m).unwrap_or(true);
    Ok(SuperiorityReport{
        candidate_fraction_of_pot:c,
        reference_fraction_of_pot:r,
        absolute_improvement_fraction_of_pot:absolute,
        relative_improvement:relative,
        claim_allowed:c<r && margin_pass && ceiling_pass,
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    fn m(subject:&str,artifact:&str,x:f64)->IndependentAccuracyMeasurement{
        IndependentAccuracyMeasurement{
            subject_id:subject.into(),tree_profile_id:"spin15-exact".into(),evaluator_profile_id:"independent-br-v1".into(),
            evaluator_source_id:"our-holdout-evaluator".into(),artifact_checksum:artifact.into(),evaluated_states:1_000_000,repetitions:3,
            normalized_exploitability_fraction_of_pot:x,
        }
    }

    #[test]
    fn materially_lower_same_evaluator_result_allows_claim(){
        let policy=SuperiorityPolicy{minimum_absolute_margin_fraction_of_pot:0.00001,candidate_max_fraction_of_pot:Some(0.0001)};
        let r=evaluate_superiority(&m("ours","a",0.00008),&m("reference","b",0.00012),&policy).unwrap();
        assert!(r.claim_allowed);
    }

    #[test]
    fn tiny_difference_below_margin_does_not_allow_claim(){
        let policy=SuperiorityPolicy{minimum_absolute_margin_fraction_of_pot:0.00001,candidate_max_fraction_of_pot:None};
        let r=evaluate_superiority(&m("ours","a",0.000099),&m("reference","b",0.000100),&policy).unwrap();
        assert!(!r.claim_allowed);
    }

    #[test]
    fn different_evaluator_cannot_support_superiority_claim(){
        let mut reference=m("reference","b",0.0002);reference.evaluator_profile_id="other".into();
        let policy=SuperiorityPolicy{minimum_absolute_margin_fraction_of_pot:0.0,candidate_max_fraction_of_pot:None};
        assert!(evaluate_superiority(&m("ours","a",0.0001),&reference,&policy).is_err());
    }

    #[test]
    fn fewer_than_three_repetitions_fails_closed(){
        let mut candidate=m("ours","a",0.0001);candidate.repetitions=2;
        let policy=SuperiorityPolicy{minimum_absolute_margin_fraction_of_pot:0.0,candidate_max_fraction_of_pot:None};
        assert!(evaluate_superiority(&candidate,&m("reference","b",0.0002),&policy).is_err());
    }
}
