use crate::cards::{Card,Combo};
use crate::evaluator::evaluate_seven;
use crate::holdem_turn_chance::TurnChanceTree;

#[derive(Debug,Clone)]
struct Row{regret:[f64;2],sum:[f64;2]}
impl Row{
    fn new()->Self{Self{regret:[0.0;2],sum:[0.0;2]}}
    fn current(&self)->[f64;2]{let a=self.regret[0].max(0.0);let b=self.regret[1].max(0.0);let z=a+b;if z>f64::EPSILON{[a/z,b/z]}else{[0.5,0.5]}}
    fn average(&self)->[f64;2]{let z=self.sum[0]+self.sum[1];if z>f64::EPSILON{[self.sum[0]/z,self.sum[1]/z]}else{[0.5,0.5]}}
}

#[derive(Debug,Clone,PartialEq)]
pub struct TurnRiverStrategy{
    pub turn_p0_root:Vec<[f64;2]>,
    pub turn_p0_facing_bet:Vec<[f64;2]>,
    pub turn_p1_after_check:Vec<[f64;2]>,
    pub turn_p1_facing_bet:Vec<[f64;2]>,
    pub river_p0_root:Vec<Vec<[f64;2]>>,
    pub river_p0_facing_bet:Vec<Vec<[f64;2]>>,
    pub river_p1_after_check:Vec<Vec<[f64;2]>>,
    pub river_p1_facing_bet:Vec<Vec<[f64;2]>>,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct TurnRiverValueReport{pub p0_value:f64,pub joint_private_states:usize,pub raw_river_transitions:usize,pub normalized:bool}

#[derive(Debug,Clone)]
pub struct HoldemTurnRiverCfr{
    chance:TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64,
    t0:Vec<Row>,t0f:Vec<Row>,t1c:Vec<Row>,t1f:Vec<Row>,
    r0:Vec<Vec<Row>>,r0f:Vec<Vec<Row>>,r1c:Vec<Vec<Row>>,r1f:Vec<Vec<Row>>,
}

impl HoldemTurnRiverCfr{
    pub fn new(chance:TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64)->Result<Self,String>{
        if !pot.is_finite()||pot<=0.0{return Err("turn-river pot must be finite and positive".into());}
        if !turn_bet.is_finite()||turn_bet<=0.0{return Err("turn bet must be finite and positive".into());}
        if !river_bet.is_finite()||river_bet<=0.0{return Err("river bet must be finite and positive".into());}
        chance.validate_mass()?;
        let n0=chance.p0_hands().len();let n1=chance.p1_hands().len();
        let mk0=|| (0..52).map(|_|(0..n0).map(|_|Row::new()).collect()).collect();
        let mk1=|| (0..52).map(|_|(0..n1).map(|_|Row::new()).collect()).collect();
        Ok(Self{chance,pot,turn_bet,river_bet,
            t0:(0..n0).map(|_|Row::new()).collect(),t0f:(0..n0).map(|_|Row::new()).collect(),
            t1c:(0..n1).map(|_|Row::new()).collect(),t1f:(0..n1).map(|_|Row::new()).collect(),
            r0:mk0(),r0f:mk0(),r1c:mk1(),r1f:mk1()})
    }

    pub fn train(&mut self,iterations:usize)->Result<(),String>{
        if iterations==0{return Err("turn-river CFR iterations must be positive".into());}
        let joint=self.chance.joint_states().to_vec();
        for _ in 0..iterations{
            for s in &joint{self.cfr_turn(s.p0_index,s.p1_index,s.probability,"",1.0,1.0);}
        }
        Ok(())
    }

    pub fn average_strategy(&self)->TurnRiverStrategy{
        TurnRiverStrategy{
            turn_p0_root:self.t0.iter().map(Row::average).collect(),turn_p0_facing_bet:self.t0f.iter().map(Row::average).collect(),
            turn_p1_after_check:self.t1c.iter().map(Row::average).collect(),turn_p1_facing_bet:self.t1f.iter().map(Row::average).collect(),
            river_p0_root:self.r0.iter().map(|v|v.iter().map(Row::average).collect()).collect(),
            river_p0_facing_bet:self.r0f.iter().map(|v|v.iter().map(Row::average).collect()).collect(),
            river_p1_after_check:self.r1c.iter().map(|v|v.iter().map(Row::average).collect()).collect(),
            river_p1_facing_bet:self.r1f.iter().map(|v|v.iter().map(Row::average).collect()).collect(),
        }
    }

    pub fn evaluate_average(&self)->Result<TurnRiverValueReport,String>{self.evaluate_strategy(&self.average_strategy())}

    pub fn evaluate_strategy(&self,s:&TurnRiverStrategy)->Result<TurnRiverValueReport,String>{
        validate_strategy(s,self.chance.p0_hands().len(),self.chance.p1_hands().len())?;
        let mut ev=0.0;
        for state in self.chance.joint_states(){ev+=state.probability*self.ev_turn(state.p0_index,state.p1_index,"",s)?;}
        Ok(TurnRiverValueReport{p0_value:ev,joint_private_states:self.chance.joint_states().len(),raw_river_transitions:self.chance.raw_transition_count(),normalized:true})
    }

    pub fn pure_checkdown_strategy(&self)->TurnRiverStrategy{
        let n0=self.chance.p0_hands().len();let n1=self.chance.p1_hands().len();
        let mut s=TurnRiverStrategy{
            turn_p0_root:vec![[1.0,0.0];n0],turn_p0_facing_bet:vec![[0.0,1.0];n0],
            turn_p1_after_check:vec![[1.0,0.0];n1],turn_p1_facing_bet:vec![[0.0,1.0];n1],
            river_p0_root:vec![vec![[1.0,0.0];n0];52],river_p0_facing_bet:vec![vec![[0.0,1.0];n0];52],
            river_p1_after_check:vec![vec![[1.0,0.0];n1];52],river_p1_facing_bet:vec![vec![[0.0,1.0];n1];52],
        };
        // Facing-bet rows are unreachable under pure checkdown but remain legal normalized policies.
        for r in &mut s.turn_p0_facing_bet{*r=[1.0,0.0];}for r in &mut s.turn_p1_facing_bet{*r=[1.0,0.0];}
        for river in 0..52{for r in &mut s.river_p0_facing_bet[river]{*r=[1.0,0.0];}for r in &mut s.river_p1_facing_bet[river]{*r=[1.0,0.0];}}
        s
    }

    pub fn exact_checkdown_value(&self)->f64{self.chance.exact_checkdown_showdown_sign()*(self.pot/2.0)}

    fn cfr_turn(&mut self,i:usize,j:usize,chance:f64,h:&str,rp0:f64,rp1:f64)->f64{
        if let Some(u)=turn_terminal(h,self.pot){return u;}
        if matches!(h,"cc"|"bc"|"cbc"){
            let pot2=if h=="cc"{self.pot}else{self.pot+2.0*self.turn_bet};
            let a=self.chance.p0_hand(i);let b=self.chance.p1_hand(j);let board=self.chance.board();
            let mut out=0.0;let mut n=0usize;
            for river in 0u8..52{
                if board.contains(&river)||a.contains(&river)||b.contains(&river){continue;}
                n+=1;out+=self.cfr_river(i,j,river,chance/44.0,pot2,"",rp0,rp1)/44.0;
            }
            assert_eq!(n,44);return out;
        }
        let (player,row)=match h{""=>(0,self.t0[i].current()),"c"=>(1,self.t1c[j].current()),"b"=>(1,self.t1f[j].current()),"cb"=>(0,self.t0f[i].current()),_=>panic!("invalid turn history {h}")};
        let mut u=[0.0;2];let mut v=0.0;
        for a in 0..2{let next=next_history(h,a);u[a]=if player==0{self.cfr_turn(i,j,chance,next,rp0*row[a],rp1)}else{self.cfr_turn(i,j,chance,next,rp0,rp1*row[a])};v+=row[a]*u[a];}
        let target=match h{""=>&mut self.t0[i],"c"=>&mut self.t1c[j],"b"=>&mut self.t1f[j],"cb"=>&mut self.t0f[i],_=>unreachable!()};
        for a in 0..2{if player==0{target.regret[a]+=chance*rp1*(u[a]-v);target.sum[a]+=chance*rp0*row[a];}else{target.regret[a]+=chance*rp0*(v-u[a]);target.sum[a]+=chance*rp1*row[a];}}
        v
    }

    fn cfr_river(&mut self,i:usize,j:usize,river:Card,chance:f64,pot:f64,h:&str,rp0:f64,rp1:f64)->f64{
        let cmp=self.showdown_cmp(i,j,river);
        if let Some(u)=river_terminal(cmp,h,pot,self.river_bet){return u;}
        let ri=river as usize;
        let (player,row)=match h{""=>(0,self.r0[ri][i].current()),"c"=>(1,self.r1c[ri][j].current()),"b"=>(1,self.r1f[ri][j].current()),"cb"=>(0,self.r0f[ri][i].current()),_=>panic!("invalid river history {h}")};
        let mut u=[0.0;2];let mut v=0.0;
        for a in 0..2{let next=next_history(h,a);u[a]=if player==0{self.cfr_river(i,j,river,chance,pot,next,rp0*row[a],rp1)}else{self.cfr_river(i,j,river,chance,pot,next,rp0,rp1*row[a])};v+=row[a]*u[a];}
        let target=match h{""=>&mut self.r0[ri][i],"c"=>&mut self.r1c[ri][j],"b"=>&mut self.r1f[ri][j],"cb"=>&mut self.r0f[ri][i],_=>unreachable!()};
        for a in 0..2{if player==0{target.regret[a]+=chance*rp1*(u[a]-v);target.sum[a]+=chance*rp0*row[a];}else{target.regret[a]+=chance*rp0*(v-u[a]);target.sum[a]+=chance*rp1*row[a];}}
        v
    }

    fn ev_turn(&self,i:usize,j:usize,h:&str,s:&TurnRiverStrategy)->Result<f64,String>{
        if let Some(u)=turn_terminal(h,self.pot){return Ok(u);}
        if matches!(h,"cc"|"bc"|"cbc"){
            let pot2=if h=="cc"{self.pot}else{self.pot+2.0*self.turn_bet};
            let a=self.chance.p0_hand(i);let b=self.chance.p1_hand(j);let board=self.chance.board();let mut out=0.0;let mut n=0usize;
            for river in 0u8..52{if board.contains(&river)||a.contains(&river)||b.contains(&river){continue;}n+=1;out+=self.ev_river(i,j,river,pot2,"",s)?/44.0;}if n!=44{return Err("turn evaluator did not enumerate exactly 44 legal rivers".into());}return Ok(out);
        }
        let row=match h{""=>s.turn_p0_root[i],"c"=>s.turn_p1_after_check[j],"b"=>s.turn_p1_facing_bet[j],"cb"=>s.turn_p0_facing_bet[i],_=>return Err("invalid turn evaluation history".into())};
        Ok(row[0]*self.ev_turn(i,j,next_history(h,0),s)?+row[1]*self.ev_turn(i,j,next_history(h,1),s)?)
    }

    fn ev_river(&self,i:usize,j:usize,river:Card,pot:f64,h:&str,s:&TurnRiverStrategy)->Result<f64,String>{
        let cmp=self.showdown_cmp(i,j,river);if let Some(u)=river_terminal(cmp,h,pot,self.river_bet){return Ok(u);}
        let r=river as usize;let row=match h{""=>s.river_p0_root[r][i],"c"=>s.river_p1_after_check[r][j],"b"=>s.river_p1_facing_bet[r][j],"cb"=>s.river_p0_facing_bet[r][i],_=>return Err("invalid river evaluation history".into())};
        Ok(row[0]*self.ev_river(i,j,river,pot,next_history(h,0),s)?+row[1]*self.ev_river(i,j,river,pot,next_history(h,1),s)?)
    }

    fn showdown_cmp(&self,i:usize,j:usize,river:Card)->i8{
        let a=self.chance.p0_hand(i);let b=self.chance.p1_hand(j);let q=self.chance.board();
        let s0=evaluate_seven([a[0],a[1],q[0],q[1],q[2],q[3],river]);let s1=evaluate_seven([b[0],b[1],q[0],q[1],q[2],q[3],river]);
        if s0>s1{1}else if s0<s1{-1}else{0}
    }
}

fn turn_terminal(h:&str,pot:f64)->Option<f64>{match h{"bf"=>Some(pot/2.0),"cbf"=>Some(-pot/2.0),_=>None}}
fn river_terminal(cmp:i8,h:&str,pot:f64,bet:f64)->Option<f64>{let show=|called:bool|cmp as f64*(pot/2.0+if called{bet}else{0.0});match h{"cc"=>Some(show(false)),"bf"=>Some(pot/2.0),"bc"=>Some(show(true)),"cbf"=>Some(-pot/2.0),"cbc"=>Some(show(true)),_=>None}}
fn next_history(h:&str,a:usize)->&'static str{match(h,a){("",0)=>"c",("",1)=>"b",("c",0)=>"cc",("c",1)=>"cb",("b",0)=>"bf",("b",1)=>"bc",("cb",0)=>"cbf",("cb",1)=>"cbc",_=>panic!("invalid transition {h} {a}")}}

fn validate_strategy(s:&TurnRiverStrategy,n0:usize,n1:usize)->Result<(),String>{
    if s.turn_p0_root.len()!=n0||s.turn_p0_facing_bet.len()!=n0||s.turn_p1_after_check.len()!=n1||s.turn_p1_facing_bet.len()!=n1{return Err("turn strategy shape mismatch".into());}
    if s.river_p0_root.len()!=52||s.river_p0_facing_bet.len()!=52||s.river_p1_after_check.len()!=52||s.river_p1_facing_bet.len()!=52{return Err("river strategy must have 52 public-card slots".into());}
    let mut rows:Vec<&[f64;2]>=Vec::new();rows.extend(s.turn_p0_root.iter());rows.extend(s.turn_p0_facing_bet.iter());rows.extend(s.turn_p1_after_check.iter());rows.extend(s.turn_p1_facing_bet.iter());
    for r in 0..52{if s.river_p0_root[r].len()!=n0||s.river_p0_facing_bet[r].len()!=n0||s.river_p1_after_check[r].len()!=n1||s.river_p1_facing_bet[r].len()!=n1{return Err("river private support shape mismatch".into());}rows.extend(s.river_p0_root[r].iter());rows.extend(s.river_p0_facing_bet[r].iter());rows.extend(s.river_p1_after_check[r].iter());rows.extend(s.river_p1_facing_bet[r].iter());}
    for row in rows{if row.iter().any(|x|!x.is_finite()||*x<0.0||*x>1.0)||(row[0]+row[1]-1.0).abs()>1e-9{return Err("invalid normalized strategy row".into());}}
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;fn c(r:u8,s:u8)->Card{r*4+s}
    fn game()->HoldemTurnRiverCfr{
        let chance=TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)]).unwrap();
        HoldemTurnRiverCfr::new(chance,4.0,4.0,8.0).unwrap()
    }
    #[test]fn pure_checkdown_value_matches_exact_turn_chance_layer(){let g=game();let s=g.pure_checkdown_strategy();let v=g.evaluate_strategy(&s).unwrap().p0_value;assert!((v-g.exact_checkdown_value()).abs()<1e-12,"strategy={v} exact={}",g.exact_checkdown_value());}
    #[test]fn average_strategy_is_normalized_after_training(){let mut g=game();g.train(100).unwrap();let s=g.average_strategy();let r=g.evaluate_strategy(&s).unwrap();assert!(r.normalized);assert!(r.p0_value.is_finite());}
}
