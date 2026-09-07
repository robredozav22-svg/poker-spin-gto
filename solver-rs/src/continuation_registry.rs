use std::collections::{HashMap,HashSet};

use crate::preflop_tree::{ContinuationContract,PreflopAction,PreflopNodeKey};
use crate::tree_catalog::TreeCatalog;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum ContinuationVerification{
    VerifiedMeasured,
    CrossChecked,
    Partial,
    Missing,
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct ContinuationKey{
    pub node:PreflopNodeKey,
    pub action:PreflopAction,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ContinuationEvidence{
    pub key:ContinuationKey,
    pub verification:ContinuationVerification,
    pub source_id:String,
    pub model_profile_id:String,
}

impl ContinuationEvidence{
    pub fn new(
        key:ContinuationKey,
        verification:ContinuationVerification,
        source_id:impl Into<String>,
        model_profile_id:impl Into<String>,
    )->Result<Self,String>{
        let source_id=source_id.into();
        let model_profile_id=model_profile_id.into();
        if source_id.trim().is_empty(){return Err("continuation source_id must not be empty".into());}
        if model_profile_id.trim().is_empty(){return Err("continuation model_profile_id must not be empty".into());}
        Ok(Self{key,verification,source_id,model_profile_id})
    }
}

#[derive(Debug,Clone)]
pub struct ContinuationRegistry{
    by_key:HashMap<ContinuationKey,ContinuationEvidence>,
}

impl ContinuationRegistry{
    pub fn new(entries:Vec<ContinuationEvidence>)->Result<Self,String>{
        let mut by_key=HashMap::new();
        for entry in entries{
            if by_key.insert(entry.key.clone(),entry).is_some(){return Err("duplicate continuation evidence key".into());}
        }
        Ok(Self{by_key})
    }

    pub fn get(&self,key:&ContinuationKey)->Option<&ContinuationEvidence>{self.by_key.get(key)}

    pub fn missing_or_unverified_required_edges(&self,catalog:&TreeCatalog)->Vec<ContinuationKey>{
        let mut missing=Vec::new();
        let mut seen=HashSet::new();
        for node in &catalog.nodes{
            for edge in &node.edges{
                if edge.continuation!=ContinuationContract::RequiresPostflopEv{continue;}
                let key=ContinuationKey{node:node.key.clone(),action:edge.action};
                if !seen.insert(key.clone()){continue;}
                let ok=self.by_key.get(&key).map(|e|e.verification==ContinuationVerification::VerifiedMeasured).unwrap_or(false);
                if !ok{missing.push(key);}
            }
        }
        missing
    }

    pub fn validate_exact_coverage(&self,catalog:&TreeCatalog)->Result<(),String>{
        let missing=self.missing_or_unverified_required_edges(catalog);
        if !missing.is_empty(){return Err(format!("exact strategy blocked: {} postflop continuation edge(s) lack VERIFIED_MEASURED coverage",missing.len()));}
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::tree_catalog::{CatalogCompleteness,TreeCatalog};
    use crate::tree_profile::screen_reference_spins_15bb_v1;

    #[test]
    fn empty_registry_detects_reference_nonallin_gaps(){
        let catalog=TreeCatalog::new(
            screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,
            crate::reference_tree_15bb::all_reference_specs(),
        ).unwrap();
        let registry=ContinuationRegistry::new(vec![]).unwrap();
        let missing=registry.missing_or_unverified_required_edges(&catalog);
        assert_eq!(missing.len(),3); // BB fold/call in multiway node + BB call vs SB raise.
        assert!(registry.validate_exact_coverage(&catalog).is_err());
    }

    #[test]
    fn crosschecked_continuation_is_not_enough_for_exact_coverage(){
        let catalog=TreeCatalog::new(
            screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,
            crate::reference_tree_15bb::all_reference_specs(),
        ).unwrap();
        let target=catalog.nodes.iter().find(|n|n.edges.iter().any(|e|e.continuation==ContinuationContract::RequiresPostflopEv)).unwrap();
        let action=target.edges.iter().find(|e|e.continuation==ContinuationContract::RequiresPostflopEv).unwrap().action;
        let evidence=ContinuationEvidence::new(
            ContinuationKey{node:target.key.clone(),action},ContinuationVerification::CrossChecked,"fixture","fixture-model",
        ).unwrap();
        let registry=ContinuationRegistry::new(vec![evidence]).unwrap();
        assert!(registry.validate_exact_coverage(&catalog).is_err());
    }

    #[test]
    fn duplicate_continuation_key_fails_closed(){
        let node=crate::reference_tree_15bb::bb_vs_btn_fold_sb_raise_3();
        let key=ContinuationKey{node:node.key.clone(),action:PreflopAction::CallTo(crate::preflop_tree::Bb100(300))};
        let a=ContinuationEvidence::new(key.clone(),ContinuationVerification::VerifiedMeasured,"a","m").unwrap();
        let b=ContinuationEvidence::new(key,ContinuationVerification::VerifiedMeasured,"b","m").unwrap();
        assert!(ContinuationRegistry::new(vec![a,b]).is_err());
    }
}
