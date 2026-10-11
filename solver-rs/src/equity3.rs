use crate::cards::{rank, suit, Card, Combo, DECK_SIZE};
use crate::evaluator::evaluate_seven;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreeWayKey(pub [Card; 6]);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThreeWayEquityEstimate {
    pub equities: [f64; 3],
    pub samples: u64,
    pub seed: u64,
}

fn permutations4() -> [[u8; 4]; 24] {
    let mut out=[[0u8;4];24];
    let mut k=0;
    for a in 0..4 { for b in 0..4 { if b==a {continue;} for c in 0..4 {
        if c==a||c==b {continue;} for d in 0..4 { if d==a||d==b||d==c {continue;}
            out[k]=[a,b,c,d]; k+=1;
        }
    }}}
    debug_assert_eq!(k,24);
    out
}

fn map_card(card:Card,p:[u8;4])->Card{rank(card)*4+p[suit(card) as usize]}
fn sorted(mut c:Combo)->Combo{if c[0]>c[1]{c.swap(0,1);}c}

pub fn canonical_threeway(a:Combo,b:Combo,c:Combo)->Result<ThreeWayKey,String>{
    let cards=[a[0],a[1],b[0],b[1],c[0],c[1]];
    for i in 0..6 { for j in (i+1)..6 { if cards[i]==cards[j] {return Err("private cards overlap".into());}}}
    let mut best:Option<[Card;6]>=None;
    for p in permutations4(){
        let aa=sorted([map_card(a[0],p),map_card(a[1],p)]);
        let bb=sorted([map_card(b[0],p),map_card(b[1],p)]);
        let cc=sorted([map_card(c[0],p),map_card(c[1],p)]);
        let candidate=[aa[0],aa[1],bb[0],bb[1],cc[0],cc[1]];
        if best.map(|x|candidate<x).unwrap_or(true){best=Some(candidate);}
    }
    Ok(ThreeWayKey(best.expect("24 permutations")))
}

fn splitmix64(state:&mut u64)->u64{
    *state=state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z=*state;
    z=(z^(z>>30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z=(z^(z>>27)).wrapping_mul(0x94D049BB133111EB);
    z^(z>>31)
}

fn key_seed(key:ThreeWayKey,user_seed:u64)->u64{
    let mut x=user_seed^0xC6A4_A793_5BD1_E995;
    for card in key.0{x=x.rotate_left(9)^(card as u64+0x517C_C1B7);}
    x
}

fn sample_board(state:&mut u64,dead:[Card;6])->[Card;5]{
    let mut deck=[0u8;DECK_SIZE]; let mut n=0usize;
    for card in 0..DECK_SIZE as Card{if !dead.contains(&card){deck[n]=card;n+=1;}}
    debug_assert_eq!(n,46);
    for i in 0..5{let j=i+(splitmix64(state) as usize%(n-i));deck.swap(i,j);}
    [deck[0],deck[1],deck[2],deck[3],deck[4]]
}

pub fn sampled_threeway_equity(
    first:Combo,second:Combo,third:Combo,samples:u64,seed:u64
)->Result<ThreeWayEquityEstimate,String>{
    if samples==0{return Err("samples must be positive".into());}
    let key=canonical_threeway(first,second,third)?;
    let hands=[[key.0[0],key.0[1]],[key.0[2],key.0[3]],[key.0[4],key.0[5]]];
    let mut state=key_seed(key,seed);
    let mut shares=[0.0f64;3];
    for _ in 0..samples{
        let board=sample_board(&mut state,key.0);
        let scores=[
            evaluate_seven([hands[0][0],hands[0][1],board[0],board[1],board[2],board[3],board[4]]),
            evaluate_seven([hands[1][0],hands[1][1],board[0],board[1],board[2],board[3],board[4]]),
            evaluate_seven([hands[2][0],hands[2][1],board[0],board[1],board[2],board[3],board[4]]),
        ];
        let best=*scores.iter().max().unwrap();
        let winners=[scores[0]==best,scores[1]==best,scores[2]==best];
        let n=winners.iter().filter(|x|**x).count() as f64;
        for i in 0..3{if winners[i]{shares[i]+=1.0/n;}}
    }
    let denom=samples as f64;
    let equities=[shares[0]/denom,shares[1]/denom,shares[2]/denom];
    Ok(ThreeWayEquityEstimate{equities,samples,seed})
}

#[cfg(test)]
mod tests{
    use super::*;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn suit_isomorphic_threeway_has_same_key(){
        let a=canonical_threeway(
            [c(12,0),c(11,0)],[c(10,1),c(10,2)],[c(9,1),c(8,1)]
        ).unwrap();
        let b=canonical_threeway(
            [c(12,3),c(11,3)],[c(10,0),c(10,1)],[c(9,0),c(8,0)]
        ).unwrap();
        assert_eq!(a,b);
    }

    #[test]
    fn sampled_threeway_is_reproducible_and_sums_to_one(){
        let args=([c(12,0),c(12,1)],[c(11,2),c(11,3)],[c(10,0),c(10,1)]);
        let a=sampled_threeway_equity(args.0,args.1,args.2,1000,55).unwrap();
        let b=sampled_threeway_equity(args.0,args.1,args.2,1000,55).unwrap();
        assert_eq!(a,b);
        assert!((a.equities.iter().sum::<f64>()-1.0).abs()<1e-12);
    }

    #[test]
    fn private_overlap_is_rejected(){
        let err=sampled_threeway_equity([c(12,0),c(11,0)],[c(12,0),c(10,1)],[c(9,2),c(8,3)],10,1);
        assert!(err.is_err());
    }
}
