use crate::preflop_tree::{GameFormat,PayoutProfile,PreflopDecisionSpec,TreeVerification};

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

    pub fn validate_node(&self,node:&PreflopDecisionSpec)->Result<(),String>{
        if node.key.tree_profile_id!=self.id{
            return Err(format!("node tree_profile_id '{}' does not match registered profile '{}'",node.key.tree_profile_id,self.id));
        }
        if node.key.format!=self.format{return Err("node format does not match tree profile".into());}
        if node.key.payout!=self.payout{return Err("node payout profile does not match tree profile".into());}
        if node.evidence.verification==TreeVerification::VerifiedExactTree && self.verification!=TreeVerification::VerifiedExactTree{
            return Err("node cannot be VERIFIED_EXACT when tree profile is not VERIFIED_EXACT".into());
        }
        Ok(())
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
    use crate::preflop_tree::{ActionEdge,Bb100,ContinuationContract,PreflopAction,PreflopNodeKey,TreeEvidence};
    use crate::tree::Player;

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

    #[test]
    fn exact_node_cannot_be_promoted_under_non_exact_profile(){
        let profile=screen_reference_spins_15bb_v1();
        let key=PreflopNodeKey::new(
            GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,profile.id.clone(),Bb100(1500),Player::Btn,vec![]
        ).unwrap();
        let node=PreflopDecisionSpec::new(
            key,
            vec![
                ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
                ActionEdge{action:PreflopAction::JamTo(Bb100(1500)),continuation:ContinuationContract::ChildDecision},
            ],
            TreeEvidence::new(TreeVerification::VerifiedExactTree,"bad-promotion").unwrap(),
        ).unwrap();
        let err=profile.validate_node(&node).unwrap_err();
        assert!(err.contains("cannot be VERIFIED_EXACT"));
    }

    #[test]
    fn matching_screen_reference_node_is_accepted(){
        let profile=screen_reference_spins_15bb_v1();
        let node=crate::reference_tree_15bb::btn_first_in();
        profile.validate_node(&node).unwrap();
    }
}
