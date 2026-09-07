use spins_solver_core::cards::Card;
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::payoff_build::{build_hu_payoff_table,build_hu_payoff_table_parallel};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let k1=canonical_hu_matchup([c(12,0),c(12,1)],[c(11,2),c(11,3)]).unwrap(); // AA vs KK
    let k2=canonical_hu_matchup([c(12,2),c(11,2)],[c(10,0),c(10,1)]).unwrap(); // AKs vs QQ
    let requested=[k2,k1,k2];

    let (serial,serial_stats)=build_hu_payoff_table(&requested,None).expect("serial exact build");
    let (parallel,parallel_stats)=build_hu_payoff_table_parallel(&requested,None,2).expect("parallel exact build");

    assert_eq!(serial_stats.requested_unique,2);
    assert_eq!(parallel_stats.requested_unique,2);
    assert_eq!(serial_stats.computed_missing,2);
    assert_eq!(parallel_stats.computed_missing,2);
    assert_eq!(serial.records,parallel.records);
    let serial_bytes=serial.encode().unwrap();
    let parallel_bytes=parallel.encode().unwrap();
    assert_eq!(serial_bytes,parallel_bytes);
    assert!(parallel.records.windows(2).all(|w|w[0].key.0<w[1].key.0));

    println!(
        "status=RESEARCH_ONLY mode=PARALLEL_PAYOFF_BUILD_EQUIVALENCE requested_unique={} serial_computed={} parallel_computed={} workers=2 records={} payload_bytes={} exact_integer_equivalence=PASS byte_identical=PASS canonical_sorted=PASS",
        parallel_stats.requested_unique,serial_stats.computed_missing,parallel_stats.computed_missing,parallel.records.len(),parallel_bytes.len()
    );
}
