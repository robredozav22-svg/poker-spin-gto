#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const exactDir=path.join(root,'data','charts','exact');
const crossDir=path.join(root,'data','crosschecks','2026');
function files(d){return !fs.existsSync(d)?[]:fs.readdirSync(d,{withFileTypes:true}).flatMap(e=>{const p=path.join(d,e.name);return e.isDirectory()?files(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
function read(p){return JSON.parse(fs.readFileSync(p,'utf8'));}
function histKey(h){return JSON.stringify((h??[]).map(x=>({actor:x.actor,type:x.type,to_bb:x.to_bb??null})));}
function actionKey(a){return `${a.type}:${a.to_bb??''}`;}
function sameActions(a,b){const x=new Set((a??[]).map(actionKey)),y=new Set((b??[]).map(actionKey));return x.size===y.size&&[...x].every(k=>y.has(k));}
function unresolved(v){return typeof v!=='string'||v.startsWith('UNRESOLVED');}
function tolerancePp(precision){return precision==='DISPLAYED_INTEGER_PERCENT'?0.51:0.15;}

export function auditSpot(ex,profile,ref,hand,spot){
  const out={exact_id:ex.id??null,crosscheck_id:ref.id??null,hand,classification:null,max_abs_diff_pp:null,actions:[]};
  if(ex.format!==profile.format||Number(ex.effective_stack_bb)!==Number(profile.effective_stack_bb)){out.classification='PROFILE_KEY_MISMATCH';return out;}
  if(ex.hero_position!==ref.hero_position||histKey(ex.history)!==histKey(ref.history)){out.classification='ACTION_HISTORY_MISMATCH';return out;}
  if(!sameActions(ex.actions,ref.actions)){out.classification='TREE_SIZING_MISMATCH';return out;}
  const cell=ex.hands?.[hand];if(!cell){out.classification='EXACT_HAND_MISSING';return out;}
  if(Number(cell.range_weight)<=0){out.classification='EXACT_HAND_UNREACHABLE';return out;}
  const strategy=cell.strategy||{};let max=0;
  for(const a of ref.actions){const expected=Number(spot[a.id]??0),actual=Number(strategy[a.id]??0);const d=Math.abs(actual-expected)*100;max=Math.max(max,d);out.actions.push({action:a.id,exact:actual,reference:expected,diff_pp:d});}
  out.max_abs_diff_pp=max;
  const profileResolved=!unresolved(profile.payout_profile)&&!unresolved(profile.ante_profile)&&ex.payout_profile===profile.payout_profile&&(ex.ante_profile??'NONE')===profile.ante_profile;
  if(!profileResolved){out.classification='PROFILE_UNRESOLVED_DIAGNOSTIC_ONLY';return out;}
  const tol=tolerancePp(spot.precision);
  out.classification=max<=tol?'HAND_SPOTCHECK_ROUNDING_PASS':'HAND_SPOTCHECK_DISAGREEMENT';
  return out;
}

function selfTest(){
  const actions=[{id:'FOLD',type:'FOLD'},{id:'LIMP',type:'LIMP'},{id:'RAISE_TO_2_2BB',type:'RAISE',to_bb:2.2},{id:'JAM_TO_15BB',type:'JAM',to_bb:15}];
  const ref={id:'r',hero_position:'SB',history:[{actor:'BTN',type:'FOLD'}],actions};
  const profile={format:'SPIN_3MAX',effective_stack_bb:15,payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE'};
  const ex={id:'e',format:'SPIN_3MAX',effective_stack_bb:15,payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',hero_position:'SB',history:ref.history,actions,hands:{AKo:{range_weight:1,strategy:{FOLD:0,LIMP:0.416,RAISE_TO_2_2BB:0.304,JAM_TO_15BB:0.28}}}};
  const spot={FOLD:0,LIMP:0.42,RAISE_TO_2_2BB:0.30,JAM_TO_15BB:0.28,precision:'DISPLAYED_INTEGER_PERCENT'};
  if(auditSpot(ex,profile,ref,'AKo',spot).classification!=='HAND_SPOTCHECK_ROUNDING_PASS')throw new Error('hand rounding pass classifier failed');
  const bad=structuredClone(ex);bad.hands.AKo.strategy.LIMP=0.39;bad.hands.AKo.strategy.RAISE_TO_2_2BB=0.33;if(auditSpot(bad,profile,ref,'AKo',spot).classification!=='HAND_SPOTCHECK_DISAGREEMENT')throw new Error('hand disagreement classifier failed');
  const unresolved={...profile,payout_profile:'UNRESOLVED_PUBLIC_SOURCE'};if(auditSpot(bad,unresolved,ref,'AKo',spot).classification!=='PROFILE_UNRESOLVED_DIAGNOSTIC_ONLY')throw new Error('unresolved profile must remain diagnostic');
  const size=structuredClone(ex);size.actions[2]={id:'RAISE_TO_2BB',type:'RAISE',to_bb:2};if(auditSpot(size,profile,ref,'AKo',spot).classification!=='TREE_SIZING_MISMATCH')throw new Error('hand tree mismatch classifier failed');
  console.log('2026 hand spot-check self-test PASS: rounding/profile/tree/fatal hand disagreement classes verified.');
}

function main(){
  const exact=files(exactDir).map(read),profiles=files(crossDir).map(read),reports=[];
  for(const ex of exact){for(const profile of profiles){if(ex.format!==profile.format||Number(ex.effective_stack_bb)!==Number(profile.effective_stack_bb))continue;for(const ref of profile.nodes??[]){if(ex.hero_position!==ref.hero_position||histKey(ex.history)!==histKey(ref.history)||!ref.hand_spot_checks)continue;for(const [hand,spot] of Object.entries(ref.hand_spot_checks))reports.push(auditSpot(ex,profile,ref,hand,spot));}}}
  const fatal=reports.filter(r=>r.classification==='HAND_SPOTCHECK_DISAGREEMENT');
  console.log(JSON.stringify({schema_version:'hand-spotcheck-audit-v1',reports},null,2));
  if(fatal.length){console.error(`2026 hand spot-check audit FAILED: ${fatal.length} compatible hand disagreement(s).`);process.exit(1);}
  console.error(`2026 hand spot-check audit PASS: ${reports.length} evaluated spot(s), ${fatal.length} compatible fatal disagreement(s).`);
}

if(process.argv.includes('--self-test'))selfTest();else main();
