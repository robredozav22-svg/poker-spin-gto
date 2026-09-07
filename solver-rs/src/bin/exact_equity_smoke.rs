use spins_solver_core::cards::Card;
use spins_solver_core::exact_equity::{exact_hu_equity,HU_PREFLOP_BOARD_COUNT};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let hero=[c(12,0),c(12,1)]; // AA
    let villain=[c(11,2),c(11,3)]; // KK
    let e=exact_hu_equity(hero,villain).expect("exact HU equity");
    assert_eq!(e.boards,HU_PREFLOP_BOARD_COUNT);
    assert_eq!(e.wins+e.losses+e.ties,e.boards);
    assert!(e.zero_sum_error()<1e-12);
    println!("status=RESEARCH_ONLY mode=EXACT_HU boards={} hero_equity={:.9} villain_equity={:.9} wins={} losses={} ties={} zero_sum_error={:.3e}",e.boards,e.hero,e.villain,e.wins,e.losses,e.ties,e.zero_sum_error());
}
