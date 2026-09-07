use std::collections::{HashMap,HashSet};

use crate::preflop_tree::{ContinuationContract,PreflopAction,PreflopNodeKey};
use crate::tree_catalog::TreeCatalog;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum ContinuationVerification{VerifiedMeasured,CrossChecked,Partial,Missing}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct ContinuationKey{pub node:PreflopNodeKey,pub action:PreflopAction}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ContinuationArtifactProof{
    pub artifact_id:String,
    pub checksum:String,
    pub required_states:u64,
    pub covered_states:u64,
}

impl ContinuationArtifactProof{
    pub fn new(artifact_id:impl Into<String>,checksum:impl Into<String>,required_states:u64,covered_states:u64)->Result<Self,String>{
        let artifact_id=artifact_id.into();let checksum=checksum.into();
        if artifact_id.trim().is_empty(){return Err("continuation artifact_id must not be empty".into());}
        if checksum.trim().is_empty(){return Err("continuation artifact checksum must not be empty".into());}
        if required_states==0{return Err("continuation required_states must be positive".into());}
        if covered_states>required_states{return Err("continuation covered_states cannot exceed required_states".into());}
        Ok(Self{artifact_id,checksum,required_states,covered_states})
    }

    pub fn is_complete(&self)->bool{self.required_states>0 && self.covered_states==self.required_states}
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ContinuationEvidence{
    pub key:ContinuationKey,
    pub verification:ContinuationVerification,
    pub source_id:String,
    pub model_profile_id:String,
    pub artifact:Option<ContinuationArtifactProof>,
}

impl ContinuationEvidence{
    pub fn new(
        key:ContinuationKey,
        verification:ContinuationVerification,
        source_id:impl Into<String>,
        model_profile_id:impl Into<String>,
        artifact:Option<ContinuationArtifactProof>,
    )->Result<Self,String>{
        let source_id=source_id.into();let model_profile_id=model_profile_id.into();
        if source_id.trim().is_empty(){return Err("continuation source_id must not be empty".into());}
        if model_profile_id.trim().is_empty(){return Err("continuation model_profile_id must not be empty".into());}
        if verification==ContinuationVerification::VerifiedMeasured{
            let proof=artifact.as_ref().ok_or_else(||"VERIFIED_MEASURED continuation requires artifact proof".to_string())?;
            if !proof.is_complete(){return Err("VERIFIED_MEASURED continuation requires complete artifact coverage".into());}
        }
        Ok(Self{key,verification,source_id,model_profile_id,artifact})
    }

    pub fn exact_ready(&self)->bool{
        self.verification==ContinuationVerification::VerifiedMeasured
            && self.artifact.as_ref().map(|a|a.is_complete()).unwrap_or(false)
    }
}

#[derive(Debug,Clone)]
pub struct ContinuationRegistry{by_key:HashMap<ContinuationKey,ContinuationEvidence>}

impl ContinuationRegistry{
    pub fn new(entries:Vec<ContinuationEvidence>)->Result<Self,String>{
        let mut by_key=HashMap::new();
        for entry in entries{if by_key.insert(entry.key.clone(),entry).is_some(){return Err("duplicate continuation evidence key".into());}}
        Ok(Self{by_key})
    }

    pub fn get(&self,key:&ContinuationKey)->Option<&ContinuationEvidence>{self.by_key.get(key)}

    pub fn missing_or_unverified_required_edges(&self,catalog:&TreeCatalog)->Vec<ContinuationKey>{
        let mut missing=Vec::new();let mut seen=HashSet::new();
        for node in &catalog.nodes{
            for edge in &node.edges{
                if edge.continuation!=ContinuationContract::RequiresPostflopEv{continue;}
                let key=ContinuationKey{node:node.key.clone(),action:edge.action};
                if !seen.insert(key.clone()){continue;}
                let ok=self.by_key.get(&key).map(|e|e.exact_ready()).unwrap_or(false);
                if !ok{missing.push(key);}
            }
        }
        missing
    }

    pub fn validate_exact_coverage(&self,catalog:&TreeCatalog)->Result<(),String>{
        let missing=self.missing_or_unverified_required_edges(catalog);
        if !missing.is_empty(){return Err(format!("exact strategy blocked: {} postflop continuation edge(s) lack VERIFIED_MEASURED complete artifact coverage",missing.len()));}
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::tree_catalog::{CatalogCompleteness,TreeCatalog};
    use crate::tree_profile::screen_reference_spins_15bb_v1;

    fn first_required(catalog:&TreeCatalog)->ContinuationKey{
        let target=catalog.nodes.iter().find(|n|n.edges.iter().any(|e|e.continuation==ContinuationContract::RequiresPostflopEv)).unwrap();
        let action=target.edges.iter().find(|e|e.continuation==ContinuationContract::RequiresPostflopEv).unwrap().action;
        ContinuationKey{node:target.key.clone(),action}
    }

    #[test]
    fn empty_registry_detects_reference_nonallin_gaps(){
        let catalog=TreeCatalog::new(screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let registry=ContinuationRegistry::new(vec![]).unwrap();
        assert_eq!(registry.missing_or_unverified_required_edges(&catalog).len(),3);
        assert!(registry.validate_exact_coverage(&catalog).is_err());
    }

    #[test]
    fn verified_measured_without_artifact_is_rejected(){
        let catalog=TreeCatalog::new(screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let key=first_required(&catalog);
        let err=ContinuationEvidence::new(key,ContinuationVerification::VerifiedMeasured,"fixture","fixture-model",None).unwrap_err();
        assert!(err.contains("requires artifact proof"));
    }

    #[test]
    fn verified_measured_with_partial_artifact_is_rejected(){
        let catalog=TreeCatalog::new(screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let key=first_required(&catalog);
        let proof=ContinuationArtifactProof::new("artifact","abc",100,99).unwrap();
        let err=ContinuationEvidence::new(key,ContinuationVerification::VerifiedMeasured,"fixture","fixture-model",Some(proof)).unwrap_err();
        assert!(err.contains("complete artifact coverage"));
    }

    #[test]
    fn crosschecked_continuation_is_not_enough_for_exact_coverage(){
        let catalog=TreeCatalog::new(screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let key=first_required(&catalog);
        let evidence=ContinuationEvidence::new(key,ContinuationVerification::CrossChecked,"fixture","fixture-model",None).unwrap();
        let registry=ContinuationRegistry::new(vec![evidence]).unwrap();
        assert!(registry.validate_exact_coverage(&catalog).is_err());
    }

    #[test]
    fn complete_verified_artifact_is_exact_ready_for_its_edge(){
        let catalog=TreeCatalog::new(screen_reference_spins_15bb_v1(),CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let key=first_required(&catalog);
        let proof=ContinuationArtifactProof::new("artifact","abc",100,100).unwrap();
        let evidence=ContinuationEvidence::new(key.clone(),ContinuationVerification::VerifiedMeasured,"fixture","fixture-model",Some(proof)).unwrap();
        assert!(evidence.exact_ready());
        let registry=ContinuationRegistry::new(vec![evidence]).unwrap();
        assert!(registry.get(&key).unwrap().exact_ready());
        assert_eq!(registry.missing_or_unverified_required_edges(&catalog).len(),2);
    }

    #[test]
    fn duplicate_continuation_key_fails_closed(){
        let node=crate::reference_tree_15bb::bb_vs_btn_fold_sb_raise_3();
        let key=ContinuationKey{node:node.key.clone(),action:PreflopAction::CallTo(crate::preflop_tree::Bb100(300))};
        let proof_a=ContinuationArtifactProof::new("a","aa",1,1).unwrap();
        let proof_b=ContinuationArtifactProof::new("b","bb",1,1).unwrap();
        let a=ContinuationEvidence::new(key.clone(),ContinuationVerification::VerifiedMeasured,"a","m",Some(proof_a)).unwrap();
        let b=ContinuationEvidence::new(key,ContinuationVerification::VerifiedMeasured,"b","m",Some(proof_b)).unwrap();
        assert!(ContinuationRegistry::new(vec![a,b]).is_err());
    }
}
