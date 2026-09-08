use crate::cards::{Card,Combo};
use crate::evaluator::evaluate_seven;
use crate::holdem_turn_chance::TurnChanceTree;
use crate::holdem_turn_river_cfr::{TurnRiverStrategy,PATH_BET_CALL,PATH_CHECK_BET_CALL,PATH_CHECK_CHECK,TURN_RIVER_PATHS};

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct TurnRiverBrReport{
    pub current_p0_value:f64,pub p0_best_response_value:f64,pub p0_value_vs_p1_best_response:f64,
    pub p0_br_gain:f64,pub p1_br_gain:f64,pub nashconv:f64,
}

pub fn evaluate_independent_br(chance:&TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy,current_p0_value:f64)->Result<TurnRiverBrReport,String>{
    if !pot.is_finite()||pot<=0.0||!turn_bet.is_finite()||turn_bet<=0.0||!river_bet.is_finite()||river_bet<=0.0{return Err("invalid two-street BR sizing".into());}
    validate_strategy_shape(chance,s)?;
    let p0_br=p0_best_response(chance,pot,turn_bet,river_bet,s);
    let vs_p1_br=p0_value_vs_p1_best_response(chance,pot,turn_bet,river_bet,s);
    let p0_br_gain=(p0_br-current_p0_value).max(0.0);let p1_br_gain=(current_p0_value-vs_p1_br).max(0.0);
    Ok(TurnRiverBrReport{current_p0_value,p0_best_response_value:p0_br,p0_value_vs_p1_best_response:vs_p1_br,p0_br_gain,p1_br_gain,nashconv:p0_br_gain+p1_br_gain})
}

fn joint_matrix(chance:&TurnChanceTree)->Vec<Vec<f64>>{
    let mut joint=vec![vec![0.0;chance.p1_hands().len()];chance.p0_hands().len()];
    for x in chance.joint_states(){joint[x.p0_index][x.p1_index]=x.probability;}joint
}

fn p0_best_response(chance:&TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let joint=joint_matrix(chance);let mut total=0.0;
    for i in 0..chance.p0_hands().len(){
        let opp=&joint[i];
        let check=p0_turn_root_check(chance,i,opp,pot,turn_bet,river_bet,s);
        let bet=p0_turn_root_bet(chance,i,opp,pot,turn_bet,river_bet,s);
        total+=check.max(bet);
    }total
}

fn p0_turn_root_check(chance:&TurnChanceTree,i:usize,opp:&[f64],pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let mut checked=vec![0.0;opp.len()];let mut bet=vec![0.0;opp.len()];
    for j in 0..opp.len(){checked[j]=opp[j]*s.turn_p1_after_check[j][0];bet[j]=opp[j]*s.turn_p1_after_check[j][1];}
    let check_cont=p0_river_br(chance,PATH_CHECK_CHECK,i,&checked,pot,river_bet,s);
    let fold=-pot/2.0*bet.iter().sum::<f64>();
    let call=p0_river_br(chance,PATH_CHECK_BET_CALL,i,&bet,pot+2.0*turn_bet,river_bet,s);
    check_cont+fold.max(call)
}

fn p0_turn_root_bet(chance:&TurnChanceTree,i:usize,opp:&[f64],pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let mut called=vec![0.0;opp.len()];let mut fold_mass=0.0;
    for j in 0..opp.len(){fold_mass+=opp[j]*s.turn_p1_facing_bet[j][0];called[j]=opp[j]*s.turn_p1_facing_bet[j][1];}
    pot/2.0*fold_mass+p0_river_br(chance,PATH_BET_CALL,i,&called,pot+2.0*turn_bet,river_bet,s)
}

fn p0_river_br(chance:&TurnChanceTree,path:usize,i:usize,opp:&[f64],pot:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let hero=chance.p0_hand(i);let board=chance.board();let mut total=0.0;
    for river in 0u8..52{
        if board.contains(&river)||hero.contains(&river){continue;}let r=river as usize;
        let mut check_show=0.0;let mut after_check_bet=vec![0.0;opp.len()];
        for j in 0..opp.len(){let v=chance.p1_hand(j);if opp[j]<=0.0||v.contains(&river)||overlap(hero,v){continue;}let w=opp[j]/44.0;let c=cmp(chance,hero,v,river);check_show+=w*s.river_p1_after_check[path][r][j][0]*show(c,pot,false,river_bet);after_check_bet[j]=w*s.river_p1_after_check[path][r][j][1];}
        let face_fold=-pot/2.0*after_check_bet.iter().sum::<f64>();
        let mut face_call=0.0;for j in 0..opp.len(){if after_check_bet[j]>0.0{face_call+=after_check_bet[j]*show(cmp(chance,hero,chance.p1_hand(j),river),pot,true,river_bet);}}
        let root_check=check_show+face_fold.max(face_call);
        let mut root_bet=0.0;
        for j in 0..opp.len(){let v=chance.p1_hand(j);if opp[j]<=0.0||v.contains(&river)||overlap(hero,v){continue;}let w=opp[j]/44.0;let row=s.river_p1_facing_bet[path][r][j];root_bet+=w*(row[0]*(pot/2.0)+row[1]*show(cmp(chance,hero,v,river),pot,true,river_bet));}
        total+=root_check.max(root_bet);
    }total
}

fn p0_value_vs_p1_best_response(chance:&TurnChanceTree,pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let joint=joint_matrix(chance);let n0=chance.p0_hands().len();let n1=chance.p1_hands().len();let mut total=0.0;
    for j in 0..n1{
        let mut p0_check=vec![0.0;n0];let mut p0_bet=vec![0.0;n0];
        for i in 0..n0{p0_check[i]=joint[i][j]*s.turn_p0_root[i][0];p0_bet[i]=joint[i][j]*s.turn_p0_root[i][1];}
        total+=p1_after_turn_check(chance,j,&p0_check,pot,turn_bet,river_bet,s);
        total+=p1_facing_turn_bet(chance,j,&p0_bet,pot,turn_bet,river_bet,s);
    }total
}

fn p1_after_turn_check(chance:&TurnChanceTree,j:usize,hero_weights:&[f64],pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let check=p1_river_br(chance,PATH_CHECK_CHECK,j,hero_weights,pot,river_bet,s);
    let mut called=vec![0.0;hero_weights.len()];let mut fold_value=0.0;
    for i in 0..hero_weights.len(){fold_value+=hero_weights[i]*s.turn_p0_facing_bet[i][0]*(-pot/2.0);called[i]=hero_weights[i]*s.turn_p0_facing_bet[i][1];}
    let bet=fold_value+p1_river_br(chance,PATH_CHECK_BET_CALL,j,&called,pot+2.0*turn_bet,river_bet,s);
    check.min(bet)
}

fn p1_facing_turn_bet(chance:&TurnChanceTree,j:usize,hero_weights:&[f64],pot:f64,turn_bet:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let fold=pot/2.0*hero_weights.iter().sum::<f64>();
    let call=p1_river_br(chance,PATH_BET_CALL,j,hero_weights,pot+2.0*turn_bet,river_bet,s);
    fold.min(call)
}

fn p1_river_br(chance:&TurnChanceTree,path:usize,j:usize,hero_weights:&[f64],pot:f64,river_bet:f64,s:&TurnRiverStrategy)->f64{
    let villain=chance.p1_hand(j);let board=chance.board();let mut total=0.0;
    for river in 0u8..52{
        if board.contains(&river)||villain.contains(&river){continue;}let r=river as usize;
        let mut after_p0_check=vec![0.0;hero_weights.len()];let mut facing_p0_bet=vec![0.0;hero_weights.len()];
        for i in 0..hero_weights.len(){let h=chance.p0_hand(i);if hero_weights[i]<=0.0||h.contains(&river)||overlap(h,villain){continue;}let w=hero_weights[i]/44.0;after_p0_check[i]=w*s.river_p0_root[path][r][i][0];facing_p0_bet[i]=w*s.river_p0_root[path][r][i][1];}
        let mut p1_check=0.0;for i in 0..hero_weights.len(){if after_p0_check[i]>0.0{p1_check+=after_p0_check[i]*show(cmp(chance,chance.p0_hand(i),villain,river),pot,false,river_bet);}}
        let mut p1_bet=0.0;for i in 0..hero_weights.len(){if after_p0_check[i]<=0.0{continue;}let row=s.river_p0_facing_bet[path][r][i];let c=cmp(chance,chance.p0_hand(i),villain,river);p1_bet+=after_p0_check[i]*(row[0]*(-pot/2.0)+row[1]*show(c,pot,true,river_bet));}
        let after_check=p1_check.min(p1_bet);
        let fold=pot/2.0*facing_p0_bet.iter().sum::<f64>();
        let mut call=0.0;for i in 0..hero_weights.len(){if facing_p0_bet[i]>0.0{call+=facing_p0_bet[i]*show(cmp(chance,chance.p0_hand(i),villain,river),pot,true,river_bet);}}
        total+=after_check+fold.min(call);
    }total
}

fn cmp(chance:&TurnChanceTree,a:Combo,b:Combo,river:Card)->i8{let q=chance.board();let x=evaluate_seven([a[0],a[1],q[0],q[1],q[2],q[3],river]);let y=evaluate_seven([b[0],b[1],q[0],q[1],q[2],q[3],river]);if x>y{1}else if x<y{-1}else{0}}
fn show(c:i8,pot:f64,called:bool,bet:f64)->f64{c as f64*(pot/2.0+if called{bet}else{0.0})}
fn overlap(a:Combo,b:Combo)->bool{a[0]==b[0]||a[0]==b[1]||a[1]==b[0]||a[1]==b[1]}

fn validate_strategy_shape(chance:&TurnChanceTree,s:&TurnRiverStrategy)->Result<(),String>{
    let n0=chance.p0_hands().len();let n1=chance.p1_hands().len();
    if s.turn_p0_root.len()!=n0||s.turn_p0_facing_bet.len()!=n0||s.turn_p1_after_check.len()!=n1||s.turn_p1_facing_bet.len()!=n1{return Err("two-street BR turn strategy shape mismatch".into());}
    fn check(x:&Vec<Vec<Vec<[f64;2]>>>,n:usize)->bool{x.len()==TURN_RIVER_PATHS&&x.iter().all(|p|p.len()==52&&p.iter().all(|r|r.len()==n))}
    if !check(&s.river_p0_root,n0)||!check(&s.river_p0_facing_bet,n0)||!check(&s.river_p1_after_check,n1)||!check(&s.river_p1_facing_bet,n1){return Err("two-street BR river public-history/private-support mismatch".into());}Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;use crate::holdem_turn_river_cfr::HoldemTurnRiverCfr;fn c(r:u8,s:u8)->Card{r*4+s}
    fn fixture()->(TurnChanceTree,HoldemTurnRiverCfr){let t=TurnChanceTree::new([c(0,0),c(5,1),c(7,2),c(9,3)],vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7)],vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8)]).unwrap();let g=HoldemTurnRiverCfr::new(t.clone(),4.0,4.0,8.0).unwrap();(t,g)}
    #[test]fn best_response_bounds_current_value_after_training(){let (t,mut g)=fixture();g.train(1000).unwrap();let s=g.average_strategy();let v=g.evaluate_strategy(&s).unwrap().p0_value;let r=evaluate_independent_br(&t,4.0,4.0,8.0,&s,v).unwrap();assert!(r.p0_best_response_value+1e-9>=v);assert!(r.p0_value_vs_p1_best_response-1e-9<=v);}
}
