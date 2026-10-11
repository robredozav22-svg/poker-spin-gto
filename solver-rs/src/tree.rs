#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Btn,
    Sb,
    Bb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Fold,
    Jam,
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    BtnRoot,
    SbAfterBtnFold,
    BbAfterBtnFoldSbJam,
    SbFacingBtnJam,
    BbAfterBtnJamSbFold,
    BbAfterBtnJamSbCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionSpec {
    pub node: NodeId,
    pub actor: Player,
    pub actions: &'static [Action],
}

const FOLD_JAM: &[Action] = &[Action::Fold, Action::Jam];
const FOLD_CALL: &[Action] = &[Action::Fold, Action::Call];

pub const DECISIONS: &[DecisionSpec] = &[
    DecisionSpec { node: NodeId::BtnRoot, actor: Player::Btn, actions: FOLD_JAM },
    DecisionSpec { node: NodeId::SbAfterBtnFold, actor: Player::Sb, actions: FOLD_JAM },
    DecisionSpec { node: NodeId::BbAfterBtnFoldSbJam, actor: Player::Bb, actions: FOLD_CALL },
    DecisionSpec { node: NodeId::SbFacingBtnJam, actor: Player::Sb, actions: FOLD_CALL },
    DecisionSpec { node: NodeId::BbAfterBtnJamSbFold, actor: Player::Bb, actions: FOLD_CALL },
    DecisionSpec { node: NodeId::BbAfterBtnJamSbCall, actor: Player::Bb, actions: FOLD_CALL },
];

pub fn spec(node: NodeId) -> &'static DecisionSpec {
    DECISIONS.iter().find(|s| s.node == node).expect("known node")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_keeps_context_distinct() {
        assert_ne!(NodeId::BbAfterBtnJamSbFold, NodeId::BbAfterBtnJamSbCall);
        assert_ne!(NodeId::BbAfterBtnFoldSbJam, NodeId::BbAfterBtnJamSbFold);
    }

    #[test]
    fn root_and_sb_nodes_have_correct_action_families() {
        assert_eq!(spec(NodeId::BtnRoot).actions, FOLD_JAM);
        assert_eq!(spec(NodeId::SbFacingBtnJam).actions, FOLD_CALL);
    }
}
