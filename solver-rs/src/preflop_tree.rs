use crate::tree::Player;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Bb100(pub u16);
impl Bb100{pub const fn from_hundredths(value:u16)->Self{Self(value)} pub const fn one_bb()->Self{Self(100)} pub fn as_bb(self)->f64{self.0 as f64/100.0}}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)] pub enum GameFormat{Spin3Max,SpinHeadsUp}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)] pub enum PayoutProfile{WinnerTakeAllChipEv,ExplicitIcmProfile(u32)}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)] pub enum PreflopAction{Fold,Check,LimpTo(Bb100),CallTo(Bb100),RaiseTo(Bb100),JamTo(Bb100)}
impl PreflopAction{pub fn amount_to(self)->Option<Bb100>{match self{Self::LimpTo(v)|Self::CallTo(v)|Self::RaiseTo(v)|Self::JamTo(v)=>Some(v),Self::Fold|Self::Check=>None}}}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)] pub struct HistoryEvent{pub actor:Player,pub action:PreflopAction}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct PreflopNodeKey{
    pub format:GameFormat,pub payout:PayoutProfile,pub tree_profile_id:String,
    pub effective_stack:Bb100,pub actor:Player,pub history:Vec<HistoryEvent>,
}
impl PreflopNodeKey{
    pub fn new(format:GameFormat,payout:PayoutProfile,tree_profile_id:impl Into<String>,effective_stack:Bb100,actor:Player,history:Vec<HistoryEvent>)->Result<Self,String>{
        if effective_stack.0==0{return Err("effective stack must be positive".into());}
        let tree_profile_id=tree_profile_id.into();if tree_profile_id.trim().is_empty(){return Err("tree_profile_id must not be empty".into());}
        for event in &history{validate_action_amount(event.action,effective_stack)?;}
        Ok(Self{format,payout,tree_profile_id,effective_stack,actor,history})
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)] pub enum TreeVerification{VerifiedExactTree,CrossChecked,ScreenReference,MissingExact}
#[derive(Debug,Clone,PartialEq,Eq,Hash)] pub struct TreeEvidence{pub verification:TreeVerification,pub source_id:String}
impl TreeEvidence{pub fn new(verification:TreeVerification,source_id:impl Into<String>)->Result<Self,String>{let source_id=source_id.into();if source_id.trim().is_empty(){return Err("tree evidence source_id must not be empty".into());}Ok(Self{verification,source_id})}}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)] pub enum ContinuationContract{ChildDecision,ExactFoldSettlement,ExactAllInShowdown,RequiresPostflopEv,Unresolved}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActionEdge{
    pub action:PreflopAction,
    pub continuation:ContinuationContract,
    /// Required only for ChildDecision. Terminal/postflop edges must not carry it.
    pub next_actor:Option<Player>,
}
impl ActionEdge{
    pub fn child(action:PreflopAction,next_actor:Player)->Self{Self{action,continuation:ContinuationContract::ChildDecision,next_actor:Some(next_actor)}}
    pub fn terminal(action:PreflopAction,continuation:ContinuationContract)->Self{Self{action,continuation,next_actor:None}}
}

#[derive(Debug,Clone,PartialEq,Eq)] pub struct PreflopDecisionSpec{pub key:PreflopNodeKey,pub edges:Vec<ActionEdge>,pub evidence:TreeEvidence}
impl PreflopDecisionSpec{
    pub fn new(key:PreflopNodeKey,edges:Vec<ActionEdge>,evidence:TreeEvidence)->Result<Self,String>{
        if edges.len()<2{return Err("decision node must expose at least two actions".into());}
        let mut seen=std::collections::HashSet::new();
        for edge in &edges{
            validate_action_amount(edge.action,key.effective_stack)?;
            if !seen.insert(edge.action){return Err(format!("duplicate action in decision node: {:?}",edge.action));}
            validate_continuation_contract(edge,key.effective_stack)?;
        }
        Ok(Self{key,edges,evidence})
    }

    pub fn child_key(&self,edge_index:usize)->Result<PreflopNodeKey,String>{
        let edge=self.edges.get(edge_index).ok_or_else(||"edge index out of bounds".to_string())?;
        if edge.continuation!=ContinuationContract::ChildDecision{return Err("edge is not a ChildDecision".into());}
        let next_actor=edge.next_actor.ok_or_else(||"ChildDecision missing next_actor".to_string())?;
        let mut history=self.key.history.clone();
        history.push(HistoryEvent{actor:self.key.actor,action:edge.action});
        PreflopNodeKey::new(self.key.format,self.key.payout,self.key.tree_profile_id.clone(),self.key.effective_stack,next_actor,history)
    }
}

fn validate_action_amount(action:PreflopAction,stack:Bb100)->Result<(),String>{
    if let Some(amount)=action.amount_to(){if amount.0==0{return Err("preflop action amount must be positive".into());}if amount>stack{return Err(format!("preflop action amount {:.2}bb exceeds effective stack {:.2}bb",amount.as_bb(),stack.as_bb()));}}
    match action{PreflopAction::JamTo(amount) if amount!=stack=>Err(format!("JamTo {:.2}bb must equal effective stack {:.2}bb",amount.as_bb(),stack.as_bb())),PreflopAction::RaiseTo(amount) if amount==stack=>Err("RaiseTo at the effective stack is non-canonical; use JamTo".into()),_=>Ok(())}
}

fn validate_continuation_contract(edge:&ActionEdge,stack:Bb100)->Result<(),String>{
    if edge.continuation==ContinuationContract::ChildDecision && edge.next_actor.is_none(){return Err("ChildDecision requires next_actor".into());}
    if edge.continuation!=ContinuationContract::ChildDecision && edge.next_actor.is_some(){return Err("non-child edge must not specify next_actor".into());}
    match (edge.action,edge.continuation){
        (PreflopAction::Fold,ContinuationContract::ExactFoldSettlement|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (PreflopAction::Fold,ContinuationContract::RequiresPostflopEv)=>Err("Fold cannot directly require postflop continuation EV".into()),
        (PreflopAction::Fold,ContinuationContract::ExactAllInShowdown)=>Err("Fold cannot directly create an all-in showdown contract".into()),
        (PreflopAction::JamTo(_),ContinuationContract::ExactAllInShowdown|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (PreflopAction::JamTo(_),ContinuationContract::RequiresPostflopEv)=>Err("all-in jam cannot require postflop continuation EV".into()),
        (PreflopAction::JamTo(_),ContinuationContract::ExactFoldSettlement)=>Err("JamTo cannot itself use ExactFoldSettlement".into()),
        (PreflopAction::CallTo(amount),ContinuationContract::ExactAllInShowdown|ContinuationContract::ChildDecision) if amount==stack=>Ok(()),
        (PreflopAction::CallTo(amount),ContinuationContract::RequiresPostflopEv) if amount==stack=>Err("all-in CallTo cannot require postflop continuation EV".into()),
        (PreflopAction::CallTo(_),ContinuationContract::RequiresPostflopEv|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (PreflopAction::RaiseTo(amount),ContinuationContract::ExactAllInShowdown) if amount<stack=>Err("non-all-in raise cannot be marked ExactAllInShowdown".into()),
        (PreflopAction::LimpTo(_)|PreflopAction::RaiseTo(_)|PreflopAction::Check,ContinuationContract::RequiresPostflopEv|ContinuationContract::ChildDecision|ContinuationContract::Unresolved)=>Ok(()),
        (_,ContinuationContract::ExactFoldSettlement)=>Err("non-fold action cannot use ExactFoldSettlement".into()),
        (_,ContinuationContract::ExactAllInShowdown)=>Err("action is not proven to complete an all-in showdown".into()),
    }
}

#[cfg(test)]
mod tests{
    use super::*;const PROFILE:&str="fixture-tree-v1";
    fn root(stack:u16)->PreflopNodeKey{PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,PROFILE,Bb100(stack),Player::Btn,vec![]).unwrap()}

    #[test] fn sizing_is_part_of_node_identity(){let a=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,PROFILE,Bb100(1500),Player::Sb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))}]).unwrap();let b=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,PROFILE,Bb100(1500),Player::Sb,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(250))}]).unwrap();assert_ne!(a,b);}
    #[test] fn solution_tree_profile_is_part_of_node_identity(){let a=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,"wizard-general",Bb100(1500),Player::Btn,vec![]).unwrap();let b=PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv,"wizard-research",Bb100(1500),Player::Btn,vec![]).unwrap();assert_ne!(a,b);}
    #[test] fn child_key_appends_parent_action_and_next_actor(){let spec=PreflopDecisionSpec::new(root(1500),vec![ActionEdge::child(PreflopAction::Fold,Player::Sb),ActionEdge::child(PreflopAction::RaiseTo(Bb100(200)),Player::Sb)],TreeEvidence::new(TreeVerification::ScreenReference,"fixture").unwrap()).unwrap();let child=spec.child_key(1).unwrap();assert_eq!(child.actor,Player::Sb);assert_eq!(child.history,vec![HistoryEvent{actor:Player::Btn,action:PreflopAction::RaiseTo(Bb100(200))}]);}
    #[test] fn child_requires_next_actor(){let err=PreflopDecisionSpec::new(root(1500),vec![ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ChildDecision,next_actor:None},ActionEdge::child(PreflopAction::JamTo(Bb100(1500)),Player::Sb)],TreeEvidence::new(TreeVerification::MissingExact,"fixture").unwrap()).unwrap_err();assert!(err.contains("requires next_actor"));}
    #[test] fn terminal_forbids_next_actor(){let err=PreflopDecisionSpec::new(root(1500),vec![ActionEdge{action:PreflopAction::Fold,continuation:ContinuationContract::ExactFoldSettlement,next_actor:Some(Player::Sb)},ActionEdge::child(PreflopAction::JamTo(Bb100(1500)),Player::Sb)],TreeEvidence::new(TreeVerification::MissingExact,"fixture").unwrap()).unwrap_err();assert!(err.contains("must not specify next_actor"));}
    #[test] fn allin_call_can_complete_showdown(){let key=PreflopNodeKey::new(GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,PROFILE,Bb100(800),Player::Bb,vec![HistoryEvent{actor:Player::Sb,action:PreflopAction::JamTo(Bb100(800))}]).unwrap();let spec=PreflopDecisionSpec::new(key,vec![ActionEdge::terminal(PreflopAction::Fold,ContinuationContract::ExactFoldSettlement),ActionEdge::terminal(PreflopAction::CallTo(Bb100(800)),ContinuationContract::ExactAllInShowdown)],TreeEvidence::new(TreeVerification::VerifiedExactTree,"fixture").unwrap()).unwrap();assert_eq!(spec.edges[1].continuation,ContinuationContract::ExactAllInShowdown);}
    #[test] fn non_allin_raise_cannot_be_declared_exact_showdown(){let err=PreflopDecisionSpec::new(root(1500),vec![ActionEdge::child(PreflopAction::Fold,Player::Sb),ActionEdge::terminal(PreflopAction::RaiseTo(Bb100(200)),ContinuationContract::ExactAllInShowdown)],TreeEvidence::new(TreeVerification::ScreenReference,"fixture").unwrap()).unwrap_err();assert!(err.contains("non-all-in raise"));}
    #[test] fn raise_to_stack_is_rejected_in_favor_of_jam(){let err=PreflopDecisionSpec::new(root(1500),vec![ActionEdge::child(PreflopAction::Fold,Player::Sb),ActionEdge::child(PreflopAction::RaiseTo(Bb100(1500)),Player::Sb)],TreeEvidence::new(TreeVerification::MissingExact,"fixture").unwrap()).unwrap_err();assert!(err.contains("use JamTo"));}
    #[test] fn empty_profile_and_evidence_source_fail_closed(){assert!(PreflopNodeKey::new(GameFormat::Spin3Max,PayoutProfile::WinnerTakeAllChipEv," ",Bb100(1500),Player::Btn,vec![]).is_err());assert!(TreeEvidence::new(TreeVerification::VerifiedExactTree," ").is_err());}
}
