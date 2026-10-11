#[derive(Debug,Clone,PartialEq)]
pub struct AccuracyPolicy{
    pub id:String,
    /// Maximum accepted normalized exploitability/convergence upper bound as a
    /// fraction of the reference pot. 0.0001 = 0.01% of pot.
    pub max_fraction_of_pot:f64,
    pub require_independent_validation:bool,
    pub require_deterministic_repeat_for_internal:bool,
    pub notes:String,
}

impl AccuracyPolicy{
    pub fn new(
        id:impl Into<String>,
        max_fraction_of_pot:f64,
        require_independent_validation:bool,
        require_deterministic_repeat_for_internal:bool,
        notes:impl Into<String>,
    )->Result<Self,String>{
        let id=id.into();
        let notes=notes.into();
        if id.trim().is_empty(){return Err("accuracy policy id must not be empty".into());}
        if !max_fraction_of_pot.is_finite()||max_fraction_of_pot<=0.0||max_fraction_of_pot>=1.0{
            return Err("max_fraction_of_pot must be finite and in (0,1)".into());
        }
        Ok(Self{id,max_fraction_of_pot,require_independent_validation,require_deterministic_repeat_for_internal,notes})
    }
}

/// Internal promotion target chosen to be numerically stricter than GTO
/// Wizard's publicly stated Spin solution range of roughly 0.1%-0.017% pot.
/// This target is 0.01% pot. It is a project acceptance target, NOT a claim
/// that our current full Spin solution already achieves or is directly metric-
/// equivalent to Wizard's published accuracy methodology.
pub fn spin_verified_exact_v1()->AccuracyPolicy{
    AccuracyPolicy::new(
        "spin-verified-exact-v1",
        0.0001,
        true,
        true,
        "Target <=0.01% pot normalized convergence/exploitability upper bound, with independent validation. Metric comparability to external solvers must be documented before comparative claims.",
    ).expect("static accuracy policy")
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn strict_spin_target_is_one_basis_point_of_pot(){
        let p=spin_verified_exact_v1();
        assert!((p.max_fraction_of_pot-0.0001).abs()<1e-15);
        assert!(p.require_independent_validation);
    }

    #[test]
    fn invalid_policy_threshold_fails_closed(){
        assert!(AccuracyPolicy::new("bad",0.0,true,true,"").is_err());
        assert!(AccuracyPolicy::new("bad",1.0,true,true,"").is_err());
    }
}
