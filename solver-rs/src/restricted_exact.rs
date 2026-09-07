use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::exact_equity::ExactEquityCache;
use crate::range::ComboRange;
use crate::regret::RegretTable;
use crate::StrategySnapshot;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactPairPayoff{
    pub sb_combo:usize,
    pub bb_combo:usize,
    pub raw_joint_weight:f64,
    pub sb_showdown_ev:f64,
    pub bb_showdown_ev:f64,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactRestrictedReport{
    pub sb_value:f64,
    pub sb_best_response_value:f64,
    pub sb_value_vs_bb_best_response:f64,
    pub sb_br_gain:f64,
    pub bb_br_gain:f64,
    pub nashconv:f64,
    pub legal_pairs:usize,
    pub joint_normalizer:f64,
}

pub struct ExactRestrictedSubgame{
    pub stack_bb:f64,
    pub sb_prior:ComboRange,
    pub bb_prior:ComboRange,
    pub sb_regrets:RegretTable, // [Fold,Jam]
    pub bb_regrets:RegretTable, // [Fold,Call]
    pub blockers:BlockerMatrix,
    pub equity_cache:ExactEquityCache,
    pub pairs:Vec<ExactPairPayoff>,
    pub joint_normalizer:f64,
}

impl ExactRestrictedSubgame{
    pub fn new(stack_bb:f64,sb_prior:ComboRange,bb_prior:ComboRange)->Result<Self,String>{
        if stack_bb<1.0{return Err("stack must be at least 1bb".into());}
        let blockers=BlockerMatrix::build();
        let combos=all_combos();
        let mut equity_cache=ExactEquityCache::new();
        let sb_support:Vec<usize>=sb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();
        let bb_support:Vec<usize>=bb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();
        let mut pairs=Vec::new();let mut z=0.0;
        for &i in &sb_support{
            for &j in &bb_support{
                if !blockers.compatible(i,j){continue;}
                let raw=sb_prior.weights()[i]*bb_prior.weights()[j];
                if raw<=0.0{continue;}
                let eq=equity_cache.get_or_compute(combos[i],combos[j])?;
                let sb_showdown=eq.hero*(2.0*stack_bb)-stack_bb;
                let bb_showdown=eq.villain*(2.0*stack_bb)-stack_bb;
                debug_assert!((sb_showdown+bb_showdown).abs()<1e-9);
                pairs.push(ExactPairPayoff{sb_combo:i,bb_combo:j,raw_joint_weight:raw,sb_showdown_ev:sb_showdown,bb_showdown_ev:bb_showdown});
                z+=raw;
            }
        }
        if z<=f64::EPSILON{return Err("no legal private-hand pairs".into());}
        Ok(Self{stack_bb,sb_prior,bb_prior,sb_regrets:RegretTable::new(COMBO_COUNT,2),bb_regrets:RegretTable::new(COMBO_COUNT,2),blockers,equity_cache,pairs,joint_normalizer:z})
    }

    pub fn sb_current_strategy(&self)->StrategySnapshot{self.sb_regrets.current_strategy_snapshot()}
    pub fn bb_current_strategy(&self)->StrategySnapshot{self.bb_regrets.current_strategy_snapshot()}
    pub fn sb_average_strategy(&self)->StrategySnapshot{self.sb_regrets.average_strategy()}
    pub fn bb_average_strategy(&self)->StrategySnapshot{self.bb_regrets.average_strategy()}

    pub fn sweep(&mut self)->Result<(),String>{
        let sb=self.sb_current_strategy();
        let bb=self.bb_current_strategy();

        let sb_support:Vec<usize>=self.sb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();
        let bb_support:Vec<usize>=self.bb_prior.weights().iter().enumerate().filter_map(|(i,w)|if *w>0.0{Some(i)}else{None}).collect();

        let mut sb_updates=Vec::new();
        for i in sb_support{
            let relevant:Vec<&ExactPairPayoff>=self.pairs.iter().filter(|p|p.sb_combo==i).collect();
            let denom: f64=relevant.iter().map(|p|p.raw_joint_weight).sum();
            if denom<=f64::EPSILON{continue;}
            let mut jam_ev=0.0;
            for p in relevant{
                let q=p.raw_joint_weight/denom;
                let call=bb.row(p.bb_combo)[1];
                jam_ev+=q*((1.0-call)*1.0+call*p.sb_showdown_ev);
            }
            sb_updates.push((i,[-0.5,jam_ev]));
        }

        let mut bb_updates=Vec::new();
        for j in bb_support{
            let relevant:Vec<&ExactPairPayoff>=self.pairs.iter().filter(|p|p.bb_combo==j).collect();
            let denom:f64=relevant.iter().map(|p|p.raw_joint_weight).sum();
            if denom<=f64::EPSILON{continue;}
            let mut jam_reach=0.0;let mut call_mass=0.0;
            for p in relevant{
                let q=p.raw_joint_weight/denom;
                let jam=sb.row(p.sb_combo)[1];
                jam_reach+=q*jam;
                call_mass+=q*jam*p.bb_showdown_ev;
            }
            if jam_reach<=f64::EPSILON{continue;}
            let call_ev=call_mass/jam_reach;
            bb_updates.push((j,[-1.0,call_ev],jam_reach));
        }

        for (i,v) in sb_updates{self.sb_regrets.update_from_action_values(i,&v,1.0,1.0);}
        for (j,v,reach) in bb_updates{self.bb_regrets.update_from_action_values(j,&v,reach,1.0);}
        Ok(())
    }

    pub fn evaluate(&self,sb:&StrategySnapshot,bb:&StrategySnapshot)->Result<ExactRestrictedReport,String>{
        if sb.infosets!=COMBO_COUNT||bb.infosets!=COMBO_COUNT||sb.actions!=2||bb.actions!=2{return Err("invalid restricted strategy shape".into());}
        let z=self.joint_normalizer;
        let mut current=0.0;
        let mut sb_marginal=vec![0.0;COMBO_COUNT];
        let mut sb_jam_mass=vec![0.0;COMBO_COUNT];
        let mut fixed_fold_mass=vec![0.0;COMBO_COUNT];
        let mut bb_fold_mass=vec![0.0;COMBO_COUNT];
        let mut bb_call_mass=vec![0.0;COMBO_COUNT];

        for p in &self.pairs{
            let q=p.raw_joint_weight/z;
            let jam=sb.row(p.sb_combo)[1];
            let call=bb.row(p.bb_combo)[1];
            let jam_value=(1.0-call)*1.0+call*p.sb_showdown_ev;
            current+=q*((1.0-jam)*-0.5+jam*jam_value);
            sb_marginal[p.sb_combo]+=q;
            sb_jam_mass[p.sb_combo]+=q*jam_value;
            fixed_fold_mass[p.bb_combo]+=q*(1.0-jam)*-0.5;
            bb_fold_mass[p.bb_combo]+=q*jam*1.0;
            bb_call_mass[p.bb_combo]+=q*jam*p.sb_showdown_ev;
        }

        let mut sb_br=0.0;
        for i in 0..COMBO_COUNT{
            if sb_marginal[i]<=f64::EPSILON{continue;}
            sb_br+=(sb_marginal[i]*-0.5).max(sb_jam_mass[i]);
        }
        let mut sb_vs_bb_br=0.0;
        for j in 0..COMBO_COUNT{
            sb_vs_bb_br+=fixed_fold_mass[j]+bb_fold_mass[j].min(bb_call_mass[j]);
        }
        let sb_gain=(sb_br-current).max(0.0);
        let bb_gain=(current-sb_vs_bb_br).max(0.0);
        Ok(ExactRestrictedReport{sb_value:current,sb_best_response_value:sb_br,sb_value_vs_bb_best_response:sb_vs_bb_br,sb_br_gain:sb_gain,bb_br_gain:bb_gain,nashconv:sb_gain+bb_gain,legal_pairs:self.pairs.len(),joint_normalizer:z})
    }
}
