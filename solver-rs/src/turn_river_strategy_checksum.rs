use sha2::{Digest,Sha256};

use crate::holdem_turn_river_cfr::TurnRiverStrategy;

pub fn checksum_turn_river_strategy(strategy:&TurnRiverStrategy)->String{
    let mut h=Sha256::new();
    h.update(b"SPINS_TURN_RIVER_STRATEGY_V1\0");
    write_rows(&mut h,&strategy.turn_p0_root);
    write_rows(&mut h,&strategy.turn_p0_facing_bet);
    write_rows(&mut h,&strategy.turn_p1_after_check);
    write_rows(&mut h,&strategy.turn_p1_facing_bet);
    write_street(&mut h,&strategy.river_p0_root);
    write_street(&mut h,&strategy.river_p0_facing_bet);
    write_street(&mut h,&strategy.river_p1_after_check);
    write_street(&mut h,&strategy.river_p1_facing_bet);
    format!("sha256:{:x}",h.finalize())
}

fn write_len(h:&mut Sha256,n:usize){h.update((n as u64).to_le_bytes());}
fn write_row(h:&mut Sha256,row:&[f64;2]){h.update(row[0].to_bits().to_le_bytes());h.update(row[1].to_bits().to_le_bytes());}
fn write_rows(h:&mut Sha256,rows:&[[f64;2]]){write_len(h,rows.len());for row in rows{write_row(h,row);}}
fn write_street(h:&mut Sha256,street:&Vec<Vec<Vec<[f64;2]>>>) {
    write_len(h,street.len());
    for path in street{
        write_len(h,path.len());
        for river in path{write_rows(h,river);}
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;
    use crate::holdem_turn_chance::TurnChanceTree;
    use crate::holdem_turn_river_cfr::HoldemTurnRiverCfr;

    fn c(r:u8,s:u8)->Card{r*4+s}
    fn strategy()->TurnRiverStrategy{
        let chance=TurnChanceTree::new(
            [c(0,0),c(5,1),c(7,2),c(9,3)],
            vec![([c(12,2),c(12,3)],1.0),([c(11,0),c(11,1)],0.7)],
            vec![([c(12,0),c(10,1)],1.0),([c(9,0),c(9,1)],0.8)],
        ).unwrap();
        let mut game=HoldemTurnRiverCfr::new(chance,4.0,4.0,8.0).unwrap();
        game.train(50).unwrap();
        game.average_strategy()
    }

    #[test]fn same_strategy_has_same_checksum(){let s=strategy();assert_eq!(checksum_turn_river_strategy(&s),checksum_turn_river_strategy(&s));}
    #[test]fn one_probability_change_changes_checksum(){let s=strategy();let mut t=s.clone();t.turn_p0_root[0]=[0.25,0.75];assert_ne!(checksum_turn_river_strategy(&s),checksum_turn_river_strategy(&t));}
    #[test]fn checksum_has_sha256_shape(){let x=checksum_turn_river_strategy(&strategy());assert!(x.starts_with("sha256:"));assert_eq!(x.len(),71);}
}
