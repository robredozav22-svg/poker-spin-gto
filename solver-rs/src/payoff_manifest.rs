use crate::payoff_table::{HuPayoffTable,ThreeWayPayoffTable,EVALUATOR_SCHEMA_VERSION,PAYOFF_TABLE_VERSION};

const MANIFEST_MAGIC:&str="SPNMAN01";

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum PayoffArtifactKind{Hu,ThreeWay}

impl PayoffArtifactKind{
    fn as_str(self)->&'static str{match self{Self::Hu=>"HU",Self::ThreeWay=>"THREEWAY"}}
    fn parse(s:&str)->Result<Self,String>{match s{"HU"=>Ok(Self::Hu),"THREEWAY"=>Ok(Self::ThreeWay),_=>Err(format!("unknown payoff artifact kind {s}"))}}
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PayoffArtifactManifest{
    pub table_schema_version:u16,
    pub evaluator_schema_version:u16,
    pub kind:PayoffArtifactKind,
    pub record_count:usize,
    pub payload_bytes:usize,
    pub checksum_fnv1a64:u64,
    pub provenance:String,
    pub generated_at:String,
}

pub fn fnv1a64(data:&[u8])->u64{
    let mut hash=0xcbf29ce484222325u64;
    for b in data{hash^=*b as u64;hash=hash.wrapping_mul(0x100000001b3);}
    hash
}

fn validate_text_field(name:&str,value:&str)->Result<(),String>{
    if value.is_empty(){return Err(format!("manifest {name} must not be empty"));}
    if value.contains('\n')||value.contains('\r'){return Err(format!("manifest {name} contains newline"));}
    Ok(())
}

impl PayoffArtifactManifest{
    pub fn for_hu(table:&HuPayoffTable,payload:&[u8],provenance:&str,generated_at:&str)->Result<Self,String>{
        HuPayoffTable::decode(payload)?;
        validate_text_field("provenance",provenance)?;
        validate_text_field("generated_at",generated_at)?;
        Ok(Self{
            table_schema_version:PAYOFF_TABLE_VERSION,
            evaluator_schema_version:EVALUATOR_SCHEMA_VERSION,
            kind:PayoffArtifactKind::Hu,
            record_count:table.records.len(),
            payload_bytes:payload.len(),
            checksum_fnv1a64:fnv1a64(payload),
            provenance:provenance.to_string(),
            generated_at:generated_at.to_string(),
        })
    }

    pub fn for_threeway(table:&ThreeWayPayoffTable,payload:&[u8],provenance:&str,generated_at:&str)->Result<Self,String>{
        ThreeWayPayoffTable::decode(payload)?;
        validate_text_field("provenance",provenance)?;
        validate_text_field("generated_at",generated_at)?;
        Ok(Self{
            table_schema_version:PAYOFF_TABLE_VERSION,
            evaluator_schema_version:EVALUATOR_SCHEMA_VERSION,
            kind:PayoffArtifactKind::ThreeWay,
            record_count:table.records.len(),
            payload_bytes:payload.len(),
            checksum_fnv1a64:fnv1a64(payload),
            provenance:provenance.to_string(),
            generated_at:generated_at.to_string(),
        })
    }

    pub fn encode_text(&self)->Result<String,String>{
        validate_text_field("provenance",&self.provenance)?;
        validate_text_field("generated_at",&self.generated_at)?;
        Ok(format!(
            "{MANIFEST_MAGIC}\ntable_schema_version={}\nevaluator_schema_version={}\nkind={}\nrecord_count={}\npayload_bytes={}\nchecksum_fnv1a64={:016x}\nprovenance={}\ngenerated_at={}\n",
            self.table_schema_version,self.evaluator_schema_version,self.kind.as_str(),self.record_count,self.payload_bytes,self.checksum_fnv1a64,self.provenance,self.generated_at
        ))
    }

    pub fn decode_text(text:&str)->Result<Self,String>{
        let mut lines=text.lines();
        if lines.next()!=Some(MANIFEST_MAGIC){return Err("invalid payoff manifest magic".into());}
        let mut table_schema=None;let mut evaluator_schema=None;let mut kind=None;let mut record_count=None;
        let mut payload_bytes=None;let mut checksum=None;let mut provenance=None;let mut generated_at=None;
        for line in lines{
            if line.is_empty(){continue;}
            let (k,v)=line.split_once('=').ok_or_else(||format!("invalid manifest line: {line}"))?;
            match k{
                "table_schema_version"=>table_schema=Some(v.parse::<u16>().map_err(|_|"invalid table schema version")?),
                "evaluator_schema_version"=>evaluator_schema=Some(v.parse::<u16>().map_err(|_|"invalid evaluator schema version")?),
                "kind"=>kind=Some(PayoffArtifactKind::parse(v)?),
                "record_count"=>record_count=Some(v.parse::<usize>().map_err(|_|"invalid record count")?),
                "payload_bytes"=>payload_bytes=Some(v.parse::<usize>().map_err(|_|"invalid payload byte count")?),
                "checksum_fnv1a64"=>checksum=Some(u64::from_str_radix(v,16).map_err(|_|"invalid FNV checksum")?),
                "provenance"=>provenance=Some(v.to_string()),
                "generated_at"=>generated_at=Some(v.to_string()),
                _=>return Err(format!("unknown manifest field {k}")),
            }
        }
        let out=Self{
            table_schema_version:table_schema.ok_or("missing table_schema_version")?,
            evaluator_schema_version:evaluator_schema.ok_or("missing evaluator_schema_version")?,
            kind:kind.ok_or("missing kind")?,
            record_count:record_count.ok_or("missing record_count")?,
            payload_bytes:payload_bytes.ok_or("missing payload_bytes")?,
            checksum_fnv1a64:checksum.ok_or("missing checksum_fnv1a64")?,
            provenance:provenance.ok_or("missing provenance")?,
            generated_at:generated_at.ok_or("missing generated_at")?,
        };
        validate_text_field("provenance",&out.provenance)?;
        validate_text_field("generated_at",&out.generated_at)?;
        if out.table_schema_version!=PAYOFF_TABLE_VERSION{return Err("manifest table schema mismatch".into());}
        if out.evaluator_schema_version!=EVALUATOR_SCHEMA_VERSION{return Err("manifest evaluator schema mismatch".into());}
        Ok(out)
    }

    pub fn verify_hu_payload(&self,payload:&[u8])->Result<HuPayoffTable,String>{
        if self.kind!=PayoffArtifactKind::Hu{return Err("manifest kind is not HU".into());}
        self.verify_common(payload)?;
        let table=HuPayoffTable::decode(payload)?;
        if table.records.len()!=self.record_count{return Err("manifest HU record count mismatch".into());}
        Ok(table)
    }

    pub fn verify_threeway_payload(&self,payload:&[u8])->Result<ThreeWayPayoffTable,String>{
        if self.kind!=PayoffArtifactKind::ThreeWay{return Err("manifest kind is not THREEWAY".into());}
        self.verify_common(payload)?;
        let table=ThreeWayPayoffTable::decode(payload)?;
        if table.records.len()!=self.record_count{return Err("manifest 3-way record count mismatch".into());}
        Ok(table)
    }

    fn verify_common(&self,payload:&[u8])->Result<(),String>{
        if self.table_schema_version!=PAYOFF_TABLE_VERSION{return Err("manifest table schema mismatch".into());}
        if self.evaluator_schema_version!=EVALUATOR_SCHEMA_VERSION{return Err("manifest evaluator schema mismatch".into());}
        if payload.len()!=self.payload_bytes{return Err("manifest payload length mismatch".into());}
        if fnv1a64(payload)!=self.checksum_fnv1a64{return Err("manifest payload checksum mismatch".into());}
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn fnv_is_deterministic_and_order_sensitive(){
        assert_eq!(fnv1a64(b"abc"),fnv1a64(b"abc"));
        assert_ne!(fnv1a64(b"abc"),fnv1a64(b"acb"));
    }

    #[test]
    fn manifest_text_roundtrip_is_stable(){
        let m=PayoffArtifactManifest{
            table_schema_version:PAYOFF_TABLE_VERSION,
            evaluator_schema_version:EVALUATOR_SCHEMA_VERSION,
            kind:PayoffArtifactKind::Hu,
            record_count:7,payload_bytes:123,checksum_fnv1a64:0x1234,
            provenance:"solver-core@test".into(),generated_at:"2026-09-07T14:55:00+05:00".into(),
        };
        let text=m.encode_text().unwrap();
        assert_eq!(PayoffArtifactManifest::decode_text(&text).unwrap(),m);
        assert_eq!(PayoffArtifactManifest::decode_text(&text).unwrap().encode_text().unwrap(),text);
    }
}
