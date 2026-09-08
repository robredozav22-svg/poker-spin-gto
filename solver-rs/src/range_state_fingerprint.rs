use std::collections::HashSet;

use sha2::{Digest,Sha256};

use crate::cards::{Card,Combo};

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct RangeStateId(String);

impl RangeStateId{
    pub fn as_str(&self)->&str{&self.0}

    pub fn parse(value:&str)->Result<Self,String>{
        let hex=value.strip_prefix("sha256:").ok_or_else(||"range_state_id must use sha256: prefix".to_string())?;
        if hex.len()!=64{return Err("range_state_id sha256 digest must contain exactly 64 hex characters".into());}
        if !hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err("range_state_id sha256 digest must be lowercase hexadecimal".into());}
        Ok(Self(value.to_string()))
    }
}

pub fn fingerprint_range_state(
    context:&str,
    board:&[Card],
    p0:&[(Combo,f64)],
    p1:&[(Combo,f64)],
)->Result<RangeStateId,String>{
    if context.trim().is_empty(){return Err("range-state fingerprint context must not be empty".into());}
    if p0.is_empty()||p1.is_empty(){return Err("range-state fingerprint requires both ranges".into());}
    validate_board(board)?;
    let a=canonical_range(board,p0,"p0")?;
    let b=canonical_range(board,p1,"p1")?;

    let mut h=Sha256::new();
    h.update(b"SPINS_RANGE_STATE_V1\0");
    write_len(&mut h,context.len());
    h.update(context.as_bytes());
    write_len(&mut h,board.len());
    for c in board{h.update([*c]);}
    write_range(&mut h,&a);
    write_range(&mut h,&b);
    let digest=h.finalize();
    RangeStateId::parse(&format!("sha256:{:x}",digest))
}

fn validate_board(board:&[Card])->Result<(),String>{
    if board.len()>5{return Err("range-state board cannot exceed 5 cards".into());}
    let mut seen=[false;52];
    for c in board{
        if *c>=52{return Err("range-state board card out of range".into());}
        if seen[*c as usize]{return Err("range-state board contains duplicate card".into());}
        seen[*c as usize]=true;
    }
    Ok(())
}

fn canonical_range(board:&[Card],range:&[(Combo,f64)],name:&str)->Result<Vec<(Combo,u64)>,String>{
    let mut seen=HashSet::new();
    let mut out=Vec::with_capacity(range.len());
    let mut positive=false;
    for (hand,w) in range{
        if hand[0]>=52||hand[1]>=52||hand[0]==hand[1]{return Err(format!("invalid {name} combo"));}
        if board.contains(&hand[0])||board.contains(&hand[1]){return Err(format!("{name} combo overlaps board"));}
        if !w.is_finite()||*w<0.0{return Err(format!("invalid {name} weight"));}
        let hand=if hand[0]<hand[1]{*hand}else{[hand[1],hand[0]]};
        if !seen.insert(hand){return Err(format!("duplicate {name} combo in range-state fingerprint input"));}
        if *w>0.0{positive=true;}
        out.push((hand,w.to_bits()));
    }
    if !positive{return Err(format!("{name} range has zero mass"));}
    out.sort_by_key(|x|(x.0[0],x.0[1]));
    Ok(out)
}

fn write_len(h:&mut Sha256,n:usize){h.update((n as u64).to_le_bytes());}
fn write_range(h:&mut Sha256,range:&[(Combo,u64)]){
    write_len(h,range.len());
    for (hand,bits) in range{
        h.update([hand[0],hand[1]]);
        h.update(bits.to_le_bytes());
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn row_order_does_not_change_fingerprint(){
        let a=vec![([1,2],0.5),([3,4],1.0)];
        let b=vec![([5,6],0.7),([7,8],0.3)];
        let x=fingerprint_range_state("edge-a",&[9,10,11],&a,&b).unwrap();
        let a2=vec![([3,4],1.0),([2,1],0.5)];
        let b2=vec![([8,7],0.3),([5,6],0.7)];
        let y=fingerprint_range_state("edge-a",&[9,10,11],&a2,&b2).unwrap();
        assert_eq!(x,y);
    }

    #[test]
    fn one_weight_change_changes_fingerprint(){
        let a=vec![([1,2],0.5),([3,4],1.0)];
        let b=vec![([5,6],0.7),([7,8],0.3)];
        let x=fingerprint_range_state("edge-a",&[9,10,11],&a,&b).unwrap();
        let mut a2=a.clone();a2[0].1=0.5000000001;
        let y=fingerprint_range_state("edge-a",&[9,10,11],&a2,&b).unwrap();
        assert_ne!(x,y);
    }

    #[test]
    fn context_change_changes_fingerprint(){
        let a=vec![([1,2],1.0)];let b=vec![([3,4],1.0)];
        assert_ne!(fingerprint_range_state("edge-a",&[9],&a,&b).unwrap(),fingerprint_range_state("edge-b",&[9],&a,&b).unwrap());
    }

    #[test]
    fn malformed_serialized_id_is_rejected(){
        assert!(RangeStateId::parse("range-v1").is_err());
        assert!(RangeStateId::parse("sha256:ABC").is_err());
        assert!(RangeStateId::parse(&format!("sha256:{}","a".repeat(64))).is_ok());
    }

    #[test]
    fn duplicate_combo_is_rejected(){
        let a=vec![([1,2],0.5),([2,1],0.5)];let b=vec![([3,4],1.0)];
        assert!(fingerprint_range_state("edge",&[9],&a,&b).is_err());
    }
}
