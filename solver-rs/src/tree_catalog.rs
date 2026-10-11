use std::collections::{HashMap,HashSet};

use crate::preflop_tree::{ContinuationContract,PreflopDecisionSpec,PreflopNodeKey,TreeVerification};
use crate::tree_profile::TreeProfileSpec;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CatalogCompleteness{
    PartialReference,
    Complete,
}

#[derive(Debug,Clone)]
pub struct TreeCatalog{
    pub profile:TreeProfileSpec,
    pub completeness:CatalogCompleteness,
    pub nodes:Vec<PreflopDecisionSpec>,
}

impl TreeCatalog{
    pub fn new(profile:TreeProfileSpec,completeness:CatalogCompleteness,nodes:Vec<PreflopDecisionSpec>)->Result<Self,String>{
        if nodes.is_empty(){return Err("tree catalog must contain at least one node".into());}
        if profile.verification==TreeVerification::VerifiedExactTree && completeness!=CatalogCompleteness::Complete{
            return Err("VERIFIED_EXACT tree profile requires a Complete catalog".into());
        }

        let mut seen=HashSet::new();
        for node in &nodes{
            profile.validate_node(node)?;
            if !seen.insert(node.key.clone()){
                return Err(format!("duplicate node key in tree catalog: {:?}",node.key));
            }
        }

        if completeness==CatalogCompleteness::Complete{
            let index:HashMap<PreflopNodeKey,usize>=nodes.iter().enumerate().map(|(i,n)|(n.key.clone(),i)).collect();
            for node in &nodes{
                for (edge_index,edge) in node.edges.iter().enumerate(){
                    if edge.continuation!=ContinuationContract::ChildDecision{continue;}
                    let child=node.child_key(edge_index)?;
                    if !index.contains_key(&child){
                        return Err(format!("complete tree catalog missing child node: {:?}",child));
                    }
                }
            }
        }

        Ok(Self{profile,completeness,nodes})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::preflop_tree::{ActionEdge,Bb100,GameFormat,PayoutProfile,PreflopAction,PreflopDecisionSpec,PreflopNodeKey,TreeEvidence};
    use crate::tree::Player;
    use crate::tree_profile::{screen_reference_spins_15bb_v1,SizingFamily,TreeFamily,TreeProfileSpec};

    #[test]
    fn screenshot_reference_can_be_partial(){
        let profile=screen_reference_spins_15bb_v1();
        let nodes=crate::reference_tree_15bb::all_reference_specs();
        let catalog=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,nodes).unwrap();
        assert_eq!(catalog.nodes.len(),4);
    }

    #[test]
    fn duplicate_node_key_fails_closed(){
        let profile=screen_reference_spins_15bb_v1();
        let node=crate::reference_tree_15bb::btn_first_in();
        assert!(TreeCatalog::new(profile,CatalogCompleteness::PartialReference,vec![node.clone(),node]).is_err());
    }

    #[test]
    fn exact_profile_cannot_use_partial_catalog(){
        let profile=TreeProfileSpec::new(
            "exact-fixture",TreeFamily::InternalResearch,GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,
            TreeVerification::VerifiedExactTree,"fixture",vec![SizingFamily::Jam],"",
        ).unwrap();
        let key=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,"exact-fixture",Bb100(800),Player::Btn,vec![]).unwrap();
        let node=PreflopDecisionSpec::new(
            key,vec![ActionEdge::child(PreflopAction::Fold,Player::Sb),ActionEdge::child(PreflopAction::JamTo(Bb100(800)),Player::Sb)],
            TreeEvidence::new(TreeVerification::VerifiedExactTree,"fixture").unwrap(),
        ).unwrap();
        let err=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,vec![node]).unwrap_err();
        assert!(err.contains("requires a Complete catalog"));
    }

    #[test]
    fn complete_catalog_requires_every_child(){
        let profile=TreeProfileSpec::new(
            "cross-fixture",TreeFamily::InternalResearch,GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,
            TreeVerification::CrossChecked,"fixture",vec![SizingFamily::Jam],"",
        ).unwrap();
        let key=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,"cross-fixture",Bb100(800),Player::Btn,vec![]).unwrap();
        let node=PreflopDecisionSpec::new(
            key,vec![ActionEdge::child(PreflopAction::Fold,Player::Sb),ActionEdge::child(PreflopAction::JamTo(Bb100(800)),Player::Sb)],
            TreeEvidence::new(TreeVerification::CrossChecked,"fixture").unwrap(),
        ).unwrap();
        let err=TreeCatalog::new(profile,CatalogCompleteness::Complete,vec![node]).unwrap_err();
        assert!(err.contains("missing child node"));
    }
}
