const assert=require('assert');
const Store=require('../ui/exact-store.js');

(async()=>{
  const hash=await Store.sha256Text('abc');
  assert.strictEqual(hash,'sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  const entry={path:'data/charts/exact/node.json',artifact_id:'node-v1',sha256:hash,captured_at:'2026-09-08T00:00:00Z'};
  assert.strictEqual(Store.validateIndexEntry('key',entry),entry);
  assert.throws(()=>Store.validateIndexEntry('key',{...entry,sha256:'sha256:bad'}),/invalid exact sha256/);
  assert.throws(()=>Store.validateIndexEntry('key',{...entry,path:'data/reference.json'}),/invalid exact node path/);
  assert.throws(()=>Store.validateIndexEntry('key',{...entry,artifact_id:''}),/missing exact artifact_id/);
  assert.throws(()=>Store.validateIndexEntry('key',{...entry,captured_at:''}),/missing exact captured_at/);

  const proof={
    status:'PROMOTED_2026_EXACT',artifact_id:entry.artifact_id,path:entry.path,sha256:entry.sha256,captured_at:entry.captured_at,
    checks:{live_2026_source:'PASS',exact_node_schema:'PASS',upi_action_conservation:'PASS',zero_reach_guard:'PASS',suit_symmetry:'PASS',aggregate_recompute:'PASS',canonical_route:'PASS',artifact_sha256:'PASS',independent_crosscheck:'PASS_COMPATIBLE',unexplained_numeric_disagreement:false},
    crosscheck:{provider:'INDEPENDENT_TEST',compatibility_resolved:true,classification:'SOLVER_TOLERANCE',captured_at:'2026-09-08T00:00:00Z'}
  };
  assert.strictEqual(Store.validatePromotionProof('key',entry,proof),proof);
  assert.throws(()=>Store.validatePromotionProof('key',entry,null),/missing promotion proof/);
  assert.throws(()=>Store.validatePromotionProof('key',entry,{...proof,status:'PENDING'}),/unpromoted/);
  const badCross=structuredClone(proof);badCross.checks.independent_crosscheck='PROFILE_UNRESOLVED';
  assert.throws(()=>Store.validatePromotionProof('key',entry,badCross),/cross-check not passed/);
  const badClass=structuredClone(proof);badClass.crosscheck.classification='UNEXPLAINED_NUMERIC_DISAGREEMENT';
  assert.throws(()=>Store.validatePromotionProof('key',entry,badClass),/classification not promotable/);
  const badSha=structuredClone(proof);badSha.sha256='sha256:'+'0'.repeat(64);
  assert.throws(()=>Store.validatePromotionProof('key',entry,badSha),/provenance mismatch/);
  console.log('Exact runtime store integrity + promotion tests passed');
})().catch(err=>{console.error(err);process.exit(1);});
