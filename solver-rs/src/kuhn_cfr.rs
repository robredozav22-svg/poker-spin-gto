/// Two-player Kuhn poker validation model.
///
/// This is intentionally independent of Hold'em. It validates the extensive-
/// form ingredients we will need for postflop continuation solving:
/// chance dealing, hidden information, sequential decisions, reach-weighted CFR,
/// average strategy, and an independently brute-forced best-response evaluator.
///
/// Cards: J=0, Q=1, K=2. Each player antes 1. One bet of size 1 is allowed.
/// Histories:
///   ""   P1: Check / Bet
///   "c"  P2: Check / Bet
///   "b"  P2: Fold / Call
///   "cb" P1: Fold / Call
/// Terminal utilities are net chips for P1.

const ACTIONS:usize=2;
const INFOSETS:usize=12;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Node{P1Root,P2AfterCheck,P2FacingBet,P1FacingBet}

#[derive(Debug,Clone,PartialEq)]
pub struct KuhnStrategy{
    /// [infoset][action0, action1]. Action semantics depend on node:
    /// root/after-check = [Check,Bet], facing-bet = [Fold,Call].
    pub probabilities:[[f64;ACTIONS];INFOSETS],
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct KuhnReport{
    pub p1_value:f64,
    pub p1_best_response_value:f64,
    pub p1_value_vs_p2_best_response:f64,
    pub p1_br_gain:f64,
    pub p2_br_gain:f64,
    pub nashconv:f64,
}

#[derive(Debug,Clone)]
pub struct KuhnCfr{
    regrets:[[f64;ACTIONS];INFOSETS],
    strategy_sum:[[f64;ACTIONS];INFOSETS],
}

impl KuhnCfr{
    pub fn new()->Self{Self{regrets:[[0.0;ACTIONS];INFOSETS],strategy_sum:[[0.0;ACTIONS];INFOSETS]}}

    pub fn train(&mut self,iterations:usize)->Result<(),String>{
        if iterations==0{return Err("Kuhn CFR iterations must be positive".into());}
        for _ in 0..iterations{
            // Enumerate all six ordered private-card deals exactly. Chance prob=1/6.
            for p1 in 0..3{
                for p2 in 0..3{
                    if p1==p2{continue;}
                    self.cfr(p1,p2,"",1.0,1.0,1.0/6.0);
                }
            }
        }
        Ok(())
    }

    pub fn average_strategy(&self)->KuhnStrategy{
        let mut out=[[0.5,0.5];INFOSETS];
        for i in 0..INFOSETS{
            let z=self.strategy_sum[i][0]+self.strategy_sum[i][1];
            if z>f64::EPSILON{out[i]=[self.strategy_sum[i][0]/z,self.strategy_sum[i][1]/z];}
        }
        KuhnStrategy{probabilities:out}
    }

    fn current_strategy(&self,infoset:usize)->[f64;2]{
        let r0=self.regrets[infoset][0].max(0.0);let r1=self.regrets[infoset][1].max(0.0);let z=r0+r1;
        if z>f64::EPSILON{[r0/z,r1/z]}else{[0.5,0.5]}
    }

    fn cfr(&mut self,p1_card:usize,p2_card:usize,history:&str,p1_reach:f64,p2_reach:f64,chance:f64)->f64{
        if let Some(u)=terminal_p1_utility(p1_card,p2_card,history){return u;}
        let (player,node)=match history{
            ""=>(0,Node::P1Root),
            "c"=>(1,Node::P2AfterCheck),
            "b"=>(1,Node::P2FacingBet),
            "cb"=>(0,Node::P1FacingBet),
            _=>panic!("invalid Kuhn history {history}"),
        };
        let card=if player==0{p1_card}else{p2_card};
        let info=infoset_index(node,card);
        let strategy=self.current_strategy(info);
        let mut util=[0.0;2];
        let mut node_util=0.0;
        for a in 0..2{
            let next=next_history(history,a);
            util[a]=if player==0{
                self.cfr(p1_card,p2_card,next,p1_reach*strategy[a],p2_reach,chance)
            }else{
                self.cfr(p1_card,p2_card,next,p1_reach,p2_reach*strategy[a],chance)
            };
            node_util+=strategy[a]*util[a];
        }
        // Regret is from the acting player's perspective. Recursion utility is P1's,
        // so P2 regrets invert sign. Counterfactual weighting uses opponent reach.
        for a in 0..2{
            if player==0{
                self.regrets[info][a]+=chance*p2_reach*(util[a]-node_util);
                self.strategy_sum[info][a]+=chance*p1_reach*strategy[a];
            }else{
                self.regrets[info][a]+=chance*p1_reach*(node_util-util[a]);
                self.strategy_sum[info][a]+=chance*p2_reach*strategy[a];
            }
        }
        node_util
    }
}

impl Default for KuhnCfr{fn default()->Self{Self::new()}}

pub fn evaluate_kuhn(strategy:&KuhnStrategy)->Result<KuhnReport,String>{
    validate_strategy(strategy)?;
    let current=expected_p1_value(strategy,strategy);
    let p1_br=brute_force_p1_best_response(strategy);
    let vs_p2_br=brute_force_p2_best_response(strategy);
    let p1_gain=(p1_br-current).max(0.0);
    let p2_gain=(current-vs_p2_br).max(0.0);
    Ok(KuhnReport{p1_value:current,p1_best_response_value:p1_br,p1_value_vs_p2_best_response:vs_p2_br,p1_br_gain:p1_gain,p2_br_gain:p2_gain,nashconv:p1_gain+p2_gain})
}

fn validate_strategy(s:&KuhnStrategy)->Result<(),String>{
    for (i,row) in s.probabilities.iter().enumerate(){
        if row.iter().any(|p|!p.is_finite()||*p<0.0||*p>1.0){return Err(format!("invalid Kuhn probability row {i}"));}
        if (row[0]+row[1]-1.0).abs()>1e-9{return Err(format!("Kuhn row {i} does not sum to 1"));}
    }
    Ok(())
}

/// Both arguments are full strategy containers; only P1 infosets from `p1s` and
/// P2 infosets from `p2s` are used. This makes best-response enumeration clean.
fn expected_p1_value(p1s:&KuhnStrategy,p2s:&KuhnStrategy)->f64{
    let mut total=0.0;
    for p1 in 0..3{for p2 in 0..3{if p1==p2{continue;}total+=(1.0/6.0)*ev_history(p1,p2,"",p1s,p2s);}}
    total
}

fn ev_history(p1:usize,p2:usize,history:&str,p1s:&KuhnStrategy,p2s:&KuhnStrategy)->f64{
    if let Some(u)=terminal_p1_utility(p1,p2,history){return u;}
    let (player,node)=match history{""=>(0,Node::P1Root),"c"=>(1,Node::P2AfterCheck),"b"=>(1,Node::P2FacingBet),"cb"=>(0,Node::P1FacingBet),_=>panic!("bad history")};
    let card=if player==0{p1}else{p2};let info=infoset_index(node,card);
    let row=if player==0{p1s.probabilities[info]}else{p2s.probabilities[info]};
    row[0]*ev_history(p1,p2,next_history(history,0),p1s,p2s)+row[1]*ev_history(p1,p2,next_history(history,1),p1s,p2s)
}

/// P1 has six binary information sets => only 64 deterministic policies.
fn brute_force_p1_best_response(opponent:&KuhnStrategy)->f64{
    let mut best=f64::NEG_INFINITY;
    for mask in 0u64..64{
        let candidate=deterministic_policy(0,mask);
        best=best.max(expected_p1_value(&candidate,opponent));
    }
    best
}

/// P2 also has six binary information sets. Since utility is P1's, P2 BR minimizes it.
fn brute_force_p2_best_response(opponent:&KuhnStrategy)->f64{
    let mut best=f64::INFINITY;
    for mask in 0u64..64{
        let candidate=deterministic_policy(1,mask);
        best=best.min(expected_p1_value(opponent,&candidate));
    }
    best
}

fn deterministic_policy(player:usize,mask:u64)->KuhnStrategy{
    let mut p=[[0.5,0.5];INFOSETS];
    let nodes=if player==0{[Node::P1Root,Node::P1FacingBet]}else{[Node::P2AfterCheck,Node::P2FacingBet]};
    let mut bit=0usize;
    for node in nodes{for card in 0..3{let action=((mask>>bit)&1) as usize;p[infoset_index(node,card)]=if action==0{[1.0,0.0]}else{[0.0,1.0]};bit+=1;}}
    KuhnStrategy{probabilities:p}
}

fn infoset_index(node:Node,card:usize)->usize{
    let base=match node{Node::P1Root=>0,Node::P2AfterCheck=>3,Node::P2FacingBet=>6,Node::P1FacingBet=>9};base+card
}

fn next_history(history:&str,action:usize)->&'static str{
    match (history,action){
        ("",0)=>"c",("",1)=>"b",
        ("c",0)=>"cc",("c",1)=>"cb",
        ("b",0)=>"bf",("b",1)=>"bc",
        ("cb",0)=>"cbf",("cb",1)=>"cbc",
        _=>panic!("invalid Kuhn transition {history} action={action}"),
    }
}

fn terminal_p1_utility(p1:usize,p2:usize,history:&str)->Option<f64>{
    let p1_wins=p1>p2;
    match history{
        "cc"=>Some(if p1_wins{1.0}else{-1.0}),
        "bf"=>Some(1.0),
        "bc"=>Some(if p1_wins{2.0}else{-2.0}),
        "cbf"=>Some(-1.0),
        "cbc"=>Some(if p1_wins{2.0}else{-2.0}),
        _=>None,
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn terminal_utilities_are_zero_sum_net_chip_values(){
        assert_eq!(terminal_p1_utility(2,0,"cc"),Some(1.0));
        assert_eq!(terminal_p1_utility(0,2,"cc"),Some(-1.0));
        assert_eq!(terminal_p1_utility(2,0,"bc"),Some(2.0));
        assert_eq!(terminal_p1_utility(0,2,"cbc"),Some(-2.0));
        assert_eq!(terminal_p1_utility(0,2,"bf"),Some(1.0));
        assert_eq!(terminal_p1_utility(2,0,"cbf"),Some(-1.0));
    }

    #[test]
    fn trained_kuhn_approaches_known_game_value(){
        let mut cfr=KuhnCfr::new();cfr.train(100_000).unwrap();
        let avg=cfr.average_strategy();let report=evaluate_kuhn(&avg).unwrap();
        let known=-1.0/18.0;
        assert!((report.p1_value-known).abs()<0.002,"value={} known={}",report.p1_value,known);
        assert!(report.nashconv<0.01,"nashconv={}",report.nashconv);
    }

    #[test]
    fn brute_force_best_response_gains_are_nonnegative(){
        let uniform=KuhnStrategy{probabilities:[[0.5,0.5];INFOSETS]};
        let r=evaluate_kuhn(&uniform).unwrap();assert!(r.p1_br_gain>=0.0&&r.p2_br_gain>=0.0&&r.nashconv>=0.0);
    }
}
