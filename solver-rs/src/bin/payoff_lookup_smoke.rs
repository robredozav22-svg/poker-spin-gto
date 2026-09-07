use spins_solver_core::cards::Card;
use spins_solver_core::equity::{canonical_hu_matchup};
use spins_solver_core::equity3::canonical_threeway;
use spins_solver_core::exact_equity::exact_hu_equity;
use spins_solver_core::exact_equity3::exact_threeway_equity;
use spins_solver_core::payoff_lookup::{HuPayoffLookup,ThreeWayPayoffLookup};
use spins_solver_core::payoff_table::{HuPayoffRecord,HuPayoffTable,ThreeWayPayoffRecord,ThreeWayPayoffTable};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let aa=[c(12,0),c(12,1)];
    let kk=[c(11,2),c(11,3)];
    let qq=[c(10,0),c(10,1)];

    let hu_direct=exact_hu_equity(aa,kk).expect("direct exact HU");
    let hu_record=HuPayoffRecord{
        key:canonical_hu_matchup(aa,kk).unwrap(),
        wins:hu_direct.wins,losses:hu_direct.losses,ties:hu_direct.ties,
    };
    let hu_bytes=HuPayoffTable{records:vec![hu_record]}.encode().unwrap();
    let hu_decoded=HuPayoffTable::decode(&hu_bytes).unwrap();
    let hu_lookup=HuPayoffLookup::from_table(hu_decoded).unwrap();
    let hu_loaded=hu_lookup.get_equity(aa,kk).unwrap().expect("HU persisted key present");
    assert_eq!(hu_loaded.wins,hu_direct.wins);
    assert_eq!(hu_loaded.losses,hu_direct.losses);
    assert_eq!(hu_loaded.ties,hu_direct.ties);
    assert!((hu_loaded.hero-hu_direct.hero).abs()<1e-15);

    let three_direct=exact_threeway_equity(aa,kk,qq).expect("direct exact 3-way");
    let three_record=ThreeWayPayoffRecord{
        key:canonical_threeway(aa,kk,qq).unwrap(),
        outright_wins:three_direct.outright_wins,
        two_way_ties:three_direct.two_way_ties,
        three_way_ties:three_direct.three_way_ties,
    };
    let three_bytes=ThreeWayPayoffTable{records:vec![three_record]}.encode().unwrap();
    let three_decoded=ThreeWayPayoffTable::decode(&three_bytes).unwrap();
    let three_lookup=ThreeWayPayoffLookup::from_table(three_decoded).unwrap();
    let three_loaded=three_lookup.get_equity(aa,kk,qq).unwrap().expect("3-way persisted key present");
    assert_eq!(three_loaded.outright_wins,three_direct.outright_wins);
    assert_eq!(three_loaded.two_way_ties,three_direct.two_way_ties);
    assert_eq!(three_loaded.three_way_ties,three_direct.three_way_ties);
    for i in 0..3{assert!((three_loaded.equities[i]-three_direct.equities[i]).abs()<1e-15);}

    let missing=[c(9,0),c(8,1)];
    assert!(hu_lookup.get_equity(aa,missing).unwrap().is_none());
    assert!(three_lookup.get_equity(aa,kk,missing).unwrap().is_none());

    println!(
        "status=RESEARCH_ONLY mode=PERSISTED_PAYOFF_LOOKUP hu_records={} hu_bytes={} hu_equity={:.9} threeway_records={} threeway_bytes={} threeway_equities={:.9},{:.9},{:.9} missing_lookup=NONE duplicate_policy=REJECT",
        hu_lookup.len(),hu_bytes.len(),hu_loaded.hero,
        three_lookup.len(),three_bytes.len(),three_loaded.equities[0],three_loaded.equities[1],three_loaded.equities[2]
    );
}
