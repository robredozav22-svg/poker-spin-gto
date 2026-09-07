use crate::preflop_tree::{GameFormat,PayoutProfile,TreeVerification};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum TreeFamily{
    ScreenReference,
    WizardGeneral,
    WizardResearch,
    WizardSimple,
    ExternalSolved,
    InternalResearch,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum SizingFamily{
    Limp,
    Raise2x,
    Raise225x,
    Raise25x,
    Raise28x,
    Raise3x,
    Jam,
    GtoSelected,
    Custom,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TreeProfileSpec{
    pub id:String,
    pub family:TreeFamily,
    pub format:GameFormat,
    pub payout:PayoutProfile,
    pub verification:TreeVerification,
    pub source_id:String,
    pub sizing_families:Vec<SizingFamily>,
    pub notes:String,
}

impl TreeProfileSpec{
    pub fn new(
        id:impl Into<String>,
        family:TreeFamily,
        format:GameFormat,
        payout:PayoutProfile,
        verification:TreeVerification,
        source_id:impl Into<String>,
        sizing_families:Vec<SizingFamily>,
        notes:impl Into<String>,
    )->Result<Self,String>{
        let id=id.into();
        let source_id=source_id.into();
        let notes=notes.into();
        if id.trim().is_empty(){return Err("tree profile id must not be empty".into());}
        if source_id.trim().is_empty(){return Err("tree profile source_id must not be empty".into());}
        if sizing_families.is_empty(){return Err("tree profile must declare at least one sizing family".into());}
        let mut seen=std::collections::HashSet::new();
        for s in &sizing_families{
            if !seen.insert(*s){return Err(format!("duplicate sizing family in tree profile: {s:?}"));}
        }
        if verification==TreeVerification::VerifiedExactTree && family==TreeFamily::ScreenReference{
            return Err("screen reference tree cannot be VERIFIED_EXACT".into());
        }
        Ok(Self{id,family,format,payout,verification,source_id,sizing_families,notes})
    }
}

pub fn screen_reference_spins_15bb_v1()->TreeProfileSpec{
    TreeProfileSpec::new(
        "screen-reference-spins-15bb-v1",
        TreeFamily::ScreenReference,
        GameFormat::Spin3Max,
        PayoutProfile::WinnerTakeAllChipEv,
        TreeVerification::ScreenReference,
        "docs/SCREEN_REFERENCE_SPINS.md",
        vec![SizingFamily::Raise2x,SizingFamily::Raise3x,SizingFamily::Jam],
        "Action paths visible in supplied screenshots only. Not a complete solver tree and not strategy data.",
    ).expect("static screenshot tree profile")
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn screenshot_profile_is_explicitly_non_exact(){
        let p=screen_reference_spins_15bb_v1();
        assert_eq!(p.verification,TreeVerification::ScreenReference);
        assert_eq!(p.family,TreeFamily::ScreenReference);
        assert_eq!(p.id,"screen-reference-spins-15bb-v1");
    }

    #[test]
    fn screen_reference_cannot_claim_exact_tree(){
        let err=TreeProfileSpec::new(
            "bad",TreeFamily::ScreenReference,GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,
            TreeVerification::VerifiedExactTree,"screen",vec![SizingFamily::Raise2x],"",
        ).unwrap_err();
        assert!(err.contains("cannot be VERIFIED_EXACT"));
    }

    #[test]
    fn duplicate_sizing_family_fails_closed(){
        assert!(TreeProfileSpec::new(
            "dup",TreeFamily::InternalResearch,GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,
            TreeVerification::MissingExact,"internal",vec![SizingFamily::Jam,SizingFamily::Jam],"",
        ).is_err());
    }
}
