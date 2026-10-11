use crate::cards::COMBO_COUNT;
use crate::leaf_ev::{LeafActionValues,ThreeWayLeafActionValues};
use crate::regret::RegretTable;

pub fn new_two_action_combo_table()->RegretTable{
    RegretTable::new(COMBO_COUNT,2)
}

fn validate_table(table:&RegretTable)->Result<(),String>{
    if table.infosets()!=COMBO_COUNT{return Err(format!("expected {COMBO_COUNT} infosets"));}
    if table.actions()!=2{return Err("leaf CFR table must have exactly 2 actions".into());}
    Ok(())
}

pub fn update_hu_leaf_combo(
    table:&mut RegretTable,
    combo_index:usize,
    values:LeafActionValues,
    opponent_reach:f64,
    own_reach:f64,
)->Result<f64,String>{
    validate_table(table)?;
    if combo_index>=COMBO_COUNT{return Err("combo index out of range".into());}
    Ok(table.update_from_action_values(combo_index,&values.action_values(),opponent_reach,own_reach))
}

pub fn update_threeway_leaf_combo(
    table:&mut RegretTable,
    combo_index:usize,
    values:ThreeWayLeafActionValues,
    opponent_reach:f64,
    own_reach:f64,
)->Result<f64,String>{
    validate_table(table)?;
    if combo_index>=COMBO_COUNT{return Err("combo index out of range".into());}
    Ok(table.update_from_action_values(combo_index,&values.action_values(),opponent_reach,own_reach))
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::range_equity::RangeEquityEstimate;
    use crate::range_equity3::ThreeWayRangeEquityEstimate;

    #[test]
    fn hu_leaf_update_moves_strategy_toward_call_when_call_ev_is_better(){
        let mut table=new_two_action_combo_table();
        let eq=RangeEquityEstimate{hero_equity:0.5,compatible_weight:1.0,compatible_combos:1,samples_per_matchup:1,seed:1};
        let values=LeafActionValues{fold_ev_bb:-1.0,call_ev_bb:0.5,call_equity:eq};
        let node=update_hu_leaf_combo(&mut table,10,values,1.0,1.0).unwrap();
        assert!((node+0.25).abs()<1e-12);
        assert_eq!(table.current_strategy(10),vec![0.0,1.0]);
    }

    #[test]
    fn threeway_leaf_uses_same_two_action_regret_contract(){
        let mut table=new_two_action_combo_table();
        let eq=ThreeWayRangeEquityEstimate{equities:[0.4,0.3,0.3],compatible_pair_weight:1.0,compatible_pairs:1,samples_per_matchup:1,seed:1};
        let values=ThreeWayLeafActionValues{fold_ev_bb:-1.0,call_ev_bb:-1.5,call_equity:eq};
        update_threeway_leaf_combo(&mut table,20,values,1.0,1.0).unwrap();
        assert_eq!(table.current_strategy(20),vec![1.0,0.0]);
    }
}
