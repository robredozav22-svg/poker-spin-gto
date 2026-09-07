import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const MANIFEST = path.join(ROOT, 'data', 'benchmarks', 'spin15-wta-v1.json');
const VALID_HERO = new Set(['BTN','SB','BB']);
const VALID_TREE = new Set(['VERIFIED_EXACT','CROSS_CHECKED','SCREEN_REFERENCE_ONLY','PARTIAL_REFERENCE','MISSING_EXACT']);
const VALID_STRATEGY = new Set(['VERIFIED_EXACT','CROSS_CHECKED','PARTIAL','MISSING_EXACT']);

function fail(msg){ throw new Error(`benchmark manifest invalid: ${msg}`); }
function nonEmpty(value,name){ if(typeof value!=='string'||!value.trim()) fail(`${name} must be non-empty`); }

if(!fs.existsSync(MANIFEST)) fail('spin15-wta-v1.json missing');
const m = JSON.parse(fs.readFileSync(MANIFEST,'utf8'));

nonEmpty(m.benchmark_id,'benchmark_id');
if(m.benchmark_id!=='spin15-wta-v1') fail('unexpected benchmark_id');
if(m.status!=='RESEARCH_ONLY') fail('15bb benchmark must remain RESEARCH_ONLY until explicit promotion');
if(m.format!=='spin3max') fail('format must be spin3max');
if(m.payout_profile!=='WTA_CHIPEV') fail('payout_profile must be WTA_CHIPEV');
if(m.stack_profile?.btn_bb!==15||m.stack_profile?.sb_bb!==15||m.stack_profile?.bb_bb!==15) fail('benchmark stack vector must be 15/15/15');
if(m.stack_profile?.symmetric!==true) fail('benchmark must declare symmetric stack vector');
if(m.accuracy_policy_id!=='spin-verified-exact-v1') fail('strict accuracy policy must be spin-verified-exact-v1');

const req=m.promotion_requirements ?? {};
if(req.complete_tree_catalog!==true) fail('complete_tree_catalog must be required');
if(req.physical_combo_rows!==1326) fail('exact promotion must require 1326 physical combos');
if(req.postflop_continuation_artifacts_for_nonallin_paths!==true) fail('postflop continuation artifacts must be required');
if(req.solver_run_evidence!==true) fail('solver_run_evidence must be required');
if(req.deterministic_repeat!==true) fail('deterministic_repeat must be required');
if(req.independent_accuracy_measurement!==true) fail('independent_accuracy_measurement must be required');
if(typeof req.max_normalized_exploitability_fraction_of_pot_where_measurable!=='number') fail('accuracy ceiling missing');
if(req.max_normalized_exploitability_fraction_of_pot_where_measurable>0.0001) fail('accuracy ceiling may not be weaker than 0.01% pot');

const ref=m.primary_reference ?? {};
if(ref.provider!=='GTO_WIZARD') fail('primary reference provider must be GTO_WIZARD for this benchmark');
if(ref.exact_tree_locked===true){
  nonEmpty(m.target_tree_profile_id,'target_tree_profile_id');
  nonEmpty(ref.reference_artifact_id,'primary_reference.reference_artifact_id');
  nonEmpty(ref.reference_checksum,'primary_reference.reference_checksum');
  if(m.target_tree_status!=='VERIFIED_EXACT_TREE') fail('locked exact tree requires VERIFIED_EXACT_TREE target status');
}else{
  if(m.target_tree_profile_id!==null) fail('unlocked tree must not claim target_tree_profile_id');
  if(m.target_tree_status==='VERIFIED_EXACT_TREE') fail('unlocked reference cannot have VERIFIED_EXACT_TREE status');
}

if(!Array.isArray(m.required_node_families)||m.required_node_families.length<10) fail('required_node_families incomplete');
const ids=new Set();
for(const [i,n] of m.required_node_families.entries()){
  nonEmpty(n.id,`required_node_families[${i}].id`);
  if(ids.has(n.id)) fail(`duplicate node family id ${n.id}`); ids.add(n.id);
  if(!VALID_HERO.has(n.hero)) fail(`${n.id}: invalid hero`);
  if(!Array.isArray(n.history_family)) fail(`${n.id}: history_family must be array`);
  if(!VALID_TREE.has(n.tree_status)) fail(`${n.id}: invalid tree_status ${n.tree_status}`);
  if(!VALID_STRATEGY.has(n.strategy_status)) fail(`${n.id}: invalid strategy_status ${n.strategy_status}`);
  if(n.tree_status==='VERIFIED_EXACT' && ref.exact_tree_locked!==true) fail(`${n.id}: exact tree node forbidden before exact reference tree lock`);
  if(n.strategy_status==='VERIFIED_EXACT'){
    if(n.tree_status!=='VERIFIED_EXACT') fail(`${n.id}: exact strategy requires exact tree node`);
    if(ref.exact_tree_locked!==true) fail(`${n.id}: exact strategy forbidden before exact reference tree lock`);
  }
}

const requiredIds=[
  'BTN_FIRST_IN','SB_FIRST_IN_AFTER_BTN_FOLD','SB_VS_BTN_RAISE','SB_VS_BTN_JAM',
  'BB_VS_BTN_RAISE_AFTER_SB_FOLD','BB_VS_BTN_RAISE_AFTER_SB_CALL','BB_VS_BTN_JAM',
  'BB_VS_SB_RAISE','BB_VS_SB_LIMP','BB_VS_SB_JAM',
  'BTN_RESPONSE_TO_SB_3BET_OR_JAM','BTN_RESPONSE_TO_BB_3BET_OR_JAM'
];
for(const id of requiredIds) if(!ids.has(id)) fail(`mandatory benchmark family missing: ${id}`);

if(m.claim_status==='ALLOWED'){
  if(ref.exact_tree_locked!==true) fail('claim cannot be ALLOWED without exact tree lock');
  if(!Array.isArray(m.claim_blockers)||m.claim_blockers.length!==0) fail('claim cannot be ALLOWED while blockers remain');
  if(m.independent_validation?.status!=='PASSED') fail('claim requires PASSED independent validation');
  for(const n of m.required_node_families){
    if(n.tree_status!=='VERIFIED_EXACT') fail(`claim requires exact tree node: ${n.id}`);
    if(n.strategy_status!=='VERIFIED_EXACT') fail(`claim requires exact strategy node: ${n.id}`);
  }
}else{
  if(m.claim_status!=='NOT_ALLOWED') fail('claim_status must be NOT_ALLOWED or ALLOWED');
  if(!Array.isArray(m.claim_blockers)||m.claim_blockers.length===0) fail('NOT_ALLOWED benchmark must explain blockers');
}

console.log(`benchmark manifest OK: ${m.benchmark_id}; nodes=${m.required_node_families.length}; claim=${m.claim_status}`);
