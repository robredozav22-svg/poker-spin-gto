use std::collections::HashSet;
use std::thread;
use std::time::{Duration,Instant};

use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,COMBO_COUNT};
use spins_solver_core::equity::{canonical_hu_matchup,HuMatchupKey};
use spins_solver_core::exact_equity::exact_hu_equity;
use spins_solver_core::payoff_table::HuPayoffRecord;

const BATCH:usize=8;
const FULL_HU_KEYS:usize=93_769;

fn deterministic_keys()->Vec<HuMatchupKey>{
    let combos=all_combos();
    let blockers=BlockerMatrix::build();
    let mut seen=HashSet::new();
    let mut keys=Vec::with_capacity(BATCH);
    'outer: for i in 0..COMBO_COUNT{
        for j in 0..COMBO_COUNT{
            if !blockers.compatible(i,j){continue;}
            let key=canonical_hu_matchup(combos[i],combos[j]).unwrap();
            if seen.insert(key){
                keys.push(key);
                if keys.len()==BATCH{break 'outer;}
            }
        }
    }
    keys.sort_by_key(|k|k.0);
    assert_eq!(keys.len(),BATCH);
    keys
}

fn compute_record(key:HuMatchupKey)->HuPayoffRecord{
    let hero=[key.0[0],key.0[1]];
    let villain=[key.0[2],key.0[3]];
    let e=exact_hu_equity(hero,villain).expect("exact HU batch matchup");
    HuPayoffRecord{key,wins:e.wins,losses:e.losses,ties:e.ties}
}

fn run_serial(keys:&[HuMatchupKey])->(Vec<HuPayoffRecord>,Duration){
    let start=Instant::now();
    let mut out:Vec<_>=keys.iter().copied().map(compute_record).collect();
    let elapsed=start.elapsed();
    out.sort_by_key(|r|r.key.0);
    (out,elapsed)
}

fn run_parallel(keys:&[HuMatchupKey],workers:usize)->(Vec<HuPayoffRecord>,Duration){
    assert!(workers>0);
    let start=Instant::now();
    let chunk=(keys.len()+workers-1)/workers;
    let mut handles=Vec::new();
    for part in keys.chunks(chunk){
        let owned=part.to_vec();
        handles.push(thread::spawn(move||owned.into_iter().map(compute_record).collect::<Vec<_>>()));
    }
    let mut out=Vec::with_capacity(keys.len());
    for h in handles{out.extend(h.join().expect("HU worker panicked"));}
    let elapsed=start.elapsed();
    out.sort_by_key(|r|r.key.0);
    (out,elapsed)
}

fn report(label:&str,elapsed:Duration){
    let seconds=elapsed.as_secs_f64();
    let rate=BATCH as f64/seconds;
    let projected=FULL_HU_KEYS as f64/rate;
    println!(
        "mode={} batch_keys={} seconds={:.6} keys_per_second={:.6} projected_full_keys={} projected_full_seconds={:.3} projected_full_minutes={:.3}",
        label,BATCH,seconds,rate,FULL_HU_KEYS,projected,projected/60.0
    );
}

fn main(){
    let keys=deterministic_keys();
    println!("status=RESEARCH_ONLY benchmark=HU_EXACT_GENERATION_ECONOMICS canonical_batch_keys={} full_hu_canonical_keys={} projection_policy=ESTIMATE_FROM_THIS_BATCH_ONLY",BATCH,FULL_HU_KEYS);

    let (serial,t1)=run_serial(&keys);
    let (two,t2)=run_parallel(&keys,2);
    let (four,t4)=run_parallel(&keys,4);

    assert_eq!(serial,two,"2-worker exact integer outcomes differ from serial");
    assert_eq!(serial,four,"4-worker exact integer outcomes differ from serial");
    assert!(serial.windows(2).all(|w|w[0].key.0<w[1].key.0));

    report("SERIAL_1",t1);
    report("PARALLEL_2",t2);
    report("PARALLEL_4",t4);

    let speedup2=t1.as_secs_f64()/t2.as_secs_f64();
    let speedup4=t1.as_secs_f64()/t4.as_secs_f64();
    println!("speedup_2_vs_serial={:.6} speedup_4_vs_serial={:.6} exact_integer_equivalence=PASS sorted_output=PASS",speedup2,speedup4);
    println!("NOTE: projected full-generation times are estimates from this 8-key CI batch, not measured 93,769-key build runtimes. Runner CPU contention can materially change parallel speedup.");
}
