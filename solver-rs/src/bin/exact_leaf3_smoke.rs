use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,Card,Combo,COMBO_COUNT};
use spins_solver_core::exact_equity3::ExactThreeWayEquityCache;
use spins_solver_core::exact_leaf3::exact_bb_after_btn_jam_sb_call_values;
use spins_solver_core::range::ComboRange;

fn c(r:u8,s:u8)->Card{r*4+s}
fn idx(target:Combo)->usize{
    all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()
}
fn singleton(i:usize)->ComboRange{
    let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()
}

fn main(){
    let btn=idx([c(12,0),c(12,1)]); // AA
    let sb=idx([c(11,2),c(11,3)]);  // KK
    let bb=idx([c(10,0),c(10,1)]);  // QQ
    let blockers=BlockerMatrix::build();
    assert!(blockers.compatible(btn,sb));
    assert!(blockers.compatible(btn,bb));
    assert!(blockers.compatible(sb,bb));

    let mut cache=ExactThreeWayEquityCache::new();
    let v=exact_bb_after_btn_jam_sb_call_values(
        8.0,bb,&singleton(btn),&singleton(sb),&blockers,&mut cache
    ).expect("exact 3-way leaf");

    assert_eq!(v.call_equity.compatible_pairs,1);
    assert!((v.call_equity.compatible_pair_weight-1.0).abs()<1e-12);
    assert_eq!(cache.misses(),1);
    assert!((v.fold_ev_bb+1.0).abs()<1e-12);
    assert!(v.call_ev_bb.is_finite());
    assert!((v.call_equity.equities.iter().sum::<f64>()-1.0).abs()<1e-9);

    println!(
        "status=RESEARCH_ONLY mode=EXACT_3WAY_LEAF node=BTN_JAM_SB_CALL_BB_DECISION fold_ev_bb={:.9} call_ev_bb={:.9} equities_bb_btn_sb={:.9},{:.9},{:.9} compatible_pairs={} cache_misses={} cache_hits={}",
        v.fold_ev_bb,v.call_ev_bb,
        v.call_equity.equities[0],v.call_equity.equities[1],v.call_equity.equities[2],
        v.call_equity.compatible_pairs,cache.misses(),cache.hits()
    );
}
