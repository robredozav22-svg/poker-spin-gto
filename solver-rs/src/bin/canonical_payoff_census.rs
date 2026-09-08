use std::collections::HashSet;
use std::time::Instant;

use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,COMBO_COUNT};
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::equity3::canonical_threeway;

fn main(){
    let combos=all_combos();
    let blockers=BlockerMatrix::build();

    let start=Instant::now();
    let mut raw_hu=0u64;
    let mut hu_keys=HashSet::new();
    for i in 0..COMBO_COUNT{
        for j in 0..COMBO_COUNT{
            if !blockers.compatible(i,j){continue;}
            raw_hu+=1;
            hu_keys.insert(canonical_hu_matchup(combos[i],combos[j]).unwrap());
        }
    }
    let hu_seconds=start.elapsed().as_secs_f64();
    let hu_unique=hu_keys.len() as u64;
    println!(
        "status=RESEARCH_ONLY census=HU_FULL ordered_legal_pairs={} unique_canonical_keys={} dedup_factor={:.6} keygen_seconds={:.6} raw_pairs_per_second={:.3}",
        raw_hu,hu_unique,raw_hu as f64/hu_unique as f64,hu_seconds,raw_hu as f64/hu_seconds
    );

    // Bounded 3-way throughput sample only. Four fixed hero combos already
    // cover millions of legal ordered triples while keeping this CI diagnostic
    // safely below the workflow timeout. This is NOT a full 1326^3 census.
    let hero_limit=4usize;
    let start3=Instant::now();
    let mut raw3=0u64;
    let mut keys3=HashSet::new();
    for hero in 0..hero_limit{
        for a in 0..COMBO_COUNT{
            if !blockers.compatible(hero,a){continue;}
            for b in 0..COMBO_COUNT{
                if !blockers.compatible(hero,b)||!blockers.compatible(a,b){continue;}
                raw3+=1;
                keys3.insert(canonical_threeway(combos[hero],combos[a],combos[b]).unwrap());
            }
        }
    }
    let sec3=start3.elapsed().as_secs_f64();
    println!(
        "status=RESEARCH_ONLY census=THREEWAY_BOUNDED hero_count={} ordered_legal_triples={} unique_canonical_keys={} observed_dedup_factor={:.6} keygen_seconds={:.6} raw_triples_per_second={:.3}",
        hero_limit,raw3,keys3.len(),raw3 as f64/keys3.len() as f64,sec3,raw3 as f64/sec3
    );
    println!("NOTE: THREEWAY_BOUNDED is a throughput/dedup sample only; never extrapolate it as an exact full-census key count without a separate proof or full enumeration.");
}
