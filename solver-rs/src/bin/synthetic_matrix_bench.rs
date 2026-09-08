use spins_solver_core::matrix_game::ZeroSumMatrixGame;
use spins_solver_core::regret::RegretTable;

fn run_case(
    name:&str,
    game:ZeroSumMatrixGame,
    expected_row:[f64;2],
    expected_col:[f64;2],
    expected_value:f64,
){
    let mut row=RegretTable::new(1,game.row_actions());
    let mut col=RegretTable::new(1,game.col_actions());
    let checkpoints=[1usize,10,100,1_000,10_000,100_000];
    println!("case={name} status=RESEARCH_ONLY iterations=100000");
    for iteration in 1..=100_000{
        game.simultaneous_sweep(&mut row,&mut col).expect("matrix sweep");
        if checkpoints.contains(&iteration){
            let ra=row.average_strategy();
            let ca=col.average_strategy();
            let r=game.evaluate(ra.row(0),ca.row(0)).expect("matrix evaluation");
            let row_err=ra.row(0).iter().zip(expected_row).map(|(a,b)|(a-b).abs()).fold(0.0f64,f64::max);
            let col_err=ca.row(0).iter().zip(expected_col).map(|(a,b)|(a-b).abs()).fold(0.0f64,f64::max);
            println!(
                "iteration={iteration} row={:.9},{:.9} col={:.9},{:.9} value={:.9} nashconv={:.9} row_max_error={row_err:.9} col_max_error={col_err:.9} value_error={:.9}",
                ra.row(0)[0],ra.row(0)[1],ca.row(0)[0],ca.row(0)[1],r.value,r.nashconv,(r.value-expected_value).abs()
            );
        }
    }
    let ra=row.average_strategy();
    let ca=col.average_strategy();
    let report=game.evaluate(ra.row(0),ca.row(0)).unwrap();
    let row_err=ra.row(0).iter().zip(expected_row).map(|(a,b)|(a-b).abs()).fold(0.0f64,f64::max);
    let col_err=ca.row(0).iter().zip(expected_col).map(|(a,b)|(a-b).abs()).fold(0.0f64,f64::max);
    let value_err=(report.value-expected_value).abs();
    assert!(row_err<0.01,"{name}: row equilibrium error {row_err}");
    assert!(col_err<0.01,"{name}: col equilibrium error {col_err}");
    assert!(value_err<0.01,"{name}: value error {value_err}");
    assert!(report.nashconv<0.02,"{name}: NashConv {}",report.nashconv);
    println!("case={name} result=PASS final_nashconv={:.9} final_row_error={row_err:.9} final_col_error={col_err:.9} final_value_error={value_err:.9}",report.nashconv);
}

fn main(){
    run_case(
        "matching_pennies",
        ZeroSumMatrixGame::new(2,2,vec![1.0,-1.0,-1.0,1.0]).unwrap(),
        [0.5,0.5],
        [0.5,0.5],
        0.0,
    );
    run_case(
        "asymmetric_4_0_neg1_2",
        ZeroSumMatrixGame::new(2,2,vec![4.0,0.0,-1.0,2.0]).unwrap(),
        [3.0/7.0,4.0/7.0],
        [2.0/7.0,5.0/7.0],
        8.0/7.0,
    );
    println!("NOTE: synthetic analytical game validation only; no poker chart data");
}
