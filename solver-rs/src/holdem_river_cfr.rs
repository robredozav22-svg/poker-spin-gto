use crate::cards::{disjoint,Card,Combo};
use crate::evaluator::{evaluate_seven,HandScore};

#[derive(Debug,Clone)]
struct RegretRow{regret:[f64;2],sum:[f64;2]}
impl RegretRow{
    fn new()->Self{Self{regret:[0.0;2],sum:[0.0;2]}}
    fn current(&self)->[f64;2]{
        let a=self.regret[0].max(0.0);let b=self.regret[1].max(0.0);let z=a+b;
        if z>f64::EPSILON{[a/z,b/z]}else{[0.5,0.5]}
    }
    fn average(&self)->[f64;2]{let z=self.sum[0]+self.sum[1];if z>f64::EPSILON{[self.sum[0]/z,self.sum[1]/z]}else{[0.5,0.5]}}
}

#[derive(Debug,Clone,PartialEq)]
pub struct RiverStrategy{
    pub p0_root:Vec<[f64;2]>,
    pub p0_facing_bet:Vec<[f64;2]>,
    pub p1_after_check:Vec<[f64;2]>,
    pub p1_facing_bet:Vec<[f64;2]>,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct RiverIndependentReport{
    pub current_p0_value:f64,
    pub p0_best_response_value:f64,
    pub p0_value_vs_p1_best_response:f64,
    pub p0_br_gain:f64,
    pub p1_br_gain:f64,
    pub nashconv:f64,
    pub legal_joint_states:usize,
    pub joint_normalizer:f64,
}

#[derive(Debug,Clone)]
pub struct RiverRangeGame{
    board:[Card;5],p0_hands:Vec<Combo>,p1_hands:Vec<Combo>,p0_weights:Vec<f64>,p1_weights:Vec<f64>,
    pot:f64,bet:f64,joint:Vec<(usize,usize,f64,i8)>,
    p0_root:Vec<RegretRow>,p0_face:Vec<RegretRow>,p1_after_check:Vec<RegretRow>,p1_face:Vec<RegretRow>,
}

impl RiverRangeGame{
    pub fn new(board:[Card;5],p0:Vec<(Combo,f64)>,p1:Vec<(Combo,f64)>,pot:f64,bet:f64)->Result<Self,String>{
        if !pot.is_finite()||pot<=0.0{return Err("river pot must be finite and positive".into());}
        if !bet.is_finite()||bet<=0.0{return Err("river bet must be finite and positive".into());}
        validate_board(board)?;
        let (p0_hands,p0_weights)=validate_range(&board,p0,"p0")?;
        let (p1_hands,p1_weights)=validate_range(&board,p1,"p1")?;
        let mut raw=Vec::new();let mut z=0.0;
        for i in 0..p0_hands.len(){for j in 0..p1_hands.len(){
            if !disjoint(p0_hands[i],p1_hands[j]){continue;}
            let mass=p0_weights[i]*p1_weights[j];if mass<=0.0{continue;}
            let cmp=showdown_cmp(board,p0_hands[i],p1_hands[j]);z+=mass;raw.push((i,j,mass,cmp));
        }}
        if z<=f64::EPSILON{return Err("river ranges have no legal joint private states".into());}
        for x in &mut raw{x.2/=z;}
        let n0=p0_hands.len();let n1=p1_hands.len();
        Ok(Self{board,p0_hands,p1_hands,p0_weights,p1_weights,pot,bet,joint:raw,
            p0_root:(0..n0).map(|_|RegretRow::new()).collect(),p0_face:(0..n0).map(|_|RegretRow::new()).collect(),
            p1_after_check:(0..n1).map(|_|RegretRow::new()).collect(),p1_face:(0..n1).map(|_|RegretRow::new()).collect()})
    }

    pub fn train(&mut self,iterations:usize)->Result<(),String>{
        if iterations==0{return Err("river CFR iterations must be positive".into());}
        for _ in 0..iterations{
            let joint=self.joint.clone();
            for (i,j,chance,cmp) in joint{self.cfr_pair(i,j,cmp,chance,"",1.0,1.0);}
        }
        Ok(())
    }

    pub fn average_strategy(&self)->RiverStrategy{
        RiverStrategy{
            p0_root:self.p0_root.iter().map(RegretRow::average).collect(),
            p0_facing_bet:self.p0_face.iter().map(RegretRow::average).collect(),
            p1_after_check:self.p1_after_check.iter().map(RegretRow::average).collect(),
            p1_facing_bet:self.p1_face.iter().map(RegretRow::average).collect(),
        }
    }

    pub fn evaluate_independent(&self,s:&RiverStrategy)->Result<RiverIndependentReport,String>{
        validate_strategy(s,self.p0_hands.len(),self.p1_hands.len())?;
        let current=self.expected_value(s,s);
        let p0_br=self.brute_force_p0_br(s)?;
        let vs_p1_br=self.brute_force_p1_br(s)?;
        let a=(p0_br-current).max(0.0);let b=(current-vs_p1_br).max(0.0);
        Ok(RiverIndependentReport{current_p0_value:current,p0_best_response_value:p0_br,p0_value_vs_p1_best_response:vs_p1_br,p0_br_gain:a,p1_br_gain:b,nashconv:a+b,legal_joint_states:self.joint.len(),joint_normalizer:self.joint.iter().map(|x|x.2).sum()})
    }

    pub fn board(&self)->[Card;5]{self.board}
    pub fn pot(&self)->f64{self.pot}
    pub fn bet(&self)->f64{self.bet}

    fn cfr_pair(&mut self,i:usize,j:usize,cmp:i8,chance:f64,h:&str,r0:f64,r1:f64)->f64{
        if let Some(u)=terminal_utility(cmp,h,self.pot,self.bet){return u;}
        let (player,row)=match h{
            ""=>(0,self.p0_root[i].current()),
            "c"=>(1,self.p1_after_check[j].current()),
            "b"=>(1,self.p1_face[j].current()),
            "cb"=>(0,self.p0_face[i].current()),
            _=>panic!("invalid river history {h}"),
        };
        let mut u=[0.0;2];let mut v=0.0;
        for a in 0..2{
            let next=next_history(h,a);
            u[a]=if player==0{self.cfr_pair(i,j,cmp,chance,next,r0*row[a],r1)}else{self.cfr_pair(i,j,cmp,chance,next,r0,r1*row[a])};
            v+=row[a]*u[a];
        }
        let target=match h{""=>&mut self.p0_root[i],"c"=>&mut self.p1_after_check[j],"b"=>&mut self.p1_face[j],"cb"=>&mut self.p0_face[i],_=>unreachable!()};
        for a in 0..2{
            if player==0{target.regret[a]+=chance*r1*(u[a]-v);target.sum[a]+=chance*r0*row[a];}
            else{target.regret[a]+=chance*r0*(v-u[a]);target.sum[a]+=chance*r1*row[a];}
        }
        v
    }

    fn expected_value(&self,p0s:&RiverStrategy,p1s:&RiverStrategy)->f64{
        self.joint.iter().map(|(i,j,p,cmp)|p*self.ev_pair(*i,*j,*cmp,"",p0s,p1s)).sum()
    }

    fn ev_pair(&self,i:usize,j:usize,cmp:i8,h:&str,p0s:&RiverStrategy,p1s:&RiverStrategy)->f64{
        if let Some(u)=terminal_utility(cmp,h,self.pot,self.bet){return u;}
        let row=match h{""=>p0s.p0_root[i],"c"=>p1s.p1_after_check[j],"b"=>p1s.p1_facing_bet[j],"cb"=>p0s.p0_facing_bet[i],_=>panic!("bad river history")};
        row[0]*self.ev_pair(i,j,cmp,next_history(h,0),p0s,p1s)+row[1]*self.ev_pair(i,j,cmp,next_history(h,1),p0s,p1s)
    }

    fn brute_force_p0_br(&self,opp:&RiverStrategy)->Result<f64,String>{
        let bits=2*self.p0_hands.len();if bits>20{return Err("p0 brute-force BR fixture support too large".into());}
        let mut best=f64::NEG_INFINITY;
        for mask in 0u64..(1u64<<bits){let p=deterministic_p0(self.p0_hands.len(),self.p1_hands.len(),mask);best=best.max(self.expected_value(&p,opp));}
        Ok(best)
    }
    fn brute_force_p1_br(&self,opp:&RiverStrategy)->Result<f64,String>{
        let bits=2*self.p1_hands.len();if bits>20{return Err("p1 brute-force BR fixture support too large".into());}
        let mut best=f64::INFINITY;
        for mask in 0u64..(1u64<<bits){let p=deterministic_p1(self.p0_hands.len(),self.p1_hands.len(),mask);best=best.min(self.expected_value(opp,&p));}
        Ok(best)
    }
}

fn validate_board(board:[Card;5])->Result<(),String>{
    let mut seen=[false;52];for c in board{if c>=52{return Err("river board card out of range".into());}if seen[c as usize]{return Err("river board contains duplicate card".into());}seen[c as usize]=true;}Ok(())
}
fn validate_range(board:&[Card;5],r:Vec<(Combo,f64)>,name:&str)->Result<(Vec<Combo>,Vec<f64>),String>{
    if r.is_empty(){return Err(format!("{name} river range must not be empty"));}
    let mut hands=Vec::new();let mut weights=Vec::new();
    for (h,w) in r{
        if h[0]>=52||h[1]>=52||h[0]==h[1]{return Err(format!("invalid {name} combo"));}
        if board.contains(&h[0])||board.contains(&h[1]){return Err(format!("{name} combo overlaps public board"));}
        if !w.is_finite()||w<0.0{return Err(format!("invalid {name} range weight"));}
        hands.push(h);weights.push(w);
    }
    if weights.iter().sum::<f64>()<=f64::EPSILON{return Err(format!("{name} range has zero mass"));}
    Ok((hands,weights))
}
fn showdown_cmp(board:[Card;5],a:Combo,b:Combo)->i8{
    let sa:HandScore=evaluate_seven([a[0],a[1],board[0],board[1],board[2],board[3],board[4]]);
    let sb:HandScore=evaluate_seven([b[0],b[1],board[0],board[1],board[2],board[3],board[4]]);
    if sa>sb{1}else if sa<sb{-1}else{0}
}
fn terminal_utility(cmp:i8,h:&str,pot:f64,bet:f64)->Option<f64>{
    let show=|called:bool|{let x=pot/2.0+if called{bet}else{0.0};cmp as f64*x};
    match h{"cc"=>Some(show(false)),"bf"=>Some(pot/2.0),"bc"=>Some(show(true)),"cbf"=>Some(-pot/2.0),"cbc"=>Some(show(true)),_=>None}
}
fn next_history(h:&str,a:usize)->&'static str{match(h,a){("",0)=>"c",("",1)=>"b",("c",0)=>"cc",("c",1)=>"cb",("b",0)=>"bf",("b",1)=>"bc",("cb",0)=>"cbf",("cb",1)=>"cbc",_=>panic!("invalid river transition")}}
fn validate_strategy(s:&RiverStrategy,n0:usize,n1:usize)->Result<(),String>{
    if s.p0_root.len()!=n0||s.p0_facing_bet.len()!=n0||s.p1_after_check.len()!=n1||s.p1_facing_bet.len()!=n1{return Err("river strategy shape mismatch".into());}
    for row in s.p0_root.iter().chain(&s.p0_facing_bet).chain(&s.p1_after_check).chain(&s.p1_facing_bet){if row.iter().any(|x|!x.is_finite()||*x<0.0||*x>1.0)||(row[0]+row[1]-1.0).abs()>1e-9{return Err("invalid river strategy row".into());}}
    Ok(())
}
fn deterministic_p0(n0:usize,n1:usize,mask:u64)->RiverStrategy{
    let mut s=RiverStrategy{p0_root:vec![[0.5,0.5];n0],p0_facing_bet:vec![[0.5,0.5];n0],p1_after_check:vec![[0.5,0.5];n1],p1_facing_bet:vec![[0.5,0.5];n1]};
    for i in 0..n0{s.p0_root[i]=pure(((mask>>i)&1)as usize);s.p0_facing_bet[i]=pure(((mask>>(n0+i))&1)as usize);}s
}
fn deterministic_p1(n0:usize,n1:usize,mask:u64)->RiverStrategy{
    let mut s=RiverStrategy{p0_root:vec![[0.5,0.5];n0],p0_facing_bet:vec![[0.5,0.5];n0],p1_after_check:vec![[0.5,0.5];n1],p1_facing_bet:vec![[0.5,0.5];n1]};
    for i in 0..n1{s.p1_after_check[i]=pure(((mask>>i)&1)as usize);s.p1_facing_bet[i]=pure(((mask>>(n1+i))&1)as usize);}s
}
fn pure(a:usize)->[f64;2]{if a==0{[1.0,0.0]}else{[0.0,1.0]}}

#[cfg(test)]
mod tests{
    use super::*;
    fn c(r:u8,s:u8)->Card{r*4+s}
    fn fixture()->RiverRangeGame{
        let board=[c(0,0),c(5,1),c(7,2),c(9,3),c(1,0)];
        let p0=vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],1.0),([c(10,0),c(8,0)],1.0),([c(6,1),c(4,1)],1.0)];
        let p1=vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],1.0),([c(8,1),c(7,1)],1.0),([c(3,1),c(2,1)],1.0)];
        RiverRangeGame::new(board,p0,p1,4.0,4.0).unwrap()
    }
    #[test]
    fn river_fixture_has_normalized_legal_joint_distribution(){let g=fixture();assert!((g.joint.iter().map(|x|x.2).sum::<f64>()-1.0).abs()<1e-12);assert!(g.joint.len()>8);}
    #[test]
    fn trained_river_strategy_has_low_independent_br_gap(){let mut g=fixture();g.train(50_000).unwrap();let r=g.evaluate_independent(&g.average_strategy()).unwrap();assert!(r.nashconv<0.01,"river nashconv={}",r.nashconv);}
    #[test]
    fn board_overlap_is_rejected(){let b=[c(0,0),c(1,0),c(2,0),c(3,0),c(4,0)];assert!(RiverRangeGame::new(b,vec![([c(0,0),c(12,0)],1.0)],vec![([c(11,0),c(10,0)],1.0)],4.0,4.0).is_err());}
}
