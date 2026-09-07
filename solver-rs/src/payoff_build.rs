use std::collections::{HashMap,HashSet};

use crate::equity::HuMatchupKey;
use crate::equity3::ThreeWayKey;
use crate::exact_equity::exact_hu_equity;
use crate::exact_equity3::exact_threeway_equity;
use crate::payoff_table::{HuPayoffRecord,HuPayoffTable,ThreeWayPayoffRecord,ThreeWayPayoffTable};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct BuildStats{
    pub requested_unique:usize,
    pub reused_existing:usize,
    pub computed_missing:usize,
    pub output_records:usize,
}

pub fn build_hu_payoff_table(
    requested:&[HuMatchupKey],
    existing:Option<&HuPayoffTable>,
)->Result<(HuPayoffTable,BuildStats),String>{
    let requested_set:HashSet<HuMatchupKey>=requested.iter().copied().collect();
    let mut by_key:HashMap<HuMatchupKey,HuPayoffRecord>=HashMap::new();
    let mut reused=0usize;

    if let Some(table)=existing{
        for r in &table.records{
            if by_key.insert(r.key,*r).is_some(){return Err(format!("duplicate HU key in existing table: {:?}",r.key.0));}
        }
    }

    for key in &requested_set{
        if by_key.contains_key(key){reused+=1;continue;}
        let hero=[key.0[0],key.0[1]];
        let villain=[key.0[2],key.0[3]];
        let e=exact_hu_equity(hero,villain)?;
        by_key.insert(*key,HuPayoffRecord{key:*key,wins:e.wins,losses:e.losses,ties:e.ties});
    }

    let mut records:Vec<_>=by_key.into_values().collect();
    records.sort_by_key(|r|r.key.0);
    let stats=BuildStats{
        requested_unique:requested_set.len(),
        reused_existing:reused,
        computed_missing:requested_set.len()-reused,
        output_records:records.len(),
    };
    Ok((HuPayoffTable{records},stats))
}

pub fn build_threeway_payoff_table(
    requested:&[ThreeWayKey],
    existing:Option<&ThreeWayPayoffTable>,
)->Result<(ThreeWayPayoffTable,BuildStats),String>{
    let requested_set:HashSet<ThreeWayKey>=requested.iter().copied().collect();
    let mut by_key:HashMap<ThreeWayKey,ThreeWayPayoffRecord>=HashMap::new();
    let mut reused=0usize;

    if let Some(table)=existing{
        for r in &table.records{
            if by_key.insert(r.key,*r).is_some(){return Err(format!("duplicate 3-way key in existing table: {:?}",r.key.0));}
        }
    }

    for key in &requested_set{
        if by_key.contains_key(key){reused+=1;continue;}
        let a=[key.0[0],key.0[1]];
        let b=[key.0[2],key.0[3]];
        let c=[key.0[4],key.0[5]];
        let e=exact_threeway_equity(a,b,c)?;
        by_key.insert(*key,ThreeWayPayoffRecord{
            key:*key,
            outright_wins:e.outright_wins,
            two_way_ties:e.two_way_ties,
            three_way_ties:e.three_way_ties,
        });
    }

    let mut records:Vec<_>=by_key.into_values().collect();
    records.sort_by_key(|r|r.key.0);
    let stats=BuildStats{
        requested_unique:requested_set.len(),
        reused_existing:reused,
        computed_missing:requested_set.len()-reused,
        output_records:records.len(),
    };
    Ok((ThreeWayPayoffTable{records},stats))
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::exact_equity::HU_PREFLOP_BOARD_COUNT;

    #[test]
    fn hu_existing_records_are_reused_and_output_sorted(){
        let k1=HuMatchupKey([1,2,3,4]);
        let k2=HuMatchupKey([0,5,6,7]);
        let existing=HuPayoffTable{records:vec![
            HuPayoffRecord{key:k1,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0},
            HuPayoffRecord{key:k2,wins:0,losses:HU_PREFLOP_BOARD_COUNT,ties:0},
        ]};
        let (built,stats)=build_hu_payoff_table(&[k1,k2,k1],Some(&existing)).unwrap();
        assert_eq!(stats.requested_unique,2);
        assert_eq!(stats.reused_existing,2);
        assert_eq!(stats.computed_missing,0);
        assert_eq!(built.records[0].key,k2);
        assert_eq!(built.records[1].key,k1);
    }

    #[test]
    fn duplicate_existing_hu_table_fails_closed(){
        let key=HuMatchupKey([1,2,3,4]);
        let r=HuPayoffRecord{key,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0};
        assert!(build_hu_payoff_table(&[],Some(&HuPayoffTable{records:vec![r,r]})).is_err());
    }
}
