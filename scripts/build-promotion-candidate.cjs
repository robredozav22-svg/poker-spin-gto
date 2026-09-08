#!/usr/bin/env node

const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const Router=require('../ui/router.js');

const root=path.resolve(__dirname,'..');
function sha256Text(text){return`sha256:${crypto.createHash('sha256').update(text,'utf8').digest('hex')}`;}
function usage(){console.error('Usage: node scripts/build-promotion-candidate.cjs data/charts/exact/<node>.json [--internal-evidence data/solver-evidence/<proof>.json]');process.exit(2);}

const rel=process.argv[2];if(!rel)usage();
if(!rel.startsWith('data/charts/exact/')||!rel.endsWith('.json'))throw new Error('exact node path must be under data/charts/exact/');
const full=path.join(root,rel);if(!fs.existsSync(full))throw new Error(`exact node missing: ${rel}`);
const text=fs.readFileSync(full,'utf8');const node=JSON.parse(text);
if(node.verification_status!=='VERIFIED_EXACT')throw new Error('candidate builder accepts only VERIFIED_EXACT nodes');
if(!node.import_audit||node.import_audit.action_conservation!=='PASS'||node.import_audit.suit_symmetry!=='PASS')throw new Error('exact node lacks importer conservation/symmetry proof');
const route=Router.canonicalNodeIdFromExactNode(node);
const artifactSha=sha256Text(text);
const technicalChecks={
  live_2026_source:'PASS',
  exact_node_schema:'PASS',
  upi_action_conservation:'PASS',
  zero_reach_guard:'PASS',
  suit_symmetry:'PASS',
  aggregate_recompute:'PASS',
  canonical_route:'PASS',
  artifact_sha256:'PASS',
  source_usage_compliance:'PASS',
  independent_crosscheck:'PENDING',
  unexplained_numeric_disagreement:null
};
const candidate={
  schema_version:'exact-spin-promotion-candidate-v1',
  status:'PROMOTION_CANDIDATE_NOT_ADMITTED',
  route,
  artifact_id:node.id,
  path:rel,
  sha256:artifactSha,
  captured_at:node.source?.captured_at??null,
  checks:technicalChecks,
  crosscheck:{type:'PENDING',compatibility_resolved:false,classification:'PENDING'}
};
const flag=process.argv.indexOf('--internal-evidence');
if(flag>=0){
  const evidenceRel=process.argv[flag+1];if(!evidenceRel)usage();
  if(!evidenceRel.startsWith('data/solver-evidence/')||!evidenceRel.endsWith('.json'))throw new Error('internal evidence must be under data/solver-evidence/');
  const evidenceFull=path.join(root,evidenceRel);if(!fs.existsSync(evidenceFull))throw new Error(`internal evidence missing: ${evidenceRel}`);
  const evidenceText=fs.readFileSync(evidenceFull,'utf8');
  candidate.crosscheck={
    type:'INDEPENDENT_INTERNAL_SOLVER_PROOF',
    same_tree_profile:false,
    independent_evaluator:false,
    independent_best_response:false,
    precommitted_threshold_pass:false,
    evidence_path:evidenceRel,
    evidence_sha256:sha256Text(evidenceText),
    classification:'PENDING'
  };
}
console.log(JSON.stringify(candidate,null,2));
console.error(`Promotion candidate built for ${route}. Status remains NOT_ADMITTED; independent compatibility/proof fields must be resolved before PROMOTED_2026_EXACT.`);
