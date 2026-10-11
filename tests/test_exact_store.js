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

  const checks={live_2026_source:'PASS',exact_node_schema:'PASS',upi_action_conservation:'PASS',zero_reach_guard:'PASS',suit_symmetry:'PASS',aggregate_recompute:'PASS',canonical_route:'PASS',artifact_sha256:'PASS',source_usage_compliance:'PASS',independent_crosscheck:'PASS_COMPATIBLE',unexplained_numeric_disagreement:false};
  const external={
    status:'PROMOTED_2026_EXACT',artifact_id:entry.artifact_id,path:entry.path,sha256:entry.sha256,captured_at:entry.captured_at,checks,
    crosscheck:{type:'COMPATIBLE_EXTERNAL_SOURCE',provider:'INDEPENDENT_TEST',compatibility_resolved:true,same_tree_profile:true,source_usage_compliant:true,classification:'SOLVER_TOLERANCE',captured_at:'2026-09-08T00:00:00Z'}
  };
  assert.strictEqual(Store.validatePromotionProof('key',entry,external),external);
  assert.throws(()=>Store.validatePromotionProof('key',entry,null),/missing promotion proof/);
  assert.throws(()=>Store.validatePromotionProof('key',entry,{...external,status:'PENDING'}),/unpromoted/);
  const badCross=structuredClone(external);badCross.checks.independent_crosscheck='PROFILE_UNRESOLVED';assert.throws(()=>Store.validatePromotionProof('key',entry,badCross),/cross-check not passed/);
  const badClass=structuredClone(external);badClass.crosscheck.classification='UNEXPLAINED_NUMERIC_DISAGREEMENT';assert.throws(()=>Store.validatePromotionProof('key',entry,badClass),/classification not promotable/);
  const noncompliant=structuredClone(external);noncompliant.crosscheck.source_usage_compliant=false;assert.throws(()=>Store.validatePromotionProof('key',entry,noncompliant),/source usage not compliant/);
  const badSha=structuredClone(external);badSha.sha256='sha256:'+'0'.repeat(64);assert.throws(()=>Store.validatePromotionProof('key',entry,badSha),/provenance mismatch/);

  const internal={...external,crosscheck:{type:'INDEPENDENT_INTERNAL_SOLVER_PROOF',same_tree_profile:true,independent_evaluator:true,independent_best_response:true,precommitted_threshold_pass:true,evidence_path:'data/solver-evidence/proof.json',evidence_sha256:'sha256:'+'1'.repeat(64),classification:'SOLVER_TOLERANCE'}};
  assert.strictEqual(Store.validatePromotionProof('key',entry,internal),internal);
  const weak=structuredClone(internal);weak.crosscheck.independent_best_response=false;assert.throws(()=>Store.validatePromotionProof('key',entry,weak),/not independently evaluated/);
  const unknown=structuredClone(external);unknown.crosscheck.type='MAGIC_SOURCE';assert.throws(()=>Store.validatePromotionProof('key',entry,unknown),/unsupported promotion crosscheck type/);

  console.log('Exact runtime store integrity + strict promotion proof tests passed');
})().catch(err=>{console.error(err);process.exit(1);});
