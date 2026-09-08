use std::collections::BTreeMap;

use crate::cards::{disjoint,Card,Combo};
use crate::evaluator::evaluate_seven;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct TurnJointState{pub p0_index:usize,pub p1_index:usize,pub probability:f64}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct RiverConditionalState{pub p0_index:usize,pub p1_index:usize,pub probability:f64}

#[derive(Debug,Clone,PartialEq)]
pub struct RiverChanceChild{pub river:Card,pub board:[Card;5],pub probability:f64,pub conditional_states:Vec<RiverConditionalState>}

#[derive(Debug,Clone,PartialEq)]
pub struct TurnChanceTree{
    board:[Card;4],p0_hands:Vec<Combo>,p1_hands:Vec<Combo>,joint:Vec<TurnJointState>,children:Vec<RiverChanceChild>,raw_transition_count:usize,
}

impl TurnChanceTree{
    pub fn new(board:[Card;4],p0:Vec<(Combo,f64)>,p1:Vec<(Combo,f64)>)->Result<Self,String>{
        validate_board4(board)?;
        let (p0_hands,p0_weights)=validate_range(&board,p0,"p0")?;
        let (p1_hands,p1_weights)=validate_range(&board,p1,"p1")?;
        let mut joint=Vec::new();let mut z=0.0;
        for i in 0..p0_hands.len(){for j in 0..p1_hands.len(){
            if !disjoint(p0_hands[i],p1_hands[j]){continue;}
            let mass=p0_weights[i]*p1_weights[j];if mass<=0.0{continue;}
            z+=mass;joint.push(TurnJointState{p0_index:i,p1_index:j,probability:mass});
        }}
        if z<=f64::EPSILON{return Err("turn ranges have no legal joint private states".into());}
        for state in &mut joint{state.probability/=z;}

        let mut by_river:BTreeMap<Card,Vec<(usize,usize,f64)>>=BTreeMap::new();let mut raw_transition_count=0usize;
        for state in &joint{
            let h0=p0_hands[state.p0_index];let h1=p1_hands[state.p1_index];let mut legal=0usize;
            for river in 0u8..52{
                if board.contains(&river)||h0.contains(&river)||h1.contains(&river){continue;}
                legal+=1;raw_transition_count+=1;
                by_river.entry(river).or_default().push((state.p0_index,state.p1_index,state.probability/44.0));
            }
            if legal!=44{return Err(format!("turn private state must have exactly 44 legal river cards, got {legal}"));}
        }

        let mut children=Vec::with_capacity(by_river.len());
        for (river,rows) in by_river{
            let probability:f64=rows.iter().map(|x|x.2).sum();if probability<=f64::EPSILON{continue;}
            let conditional_states=rows.into_iter().map(|(p0_index,p1_index,mass)|RiverConditionalState{p0_index,p1_index,probability:mass/probability}).collect();
            let mut board5=[0u8;5];board5[..4].copy_from_slice(&board);board5[4]=river;
            children.push(RiverChanceChild{river,board:board5,probability,conditional_states});
        }
        let tree=Self{board,p0_hands,p1_hands,joint,children,raw_transition_count};tree.validate_mass()?;Ok(tree)
    }

    pub fn board(&self)->[Card;4]{self.board}
    pub fn p0_hands(&self)->&[Combo]{&self.p0_hands}
    pub fn p1_hands(&self)->&[Combo]{&self.p1_hands}
    pub fn p0_hand(&self,index:usize)->Combo{self.p0_hands[index]}
    pub fn p1_hand(&self,index:usize)->Combo{self.p1_hands[index]}
    pub fn joint_states(&self)->&[TurnJointState]{&self.joint}
    pub fn children(&self)->&[RiverChanceChild]{&self.children}
    pub fn raw_transition_count(&self)->usize{self.raw_transition_count}

    pub fn validate_mass(&self)->Result<(),String>{
        let joint_sum:f64=self.joint.iter().map(|x|x.probability).sum();if (joint_sum-1.0).abs()>1e-12{return Err(format!("turn joint mass not normalized: {joint_sum}"));}
        let child_sum:f64=self.children.iter().map(|x|x.probability).sum();if (child_sum-1.0).abs()>1e-12{return Err(format!("river child chance mass not normalized: {child_sum}"));}
        for child in &self.children{let s:f64=child.conditional_states.iter().map(|x|x.probability).sum();if (s-1.0).abs()>1e-12{return Err(format!("conditional river state mass not normalized for card {}: {s}",child.river));}}
        Ok(())
    }

    pub fn exact_checkdown_showdown_sign(&self)->f64{
        let mut ev=0.0;
        for child in &self.children{for state in &child.conditional_states{
            let h0=self.p0_hands[state.p0_index];let h1=self.p1_hands[state.p1_index];
            let s0=evaluate_seven([h0[0],h0[1],child.board[0],child.board[1],child.board[2],child.board[3],child.board[4]]);
            let s1=evaluate_seven([h1[0],h1[1],child.board[0],child.board[1],child.board[2],child.board[3],child.board[4]]);
            ev+=child.probability*state.probability*if s0>s1{1.0}else if s0<s1{-1.0}else{0.0};
        }}ev
    }

    pub fn direct_checkdown_showdown_sign(&self)->f64{
        let mut ev=0.0;
        for state in &self.joint{
            let h0=self.p0_hands[state.p0_index];let h1=self.p1_hands[state.p1_index];
            for river in 0u8..52{
                if self.board.contains(&river)||h0.contains(&river)||h1.contains(&river){continue;}
                let board=[self.board[0],self.board[1],self.board[2],self.board[3],river];
                let s0=evaluate_seven([h0[0],h0[1],board[0],board[1],board[2],board[3],board[4]]);
                let s1=evaluate_seven([h1[0],h1[1],board[0],board[1],board[2],board[3],board[4]]);
                ev+=state.probability*(1.0/44.0)*if s0>s1{1.0}else if s0<s1{-1.0}else{0.0};
            }
        }ev
    }
}

fn validate_board4(board:[Card;4])->Result<(),String>{let mut seen=[false;52];for c in board{if c>=52{return Err("turn board card out of range".into());}if seen[c as usize]{return Err("turn board contains duplicate card".into());}seen[c as usize]=true;}Ok(())}
fn validate_range(board:&[Card;4],range:Vec<(Combo,f64)>,name:&str)->Result<(Vec<Combo>,Vec<f64>),String>{
    if range.is_empty(){return Err(format!("{name} turn range must not be empty"));}
    let mut hands=Vec::new();let mut weights=Vec::new();
    for (h,w) in range{if h[0]>=52||h[1]>=52||h[0]==h[1]{return Err(format!("invalid {name} combo"));}if board.contains(&h[0])||board.contains(&h[1]){return Err(format!("{name} combo overlaps turn board"));}if !w.is_finite()||w<0.0{return Err(format!("invalid {name} range weight"));}hands.push(h);weights.push(w);}
    if weights.iter().sum::<f64>()<=f64::EPSILON{return Err(format!("{name} range has zero mass"));}Ok((hands,weights))
}

#[cfg(test)]
mod tests{
    use super::*;fn c(r:u8,s:u8)->Card{r*4+s}
    fn fixture()->TurnChanceTree{TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4),([c(6,1),c(4,1)],0.2)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5),([c(3,1),c(2,1)],0.3)]).unwrap()}
    #[test]fn every_joint_private_state_has_44_exact_river_transitions(){let t=fixture();assert_eq!(t.raw_transition_count(),t.joint_states().len()*44);t.validate_mass().unwrap();}
    #[test]fn child_aggregation_matches_direct_state_first_enumeration(){let t=fixture();let a=t.exact_checkdown_showdown_sign();let b=t.direct_checkdown_showdown_sign();assert!((a-b).abs()<1e-12,"aggregated={a} direct={b}");}
    #[test]fn immutable_private_support_getters_are_consistent(){let t=fixture();assert_eq!(t.p0_hand(0),t.p0_hands()[0]);assert_eq!(t.p1_hand(0),t.p1_hands()[0]);}
    #[test]fn overlapping_board_card_is_rejected(){let board=[c(0,0),c(1,0),c(2,0),c(3,0)];assert!(TurnChanceTree::new(board,vec![([c(0,0),c(12,0)],1.0)],vec![([c(11,0),c(10,0)],1.0)]).is_err());}
}
