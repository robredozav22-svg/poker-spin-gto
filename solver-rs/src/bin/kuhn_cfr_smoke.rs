use spins_solver_core::kuhn_cfr::{evaluate_kuhn,KuhnCfr};

fn main(){
    let iterations=200_000usize;
    let mut cfr=KuhnCfr::new();
    cfr.train(iterations).expect("Kuhn CFR training");
    let strategy=cfr.average_strategy();
    let report=evaluate_kuhn(&strategy).expect("Kuhn independent brute-force evaluation");
    let known=-1.0/18.0;
    let value_error=(report.p1_value-known).abs();
    assert!(value_error<0.0015,"Kuhn value error too large: {value_error}");
    assert!(report.nashconv<0.008,"Kuhn NashConv too large: {}",report.nashconv);
    println!(
        "status=RESEARCH_ONLY mode=KUHN_CFR_ANALYTICAL_VALIDATION iterations={} known_p1_value={:.12} measured_p1_value={:.12} value_abs_error={:.12} p1_br={:.12} p1_vs_p2_br={:.12} p1_br_gain={:.12} p2_br_gain={:.12} nashconv={:.12} independent_brute_force_evaluator=PASS analytical_value_gate=PASS",
        iterations,known,report.p1_value,value_error,report.p1_best_response_value,report.p1_value_vs_p2_best_response,report.p1_br_gain,report.p2_br_gain,report.nashconv
    );
}
