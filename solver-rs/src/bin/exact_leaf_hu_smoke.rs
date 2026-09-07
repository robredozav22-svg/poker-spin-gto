use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,Card,Combo,COMBO_COUNT};
use spins_solver_core::exact_equity::ExactEquityCache;
use spins_solver_core::exact_leaf::exact_bb_leaf_action_values;
use spins_solver_core::leaf_ev::BbHuLeaf;
use spins_solver_core::range::ComboRange;

fn c(r:u8,s:u8)->Card{r*4+s}
fn idx(target:Combo)->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
fn singleton(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}

fn main(){
    let jammer=idx([c(12,0),c(12,1)]); // AA
    let bb=idx([c(11,2),c(11,3)]);     // KK
    let blockers=BlockerMatrix::build();
    assert!(blockers.compatible(jammer,bb));
    let range=singleton(jammer);
    let mut cache=ExactEquityCache::new();

    let sb_jam=exact_bb_leaf_action_values(
        BbHuLeaf::AfterBtnFoldSbJam,8.0,bb,&range,&blockers,&mut cache
    ).expect("exact BB vs SB jam");
    assert_eq!(sb_jam.fold_ev_bb,-1.0);
    assert_eq!(sb_jam.call_equity.compatible_combos,1);
    assert!((sb_jam.call_equity.compatible_weight-1.0).abs()<1e-12);
    assert!((sb_jam.call_equity.hero_equity-0.187445103).abs()<1e-8);
    assert_eq!(cache.misses(),1);

    let btn_jam=exact_bb_leaf_action_values(
        BbHuLeaf::AfterBtnJamSbFold,8.0,bb,&range,&blockers,&mut cache
    ).expect("exact BB vs BTN jam");
    assert_eq!(btn_jam.fold_ev_bb,-1.0);
    assert_eq!(btn_jam.call_equity.compatible_combos,1);
    assert!((btn_jam.call_equity.hero_equity-sb_jam.call_equity.hero_equity).abs()<1e-12);
    assert_eq!(cache.misses(),1);
    assert_eq!(cache.hits(),1);

    println!(
        "status=RESEARCH_ONLY mode=EXACT_HU_LEAVES bb=KK jammer=AA equity_bb={:.9} sb_jam_fold_ev={:.9} sb_jam_call_ev={:.9} btn_jam_fold_ev={:.9} btn_jam_call_ev={:.9} cache_misses={} cache_hits={}",
        sb_jam.call_equity.hero_equity,
        sb_jam.fold_ev_bb,sb_jam.call_ev_bb,
        btn_jam.fold_ev_bb,btn_jam.call_ev_bb,
        cache.misses(),cache.hits()
    );
}
