use spins_solver_core::leduc_cfr::LeducCfr;

fn main(){
    let iterations=50_000usize;
    let mut cfr=LeducCfr::new();
    cfr.train(iterations).expect("Leduc CFR training");
    let report=cfr.evaluate_average().expect("Leduc average evaluation");
    assert!(report.absolute_error<0.004,"Leduc value error too large: {}",report.absolute_error);
    assert!(report.infosets>100,"Leduc infoset count unexpectedly small: {}",report.infosets);
    println!(
        "status=RESEARCH_ONLY mode=LEDUC_CFR_SEQUENCE_FORM_VALUE_VALIDATION iterations={} known_p0_value={:.12} measured_p0_value={:.12} value_abs_error={:.12} infosets={} chance_deals=120 public_card_round=ENABLED bet_sizes=2,4 max_raise_per_round=1 value_gate=PASS",
        iterations,report.known_value,report.p0_value,report.absolute_error,report.infosets
    );
}
