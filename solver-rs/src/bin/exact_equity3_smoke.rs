use spins_solver_core::cards::Card;
use spins_solver_core::exact_equity3::{exact_threeway_equity,THREEWAY_PREFLOP_BOARD_COUNT};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let first=[c(12,0),c(12,1)]; // AA
    let second=[c(11,2),c(11,3)]; // KK
    let third=[c(10,0),c(10,1)]; // QQ
    let e=exact_threeway_equity(first,second,third).expect("exact 3-way equity");
    assert_eq!(e.boards,THREEWAY_PREFLOP_BOARD_COUNT);
    let classified=e.outright_wins.iter().sum::<u64>()+e.two_way_ties.iter().sum::<u64>()+e.three_way_ties;
    assert_eq!(classified,e.boards);
    assert!(e.zero_sum_error()<1e-12);
    println!("status=RESEARCH_ONLY mode=EXACT_3WAY boards={} equities={:.9},{:.9},{:.9} outright={},{},{} two_way_ties={},{},{} three_way_ties={} zero_sum_error={:.3e}",e.boards,e.equities[0],e.equities[1],e.equities[2],e.outright_wins[0],e.outright_wins[1],e.outright_wins[2],e.two_way_ties[0],e.two_way_ties[1],e.two_way_ties[2],e.three_way_ties,e.zero_sum_error());
}
