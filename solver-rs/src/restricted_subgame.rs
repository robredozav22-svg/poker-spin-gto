use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::equity_cache::EquityCache;
use crate::leaf_ev::{sampled_bb_leaf_action_values,BbHuLeaf};
use crate::range::ComboRange;
use crate::regret::RegretTable;
use crate::sb_ev::sampled_sb_fold_jam_values;
use crate::strategy_range::condition_range_on_action;
use crate::StrategySnapshot;

pub const FOLD_ACTION:usize=0;
pub const AGGRESSIVE_ACTION:usize=1; // SB Jam or BB Call depending on table.

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum RestrictedSubgameStatus{
    ResearchOnly,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct SweepDiagnostics{
    pub sb_updated_combos:usize,
    pub bb_updated_combos:usize,
    pub sb_jam_probability:f64,
    pub hu_cache_hits:u64,
    pub hu_cache_misses:u64,
}

pub struct SbJamBbResponseSubgame{
    pub status:RestrictedSubgameStatus,
    pub stack_bb:f64,
    pub sb_prior:ComboRange,
    pub bb_prior:ComboRange,
    pub sb_regrets:RegretTable, // [Fold, Jam]
    pub bb_regrets:RegretTable, // [Fold, Call]
    pub blockers:BlockerMatrix,
    pub equity_cache:EquityCache,
}

impl SbJamBbResponseSubgame{
    pub fn new(stack_bb:f64,sb_prior:ComboRange,bb_prior:ComboRange)->Result<Self,String>{
        if stack_bb<1.0{return Err("stack must be at least 1bb".into());}
        Ok(Self{
            status:RestrictedSubgameStatus::ResearchOnly,
            stack_bb,
            sb_prior,
            bb_prior,
            sb_regrets:RegretTable::new(COMBO_COUNT,2),
            bb_regrets:RegretTable::new(COMBO_COUNT,2),
            blockers:BlockerMatrix::build(),
            equity_cache:EquityCache::new(),
        })
    }

    pub fn sb_current_strategy(&self)->StrategySnapshot{self.sb_regrets.current_strategy_snapshot()}
    pub fn bb_current_strategy(&self)->StrategySnapshot{self.bb_regrets.current_strategy_snapshot()}
    pub fn sb_average_strategy(&self)->StrategySnapshot{self.sb_regrets.average_strategy()}
    pub fn bb_average_strategy(&self)->StrategySnapshot{self.bb_regrets.average_strategy()}

    /// One simultaneous-style sweep from snapshots frozen at sweep start.
    /// Payoff integration already conditions opponent ranges on each hero's
    /// private cards, so regret scaling is 1.0 for each reachable private type.
    pub fn sweep(&mut self,samples_per_matchup:u64,seed:u64)->Result<SweepDiagnostics,String>{
        if samples_per_matchup==0{return Err("samples_per_matchup must be positive".into());}
        let sb_snapshot=self.sb_current_strategy();
        let bb_snapshot=self.bb_current_strategy();

        let sb_jam=condition_range_on_action(&self.sb_prior,&sb_snapshot,AGGRESSIVE_ACTION).ok();
        let sb_jam_probability=sb_jam.as_ref().map(|x|x.action_probability).unwrap_or(0.0);

        let mut bb_updates:Vec<(usize,[f64;2])>=Vec::new();
        if let Some(jam)=sb_jam.as_ref(){
            for bb_combo in positive_support(&self.bb_prior){
                match sampled_bb_leaf_action_values(
                    BbHuLeaf::AfterBtnFoldSbJam,
                    self.stack_bb,
                    bb_combo,
                    &jam.range,
                    &self.blockers,
                    &mut self.equity_cache,
                    samples_per_matchup,
                    seed,
                ){
                    Ok(v)=>bb_updates.push((bb_combo,v.action_values())),
                    Err(e) if e.contains("compatible")=>{}, // private state unreachable after blockers
                    Err(e)=>return Err(e),
                }
            }
        }

        let mut sb_updates:Vec<(usize,[f64;2])>=Vec::new();
        for sb_combo in positive_support(&self.sb_prior){
            match sampled_sb_fold_jam_values(
                self.stack_bb,
                sb_combo,
                &self.bb_prior,
                &bb_snapshot,
                &self.blockers,
                &mut self.equity_cache,
                samples_per_matchup,
                seed,
            ){
                Ok(v)=>sb_updates.push((sb_combo,v.action_values())),
                Err(e) if e.contains("compatible")=>{},
                Err(e)=>return Err(e),
            }
        }

        for (i,values) in &bb_updates{self.bb_regrets.update_from_action_values(*i,values,1.0,1.0);}
        for (i,values) in &sb_updates{self.sb_regrets.update_from_action_values(*i,values,1.0,1.0);}

        Ok(SweepDiagnostics{
            sb_updated_combos:sb_updates.len(),
            bb_updated_combos:bb_updates.len(),
            sb_jam_probability,
            hu_cache_hits:self.equity_cache.hits(),
            hu_cache_misses:self.equity_cache.misses(),
        })
    }
}

fn positive_support(range:&ComboRange)->impl Iterator<Item=usize>+'_ {
    range.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None})
}

#[cfg(test)]
mod tests{
    use super::*;

    fn one_combo_range(index:usize)->ComboRange{
        let mut w=vec![0.0;COMBO_COUNT];w[index]=1.0;ComboRange::from_weights(w).unwrap()
    }

    #[test]
    fn one_sparse_coupled_sweep_updates_both_players(){
        let blockers=BlockerMatrix::build();
        let sb=0usize;
        let bb=(0..COMBO_COUNT).find(|i|blockers.compatible(sb,*i)).unwrap();
        let mut game=SbJamBbResponseSubgame::new(8.0,one_combo_range(sb),one_combo_range(bb)).unwrap();
        let d=game.sweep(40,20260907).unwrap();
        assert_eq!(game.status,RestrictedSubgameStatus::ResearchOnly);
        assert_eq!(d.sb_updated_combos,1);
        assert_eq!(d.bb_updated_combos,1);
        assert!((d.sb_jam_probability-0.5).abs()<1e-12); // zero regrets => uniform Fold/Jam
        let ss=game.sb_current_strategy();let bs=game.bb_current_strategy();
        assert!((ss.row(sb).iter().sum::<f64>()-1.0).abs()<1e-12);
        assert!((bs.row(bb).iter().sum::<f64>()-1.0).abs()<1e-12);
        assert!(d.hu_cache_misses>0);
    }

    #[test]
    fn research_status_is_not_a_promotion_status(){
        let game=SbJamBbResponseSubgame::new(8.0,ComboRange::uniform(),ComboRange::uniform()).unwrap();
        assert_eq!(game.status,RestrictedSubgameStatus::ResearchOnly);
    }
}
