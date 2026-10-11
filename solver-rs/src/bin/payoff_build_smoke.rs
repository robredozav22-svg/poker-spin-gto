use spins_solver_core::cards::Card;
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::equity3::canonical_threeway;
use spins_solver_core::payoff_build::{build_hu_payoff_table,build_threeway_payoff_table};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let aa=[c(12,0),c(12,1)];
    let kk=[c(11,2),c(11,3)];
    let qq=[c(10,0),c(10,1)];
    let aks=[c(12,2),c(11,2)];
    let jj=[c(9,0),c(9,1)];

    let hu1=canonical_hu_matchup(aa,kk).unwrap();
    let hu2=canonical_hu_matchup(aks,jj).unwrap();

    let (first,first_stats)=build_hu_payoff_table(&[hu1],None).expect("first HU build");
    assert_eq!(first_stats.requested_unique,1);
    assert_eq!(first_stats.reused_existing,0);
    assert_eq!(first_stats.computed_missing,1);
    assert_eq!(first.records.len(),1);

    let (second,second_stats)=build_hu_payoff_table(&[hu2,hu1,hu2],Some(&first)).expect("incremental HU build");
    assert_eq!(second_stats.requested_unique,2);
    assert_eq!(second_stats.reused_existing,1);
    assert_eq!(second_stats.computed_missing,1);
    assert_eq!(second.records.len(),2);
    assert!(second.records.windows(2).all(|w|w[0].key.0<w[1].key.0));

    let (third,third_stats)=build_hu_payoff_table(&[hu1,hu2],Some(&second)).expect("idempotent HU rebuild");
    assert_eq!(third_stats.computed_missing,0);
    assert_eq!(third_stats.reused_existing,2);
    let second_bytes=second.encode().unwrap();
    let third_bytes=third.encode().unwrap();
    assert_eq!(second_bytes,third_bytes);

    let tkey=canonical_threeway(aa,kk,qq).unwrap();
    let (three_first,three_first_stats)=build_threeway_payoff_table(&[tkey],None).expect("first 3-way build");
    assert_eq!(three_first_stats.computed_missing,1);
    let (three_second,three_second_stats)=build_threeway_payoff_table(&[tkey,tkey],Some(&three_first)).expect("3-way reuse build");
    assert_eq!(three_second_stats.requested_unique,1);
    assert_eq!(three_second_stats.reused_existing,1);
    assert_eq!(three_second_stats.computed_missing,0);
    assert_eq!(three_first.encode().unwrap(),three_second.encode().unwrap());

    println!(
        "status=RESEARCH_ONLY mode=PAYOFF_BUILD_COMPUTE_MISSING hu_first_computed={} hu_incremental_reused={} hu_incremental_computed={} hu_output_records={} hu_idempotent_bytes={} threeway_first_computed={} threeway_reuse={} threeway_recomputed={} threeway_output_records={}",
        first_stats.computed_missing,
        second_stats.reused_existing,
        second_stats.computed_missing,
        second_stats.output_records,
        second_bytes.len(),
        three_first_stats.computed_missing,
        three_second_stats.reused_existing,
        three_second_stats.computed_missing,
        three_second_stats.output_records,
    );
}
