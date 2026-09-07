use spins_solver_core::blockers::BlockerMatrix;
use spins_solver_core::cards::{all_combos,Card,Combo,COMBO_COUNT};
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::equity3::canonical_threeway;
use spins_solver_core::exact_equity::{exact_hu_equity,ExactEquityCache};
use spins_solver_core::exact_equity3::{exact_threeway_equity,ExactThreeWayEquityCache};
use spins_solver_core::exact_leaf::{exact_bb_leaf_action_values,persisted_bb_leaf_action_values};
use spins_solver_core::exact_leaf3::{exact_bb_after_btn_jam_sb_call_values,persisted_bb_after_btn_jam_sb_call_values};
use spins_solver_core::leaf_ev::BbHuLeaf;
use spins_solver_core::payoff_lookup::{HuPayoffLookup,ThreeWayPayoffLookup};
use spins_solver_core::payoff_table::{HuPayoffRecord,HuPayoffTable,ThreeWayPayoffRecord,ThreeWayPayoffTable};
use spins_solver_core::range::ComboRange;

fn c(r:u8,s:u8)->Card{r*4+s}
fn idx(target:Combo)->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
fn singleton(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}

fn main(){
    let aa=[c(12,0),c(12,1)];
    let kk=[c(11,2),c(11,3)];
    let qq=[c(10,0),c(10,1)];
    let aa_i=idx(aa);let kk_i=idx(kk);let qq_i=idx(qq);
    let blockers=BlockerMatrix::build();

    // HU runtime persisted table contains exactly the required KK(hero) vs AA(jammer) payoff.
    let hu_e=exact_hu_equity(kk,aa).expect("HU fixture equity");
    let hu_table=HuPayoffTable{records:vec![HuPayoffRecord{
        key:canonical_hu_matchup(kk,aa).unwrap(),wins:hu_e.wins,losses:hu_e.losses,ties:hu_e.ties,
    }]};
    let hu_lookup=HuPayoffLookup::from_table(hu_table).unwrap();
    let jammer=singleton(aa_i);
    let mut hu_cache=ExactEquityCache::new();
    let hu_direct=exact_bb_leaf_action_values(BbHuLeaf::AfterBtnFoldSbJam,8.0,kk_i,&jammer,&blockers,&mut hu_cache).unwrap();
    let hu_persisted=persisted_bb_leaf_action_values(BbHuLeaf::AfterBtnFoldSbJam,8.0,kk_i,&jammer,&blockers,&hu_lookup).unwrap();
    assert!((hu_direct.call_equity.hero_equity-hu_persisted.call_equity.hero_equity).abs()<1e-12);
    assert!((hu_direct.fold_ev_bb-hu_persisted.fold_ev_bb).abs()<1e-12);
    assert!((hu_direct.call_ev_bb-hu_persisted.call_ev_bb).abs()<1e-12);

    // 3-way runtime persisted table uses role order BB=QQ, BTN=AA, SB=KK.
    let three_e=exact_threeway_equity(qq,aa,kk).expect("3-way fixture equity");
    let three_table=ThreeWayPayoffTable{records:vec![ThreeWayPayoffRecord{
        key:canonical_threeway(qq,aa,kk).unwrap(),
        outright_wins:three_e.outright_wins,
        two_way_ties:three_e.two_way_ties,
        three_way_ties:three_e.three_way_ties,
    }]};
    let three_lookup=ThreeWayPayoffLookup::from_table(three_table).unwrap();
    let btn=singleton(aa_i);let sb=singleton(kk_i);
    let mut three_cache=ExactThreeWayEquityCache::new();
    let three_direct=exact_bb_after_btn_jam_sb_call_values(8.0,qq_i,&btn,&sb,&blockers,&mut three_cache).unwrap();
    let three_persisted=persisted_bb_after_btn_jam_sb_call_values(8.0,qq_i,&btn,&sb,&blockers,&three_lookup).unwrap();
    for i in 0..3{assert!((three_direct.call_equity.equities[i]-three_persisted.call_equity.equities[i]).abs()<1e-12);}
    assert!((three_direct.fold_ev_bb-three_persisted.fold_ev_bb).abs()<1e-12);
    assert!((three_direct.call_ev_bb-three_persisted.call_ev_bb).abs()<1e-12);

    // Deliberately incomplete runtime tables must fail closed.
    let empty_hu=HuPayoffLookup::from_table(HuPayoffTable{records:vec![]}).unwrap();
    let hu_missing=persisted_bb_leaf_action_values(BbHuLeaf::AfterBtnFoldSbJam,8.0,kk_i,&jammer,&blockers,&empty_hu).unwrap_err();
    assert!(hu_missing.contains("missing exact HU payoff key"));

    let empty_three=ThreeWayPayoffLookup::from_table(ThreeWayPayoffTable{records:vec![]}).unwrap();
    let three_missing=persisted_bb_after_btn_jam_sb_call_values(8.0,qq_i,&btn,&sb,&blockers,&empty_three).unwrap_err();
    assert!(three_missing.contains("missing exact 3-way payoff key"));

    println!(
        "status=RESEARCH_ONLY mode=PERSISTED_EXACT_LEAF_EQUIVALENCE hu_equity={:.9} hu_fold_ev={:.9} hu_call_ev={:.9} three_equities_bb_btn_sb={:.9},{:.9},{:.9} three_fold_ev={:.9} three_call_ev={:.9} direct_vs_persisted=PASS missing_hu=REJECT missing_threeway=REJECT sampled_fallback=NONE",
        hu_persisted.call_equity.hero_equity,hu_persisted.fold_ev_bb,hu_persisted.call_ev_bb,
        three_persisted.call_equity.equities[0],three_persisted.call_equity.equities[1],three_persisted.call_equity.equities[2],
        three_persisted.fold_ev_bb,three_persisted.call_ev_bb
    );
}
