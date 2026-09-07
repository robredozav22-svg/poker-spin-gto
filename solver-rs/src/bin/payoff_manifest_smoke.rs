use spins_solver_core::cards::Card;
use spins_solver_core::equity::canonical_hu_matchup;
use spins_solver_core::exact_equity::exact_hu_equity;
use spins_solver_core::payoff_manifest::PayoffArtifactManifest;
use spins_solver_core::payoff_table::{HuPayoffRecord,HuPayoffTable};

fn c(r:u8,s:u8)->Card{r*4+s}

fn main(){
    let aa=[c(12,0),c(12,1)];
    let kk=[c(11,2),c(11,3)];
    let direct=exact_hu_equity(aa,kk).expect("exact HU");
    let key=canonical_hu_matchup(aa,kk).unwrap();
    let table=HuPayoffTable{records:vec![HuPayoffRecord{
        key,wins:direct.wins,losses:direct.losses,ties:direct.ties,
    }]};
    let payload=table.encode().unwrap();
    let manifest=PayoffArtifactManifest::for_hu(
        &table,&payload,
        "solver-core@payoff-manifest-smoke",
        "2026-09-07T14:55:00+05:00",
    ).unwrap();
    let text=manifest.encode_text().unwrap();
    let decoded=PayoffArtifactManifest::decode_text(&text).unwrap();
    let verified=decoded.verify_hu_payload(&payload).expect("verified payload");
    assert_eq!(verified.records,table.records);

    let mut corrupted=payload.clone();
    let last=corrupted.len()-1;
    corrupted[last]^=0x01;
    assert!(decoded.verify_hu_payload(&corrupted).is_err());

    println!(
        "status=RESEARCH_ONLY mode=PAYOFF_MANIFEST_INTEGRITY kind=HU records={} payload_bytes={} checksum_fnv1a64={:016x} manifest_bytes={} corruption_check=REJECT provenance={} generated_at={}",
        manifest.record_count,manifest.payload_bytes,manifest.checksum_fnv1a64,text.len(),manifest.provenance,manifest.generated_at
    );
}
