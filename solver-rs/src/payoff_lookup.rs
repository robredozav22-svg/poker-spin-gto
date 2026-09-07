use std::collections::HashMap;

use crate::cards::Combo;
use crate::equity::{canonical_hu_matchup,HuMatchupKey};
use crate::equity3::{canonical_threeway,ThreeWayKey};
use crate::exact_equity::ExactEquity;
use crate::exact_equity3::ExactThreeWayEquity;
use crate::payoff_table::{HuPayoffRecord,HuPayoffTable,ThreeWayPayoffRecord,ThreeWayPayoffTable};

#[derive(Debug,Clone)]
pub struct HuPayoffLookup{
    records:HashMap<HuMatchupKey,HuPayoffRecord>,
}

impl HuPayoffLookup{
    pub fn from_table(table:HuPayoffTable)->Result<Self,String>{
        let mut records=HashMap::with_capacity(table.records.len());
        for record in table.records{
            if records.insert(record.key,record).is_some(){
                return Err(format!("duplicate HU canonical payoff key: {:?}",record.key.0));
            }
        }
        Ok(Self{records})
    }
    pub fn len(&self)->usize{self.records.len()}
    pub fn is_empty(&self)->bool{self.records.is_empty()}
    pub fn get_record_by_key(&self,key:HuMatchupKey)->Option<HuPayoffRecord>{self.records.get(&key).copied()}
    pub fn get_equity_by_key(&self,key:HuMatchupKey)->Option<ExactEquity>{self.get_record_by_key(key).map(|r|r.equity())}
    pub fn get_record(&self,hero:Combo,villain:Combo)->Result<Option<HuPayoffRecord>,String>{
        Ok(self.get_record_by_key(canonical_hu_matchup(hero,villain)?))
    }
    pub fn get_equity(&self,hero:Combo,villain:Combo)->Result<Option<ExactEquity>,String>{
        Ok(self.get_record(hero,villain)?.map(|r|r.equity()))
    }
}

#[derive(Debug,Clone)]
pub struct ThreeWayPayoffLookup{
    records:HashMap<ThreeWayKey,ThreeWayPayoffRecord>,
}

impl ThreeWayPayoffLookup{
    pub fn from_table(table:ThreeWayPayoffTable)->Result<Self,String>{
        let mut records=HashMap::with_capacity(table.records.len());
        for record in table.records{
            if records.insert(record.key,record).is_some(){
                return Err(format!("duplicate three-way canonical payoff key: {:?}",record.key.0));
            }
        }
        Ok(Self{records})
    }
    pub fn len(&self)->usize{self.records.len()}
    pub fn is_empty(&self)->bool{self.records.is_empty()}
    pub fn get_record_by_key(&self,key:ThreeWayKey)->Option<ThreeWayPayoffRecord>{self.records.get(&key).copied()}
    pub fn get_equity_by_key(&self,key:ThreeWayKey)->Option<ExactThreeWayEquity>{self.get_record_by_key(key).map(|r|r.equity())}
    pub fn get_record(&self,a:Combo,b:Combo,c:Combo)->Result<Option<ThreeWayPayoffRecord>,String>{
        Ok(self.get_record_by_key(canonical_threeway(a,b,c)?))
    }
    pub fn get_equity(&self,a:Combo,b:Combo,c:Combo)->Result<Option<ExactThreeWayEquity>,String>{
        Ok(self.get_record(a,b,c)?.map(|r|r.equity()))
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::exact_equity::{HU_PREFLOP_BOARD_COUNT};
    use crate::exact_equity3::THREEWAY_PREFLOP_BOARD_COUNT;

    #[test]
    fn hu_duplicate_key_fails_closed(){
        let key=HuMatchupKey([1,2,3,4]);
        let a=HuPayoffRecord{key,wins:HU_PREFLOP_BOARD_COUNT,losses:0,ties:0};
        let b=HuPayoffRecord{key,wins:0,losses:HU_PREFLOP_BOARD_COUNT,ties:0};
        assert!(HuPayoffLookup::from_table(HuPayoffTable{records:vec![a,b]}).is_err());
    }

    #[test]
    fn threeway_duplicate_key_fails_closed(){
        let key=ThreeWayKey([1,2,3,4,5,6]);
        let a=ThreeWayPayoffRecord{key,outright_wins:[THREEWAY_PREFLOP_BOARD_COUNT,0,0],two_way_ties:[0,0,0],three_way_ties:0};
        assert!(ThreeWayPayoffLookup::from_table(ThreeWayPayoffTable{records:vec![a,a]}).is_err());
    }

    #[test]
    fn missing_key_is_explicit_none(){
        let hu=HuPayoffLookup::from_table(HuPayoffTable{records:vec![]}).unwrap();
        assert!(hu.get_record_by_key(HuMatchupKey([1,2,3,4])).is_none());
        let three=ThreeWayPayoffLookup::from_table(ThreeWayPayoffTable{records:vec![]}).unwrap();
        assert!(three.get_record_by_key(ThreeWayKey([1,2,3,4,5,6])).is_none());
    }
}
