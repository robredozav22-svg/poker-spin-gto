use spins_solver_core::cards::Card;
use spins_solver_core::equity::sampled_hu_equity;
use spins_solver_core::equity3::sampled_threeway_equity;
use spins_solver_core::exact_equity::exact_hu_equity;
use spins_solver_core::exact_equity3::exact_threeway_equity;

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let aa=[c(12,0),c(12,1)];
    let kk=[c(11,2),c(11,3)];
    let qq=[c(10,0),c(10,1)];

    let exact_hu=exact_hu_equity(aa,kk).expect("exact HU");
    let sampled_hu=sampled_hu_equity(aa,kk,200_000,20260907).expect("sampled HU");
    let hu_abs=(sampled_hu.hero-exact_hu.hero).abs();
    assert!(hu_abs<=0.01,"HU sampled/exact difference too large: {hu_abs}");

    let exact_3=exact_threeway_equity(aa,kk,qq).expect("exact 3-way");
    let sampled_3=sampled_threeway_equity(aa,kk,qq,200_000,20260907).expect("sampled 3-way");
    let diffs=[
        (sampled_3.equities[0]-exact_3.equities[0]).abs(),
        (sampled_3.equities[1]-exact_3.equities[1]).abs(),
        (sampled_3.equities[2]-exact_3.equities[2]).abs(),
    ];
    let max_3=diffs.into_iter().fold(0.0f64,f64::max);
    assert!(max_3<=0.01,"3-way sampled/exact max difference too large: {max_3}");

    println!(
        "status=RESEARCH_ONLY policy=EXACT_CANONICAL_SAMPLED_CROSSCHECK hu_exact={:.9} hu_sampled={:.9} hu_abs_diff={:.9} threeway_exact={:.9},{:.9},{:.9} threeway_sampled={:.9},{:.9},{:.9} threeway_max_abs_diff={:.9}",
        exact_hu.hero,sampled_hu.hero,hu_abs,
        exact_3.equities[0],exact_3.equities[1],exact_3.equities[2],
        sampled_3.equities[0],sampled_3.equities[1],sampled_3.equities[2],
        max_3
    );
}
