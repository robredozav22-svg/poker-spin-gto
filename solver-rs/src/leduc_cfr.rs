use std::collections::HashMap;

#[derive(Debug,Clone)]
struct RegretNode{regrets:Vec<f64>,strategy_sum:Vec<f64>}

impl RegretNode{
    fn new(actions:usize)->Self{Self{regrets:vec![0.0;actions],strategy_sum:vec![0.0;actions]}}
    fn strategy(&self)->Vec<f64>{
        let pos:Vec<f64>=self.regrets.iter().map(|r|r.max(0.0)).collect();
        let z: f64=pos.iter().sum();
        if z>f64::EPSILON{pos.into_iter().map(|x|x/z).collect()}else{vec![1.0/self.regrets.len() as f64;self.regrets.len()]}
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct LeducAverageStrategy{pub rows:HashMap<String,Vec<f64>>}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct LeducValueReport{pub p0_value:f64,pub known_value:f64,pub absolute_error:f64,pub infosets:usize}

#[derive(Debug,Clone)]
pub struct LeducCfr{nodes:HashMap<String,RegretNode>}

impl LeducCfr{
    pub fn new()->Self{Self{nodes:HashMap::new()}}

    pub fn train(&mut self,iterations:usize)->Result<(),String>{
        if iterations==0{return Err("Leduc CFR iterations must be positive".into());}
        for _ in 0..iterations{
            // Six physical cards: rank = card/2, two copies of J/Q/K.
            // Exact ordered chance enumeration: 6*5*4 = 120 deals.
            for p0 in 0u8..6{
                for p1 in 0u8..6{
                    if p1==p0{continue;}
                    for public in 0u8..6{
                        if public==p0||public==p1{continue;}
                        let mut state=State::new(p0,p1,public);
                        self.cfr(&mut state,1.0,1.0,1.0/120.0);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn average_strategy(&self)->LeducAverageStrategy{
        let mut rows=HashMap::new();
        for (k,n) in &self.nodes{
            let z:f64=n.strategy_sum.iter().sum();
            let row=if z>f64::EPSILON{n.strategy_sum.iter().map(|x|x/z).collect()}else{vec![1.0/n.strategy_sum.len() as f64;n.strategy_sum.len()]};
            rows.insert(k.clone(),row);
        }
        LeducAverageStrategy{rows}
    }

    pub fn evaluate_average(&self)->Result<LeducValueReport,String>{
        let avg=self.average_strategy();
        let value=evaluate_strategy_value(&avg)?;
        let known=-0.085606424078f64;
        Ok(LeducValueReport{p0_value:value,known_value:known,absolute_error:(value-known).abs(),infosets:avg.rows.len()})
    }

    fn cfr(&mut self,state:&mut State,p0_reach:f64,p1_reach:f64,chance:f64)->f64{
        if let Some(u)=state.terminal_utility(){return u;}
        if state.round_closed(){state.advance_round();let u=self.cfr(state,p0_reach,p1_reach,chance);state.rewind_round();return u;}

        let player=state.actor;
        let actions=state.legal_actions();
        let key=state.infoset_key(player);
        let strategy={
            let node=self.nodes.entry(key.clone()).or_insert_with(||RegretNode::new(actions.len()));
            assert_eq!(node.regrets.len(),actions.len(),"Leduc action-count mismatch at infoset {key}");
            node.strategy()
        };
        let snapshot=state.clone();
        let mut action_utils=vec![0.0;actions.len()];
        let mut node_util=0.0;
        for (a_i,action) in actions.iter().enumerate(){
            *state=snapshot.clone();state.apply(*action);
            action_utils[a_i]=if player==0{self.cfr(state,p0_reach*strategy[a_i],p1_reach,chance)}else{self.cfr(state,p0_reach,p1_reach*strategy[a_i],chance)};
            node_util+=strategy[a_i]*action_utils[a_i];
        }
        *state=snapshot;
        let node=self.nodes.get_mut(&key).unwrap();
        for i in 0..actions.len(){
            if player==0{
                node.regrets[i]+=chance*p1_reach*(action_utils[i]-node_util);
                node.strategy_sum[i]+=chance*p0_reach*strategy[i];
            }else{
                node.regrets[i]+=chance*p0_reach*(node_util-action_utils[i]);
                node.strategy_sum[i]+=chance*p1_reach*strategy[i];
            }
        }
        node_util
    }
}

impl Default for LeducCfr{fn default()->Self{Self::new()}}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Action{Check,Bet,Fold,Call,Raise}

#[derive(Debug,Clone)]
struct State{
    p0:u8,p1:u8,public:u8,
    round:u8,actor:usize,
    contrib:[i32;2],
    history0:String,history1:String,
    saved_round0:Option<(usize,[i32;2])>,
}

impl State{
    fn new(p0:u8,p1:u8,public:u8)->Self{Self{p0,p1,public,round:0,actor:0,contrib:[1,1],history0:String::new(),history1:String::new(),saved_round0:None}}
    fn h(&self)->&str{if self.round==0{&self.history0}else{&self.history1}}
    fn h_mut(&mut self)->&mut String{if self.round==0{&mut self.history0}else{&mut self.history1}}
    fn bet_size(&self)->i32{if self.round==0{2}else{4}}

    fn legal_actions(&self)->Vec<Action>{
        match self.h(){
            ""|"x"=>vec![Action::Check,Action::Bet],
            "b"|"xb"=>vec![Action::Fold,Action::Call,Action::Raise],
            "br"|"xbr"=>vec![Action::Fold,Action::Call],
            h=>panic!("no legal actions for closed/invalid Leduc history {h}"),
        }
    }

    fn apply(&mut self,a:Action){
        let size=self.bet_size();let other=1-self.actor;
        match a{
            Action::Check=>self.h_mut().push('x'),
            Action::Bet=>{self.contrib[self.actor]+=size;self.h_mut().push('b');}
            Action::Fold=>self.h_mut().push('f'),
            Action::Call=>{let need=self.contrib[other]-self.contrib[self.actor];assert!(need>=0);self.contrib[self.actor]+=need;self.h_mut().push('c');}
            Action::Raise=>{let need=self.contrib[other]-self.contrib[self.actor];assert!(need>=0);self.contrib[self.actor]+=need+size;self.h_mut().push('r');}
        }
        if !matches!(a,Action::Fold){self.actor=other;}
    }

    fn round_closed(&self)->bool{matches!(self.h(),"xx"|"bc"|"xbc"|"brc"|"xbrc")&&self.round==0}

    fn advance_round(&mut self){
        assert!(self.round_closed());
        self.saved_round0=Some((self.actor,self.contrib));
        self.round=1;self.actor=0;self.history1.clear();
    }
    fn rewind_round(&mut self){
        let (actor,contrib)=self.saved_round0.take().unwrap();
        self.round=0;self.actor=actor;self.contrib=contrib;self.history1.clear();
    }

    fn terminal_utility(&self)->Option<f64>{
        let h=self.h();
        if matches!(h,"bf"|"xbf"|"brf"|"xbrf"){
            // actor switched only for non-fold, so actor remains the folder.
            return Some(if self.actor==0{-(self.contrib[0] as f64)}else{self.contrib[1] as f64});
        }
        if self.round==1 && matches!(h,"xx"|"bc"|"xbc"|"brc"|"xbrc"){
            return Some(self.showdown_utility());
        }
        None
    }

    fn showdown_utility(&self)->f64{
        assert_eq!(self.contrib[0],self.contrib[1],"showdown contributions must match");
        let r0=self.p0/2;let r1=self.p1/2;let board=self.public/2;
        let p0_pair=r0==board;let p1_pair=r1==board;
        let cmp=if p0_pair!=p1_pair{p0_pair.cmp(&p1_pair)}else{r0.cmp(&r1)};
        match cmp{std::cmp::Ordering::Greater=>self.contrib[1] as f64,std::cmp::Ordering::Less=>-(self.contrib[0] as f64),std::cmp::Ordering::Equal=>0.0}
    }

    fn infoset_key(&self,player:usize)->String{
        let private=if player==0{self.p0/2}else{self.p1/2};
        if self.round==0{format!("p{player}:r0:c{private}:{}",self.history0)}
        else{format!("p{player}:r1:c{private}:b{}:pre{}:post{}",self.public/2,self.history0,self.history1)}
    }
}

pub fn evaluate_strategy_value(strategy:&LeducAverageStrategy)->Result<f64,String>{
    let mut total=0.0;
    for p0 in 0u8..6{for p1 in 0u8..6{if p1==p0{continue;}for public in 0u8..6{if public==p0||public==p1{continue;}
        let mut state=State::new(p0,p1,public);total+=(1.0/120.0)*ev_state(&mut state,strategy)?;
    }}}
    Ok(total)
}

fn ev_state(state:&mut State,s:&LeducAverageStrategy)->Result<f64,String>{
    if let Some(u)=state.terminal_utility(){return Ok(u);}
    if state.round_closed(){state.advance_round();let u=ev_state(state,s)?;state.rewind_round();return Ok(u);}
    let player=state.actor;let actions=state.legal_actions();let key=state.infoset_key(player);
    let row=s.rows.get(&key).ok_or_else(||format!("missing Leduc strategy row {key}"))?;
    if row.len()!=actions.len(){return Err(format!("Leduc strategy action mismatch {key}"));}
    let snapshot=state.clone();let mut out=0.0;
    for (i,a) in actions.iter().enumerate(){*state=snapshot.clone();state.apply(*a);out+=row[i]*ev_state(state,s)?;}
    *state=snapshot;Ok(out)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn chance_deals_are_exactly_120(){let mut n=0;for a in 0..6{for b in 0..6{if b==a{continue;}for c in 0..6{if c==a||c==b{continue;}n+=1;}}}assert_eq!(n,120);}

    #[test]
    fn showdown_pair_beats_high_card(){
        let mut s=State::new(0,2,1);s.round=1;s.contrib=[5,5];s.history1="xx".into();assert_eq!(s.showdown_utility(),5.0);
    }

    #[test]
    fn trained_leduc_approaches_sequence_form_value(){
        let mut cfr=LeducCfr::new();cfr.train(20_000).unwrap();let r=cfr.evaluate_average().unwrap();
        assert!(r.absolute_error<0.008,"value={} known={} error={}",r.p0_value,r.known_value,r.absolute_error);
        assert!(r.infosets>100,"unexpectedly small infoset count {}",r.infosets);
    }
}
