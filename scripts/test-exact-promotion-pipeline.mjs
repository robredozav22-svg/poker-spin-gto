#!/usr/bin/env node

import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {importCapture} from './import-gtowizard-upi-node.mjs';

const require=createRequire(import.meta.url);
const Router=require('../ui/router.js');
const Store=require('../ui/exact-store.js');
const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
function classes169(){const out=[];for(let i=0;i<13;i++)for(let j=0;j<13;j++)out.push(i===j?RANKS[i]+RANKS[j]:(i<j?RANKS[i]+RANKS[j]+'s':RANKS[j]+RANKS[i]+'o'));return out;}
const hands=classes169();if(hands.length!==169||new Set(hands).size!==169)throw new Error('integration hand universe invalid');
const weighted=p=>hands.map(h=>`${h}: ${p}`).join(',');
const capture={schema_version:'gtowizard-upi-capture-v1',id:'integration-15bb-btn-root-v1',format:'SPIN_3MAX',payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',effective_stack_bb:15,hero_position:'BTN',history:[],source:{provider:'GTO Wizard',solution_family:'Research',node_id:'INTEGRATION_FIXTURE_ONLY',captured_at:'2026-09-08T08:40:00Z'},full_range_upi:weighted(1),actions:[{id:'FOLD',type:'FOLD',range_upi:weighted(0.6)},{id:'RAISE_TO_2BB',type:'RAISE',to_bb:2,range_upi:weighted(0.3)},{id:'JAM_TO_15BB',type:'JAM',to_bb:15,range_upi:weighted(0.1)}]};
const node=importCapture(capture);
if(node.import_audit.physical_combos_checked!==1326||node.import_audit.hand_classes_derived!==169)throw new Error('1326/169 audit missing');
if(node.import_audit.action_conservation!=='PASS'||node.import_audit.suit_symmetry!=='PASS')throw new Error('import proof missing');
if(Math.abs(node.aggregate.FOLD-0.6)>1e-10||Math.abs(node.aggregate.RAISE_TO_2BB-0.3)>1e-10||Math.abs(node.aggregate.JAM_TO_15BB-0.1)>1e-10)throw new Error('aggregate reconstruction failed');
const route=Router.canonicalNodeIdFromExactNode(node);if(route!=='3MAX/WTA_CHIPEV_BASELINE/NONE/15/ROOT')throw new Error(`unexpected route ${route}`);
const text=JSON.stringify(node,null,2)+'\n';const sha256=`sha256:${crypto.createHash('sha256').update(text,'utf8').digest('hex')}`;
const entry={path:'data/charts/exact/integration-fixture.json',artifact_id:node.id,sha256,captured_at:node.source.captured_at};Store.validateIndexEntry(route,entry);
const proof={status:'PROMOTED_2026_EXACT',artifact_id:entry.artifact_id,path:entry.path,sha256:entry.sha256,captured_at:entry.captured_at,checks:{live_2026_source:'PASS',exact_node_schema:'PASS',upi_action_conservation:'PASS',zero_reach_guard:'PASS',suit_symmetry:'PASS',aggregate_recompute:'PASS',canonical_route:'PASS',artifact_sha256:'PASS',independent_crosscheck:'PASS_COMPATIBLE',unexplained_numeric_disagreement:false},crosscheck:{provider:'INTEGRATION_INDEPENDENT_FIXTURE',compatibility_resolved:true,classification:'ROUNDING_ONLY',captured_at:'2026-09-08T08:40:01Z'}};
Store.validatePromotionProof(route,entry,proof);
if(Router.canonicalNodeIdFromExactNode({...node,ante_profile:'SPIN_PLUS_ANTE_0_2BB'})===route)throw new Error('ante collision not blocked');
if(Router.canonicalNodeIdFromExactNode({...node,payout_profile:'HIGH_MULTIPLIER_ICM'})===route)throw new Error('payout collision not blocked');
let rejected=0;try{Store.validatePromotionProof(route,entry,{...proof,sha256:'sha256:'+'0'.repeat(64)});}catch{rejected++;}
const badCross=structuredClone(proof);badCross.checks.independent_crosscheck='PROFILE_UNRESOLVED';try{Store.validatePromotionProof(route,entry,badCross);}catch{rejected++;}
const fatal=structuredClone(proof);fatal.checks.unexplained_numeric_disagreement=true;fatal.crosscheck.classification='UNEXPLAINED_NUMERIC_DISAGREEMENT';try{Store.validatePromotionProof(route,entry,fatal);}catch{rejected++;}
const zeroCapture=structuredClone(capture);zeroCapture.id='integration-zero-reach';zeroCapture.full_range_upi='AA: 1';zeroCapture.actions=[{id:'FOLD',type:'FOLD',range_upi:'AA: 0.4'},{id:'JAM_TO_15BB',type:'JAM',to_bb:15,range_upi:'AA: 0.6'}];const zero=importCapture(zeroCapture);if(zero.hands.KKs.range_weight!==0||zero.hands.KKs.strategy!==null)throw new Error('zero reach became action');
if(rejected!==3)throw new Error(`expected 3 corrupted promotion proofs rejected, got ${rejected}`);
console.log(`Exact promotion pipeline PASS: live-2026 capture -> 1326 -> 169 -> ${route} -> SHA-256 -> promotion; ante/payout/SHA/crosscheck/zero-reach guards verified.`);
