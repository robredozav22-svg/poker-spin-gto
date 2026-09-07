use std::collections::HashSet;

use crate::cards::COMBO_COUNT;
use crate::continuation_registry::ContinuationRegistry;
use crate::preflop_tree::{PreflopAction,PreflopNodeKey,TreeVerification};
use crate::tree_catalog::{CatalogCompleteness,TreeCatalog};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum StrategyVerification{VerifiedExact,CrossChecked,Partial,MissingExact}

#[derive(Debug,Clone,PartialEq)]
pub struct ComboActionFrequency{pub combo_index:usize,pub frequencies:Vec<(PreflopAction,f64)>}

#[derive(Debug,Clone,PartialEq)]
pub struct StrategyRecord{
    pub node:PreflopNodeKey,
    pub verification:StrategyVerification,
    pub source_id:String,
    pub solver_profile_id:String,
    pub combos:Vec<ComboActionFrequency>,
}

impl StrategyRecord{
    pub fn new(
        catalog:&TreeCatalog,
        continuations:Option<&ContinuationRegistry>,
        node:PreflopNodeKey,
        verification:StrategyVerification,
        source_id:impl Into<String>,
        solver_profile_id:impl Into<String>,
        combos:Vec<ComboActionFrequency>,
    )->Result<Self,String>{
        let source_id=source_id.into();
        let solver_profile_id=solver_profile_id.into();
        if source_id.trim().is_empty(){return Err("strategy source_id must not be empty".into());}
        if solver_profile_id.trim().is_empty(){return Err("solver_profile_id must not be empty".into());}
        if node.tree_profile_id!=catalog.profile.id{return Err("strategy node tree profile does not match catalog".into());}
        let node_spec=catalog.nodes.iter().find(|n|n.key==node).ok_or_else(||"strategy node is not present in tree catalog".to_string())?;

        if verification==StrategyVerification::VerifiedExact{
            if catalog.profile.verification!=TreeVerification::VerifiedExactTree{return Err("VERIFIED_EXACT strategy requires VERIFIED_EXACT tree profile".into());}
            if catalog.completeness!=CatalogCompleteness::Complete{return Err("VERIFIED_EXACT strategy requires Complete tree catalog".into());}
            let registry=continuations.ok_or_else(||"VERIFIED_EXACT strategy requires continuation registry".to_string())?;
            registry.validate_exact_coverage(catalog)?;
            if combos.len()!=COMBO_COUNT{return Err(format!("VERIFIED_EXACT strategy requires all {COMBO_COUNT} physical combos"));}
        }

        let legal_actions:HashSet<PreflopAction>=node_spec.edges.iter().map(|e|e.action).collect();
        validate_combo_rows(&combos,&legal_actions)?;
        Ok(Self{node,verification,source_id,solver_profile_id,combos})
    }
}

fn validate_combo_rows(rows:&[ComboActionFrequency],legal_actions:&HashSet<PreflopAction>)->Result<(),String>{
    let mut seen_combos=HashSet::new();
    for row in rows{
        if row.combo_index>=COMBO_COUNT{return Err(format!("combo index {} out of bounds",row.combo_index));}
        if !seen_combos.insert(row.combo_index){return Err(format!("duplicate combo index {}",row.combo_index));}
        if row.frequencies.is_empty(){return Err(format!("combo {} has no action frequencies",row.combo_index));}
        let mut seen_actions=HashSet::new();
        let mut sum=0.0f64;
        for (action,p) in &row.frequencies{
            if !legal_actions.contains(action){return Err(format!("combo {} uses action {:?} not legal at this node",row.combo_index,action));}
            if !seen_actions.insert(*action){return Err(format!("combo {} repeats action {:?}",row.combo_index,action));}
            if !p.is_finite() || *p<0.0 || *p>1.0{return Err(format!("combo {} has invalid frequency {}",row.combo_index,p));}
            sum+=*p;
        }
        if (sum-1.0).abs()>1e-9{return Err(format!("combo {} action frequencies sum to {:.12}, expected 1",row.combo_index,sum));}
    }
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::continuation_registry::ContinuationRegistry;
    use crate::preflop_tree::{ActionEdge,Bb100,GameFormat,PayoutProfile,PreflopDecisionSpec,TreeEvidence};
    use crate::tree::Player;
    use crate::tree_catalog::{CatalogCompleteness,TreeCatalog};
    use crate::tree_profile::{screen_reference_spins_15bb_v1,SizingFamily,TreeFamily,TreeProfileSpec};

    #[test]
    fn screenshot_tree_cannot_promote_strategy_to_exact(){
        let profile=screen_reference_spins_15bb_v1();
        let catalog=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let node=crate::reference_tree_15bb::btn_first_in().key;
        let err=StrategyRecord::new(&catalog,None,node,StrategyVerification::VerifiedExact,"screen","none",vec![]).unwrap_err();
        assert!(err.contains("requires VERIFIED_EXACT tree profile"));
    }

    #[test]
    fn partial_reference_strategy_can_store_no_combo_rows(){
        let profile=screen_reference_spins_15bb_v1();
        let catalog=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let node=crate::reference_tree_15bb::btn_first_in().key;
        let r=StrategyRecord::new(&catalog,None,node,StrategyVerification::MissingExact,"screen","none",vec![]).unwrap();
        assert!(r.combos.is_empty());
    }

    #[test]
    fn malformed_frequency_row_fails_closed(){
        let profile=screen_reference_spins_15bb_v1();
        let catalog=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let node=crate::reference_tree_15bb::btn_first_in().key;
        let bad=ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::Fold,0.7),(PreflopAction::RaiseTo(Bb100(200)),0.4)]};
        assert!(StrategyRecord::new(&catalog,None,node,StrategyVerification::Partial,"fixture","fixture",vec![bad]).is_err());
    }

    #[test]
    fn action_not_present_at_node_fails_closed(){
        let profile=screen_reference_spins_15bb_v1();
        let catalog=TreeCatalog::new(profile,CatalogCompleteness::PartialReference,crate::reference_tree_15bb::all_reference_specs()).unwrap();
        let node=crate::reference_tree_15bb::btn_first_in().key;
        let bad=ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::CallTo(Bb100(200)),1.0)]};
        let err=StrategyRecord::new(&catalog,None,node,StrategyVerification::Partial,"fixture","fixture",vec![bad]).unwrap_err();
        assert!(err.contains("not legal at this node"));
    }

    fn exact_pushfold_catalog()->(TreeCatalog,PreflopNodeKey){
        let profile=TreeProfileSpec::new(
            "exact-fixture",TreeFamily::InternalResearch,GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,
            TreeVerification::VerifiedExactTree,"fixture",vec![SizingFamily::Jam],"",
        ).unwrap();
        let root=PreflopNodeKey::new(GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,"exact-fixture",Bb100(800),Player::Sb,vec![]).unwrap();
        let child=PreflopNodeKey::new(GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,"exact-fixture",Bb100(800),Player::Bb,vec![crate::preflop_tree::HistoryEvent{actor:Player::Sb,action:PreflopAction::JamTo(Bb100(800))}]).unwrap();
        let root_spec=PreflopDecisionSpec::new(
            root.clone(),vec![ActionEdge::terminal(PreflopAction::Fold,crate::preflop_tree::ContinuationContract::ExactFoldSettlement),ActionEdge::child(PreflopAction::JamTo(Bb100(800)),Player::Bb)],
            TreeEvidence::new(TreeVerification::VerifiedExactTree,"fixture").unwrap(),
        ).unwrap();
        let child_spec=PreflopDecisionSpec::new(
            child,vec![ActionEdge::terminal(PreflopAction::Fold,crate::preflop_tree::ContinuationContract::ExactFoldSettlement),ActionEdge::terminal(PreflopAction::CallTo(Bb100(800)),crate::preflop_tree::ContinuationContract::ExactAllInShowdown)],
            TreeEvidence::new(TreeVerification::VerifiedExactTree,"fixture").unwrap(),
        ).unwrap();
        (TreeCatalog::new(profile,CatalogCompleteness::Complete,vec![root_spec,child_spec]).unwrap(),root)
    }

    #[test]
    fn exact_requires_continuation_registry_even_when_no_postflop_edges(){
        let (catalog,root)=exact_pushfold_catalog();
        let err=StrategyRecord::new(&catalog,None,root,StrategyVerification::VerifiedExact,"fixture","solver",vec![]).unwrap_err();
        assert!(err.contains("requires continuation registry"));
    }

    #[test]
    fn exact_requires_all_1326_physical_combos_after_coverage_gate(){
        let (catalog,root)=exact_pushfold_catalog();
        let registry=ContinuationRegistry::new(vec![]).unwrap();
        let one=ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::Fold,0.5),(PreflopAction::JamTo(Bb100(800)),0.5)]};
        let err=StrategyRecord::new(&catalog,Some(&registry),root,StrategyVerification::VerifiedExact,"fixture","solver",vec![one]).unwrap_err();
        assert!(err.contains("all 1326 physical combos"));
    }
}
