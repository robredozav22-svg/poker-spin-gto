#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const exactDir=path.join(root,'data','charts','exact');
const crossDir=path.join(root,'data','crosschecks','2026');
const ROUNDING_PP=0.15,TOLERANCE_PP=0.50;
const readJson=p=>JSON.parse(fs.readFileSync(p,'utf8'));
const files=dir=>!fs.existsSync(dir)?[]:fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?files(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});
const histKey=h=>JSON.stringify((h??[]).map(x=>({actor:x.actor,type:x.type,to_bb:x.to_bb??null})));
const actionKey=a=>`${a.type}:${a.to_bb??''}`;
const actionSet=n=>new Set((n.actions??[]).map(actionKey));
const sameSet=(a,b)=>a.size===b.size&&[...a].every(x=>b.has(x));
const unresolved=v=>typeof v!=='string'||!v.trim()||v.startsWith('UNRESOLVED');

export function compareNode(ex,profile,ref){
  const r={exact_id:ex.id??null,crosscheck_id:ref.id??null,provider:profile.source?.provider??'UNKNOWN',classification:null,numeric:null,reasons:[]};
  if(ex.format!==profile.format){r.classification='FORMAT_MISMATCH';return r;}
  if(Number(ex.effective_stack_bb)!==Number(profile.effective_stack_bb)){r.classification='STACK_PROFILE_MISMATCH';return r;}
  if(ex.hero_position!==ref.hero_position||histKey(ex.history)!==histKey(ref.history)){r.classification='ACTION_HISTORY_MISMATCH';return r;}
  if(!sameSet(actionSet(ex),actionSet(ref))){r.classification='TREE_SIZING_MISMATCH';r.reasons.push(`exact=${[...actionSet(ex)].join(',')} ref=${[...actionSet(ref)].join(',')}`);return r;}
  if(unresolved(profile.payout_profile)){r.classification='PAYOUT_PROFILE_UNRESOLVED';return r;}
  if(ex.payout_profile!==profile.payout_profile){r.classification='PAYOUT_PROFILE_MISMATCH';return r;}
  if(unresolved(profile.ante_profile)){r.classification='ANTE_PROFILE_UNRESOLVED';return r;}
  if((ex.ante_profile??'NONE')!==profile.ante_profile){r.classification='ANTE_FORMAT_MISMATCH';return r;}
  if(!ex.aggregate||!ref.aggregate){r.classification='NO_NUMERIC_AGGREGATE';return r;}
  const diffs=[];
  for(const a of ex.actions){const id=a.id;if(!(id in ex.aggregate)||!(id in ref.aggregate)){r.classification='ACTION_ID_MAPPING_MISMATCH';return r;}diffs.push({action:id,exact:Number(ex.aggregate[id]),reference:Number(ref.aggregate[id]),diff_pp:Math.abs(Number(ex.aggregate[id])-Number(ref.aggregate[id]))*100});}
  const max=Math.max(...diffs.map(x=>x.diff_pp));r.numeric={max_abs_diff_pp:max,actions:diffs};
  r.classification=max<=ROUNDING_PP?'ROUNDING_ONLY':max<=TOLERANCE_PP?'SOLVER_TOLERANCE':'UNEXPLAINED_NUMERIC_DISAGREEMENT';return r;
}

function selfTest(){
  const baseProfile={source:{provider:'TEST'},format:'SPIN_3MAX',effective_stack_bb:15,payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE'};
  const ref={id:'r',hero_position:'BTN',history:[],actions:[{id:'FOLD',type:'FOLD'},{id:'RAISE_TO_2BB',type:'RAISE',to_bb:2}],aggregate:{FOLD:0.60,RAISE_TO_2BB:0.40}};
  const exact={id:'e',format:'SPIN_3MAX',effective_stack_bb:15,payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',hero_position:'BTN',history:[],actions:ref.actions,aggregate:{FOLD:0.601,RAISE_TO_2BB:0.399}};
  if(compareNode(exact,baseProfile,ref).classification!=='ROUNDING_ONLY')throw new Error('rounding classifier failed');
  const tol=structuredClone(exact);tol.aggregate={FOLD:0.604,RAISE_TO_2BB:0.396};if(compareNode(tol,baseProfile,ref).classification!=='SOLVER_TOLERANCE')throw new Error('solver tolerance classifier failed');
  const bad=structuredClone(exact);bad.aggregate={FOLD:0.62,RAISE_TO_2BB:0.38};if(compareNode(bad,baseProfile,ref).classification!=='UNEXPLAINED_NUMERIC_DISAGREEMENT')throw new Error('numeric disagreement classifier failed');
  const sizing=structuredClone(exact);sizing.actions=[{id:'FOLD',type:'FOLD'},{id:'RAISE_TO_2_2BB',type:'RAISE',to_bb:2.2}];if(compareNode(sizing,baseProfile,ref).classification!=='TREE_SIZING_MISMATCH')throw new Error('tree sizing classifier failed');
  const unresolvedProfile={...baseProfile,payout_profile:'UNRESOLVED_PUBLIC_SOURCE'};if(compareNode(exact,unresolvedProfile,ref).classification!=='PAYOUT_PROFILE_UNRESOLVED')throw new Error('unresolved payout gate failed');
  console.log('2026 cross-check classifier self-test PASS: rounding/tolerance/tree/unresolved/fatal disagreement classes verified.');
}

function main(){
  const exactNodes=files(exactDir).map(p=>readJson(p));const profiles=files(crossDir).map(p=>readJson(p));const reports=[];
  for(const ex of exactNodes){let matched=false;for(const profile of profiles){for(const ref of profile.nodes??[]){if(ex.format===profile.format&&Number(ex.effective_stack_bb)===Number(profile.effective_stack_bb)&&ex.hero_position===ref.hero_position&&histKey(ex.history)===histKey(ref.history)){matched=true;reports.push(compareNode(ex,profile,ref));}}}if(!matched)reports.push({exact_id:ex.id??null,classification:'NO_COMPATIBLE_CURRENT_CROSSCHECK',reasons:[]});}
  const fatal=reports.filter(r=>r.classification==='UNEXPLAINED_NUMERIC_DISAGREEMENT');
  console.log(JSON.stringify({schema_version:'crosscheck-audit-report-v1',exact_nodes:exactNodes.length,crosscheck_profiles:profiles.length,reports},null,2));
  if(fatal.length){console.error(`2026 cross-check audit FAILED: ${fatal.length} unexplained numeric disagreement(s).`);process.exit(1);}
  console.error(`2026 cross-check audit PASS: ${exactNodes.length} exact node(s), ${profiles.length} current cross-check profile(s).`);
}

if(process.argv.includes('--self-test'))selfTest();else main();
