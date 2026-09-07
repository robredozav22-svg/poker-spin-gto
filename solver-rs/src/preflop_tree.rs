use crate::tree::Player;

/// Hundredths of a big blind. Fixed-point amounts are used in node identity so
/// 2.00bb, 2.20bb, 2.50bb, 2.80bb and 3.00bb remain deterministic/hash-safe.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Bb100(pub u16);

impl Bb100{
    pub const fn from_hundredths(value:u16)->Self{Self(value)}
    pub const fn one_bb()->Self{Self(100)}
    pub fn as_bb(self)->f64{self.0 as f64/100.0}
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum GameFormat{
    Spin3Max,
    SpinHeadsUp,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum PayoutProfile{
    WinnerTakeAllChipEv,
    /// Metadata ID only. Its actual payout vector must be defined and validated
    /// before any solver run may use this profile.
    ExplicitIcmProfile(u32),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum PreflopAction{
    Fold,
    Check,
    LimpTo(Bb100),
    CallTo(Bb100),
    RaiseTo(Bb100),
    JamTo(Bb100),
}

impl PreflopAction{
    pub fn amount_to(self)->Option<Bb100>{
        match self{
            Self::LimpTo(v)|Self::CallTo(v)|Self::RaiseTo(v)|Self::JamTo(v)=>Some(v),
            Self::Fold|Self::Check=>None,
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct HistoryEvent{
    pub actor:Player,
    pub action:PreflopAction,
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct PreflopNodeKey{
    pub format:GameFormat,
    pub payout:PayoutProfile,
    pub effective_stack:Bb100,
    pub actor:Player,
    pub history:Vec<HistoryEvent>,
}

impl PreflopNodeKey{
    pub fn new(
        format:GameFormat,
        payout:PayoutProfile,
        effective_stack:Bb100,
        actor:Player,
        history:Vec<HistoryEvent>,
    )->Result<Self,String>{
        if effective_stack.0==0{return Err("effective stack must be positive".into());}
        for event in &history{validate_action_amount(event.action,effective_stack)?;}
        Ok(Self{format,payout,effective_stack,actor,history})
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum TreeVerification{
    VerifiedExactTree,
    CrossChecked,
    ScreenReference,
    MissingExact,
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct TreeEvidence{
    pub verification:TreeVerification,
    pub source_id:String,
}

impl TreeEvidence{
    pub fn new(verification:TreeVerification,source_id:impl Into<String>)->Result<Self,String>{
        let source_id=source_id.into();
        if source_id.trim().is_empty(){return Err("tree evidence source_id must not be empty".into());}
        Ok(Self{verification,source_id})
    }
}

/// Contract for the mathematical object required after an action.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum ContinuationContract{
    ChildDecision,
    ExactFoldSettlement,
    ExactAllInShowdown,
    RequiresPostflopEv,
    Unresolved,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActionEdge{
    pub action:PreflopAction,
    pub continuation:ContinuationContract,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PreflopDecisionSpec{
    pub key:PreflopNodeKey,
    pub edges:Vec<ActionEdge>,
    pub evidence:TreeEvidence,
}

impl PreflopDecisionSpec{
    pub fn new(
        key:PreflopNodeKey,
        edges:Vec<ActionEdge>,
        evidence:TreeEvidence,
    )->Result<Self,String>{
        if edges.len()<2{return Err("decision node must expose at least two actions".into());}
        let mut seen=std::collections::HashSet::new();
        for edge in &edges{
            validate_action_amount(edge.action,key.effective_stack)?;
            if !seen.insert(edge.action){return Err(format!("duplicate action in decision node: {:?}",edge.action));}
            validate_continuation_contract(edge,key.effective_stack)?;
        }
        Ok(Self{key,edges,evidence})
    }
}

fn validate_action_amount(action:PreflopAction,stack:Bb100)->Result<(),String>{
    if let Some(amount)=action.amount_to(){
        if amount.0==0{return Err("preflop action amount must be positive".into());}
        if amount>stack{return Err(format!("preflop action amount {:.2}bb exceeds effective stack {:.2}bb",amount.as_bb(),stack.as_bb()));}
    }
    match action{
        PreflopAction::JamTo(amount) if amount!=stack=>Err(format!("JamTo {:.2}bb must equal effective stack {:.2}bb",amount.as_bb(),stack.as_bb())),
        PreflopAction::RaiseTo(amount) if amount==stack=>Err("RaiseTo at the effective stack is non-canonical; use JamTo".into()),
        _=>Ok(()),
    }
}

fn validate_continuation_contract(edge:&ActionEdge,stack:Bb100)->Result<(),String>{
    match (edge.action,edge.continuation){
        // In multi-player preflop a fold may either end the hand or pass action
        // to another remaining player.
        (PreflopAction::Fold,ContinuationContract::ExactFoldSettlement|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (PreflopAction::Fold,ContinuationContract::RequiresPostflopEv)=>Err("Fold cannot directly require postflop continuation EV".into()),
        (PreflopAction::Fold,ContinuationContract::ExactAllInShowdown)=>Err("Fold cannot directly create an all-in showdown contract".into()),

        // JamTo validity (amount == effective stack) is enforced separately.
        (PreflopAction::JamTo(_),ContinuationContract::ExactAllInShowdown|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (PreflopAction::JamTo(_),ContinuationContract::RequiresPostflopEv)=>Err("all-in jam cannot require postflop continuation EV".into()),
        (PreflopAction::JamTo(_),ContinuationContract::ExactFoldSettlement)=>Err("JamTo cannot itself use ExactFoldSettlement".into()),

        // An all-in call may complete showdown or, in a 3-player hand, pass the
        // decision to the third player. It must not be valued as postflop play.
        (PreflopAction::CallTo(amount),ContinuationContract::ExactAllInShowdown|ContinuationContract::ChildDecision) if amount==stack=>Ok(()),
        (PreflopAction::CallTo(amount),ContinuationContract::RequiresPostflopEv) if amount==stack=>Err("all-in CallTo cannot require postflop continuation EV".into()),
        (PreflopAction::CallTo(_),ContinuationContract::RequiresPostflopEv|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),

        (PreflopAction::RaiseTo(amount),ContinuationContract::ExactAllInShowdown) if amount<stack=>Err("non-all-in raise cannot be marked ExactAllInShowdown".into()),
        (PreflopAction::LimpTo(_)|PreflopAction::RaiseTo(_)|PreflopAction::Check,ContinuationContract::RequiresPostflopEv|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),

        (_,ContinuationContract::Unresolved)=>Ok(()),
        (_,ContinuationContract::ExactFoldSettlement)=>Err("non-fold action cannot use ExactFoldSettlement".into()),
        (_,ContinuationContract::ExactAllInShowdown)=>Err("action is not proven to complete an all-in showdown".into()),
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn root(stack:u16)->PreflopNodeKey{
        PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,Bb100(stack),Player::Btn,vec![]).unwrap()
    }

    #[test]
    fn sizing_is_part_of_node_identity(){
        let a=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,Bb100(1500),Player::Sb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))}]).unwrap();
        let b=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,Bb100(1500),Player::Sb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(250))}]).unwrap();
        assert_ne!(a,b);
    }

    #[test]
    fn action_order_and_actor_are_part_of_node_identity(){
        let a=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,Bb100(1500),Player::Bb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))},HistoryEvent{actor:Player::Sb,action:PreflopAction::CallTo(Bb100(200))}]).unwrap();
        let b=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,Bb100(1500),Player::Bb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::Fold},HistoryEvent{actor:Player::Sb,action:PreflopAction::RaiseTo(Bb100(200))}]).unwrap();
        assert_ne!(a,b);
    }

    #[test]
    fn btn_fold_can_pass_action_to_sb(){
        let spec=PreflopDecisionSpec::new(root(1500),vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::JamTo(Bb100(1500)),continuation:ContinuationContract::ChildDecision},
        ],TreeEvidence::new(TreeVerification::ScreenReference,"fixture").unwrap()).unwrap();
        assert_eq!(spec.edges[0].continuation,ContinuationContract::ChildDecision);
    }

    #[test]
    fn allin_call_can_complete_showdown(){
        let key=PreflopNodeKey::new(GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,Bb100(800),Player::Bb,vec![HistoryEvent{actor:Player::Sb,action:PreflopAction::JamTo(Bb100(800))}]).unwrap();
        let spec=PreflopDecisionSpec::new(key,vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ExactFoldSettlement},
            ActionEdge{action:PreflopAction::CallTo(Bb100(800)),continuation:ContinuationContract::ExactAllInShowdown},
        ],TreeEvidence::new(TreeVerification::VerifiedExactTree,"fixture").unwrap()).unwrap();
        assert_eq!(spec.edges[1].continuation,ContinuationContract::ExactAllInShowdown);
    }

    #[test]
    fn non_allin_raise_cannot_be_declared_exact_showdown(){
        let err=PreflopDecisionSpec::new(root(1500),vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::RaiseTo(Bb100(200)),continuation:ContinuationContract::ExactAllInShowdown},
        ],TreeEvidence::new(TreeVerification::ScreenReference,"fixture").unwrap()).unwrap_err();
        assert!(err.contains("non-all-in raise"));
    }

    #[test]
    fn non_allin_path_can_require_postflop_ev(){
        let spec=PreflopDecisionSpec::new(root(1500),vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::RaiseTo(Bb100(200)),continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::JamTo(Bb100(1500)),continuation:ContinuationContract::ChildDecision},
        ],TreeEvidence::new(TreeVerification::ScreenReference,"fixture").unwrap()).unwrap();
        assert_eq!(spec.edges.len(),3);
    }

    #[test]
    fn raise_to_stack_is_rejected_in_favor_of_jam(){
        let err=PreflopDecisionSpec::new(root(1500),vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::RaiseTo(Bb100(1500)),continuation:ContinuationContract::ChildDecision},
        ],TreeEvidence::new(TreeVerification::MissingExact,"fixture").unwrap()).unwrap_err();
        assert!(err.contains("use JamTo"));
    }

    #[test]
    fn jam_amount_must_equal_effective_stack(){
        let err=PreflopDecisionSpec::new(root(1500),vec![
            ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision},
            ActionEdge{action:PreflopAction::JamTo(Bb100(1499)),continuation:ContinuationContract::ChildDecision},
        ],TreeEvidence::new(TreeVerification::MissingExact,"fixture").unwrap()).unwrap_err();
        assert!(err.contains("must equal effective stack"));
    }

    #[test]
    fn empty_evidence_source_fails_closed(){
        assert!(TreeEvidence::new(TreeVerification::VerifiedExactTree," ").is_err());
    }
}
