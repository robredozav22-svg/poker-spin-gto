use crate::cards::{disjoint,Card};
use crate::evaluator::evaluate_seven;
use crate::holdem_turn_chance::TurnChanceTree;

pub const TURN_RIVER_PATHS:usize=3;
pub const PATH_CHECK_CHECK:usize=0;
pub const PATH_BET_CALL:usize=1;
pub const PATH_CHECK_BET_CALL:usize=2;

#[derive(Debug,Clone)]
struct Row{regret:[f64;2],sum:[f64;2]}
impl Row{
    fn new()->Self{Self{regret:[0.0;2],sum:[0.0;2]}}
    fn current(&self)->[f64;2]{let a=self.regret[0].max(0.0);let b=self.regret[1].max(0.0);let z=a+b;if z>f64::EPSILON{[a/z,b/z]}else{[0.5,0.5]}}
    fn average(&self)->[f64;2]{let z=self.sum[0]+self.sum[1];if z>f64::EPSILON{[self.sum[0]/z,self.sum[1]/z]}else{[0.5,0.5]}}
}

type StrategyStreet=Vec<Vec<Vec<[f64;2]>>>;
type RegretStreet=Vec<Vec<Vec<Row>>>;

#[derive(Debug,Clone,PartialEq)]
pub struct TurnRiverStrategy{
    pub turn_p0_root:Vec<[f64;2]>,pub turn_p0_facing_bet:Vec<[f64;2]>,
    pub turn_p1_after_check:Vec<[f64;2]>,pub turn_p1_facing_bet:Vec<[f64;2]>,
    pub river_p0_root:StrategyStreet,pub river_p0_facing_bet:StrategyStreet,
    pub river_p1_after_check:StrategyStreet,pub river_p1_facing_bet:StrategyStreet,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct TurnRiverValueReport{pub p0_value:f64,pub joint_private_states:usize,pub raw_river_transitions:usize,pub normalized:bool}

#[derive(Debug,Clone)]
pub struct HoldemTurnRiverCfr{
    chance:TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64,
    t0:Vec<Row>,t0f:Vec<Row>,t1c:Vec<Row>,t1f:Vec<Row>,
    r0:RegretStreet,r0f:RegretStreet,r1c:RegretStreet,r1f:RegretStreet,
    showdown:Vec<Vec<Vec<i8>>>,
}

impl HoldemTurnRiverCfr{
    pub fn new(chance:TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64)->Result<Self,String>{
        if !pot.is_finite()||pot<=0.0{return Err("turn-river pot must be finite and positive".into());}
        if !turn_bet.is_finite()||turn_bet<=0.0{return Err("turn bet must be finite and positive".into());}
        if !river_bet.is_finite()||river_bet<=0.0{return Err("river bet must be finite and positive".into());}
        chance.validate_mass()?;
        let n0=chance.p0_hands().len();let n1=chance.p1_hands().len();
        let mk0=|| (0..TURN_RIVER_PATHS).map(|_|(0..52).map(|_|(0..n0).map(|_|Row::new()).collect()).collect()).collect();
        let mk1=|| (0..TURN_RIVER_PATHS).map(|_|(0..52).map(|_|(0..n1).map(|_|Row::new()).collect()).collect()).collect();
        let showdown=precompute_showdowns(&chance);
        Ok(Self{chance,pot,turn_bet,river_bet,
            t0:(0..n0).map(|_|Row::new()).collect(),t0f:(0..n0).map(|_|Row::new()).collect(),
            t1c:(0..n1).map(|_|Row::new()).collect(),t1f:(0..n1).map(|_|Row::new()).collect(),
            r0:mk0(),r0f:mk0(),r1c:mk1(),r1f:mk1(),showdown})
    }

    pub fn chance_tree(&self)->&TurnChanceTree{&self.chance}
    pub fn pot(&self)->f64{self.pot}
    pub fn turn_bet(&self)->f64{self.turn_bet}
    pub fn river_bet(&self)->f64{self.river_bet}

    pub fn train(&mut self,iterations:usize)->Result<(),String>{
        if iterations==0{return Err("turn-river CFR iterations must be positive".into());}
        let joint=self.chance.joint_states().to_vec();
        for _ in 0..iterations{for s in &joint{self.cfr_turn(s.p0_index,s.p1_index,s.probability,"",1.0,1.0);}}
        Ok(())
    }

    pub fn average_strategy(&self)->TurnRiverStrategy{
        fn avg(x:&RegretStreet)->StrategyStreet{x.iter().map(|path|path.iter().map(|river|river.iter().map(Row::average).collect()).collect()).collect()}
        TurnRiverStrategy{
            turn_p0_root:self.t0.iter().map(Row::average).collect(),turn_p0_facing_bet:self.t0f.iter().map(Row::average).collect(),
            turn_p1_after_check:self.t1c.iter().map(Row::average).collect(),turn_p1_facing_bet:self.t1f.iter().map(Row::average).collect(),
            river_p0_root:avg(&self.r0),river_p0_facing_bet:avg(&self.r0f),river_p1_after_check:avg(&self.r1c),river_p1_facing_bet:avg(&self.r1f),
        }
    }

    pub fn evaluate_average(&self)->Result<TurnRiverValueReport,String>{self.evaluate_strategy(&self.average_strategy())}

    pub fn evaluate_strategy(&self,s:&TurnRiverStrategy)->Result<TurnRiverValueReport,String>{
        validate_strategy(s,self.chance.p0_hands().len(),self.chance.p1_hands().len())?;
        let mut ev=0.0;for x in self.chance.joint_states(){ev+=x.probability*self.ev_turn(x.p0_index,x.p1_index,"",s)?;}
        Ok(TurnRiverValueReport{p0_value:ev,joint_private_states:self.chance.joint_states().len(),raw_river_transitions:self.chance.raw_transition_count(),normalized:true})
    }

    pub fn pure_checkdown_strategy(&self)->TurnRiverStrategy{
        let n0=self.chance.p0_hands().len();let n1=self.chance.p1_hands().len();
        let mk0=|row:[f64;2]|vec![vec![vec![row;n0];52];TURN_RIVER_PATHS];
        let mk1=|row:[f64;2]|vec![vec![vec![row;n1];52];TURN_RIVER_PATHS];
        TurnRiverStrategy{
            turn_p0_root:vec![[1.0,0.0];n0],turn_p0_facing_bet:vec![[1.0,0.0];n0],
            turn_p1_after_check:vec![[1.0,0.0];n1],turn_p1_facing_bet:vec![[1.0,0.0];n1],
            river_p0_root:mk0([1.0,0.0]),river_p0_facing_bet:mk0([1.0,0.0]),
            river_p1_after_check:mk1([1.0,0.0]),river_p1_facing_bet:mk1([1.0,0.0]),
        }
    }

    pub fn exact_checkdown_value(&self)->f64{self.chance.exact_checkdown_showdown_sign()*(self.pot/2.0)}

    fn cfr_turn(&mut self,i:usize,j:usize,chance:f64,h:&str,rp0:f64,rp1:f64)->f64{
        if let Some(u)=turn_terminal(h,self.pot){return u;}
        if let Some(path)=turn_path(h){
            let pot2=if path==PATH_CHECK_CHECK{self.pot}else{self.pot+2.0*self.turn_bet};
            let a=self.chance.p0_hand(i);let b=self.chance.p1_hand(j);let board=self.chance.board();let mut out=0.0;let mut n=0;
            for river in 0u8..52{if board.contains(&river)||a.contains(&river)||b.contains(&river){continue;}n+=1;out+=self.cfr_river(path,i,j,river,chance/44.0,pot2,"",rp0,rp1)/44.0;}
            assert_eq!(n,44);return out;
        }
        let (player,row)=match h{""=>(0,self.t0[i].current()),"c"=>(1,self.t1c[j].current()),"b"=>(1,self.t1f[j].current()),"cb"=>(0,self.t0f[i].current()),_=>panic!("invalid turn history {h}")};
        let mut u=[0.0;2];let mut v=0.0;for a in 0..2{let next=next_history(h,a);u[a]=if player==0{self.cfr_turn(i,j,chance,next,rp0*row[a],rp1)}else{self.cfr_turn(i,j,chance,next,rp0,rp1*row[a])};v+=row[a]*u[a];}
        let target=match h{""=>&mut self.t0[i],"c"=>&mut self.t1c[j],"b"=>&mut self.t1f[j],"cb"=>&mut self.t0f[i],_=>unreachable!()};
        for a in 0..2{if player==0{target.regret[a]+=chance*rp1*(u[a]-v);target.sum[a]+=chance*rp0*row[a];}else{target.regret[a]+=chance*rp0*(v-u[a]);target.sum[a]+=chance*rp1*row[a];}}v
    }

    fn cfr_river(&mut self,path:usize,i:usize,j:usize,river:Card,chance:f64,pot:f64,h:&str,rp0:f64,rp1:f64)->f64{
        let cmp=self.showdown[river as usize][i][j];if let Some(u)=river_terminal(cmp,h,pot,self.river_bet){return u;}
        let r=river as usize;let (player,row)=match h{""=>(0,self.r0[path][r][i].current()),"c"=>(1,self.r1c[path][r][j].current()),"b"=>(1,self.r1f[path][r][j].current()),"cb"=>(0,self.r0f[path][r][i].current()),_=>panic!("invalid river history {h}")};
        let mut u=[0.0;2];let mut v=0.0;for a in 0..2{let next=next_history(h,a);u[a]=if player==0{self.cfr_river(path,i,j,river,chance,pot,next,rp0*row[a],rp1)}else{self.cfr_river(path,i,j,river,chance,pot,next,rp0,rp1*row[a])};v+=row[a]*u[a];}
        let target=match h{""=>&mut self.r0[path][r][i],"c"=>&mut self.r1c[path][r][j],"b"=>&mut self.r1f[path][r][j],"cb"=>&mut self.r0f[path][r][i],_=>unreachable!()};
        for a in 0..2{if player==0{target.regret[a]+=chance*rp1*(u[a]-v);target.sum[a]+=chance*rp0*row[a];}else{target.regret[a]+=chance*rp0*(v-u[a]);target.sum[a]+=chance*rp1*row[a];}}v
    }

    fn ev_turn(&self,i:usize,j:usize,h:&str,s:&TurnRiverStrategy)->Result<f64,String>{
        if let Some(u)=turn_terminal(h,self.pot){return Ok(u);}
        if let Some(path)=turn_path(h){let pot2=if path==PATH_CHECK_CHECK{self.pot}else{self.pot+2.0*self.turn_bet};let a=self.chance.p0_hand(i);let b=self.chance.p1_hand(j);let board=self.chance.board();let mut out=0.0;let mut n=0;for river in 0u8..52{if board.contains(&river)||a.contains(&river)||b.contains(&river){continue;}n+=1;out+=self.ev_river(path,i,j,river,pot2,"",s)?/44.0;}if n!=44{return Err("turn evaluator did not enumerate exactly 44 legal rivers".into());}return Ok(out);}
        let row=match h{""=>s.turn_p0_root[i],"c"=>s.turn_p1_after_check[j],"b"=>s.turn_p1_facing_bet[j],"cb"=>s.turn_p0_facing_bet[i],_=>return Err("invalid turn evaluation history".into())};
        Ok(row[0]*self.ev_turn(i,j,next_history(h,0),s)?+row[1]*self.ev_turn(i,j,next_history(h,1),s)?)
    }

    fn ev_river(&self,path:usize,i:usize,j:usize,river:Card,pot:f64,h:&str,s:&TurnRiverStrategy)->Result<f64,String>{
        let cmp=self.showdown[river as usize][i][j];if let Some(u)=river_terminal(cmp,h,pot,self.river_bet){return Ok(u);}
        let r=river as usize;let row=match h{""=>s.river_p0_root[path][r][i],"c"=>s.river_p1_after_check[path][r][j],"b"=>s.river_p1_facing_bet[path][r][j],"cb"=>s.river_p0_facing_bet[path][r][i],_=>return Err("invalid river evaluation history".into())};
        Ok(row[0]*self.ev_river(path,i,j,river,pot,next_history(h,0),s)?+row[1]*self.ev_river(path,i,j,river,pot,next_history(h,1),s)?)
    }
}

fn precompute_showdowns(chance:&TurnChanceTree)->Vec<Vec<Vec<i8>>>{
    let n0=chance.p0_hands().len();let n1=chance.p1_hands().len();let q=chance.board();let mut out=vec![vec![vec![0i8;n1];n0];52];
    for river in 0u8..52{if q.contains(&river){continue;}for i in 0..n0{let a=chance.p0_hand(i);if a.contains(&river){continue;}for j in 0..n1{let b=chance.p1_hand(j);if b.contains(&river)||!disjoint(a,b){continue;}let x=evaluate_seven([a[0],a[1],q[0],q[1],q[2],q[3],river]);let y=evaluate_seven([b[0],b[1],q[0],q[1],q[2],q[3],river]);out[river as usize][i][j]=if x>y{1}else if x<y{-1}else{0};}}}out
}

pub fn turn_path(h:&str)->Option<usize>{match h{"cc"=>Some(PATH_CHECK_CHECK),"bc"=>Some(PATH_BET_CALL),"cbc"=>Some(PATH_CHECK_BET_CALL),_=>None}}
fn turn_terminal(h:&str,pot:f64)->Option<f64>{match h{"bf"=>Some(pot/2.0),"cbf"=>Some(-pot/2.0),_=>None}}
fn river_terminal(cmp:i8,h:&str,pot:f64,bet:f64)->Option<f64>{let show=|called:bool|cmp as f64*(pot/2.0+if called{bet}else{0.0});match h{"cc"=>Some(show(false)),"bf"=>Some(pot/2.0),"bc"=>Some(show(true)),"cbf"=>Some(-pot/2.0),"cbc"=>Some(show(true)),_=>None}}
fn next_history(h:&str,a:usize)->&'static str{match(h,a){("",0)=>"c",("",1)=>"b",("c",0)=>"cc",("c",1)=>"cb",("b",0)=>"bf",("b",1)=>"bc",("cb",0)=>"cbf",("cb",1)=>"cbc",_=>panic!("invalid transition {h} {a}")}}

fn validate_strategy(s:&TurnRiverStrategy,n0:usize,n1:usize)->Result<(),String>{
    if s.turn_p0_root.len()!=n0||s.turn_p0_facing_bet.len()!=n0||s.turn_p1_after_check.len()!=n1||s.turn_p1_facing_bet.len()!=n1{return Err("turn strategy shape mismatch".into());}
    fn collect<'a>(x:&'a StrategyStreet,n:usize,rows:&mut Vec<&'a [f64;2]>)->Result<(),String>{if x.len()!=TURN_RIVER_PATHS{return Err("river strategy must separate all turn public histories".into());}for path in x{if path.len()!=52{return Err("river strategy must have 52 public-card slots per turn history".into());}for river in path{if river.len()!=n{return Err("river private support shape mismatch".into());}rows.extend(river.iter());}}Ok(())}
    let mut rows:Vec<&[f64;2]>=Vec::new();rows.extend(s.turn_p0_root.iter());rows.extend(s.turn_p0_facing_bet.iter());rows.extend(s.turn_p1_after_check.iter());rows.extend(s.turn_p1_facing_bet.iter());collect(&s.river_p0_root,n0,&mut rows)?;collect(&s.river_p0_facing_bet,n0,&mut rows)?;collect(&s.river_p1_after_check,n1,&mut rows)?;collect(&s.river_p1_facing_bet,n1,&mut rows)?;
    for row in rows{if row.iter().any(|x|!x.is_finite()||*x<0.0||*x>1.0)||(row[0]+row[1]-1.0).abs()>1e-9{return Err("invalid normalized strategy row".into());}}Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;fn c(r:u8,s:u8)->Card{r*4+s}
    fn game()->HoldemTurnRiverCfr{let chance=TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7),([c(10,0),c(8,0)],0.4)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8),([c(8,1),c(7,1)],0.5)]).unwrap();HoldemTurnRiverCfr::new(chance,4.0,4.0,8.0).unwrap()}
    #[test]fn pure_checkdown_value_matches_exact_turn_chance_layer(){let g=game();let v=g.evaluate_strategy(&g.pure_checkdown_strategy()).unwrap().p0_value;assert!((v-g.exact_checkdown_value()).abs()<1e-12);}
    #[test]fn river_infosets_are_separate_for_three_public_turn_histories(){let g=game();let s=g.average_strategy();assert_eq!(s.river_p0_root.len(),3);assert_eq!(s.river_p1_after_check.len(),3);}
    #[test]fn average_strategy_is_normalized_after_training(){let mut g=game();g.train(100).unwrap();assert!(g.evaluate_average().unwrap().p0_value.is_finite());}
}
