use std::collections::BTreeMap;

use crate::cards::{all_combos,rank,suit,Combo,COMBO_COUNT};
use crate::StrategySnapshot;

#[derive(Debug,Clone,PartialEq)]
pub struct ClassStrategy{
    pub hand:String,
    pub combo_count:usize,
    pub actions:Vec<f64>,
    pub max_variant_deviation:f64,
}

fn rank_char(r:u8)->char{
    const RANKS:[char;13]=['2','3','4','5','6','7','8','9','T','J','Q','K','A'];
    RANKS[r as usize]
}

pub fn class_label(combo:Combo)->String{
    let r0=rank(combo[0]);let r1=rank(combo[1]);
    if r0==r1{return format!("{}{}",rank_char(r0),rank_char(r1));}
    let hi=r0.max(r1);let lo=r0.min(r1);
    let suffix=if suit(combo[0])==suit(combo[1]){'s'}else{'o'};
    format!("{}{}{}",rank_char(hi),rank_char(lo),suffix)
}

pub fn aggregate_to_169(strategy:&StrategySnapshot)->Result<BTreeMap<String,ClassStrategy>,String>{
    if strategy.infosets!=COMBO_COUNT{return Err(format!("expected {COMBO_COUNT} combo infosets"));}
    if strategy.actions==0{return Err("strategy has zero actions".into());}
    if strategy.probabilities.len()!=strategy.infosets*strategy.actions{return Err("strategy shape mismatch".into());}

    let combos=all_combos();
    let mut rows:BTreeMap<String,Vec<Vec<f64>>>=BTreeMap::new();
    for (i,combo) in combos.iter().copied().enumerate(){
        let row=strategy.row(i);
        if row.iter().any(|p|!p.is_finite()||*p<0.0||*p>1.0){return Err(format!("invalid probability at combo {i}"));}
        if (row.iter().sum::<f64>()-1.0).abs()>1e-9{return Err(format!("action probabilities at combo {i} do not sum to 1"));}
        rows.entry(class_label(combo)).or_default().push(row.to_vec());
    }
    if rows.len()!=169{return Err(format!("expected 169 hand classes, got {}",rows.len()));}

    let mut out=BTreeMap::new();
    for (hand,variants) in rows{
        let expected=if hand.len()==2{6}else if hand.ends_with('s'){4}else{12};
        if variants.len()!=expected{return Err(format!("{hand} has {} variants, expected {expected}",variants.len()));}
        let mut mean=vec![0.0;strategy.actions];
        for row in &variants{for a in 0..strategy.actions{mean[a]+=row[a];}}
        for value in &mut mean{*value/=variants.len() as f64;}
        let mut max_dev=0.0f64;
        for row in &variants{for a in 0..strategy.actions{max_dev=max_dev.max((row[a]-mean[a]).abs());}}
        out.insert(hand.clone(),ClassStrategy{hand,combo_count:variants.len(),actions:mean,max_variant_deviation:max_dev});
    }
    Ok(out)
}

pub fn maximum_suit_dispersion(classes:&BTreeMap<String,ClassStrategy>)->f64{
    classes.values().map(|c|c.max_variant_deviation).fold(0.0f64,f64::max)
}

#[cfg(test)]
mod tests{
    use super::*;

    fn constant_snapshot(fold:f64)->StrategySnapshot{
        let mut p=Vec::with_capacity(COMBO_COUNT*2);
        for _ in 0..COMBO_COUNT{p.push(fold);p.push(1.0-fold);}
        StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities:p}
    }

    #[test]
    fn exact_169_counts_are_preserved(){
        let classes=aggregate_to_169(&constant_snapshot(0.25)).unwrap();
        assert_eq!(classes.len(),169);
        assert_eq!(classes["AA"].combo_count,6);
        assert_eq!(classes["AKs"].combo_count,4);
        assert_eq!(classes["AKo"].combo_count,12);
        assert_eq!(classes["32s"].combo_count,4);
        assert_eq!(maximum_suit_dispersion(&classes),0.0);
        assert_eq!(classes["AKs"].actions,vec![0.25,0.75]);
    }

    #[test]
    fn one_suit_variant_outlier_is_visible_not_hidden_by_average(){
        let combos=all_combos();
        let mut s=constant_snapshot(0.5);
        let aa=combos.iter().position(|c|class_label(*c)=="AA").unwrap();
        s.probabilities[aa*2]=1.0;s.probabilities[aa*2+1]=0.0;
        let classes=aggregate_to_169(&s).unwrap();
        assert!(classes["AA"].max_variant_deviation>0.4);
        assert!(maximum_suit_dispersion(&classes)>0.4);
    }
}
