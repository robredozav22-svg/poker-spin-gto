use crate::equity::{HuMatchupKey};
use crate::equity3::ThreeWayKey;
use crate::exact_equity::{ExactEquity,HU_PREFLOP_BOARD_COUNT};
use crate::exact_equity3::{ExactThreeWayEquity,THREEWAY_PREFLOP_BOARD_COUNT};

const MAGIC:[u8;8]=*b"SPNPAY01";
pub const PAYOFF_TABLE_VERSION:u16=1;
pub const EVALUATOR_SCHEMA_VERSION:u16=1;
const KIND_HU:u8=1;
const KIND_THREEWAY:u8=2;
const HEADER_LEN:usize=8+2+2+1+3+8+8;
const HU_RECORD_LEN:usize=4+8*3;
const THREEWAY_RECORD_LEN:usize=6+8*7;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct HuPayoffRecord{
    pub key:HuMatchupKey,
    pub wins:u64,
    pub losses:u64,
    pub ties:u64,
}

impl HuPayoffRecord{
    pub fn equity(self)->ExactEquity{
        let boards=self.wins+self.losses+self.ties;
        let n=boards as f64;
        ExactEquity{
            hero:(self.wins as f64+0.5*self.ties as f64)/n,
            villain:(self.losses as f64+0.5*self.ties as f64)/n,
            wins:self.wins,losses:self.losses,ties:self.ties,boards,
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ThreeWayPayoffRecord{
    pub key:ThreeWayKey,
    pub outright_wins:[u64;3],
    pub two_way_ties:[u64;3],
    pub three_way_ties:u64,
}

impl ThreeWayPayoffRecord{
    pub fn equity(self)->ExactThreeWayEquity{
        let boards=self.outright_wins.iter().sum::<u64>()+self.two_way_ties.iter().sum::<u64>()+self.three_way_ties;
        let mut shares=[0.0f64;3];
        for i in 0..3{shares[i]+=self.outright_wins[i] as f64;}
        shares[0]+=0.5*(self.two_way_ties[0]+self.two_way_ties[1]) as f64;
        shares[1]+=0.5*(self.two_way_ties[0]+self.two_way_ties[2]) as f64;
        shares[2]+=0.5*(self.two_way_ties[1]+self.two_way_ties[2]) as f64;
        for s in &mut shares{*s+=self.three_way_ties as f64/3.0;*s/=boards as f64;}
        ExactThreeWayEquity{equities:shares,outright_wins:self.outright_wins,two_way_ties:self.two_way_ties,three_way_ties:self.three_way_ties,boards}
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct HuPayoffTable{pub records:Vec<HuPayoffRecord>}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ThreeWayPayoffTable{pub records:Vec<ThreeWayPayoffRecord>}

fn push_u16(out:&mut Vec<u8>,v:u16){out.extend_from_slice(&v.to_le_bytes());}
fn push_u64(out:&mut Vec<u8>,v:u64){out.extend_from_slice(&v.to_le_bytes());}
fn take_u16(data:&[u8],p:&mut usize)->Result<u16,String>{if *p+2>data.len(){return Err("truncated u16".into());}let v=u16::from_le_bytes([data[*p],data[*p+1]]);*p+=2;Ok(v)}
fn take_u64(data:&[u8],p:&mut usize)->Result<u64,String>{if *p+8>data.len(){return Err("truncated u64".into());}let mut b=[0u8;8];b.copy_from_slice(&data[*p..*p+8]);*p+=8;Ok(u64::from_le_bytes(b))}

fn write_header(out:&mut Vec<u8>,kind:u8,boards:u64,count:u64){
    out.extend_from_slice(&MAGIC);
    push_u16(out,PAYOFF_TABLE_VERSION);
    push_u16(out,EVALUATOR_SCHEMA_VERSION);
    out.push(kind);out.extend_from_slice(&[0u8;3]);
    push_u64(out,boards);push_u64(out,count);
}

fn read_header(data:&[u8],expected_kind:u8,expected_boards:u64)->Result<(usize,u64),String>{
    if data.len()<HEADER_LEN{return Err("payoff table header truncated".into());}
    if data[..8]!=MAGIC{return Err("invalid payoff table magic".into());}
    let mut p=8usize;
    let version=take_u16(data,&mut p)?;
    let evaluator=take_u16(data,&mut p)?;
    if version!=PAYOFF_TABLE_VERSION{return Err(format!("unsupported payoff table version {version}"));}
    if evaluator!=EVALUATOR_SCHEMA_VERSION{return Err(format!("unsupported evaluator schema {evaluator}"));}
    let kind=data[p];p+=1;
    if kind!=expected_kind{return Err("payoff table kind mismatch".into());}
    p+=3;
    let boards=take_u64(data,&mut p)?;
    if boards!=expected_boards{return Err(format!("board-count provenance mismatch: {boards}"));}
    let count=take_u64(data,&mut p)?;
    Ok((p,count))
}

impl HuPayoffTable{
    pub fn encode(&self)->Result<Vec<u8>,String>{
        let mut out=Vec::with_capacity(HEADER_LEN+self.records.len()*HU_RECORD_LEN);
        write_header(&mut out,KIND_HU,HU_PREFLOP_BOARD_COUNT,self.records.len() as u64);
        for r in &self.records{
            if r.wins+r.losses+r.ties!=HU_PREFLOP_BOARD_COUNT{return Err("HU record board count mismatch".into());}
            out.extend_from_slice(&r.key.0);push_u64(&mut out,r.wins);push_u64(&mut out,r.losses);push_u64(&mut out,r.ties);
        }
        Ok(out)
    }
    pub fn decode(data:&[u8])->Result<Self,String>{
        let (mut p,count)=read_header(data,KIND_HU,HU_PREFLOP_BOARD_COUNT)?;
        let expected=HEADER_LEN+(count as usize).checked_mul(HU_RECORD_LEN).ok_or("HU table size overflow")?;
        if data.len()!=expected{return Err(format!("HU table length mismatch: got {}, expected {expected}",data.len()));}
        let mut records=Vec::with_capacity(count as usize);
        for _ in 0..count{
            let mut key=[0u8;4];key.copy_from_slice(&data[p..p+4]);p+=4;
            let wins=take_u64(data,&mut p)?;let losses=take_u64(data,&mut p)?;let ties=take_u64(data,&mut p)?;
            if wins+losses+ties!=HU_PREFLOP_BOARD_COUNT{return Err("HU decoded record board count mismatch".into());}
            records.push(HuPayoffRecord{key:HuMatchupKey(key),wins,losses,ties});
        }
        Ok(Self{records})
    }
}

impl ThreeWayPayoffTable{
    pub fn encode(&self)->Result<Vec<u8>,String>{
        let mut out=Vec::with_capacity(HEADER_LEN+self.records.len()*THREEWAY_RECORD_LEN);
        write_header(&mut out,KIND_THREEWAY,THREEWAY_PREFLOP_BOARD_COUNT,self.records.len() as u64);
        for r in &self.records{
            let boards=r.outright_wins.iter().sum::<u64>()+r.two_way_ties.iter().sum::<u64>()+r.three_way_ties;
            if boards!=THREEWAY_PREFLOP_BOARD_COUNT{return Err("3-way record board count mismatch".into());}
            out.extend_from_slice(&r.key.0);
            for v in r.outright_wins{push_u64(&mut out,v);}for v in r.two_way_ties{push_u64(&mut out,v);}push_u64(&mut out,r.three_way_ties);
        }
        Ok(out)
    }
    pub fn decode(data:&[u8])->Result<Self,String>{
        let (mut p,count)=read_header(data,KIND_THREEWAY,THREEWAY_PREFLOP_BOARD_COUNT)?;
        let expected=HEADER_LEN+(count as usize).checked_mul(THREEWAY_RECORD_LEN).ok_or("3-way table size overflow")?;
        if data.len()!=expected{return Err(format!("3-way table length mismatch: got {}, expected {expected}",data.len()));}
        let mut records=Vec::with_capacity(count as usize);
        for _ in 0..count{
            let mut key=[0u8;6];key.copy_from_slice(&data[p..p+6]);p+=6;
            let mut outright=[0u64;3];let mut two=[0u64;3];
            for v in &mut outright{*v=take_u64(data,&mut p)?;}for v in &mut two{*v=take_u64(data,&mut p)?;}let three=take_u64(data,&mut p)?;
            let boards=outright.iter().sum::<u64>()+two.iter().sum::<u64>()+three;
            if boards!=THREEWAY_PREFLOP_BOARD_COUNT{return Err("3-way decoded record board count mismatch".into());}
            records.push(ThreeWayPayoffRecord{key:ThreeWayKey(key),outright_wins:outright,two_way_ties:two,three_way_ties:three});
        }
        Ok(Self{records})
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn hu_roundtrip_preserves_exact_integer_counts(){
        let r=HuPayoffRecord{key:HuMatchupKey([1,2,3,4]),wins:1_000_000,losses:700_000,ties:12_304};
        let t=HuPayoffTable{records:vec![r]};let bytes=t.encode().unwrap();let d=HuPayoffTable::decode(&bytes).unwrap();assert_eq!(d,t);assert_eq!(d.records[0].equity().boards,HU_PREFLOP_BOARD_COUNT);
    }

    #[test]
    fn threeway_roundtrip_preserves_tie_provenance(){
        let r=ThreeWayPayoffRecord{key:ThreeWayKey([1,2,3,4,5,6]),outright_wins:[500_000,400_000,300_000],two_way_ties:[50_000,40_000,30_000],three_way_ties:50_754};
        let t=ThreeWayPayoffTable{records:vec![r]};let bytes=t.encode().unwrap();let d=ThreeWayPayoffTable::decode(&bytes).unwrap();assert_eq!(d,t);assert!((d.records[0].equity().equities.iter().sum::<f64>()-1.0).abs()<1e-12);
    }

    #[test]
    fn corrupted_magic_fails_closed(){
        let t=HuPayoffTable{records:vec![]};let mut bytes=t.encode().unwrap();bytes[0]^=0xff;assert!(HuPayoffTable::decode(&bytes).is_err());
    }
}
