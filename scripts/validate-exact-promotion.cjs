#!/usr/bin/env node

const fs=require('fs');
const path=require('path');

const root=path.resolve(__dirname,'..');
const index=JSON.parse(fs.readFileSync(path.join(root,'data','charts','exact-index.json'),'utf8'));
const manifest=JSON.parse(fs.readFileSync(path.join(root,'data','charts','exact-promotion-manifest.json'),'utf8'));
const errors=[];
function fail(x){errors.push(x);}

function validateExternal(indexKey,cross,bad){
  if(typeof cross.provider!=='string'||!cross.provider.trim())bad(`${indexKey}: external crosscheck.provider missing`);
  if(cross.compatibility_resolved!==true)bad(`${indexKey}: external crosscheck compatibility must be resolved`);
  if(cross.same_tree_profile!==true)bad(`${indexKey}: external crosscheck must prove same_tree_profile`);
  if(cross.source_usage_compliant!==true)bad(`${indexKey}: external crosscheck must be source-usage compliant`);
  if(!['ROUNDING_ONLY','SOLVER_TOLERANCE','NONE'].includes(cross.classification))bad(`${indexKey}: external crosscheck classification not promotable: ${cross.classification}`);
  if(typeof cross.captured_at!=='string'||!cross.captured_at.trim())bad(`${indexKey}: external crosscheck captured_at missing`);
}

function validateInternal(indexKey,cross,bad){
  if(cross.same_tree_profile!==true)bad(`${indexKey}: internal solver proof must prove same_tree_profile`);
  if(cross.independent_evaluator!==true)bad(`${indexKey}: internal solver proof requires independent_evaluator`);
  if(cross.independent_best_response!==true)bad(`${indexKey}: internal solver proof requires independent_best_response`);
  if(cross.precommitted_threshold_pass!==true)bad(`${indexKey}: internal solver proof requires precommitted threshold PASS`);
  if(typeof cross.evidence_path!=='string'||!cross.evidence_path.startsWith('data/solver-evidence/')||!cross.evidence_path.endsWith('.json'))bad(`${indexKey}: internal solver evidence_path invalid`);
  else if(!fs.existsSync(path.join(root,cross.evidence_path)))bad(`${indexKey}: internal solver evidence file missing ${cross.evidence_path}`);
  if(typeof cross.evidence_sha256!=='string'||!/^sha256:[0-9a-f]{64}$/.test(cross.evidence_sha256))bad(`${indexKey}: internal solver evidence_sha256 invalid`);
  if(!['SOLVER_TOLERANCE','NONE'].includes(cross.classification))bad(`${indexKey}: internal solver classification not promotable: ${cross.classification}`);
}

function validatePromotion(indexKey,indexEntry,p){
  const out=[];const bad=x=>out.push(x);
  if(!p||typeof p!=='object')return [`${indexKey}: promotion proof missing`];
  if(p.status!=='PROMOTED_2026_EXACT')bad(`${indexKey}: promotion status must be PROMOTED_2026_EXACT`);
  if(p.artifact_id!==indexEntry.artifact_id)bad(`${indexKey}: promotion artifact_id mismatch`);
  if(p.path!==indexEntry.path)bad(`${indexKey}: promotion path mismatch`);
  if(p.sha256!==indexEntry.sha256)bad(`${indexKey}: promotion sha256 mismatch`);
  if(p.captured_at!==indexEntry.captured_at)bad(`${indexKey}: promotion captured_at mismatch`);
  const checks=p.checks;
  if(!checks||typeof checks!=='object')bad(`${indexKey}: promotion checks missing`);else{
    for(const k of ['live_2026_source','exact_node_schema','upi_action_conservation','zero_reach_guard','suit_symmetry','aggregate_recompute','canonical_route','artifact_sha256','source_usage_compliance'])if(checks[k]!=='PASS')bad(`${indexKey}: ${k} must be PASS`);
    if(checks.independent_crosscheck!=='PASS_COMPATIBLE')bad(`${indexKey}: independent_crosscheck must be PASS_COMPATIBLE`);
    if(checks.unexplained_numeric_disagreement!==false)bad(`${indexKey}: unexplained_numeric_disagreement must be false`);
  }
  const cross=p.crosscheck;
  if(!cross||typeof cross!=='object')bad(`${indexKey}: crosscheck evidence missing`);else{
    if(cross.type==='COMPATIBLE_EXTERNAL_SOURCE')validateExternal(indexKey,cross,bad);
    else if(cross.type==='INDEPENDENT_INTERNAL_SOLVER_PROOF')validateInternal(indexKey,cross,bad);
    else bad(`${indexKey}: unsupported crosscheck.type ${cross.type}`);
  }
  return out;
}

function baseGood(idx){return{status:'PROMOTED_2026_EXACT',artifact_id:'a',path:idx.path,sha256:idx.sha256,captured_at:idx.captured_at,checks:{live_2026_source:'PASS',exact_node_schema:'PASS',upi_action_conservation:'PASS',zero_reach_guard:'PASS',suit_symmetry:'PASS',aggregate_recompute:'PASS',canonical_route:'PASS',artifact_sha256:'PASS',source_usage_compliance:'PASS',independent_crosscheck:'PASS_COMPATIBLE',unexplained_numeric_disagreement:false},crosscheck:{type:'COMPATIBLE_EXTERNAL_SOURCE',provider:'INDEPENDENT_TEST',compatibility_resolved:true,same_tree_profile:true,source_usage_compliant:true,classification:'SOLVER_TOLERANCE',captured_at:'2026-09-08T00:00:00Z'}};}

function selfTest(){
  const idx={artifact_id:'a',path:'data/charts/exact/a.json',sha256:'sha256:'+'1'.repeat(64),captured_at:'2026-09-08T00:00:00Z'};
  const good=baseGood(idx);
  if(validatePromotion('k',idx,good).length)throw new Error('valid external promotion proof rejected');
  const noncompliant=structuredClone(good);noncompliant.crosscheck.source_usage_compliant=false;if(!validatePromotion('k',idx,noncompliant).some(x=>x.includes('source-usage compliant')))throw new Error('noncompliant external source not rejected');
  const noCross=structuredClone(good);noCross.checks.independent_crosscheck='PROFILE_UNRESOLVED';if(!validatePromotion('k',idx,noCross).some(x=>x.includes('independent_crosscheck')))throw new Error('unresolved crosscheck not rejected');
  const fatal=structuredClone(good);fatal.checks.unexplained_numeric_disagreement=true;fatal.crosscheck.classification='UNEXPLAINED_NUMERIC_DISAGREEMENT';if(validatePromotion('k',idx,fatal).length<2)throw new Error('fatal disagreement not rejected');
  const wrongSha=structuredClone(good);wrongSha.sha256='sha256:'+'2'.repeat(64);if(!validatePromotion('k',idx,wrongSha).some(x=>x.includes('sha256 mismatch')))throw new Error('sha mismatch not rejected');
  const internal=baseGood(idx);internal.crosscheck={type:'INDEPENDENT_INTERNAL_SOLVER_PROOF',same_tree_profile:true,independent_evaluator:true,independent_best_response:true,precommitted_threshold_pass:true,evidence_path:'data/solver-evidence/turn-river-research-fixture-v1.json',evidence_sha256:'sha256:'+'3'.repeat(64),classification:'SOLVER_TOLERANCE'};
  if(validatePromotion('k',idx,internal).length)throw new Error('valid internal proof rejected');
  const weak=structuredClone(internal);weak.crosscheck.independent_best_response=false;if(!validatePromotion('k',idx,weak).some(x=>x.includes('independent_best_response')))throw new Error('weak internal proof not rejected');
  console.log('Exact promotion validator self-test PASS: compliant external and independent internal proof types enforced; weak/noncompliant evidence fails closed.');
}

function main(){
  if(manifest.schema_version!=='exact-spin-promotion-manifest-v1')fail('promotion manifest schema_version mismatch');
  if(!manifest.nodes||typeof manifest.nodes!=='object'||Array.isArray(manifest.nodes))fail('promotion manifest nodes must be object');
  const indexedArtifacts=new Set();
  for(const [key,entry] of Object.entries(index.nodes??{})){
    indexedArtifacts.add(entry.artifact_id);
    const p=manifest.nodes?.[entry.artifact_id];
    for(const e of validatePromotion(key,entry,p))fail(e);
  }
  for(const [artifact,p] of Object.entries(manifest.nodes??{})){
    if(!indexedArtifacts.has(artifact))fail(`${artifact}: promotion proof is orphaned; promoted artifacts must be runtime-indexed`);
    if(p?.artifact_id!==artifact)fail(`${artifact}: manifest key/artifact_id mismatch`);
  }
  if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact promotion validation FAILED: ${errors.length} error(s).`);process.exit(1);}
  console.log(`Exact promotion validation PASS: ${Object.keys(index.nodes??{}).length} runtime node(s), every admission has compliant independent evidence.`);
}

if(process.argv.includes('--self-test'))selfTest();else main();
