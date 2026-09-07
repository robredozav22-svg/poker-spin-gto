use std::time::Instant;
use std::thread;

use spins_solver_core::cards::Card;
use spins_solver_core::equity::{canonical_hu_matchup,HuMatchupKey};
use spins_solver_core::exact_equity::exact_hu_equity;
use spins_solver_core::payoff_table::HuPayoffRecord;

const FULL_HU_CANONICAL_KEYS:usize=93_769;

fn c(r:u8,s:u8)->Card{r*4+s}

fn fixtures()->Vec<HuMatchupKey>{
    let raw=[
        ([c(12,0),c(12,1)],[c(11,2),c(11,3)]), // AA vs KK
        ([c(12,0),c(11,0)],[c(10,1),c(10,2)]), // AKs vs QQ
        ([c(12,0),c(11,1)],[c(9,2),c(9,3)]),   // AKo vs JJ
        ([c(8,0),c(8,1)],[c(12,2),c(7,2)]),    // TT vs A9s
        ([c(6,0),c(6,1)],[c(5,2),c(4,2)]),     // 88 vs 76s
        ([c(12,0),c(3,0)],[c(11,1),c(10,1)]),  // A5s vs KQs
        ([c(7,0),c(6,1)],[c(5,2),c(4,3)]),     // 98o vs 76o
        ([c(2,0),c(2,1)],[c(12,2),c(1,2)]),    // 44 vs A3s
    ];
    let mut keys:Vec<_>=raw.into_iter().map(|(a,b)|canonical_hu_matchup(a,b).unwrap()).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(),8,"benchmark fixtures must produce 8 unique canonical HU keys");
    keys
}

fn compute_one(key:HuMatchupKey)->HuPayoffRecord{
    let hero=[key.0[0],key.0[1]];
    let villain=[key.0[2],key.0[3]];
    let e=exact_hu_equity(hero,villain).expect("exact HU batch equity");
    HuPayoffRecord{key,wins:e.wins,losses:e.losses,ties:e.ties}
}

fn run(keys:&[HuMatchupKey],workers:usize)->(Vec<HuPayoffRecord>,f64){
    assert!(workers>=1);
    let started=Instant::now();
    let mut records=Vec::with_capacity(keys.len());
    if workers==1{
        records.extend(keys.iter().copied().map(compute_one));
    }else{
        let chunk=(keys.len()+workers-1)/workers;
        thread::scope(|scope|{
            let mut handles=Vec::new();
            for part in keys.chunks(chunk){
                handles.push(scope.spawn(move || part.iter().copied().map(compute_one).collect::<Vec<_>>()));
            }
            for h in handles{records.extend(h.join().expect("HU benchmark worker panicked"));}
        });
    }
    records.sort_by_key(|r|r.key.0);
    (records,started.elapsed().as_secs_f64())
}

fn main(){
    let keys=fixtures();
    let (serial,t1)=run(&keys,1);
    let (two,t2)=run(&keys,2);
    let (four,t4)=run(&keys,4);
    assert_eq!(serial,two,"2-worker exact integer outcomes differ from serial baseline");
    assert_eq!(serial,four,"4-worker exact integer outcomes differ from serial baseline");

    for (workers,seconds) in [(1usize,t1),(2,t2),(4,t4)]{
        let kps=keys.len() as f64/seconds;
        let projected_seconds=FULL_HU_CANONICAL_KEYS as f64/kps;
        println!(
            "status=RESEARCH_ONLY mode=HU_BATCH_ECONOMICS workers={} batch_keys={} wall_seconds={:.6} keys_per_second={:.6} projected_full_keys={} projected_full_seconds={:.3} projection=ESTIMATE_ONLY",
            workers,keys.len(),seconds,kps,FULL_HU_CANONICAL_KEYS,projected_seconds
        );
    }
    println!("status=RESEARCH_ONLY exact_integer_crosscheck=PASS deterministic_sort=PASS note=parallel_speedup_depends_on_runner_cpu_allocation");
}
