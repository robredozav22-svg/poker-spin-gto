use std::collections::{HashMap,HashSet};
use std::thread;

use crate::equity::{canonical_hu_matchup,HuMatchupKey};
use crate::equity3::{canonical_threeway,ThreeWayKey};
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

fn validate_hu_key(key:HuMatchupKey)->Result<(),String>{
    let hero=[key.0[0],key.0[1]];
    let villain=[key.0[2],key.0[3]];
    if canonical_hu_matchup(hero,villain)?!=key{return Err(format!("non-canonical HU payoff key: {:?}",key.0));}
    Ok(())
}

fn validate_threeway_key(key:ThreeWayKey)->Result<(),String>{
    let a=[key.0[0],key.0[1]];
    let b=[key.0[2],key.0[3]];
    let c=[key.0[4],key.0[5]];
    if canonical_threeway(a,b,c)?!=key{return Err(format!("non-canonical 3-way payoff key: {:?}",key.0));}
    Ok(())
}

fn existing_hu_map(existing:Option<&HuPayoffTable>)->Result<HashMap<HuMatchupKey,HuPayoffRecord>,String>{
    let mut by_key=HashMap::new();
    if let Some(table)=existing{
        by_key.reserve(table.records.len());
        for r in &table.records{
            validate_hu_key(r.key)?;
            if by_key.insert(r.key,*r).is_some(){return Err(format!("duplicate HU key in existing table: {:?}",r.key.0));}
        }
    }
    Ok(by_key)
}

fn exact_hu_record(key:HuMatchupKey)->Result<HuPayoffRecord,String>{
    validate_hu_key(key)?;
    let hero=[key.0[0],key.0[1]];
    let villain=[key.0[2],key.0[3]];
    let e=exact_hu_equity(hero,villain)?;
    Ok(HuPayoffRecord{key,wins:e.wins,losses:e.losses,ties:e.ties})
}

pub fn build_hu_payoff_table(
    requested:&[HuMatchupKey],
    existing:Option<&HuPayoffTable>,
)->Result<(HuPayoffTable,BuildStats),String>{
    for key in requested{validate_hu_key(*key)?;}
    let requested_set:HashSet<HuMatchupKey>=requested.iter().copied().collect();
    let mut by_key=existing_hu_map(existing)?;
    let reused=requested_set.iter().filter(|k|by_key.contains_key(k)).count();

    for key in &requested_set{
        if by_key.contains_key(key){continue;}
        let record=exact_hu_record(*key)?;
        by_key.insert(*key,record);
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

/// Build-only parallel exact HU generator.
///
/// Only missing requested canonical keys are dispatched to workers. Existing
/// trusted records are reused unchanged. Worker completion order is discarded:
/// final output is canonical-key sorted, so the encoded artifact is byte-
/// identical to the serial builder for identical exact inputs.
pub fn build_hu_payoff_table_parallel(
    requested:&[HuMatchupKey],
    existing:Option<&HuPayoffTable>,
    workers:usize,
)->Result<(HuPayoffTable,BuildStats),String>{
    if workers==0{return Err("HU payoff builder workers must be positive".into());}
    for key in requested{validate_hu_key(*key)?;}
    let requested_set:HashSet<HuMatchupKey>=requested.iter().copied().collect();
    let mut by_key=existing_hu_map(existing)?;
    let reused=requested_set.iter().filter(|k|by_key.contains_key(k)).count();

    let mut missing:Vec<HuMatchupKey>=requested_set.iter().copied().filter(|k|!by_key.contains_key(k)).collect();
    missing.sort_by_key(|k|k.0);

    if !missing.is_empty(){
        let active_workers=workers.min(missing.len());
        let chunk_size=(missing.len()+active_workers-1)/active_workers;
        let mut handles=Vec::with_capacity(active_workers);
        for chunk in missing.chunks(chunk_size){
            let owned=chunk.to_vec();
            handles.push(thread::spawn(move||->Result<Vec<HuPayoffRecord>,String>{
                owned.into_iter().map(exact_hu_record).collect()
            }));
        }
        for handle in handles{
            let computed=handle.join().map_err(|_|"HU payoff build worker panicked".to_string())??;
            for record in computed{
                if by_key.insert(record.key,record).is_some(){return Err(format!("parallel HU build produced duplicate key: {:?}",record.key.0));}
            }
        }
    }

    let mut records:Vec<_>=by_key.into_values().collect();
    records.sort_by_key(|r|r.key.0);
    let stats=BuildStats{
        requested_unique:requested_set.len(),
        reused_existing:reused,
        computed_missing:missing.len(),
        output_records:records.len(),
    };
    Ok((HuPayoffTable{records},stats))
}

pub fn build_threeway_payoff_table(
    requested:&[ThreeWayKey],
    existing:Option<&ThreeWayPayoffTable>,
)->Result<(ThreeWayPayoffTable,BuildStats),String>{
    for key in requested{validate_threeway_key(*key)?;}
    let requested_set:HashSet<ThreeWayKey>=requested.iter().copied().collect();
    let mut by_key:HashMap<ThreeWayKey,ThreeWayPayoffRecord>=HashMap::new();
    let mut reused=0usize;

    if let Some(table)=existing{
        for r in &table.records{
            validate_threeway_key(r.key)?;
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
    use crate::cards::Card;
    use crate::exact_equity::HU_PREFLOP_BOARD_COUNT;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn hu_existing_records_are_reused_and_output_sorted(){
        let k1=canonical_hu_matchup([c(12,0),c(12,1)],[c(11,2),c(11,3)]).unwrap();
        let k2=canonical_hu_matchup([c(10,0),c(9,0)],[c(8,1),c(7,2)]).unwrap();
        let existing=HuPayoffTable{records:vec![
            HuPayoffRecord{key:k1,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0},
            HuPayoffRecord{key:k2,wins:0,losses:HU_PREFLOP_BOARD_COUNT,ties:0},
        ]};
        let (built,stats)=build_hu_payoff_table(&[k1,k2,k1],Some(&existing)).unwrap();
        assert_eq!(stats.requested_unique,2);
        assert_eq!(stats.reused_existing,2);
        assert_eq!(stats.computed_missing,0);
        assert!(built.records.windows(2).all(|w|w[0].key.0<w[1].key.0));
    }

    #[test]
    fn parallel_hu_all_reused_matches_serial_bytes(){
        let k1=canonical_hu_matchup([c(12,0),c(12,1)],[c(11,2),c(11,3)]).unwrap();
        let k2=canonical_hu_matchup([c(10,0),c(9,0)],[c(8,1),c(7,2)]).unwrap();
        let existing=HuPayoffTable{records:vec![
            HuPayoffRecord{key:k1,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0},
            HuPayoffRecord{key:k2,wins:0,losses:HU_PREFLOP_BOARD_COUNT,ties:0},
        ]};
        let (serial,_)=build_hu_payoff_table(&[k1,k2],Some(&existing)).unwrap();
        let (parallel,stats)=build_hu_payoff_table_parallel(&[k2,k1,k2],Some(&existing),4).unwrap();
        assert_eq!(stats.reused_existing,2);
        assert_eq!(stats.computed_missing,0);
        assert_eq!(serial.encode().unwrap(),parallel.encode().unwrap());
    }

    #[test]
    fn parallel_hu_zero_workers_fails_closed(){
        assert!(build_hu_payoff_table_parallel(&[],None,0).is_err());
    }

    #[test]
    fn duplicate_existing_hu_table_fails_closed(){
        let key=canonical_hu_matchup([c(12,0),c(12,1)],[c(11,2),c(11,3)]).unwrap();
        let r=HuPayoffRecord{key,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0};
        assert!(build_hu_payoff_table(&[],Some(&HuPayoffTable{records:vec![r,r]})).is_err());
        assert!(build_hu_payoff_table_parallel(&[],Some(&HuPayoffTable{records:vec![r,r]}),2).is_err());
    }

    #[test]
    fn noncanonical_requested_key_fails_closed(){
        let canonical=canonical_hu_matchup([c(12,0),c(12,1)],[c(11,2),c(11,3)]).unwrap();
        let mut bad=canonical.0;bad.swap(0,1);
        assert!(build_hu_payoff_table(&[HuMatchupKey(bad)],None).is_err());
        assert!(build_hu_payoff_table_parallel(&[HuMatchupKey(bad)],None,2).is_err());
    }
}
