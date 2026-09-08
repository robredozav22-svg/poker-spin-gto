use crate::preflop_tree::{
    ActionEdge,Bb100,ContinuationContract,GameFormat,HistoryEvent,PayoutProfile,
    PreflopAction,PreflopDecisionSpec,PreflopNodeKey,TreeEvidence,TreeVerification,
};
use crate::tree::Player;

const STACK:Bb100=Bb100(1500);
const TREE_PROFILE:&str="screen-reference-spins-15bb-v1";

fn evidence(id:&str)->TreeEvidence{
    TreeEvidence::new(TreeVerification::ScreenReference,id).expect("static reference evidence")
}

fn key(actor:Player,history:Vec<HistoryEvent>)->PreflopNodeKey{
    PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,TREE_PROFILE,STACK,actor,history)
        .expect("static 15bb reference key")
}

/// Screenshot reference only. This module records visible action-tree structure,
/// never per-hand strategy frequencies.
pub fn btn_first_in()->PreflopDecisionSpec{
    PreflopDecisionSpec::new(
        key(Player::Btn,vec![]),
        vec![
            ActionEdge::child(PreflopAction::Fold,Player::Sb),
            ActionEdge::child(PreflopAction::RaiseTo(Bb100(200)),Player::Sb),
            ActionEdge::child(PreflopAction::JamTo(STACK),Player::Sb),
        ],
        evidence("SCREEN_REFERENCE_15BB_BTN_FIRST_IN"),
    ).expect("valid BTN screenshot reference")
}

pub fn sb_vs_btn_raise_2()->PreflopDecisionSpec{
    PreflopDecisionSpec::new(
        key(Player::Sb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))}]),
        vec![
            ActionEdge::child(PreflopAction::Fold,Player::Bb),
            ActionEdge::child(PreflopAction::CallTo(Bb100(200)),Player::Bb),
            ActionEdge::child(PreflopAction::JamTo(STACK),Player::Bb),
        ],
        evidence("SCREEN_REFERENCE_15BB_SB_VS_BTN_RAISE_2"),
    ).expect("valid SB screenshot reference")
}

pub fn bb_vs_btn_raise_2_sb_call()->PreflopDecisionSpec{
    PreflopDecisionSpec::new(
        key(Player::Bb,vec![
            HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))},
            HistoryEvent{actor:Player::Sb,action:PreflopAction::CallTo(Bb100(200))},
        ]),
        vec![
            ActionEdge::terminal(PreflopAction::Fold,ContinuationContract::RequiresPostflopEv),
            ActionEdge::terminal(PreflopAction::CallTo(Bb100(200)),ContinuationContract::RequiresPostflopEv),
            ActionEdge::child(PreflopAction::JamTo(STACK),Player::Btn),
        ],
        evidence("SCREEN_REFERENCE_15BB_BB_VS_BTN_RAISE_2_SB_CALL"),
    ).expect("valid BB multiway screenshot reference")
}

pub fn bb_vs_btn_fold_sb_raise_3()->PreflopDecisionSpec{
    PreflopDecisionSpec::new(
        key(Player::Bb,vec![
            HistoryEvent{actor:Player::Btn,action:PreflopAction::Fold},
            HistoryEvent{actor:Player::Sb,action:PreflopAction::RaiseTo(Bb100(300))},
        ]),
        vec![
            ActionEdge::terminal(PreflopAction::Fold,ContinuationContract::ExactFoldSettlement),
            ActionEdge::terminal(PreflopAction::CallTo(Bb100(300)),ContinuationContract::RequiresPostflopEv),
            ActionEdge::child(PreflopAction::JamTo(STACK),Player::Sb),
        ],
        evidence("SCREEN_REFERENCE_15BB_BB_VS_BTN_FOLD_SB_RAISE_3"),
    ).expect("valid BB vs SB screenshot reference")
}

pub fn all_reference_specs()->Vec<PreflopDecisionSpec>{
    vec![btn_first_in(),sb_vs_btn_raise_2(),bb_vs_btn_raise_2_sb_call(),bb_vs_btn_fold_sb_raise_3()]
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn every_reference_node_is_screen_reference_not_exact(){
        for spec in all_reference_specs(){
            assert_eq!(spec.evidence.verification,TreeVerification::ScreenReference);
            assert_eq!(spec.key.tree_profile_id,TREE_PROFILE);
        }
    }

    #[test]
    fn btn_raise_child_key_is_exact_sb_history(){
        let root=btn_first_in();
        let child=root.child_key(1).unwrap();
        assert_eq!(child.actor,Player::Sb);
        assert_eq!(child.history,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))}]);
        assert_eq!(child,sb_vs_btn_raise_2().key);
    }

    #[test]
    fn screenshot_nodes_have_distinct_full_histories(){
        let multiway=bb_vs_btn_raise_2_sb_call();
        let sb_only=bb_vs_btn_fold_sb_raise_3();
        assert_ne!(multiway.key,sb_only.key);
        assert_eq!(multiway.key.history[0].action,PreflopAction::RaiseTo(Bb100(200)));
        assert_eq!(sb_only.key.history[1].action,PreflopAction::RaiseTo(Bb100(300)));
    }

    #[test]
    fn nonallin_closed_paths_require_postflop_ev(){
        let spec=bb_vs_btn_raise_2_sb_call();
        assert_eq!(spec.edges[0].continuation,ContinuationContract::RequiresPostflopEv);
        assert_eq!(spec.edges[1].continuation,ContinuationContract::RequiresPostflopEv);
        assert_eq!(spec.edges[2].continuation,ContinuationContract::ChildDecision);
        assert_eq!(spec.edges[2].next_actor,Some(Player::Btn));
    }

    #[test]
    fn btn_fold_sb_raise_bb_fold_is_exact_fold_settlement(){
        let spec=bb_vs_btn_fold_sb_raise_3();
        assert_eq!(spec.edges[0].continuation,ContinuationContract::ExactFoldSettlement);
        assert_eq!(spec.edges[2].next_actor,Some(Player::Sb));
    }
}
