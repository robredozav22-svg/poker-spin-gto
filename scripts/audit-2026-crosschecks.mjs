#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const exactDir=path.join(root,'data','charts','exact');
const crossDir=path.join(root,'data','crosschecks','2026');
const ROUNDING_PP=0.15;
const TOLERANCE_PP=0.50;

function readJson(p){return JSON.parse(fs.readFileSync(p,'utf8'));}
function files(dir){if(!fs.existsSync(dir))return[];return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?files(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
function histKey(h){return JSON.stringify((h??[]).map(x=>({actor:x.actor,type:x.type,to_bb:x.to_bb??null})));}
function actionKey(a){return `${a.type}:${a.to_bb??''}`;}
function nodeIdentity(n){return `${n.format}|${n.effective_stack_bb}|${n.hero_position}|${histKey(n.history)}`;}
function actionSet(n){return new Set((n.actions??[]).map(actionKey));}
function sameSet(a,b){return a.size===b.size&&[...a].every(x=>b.has(x));}
function unresolved(v){return typeof v!=='string'||!v.trim()||v.startsWith('UNRESOLVED');}
function compareNode(ex,profile,ref){
  const result={exact_id:ex.id??null,crosscheck_id:ref.id??null,provider:profile.source?.provider??'UNKNOWN',classification:null,numeric:null,reasons:[]};
  if(ex.format!==profile.format){result.classification='FORMAT_MISMATCH';result.reasons.push(`${ex.format} vs ${profile.format}`);return result;}
  if(Number(ex.effective_stack_bb)!==Number(profile.effective_stack_bb)){result.classification='STACK_PROFILE_MISMATCH';return result;}
  if(ex.hero_position!==ref.hero_position||histKey(ex.history)!==histKey(ref.history)){result.classification='ACTION_HISTORY_MISMATCH';return result;}
  if(!sameSet(actionSet(ex),actionSet(ref))){result.classification='TREE_SIZING_MISMATCH';result.reasons.push(`exact=${[...actionSet(ex)].join(',')} ref=${[...actionSet(ref)].join(',')}`);return result;}
  if(unresolved(profile.payout_profile)){result.classification='PAYOUT_PROFILE_UNRESOLVED';return result;}
  if(ex.payout_profile!==profile.payout_profile){result.classification='PAYOUT_PROFILE_MISMATCH';return result;}
  if(unresolved(profile.ante_profile)){result.classification='ANTE_PROFILE_UNRESOLVED';return result;}
  const exAnte=ex.ante_profile??'NONE';
  if(exAnte!==profile.ante_profile){result.classification='ANTE_FORMAT_MISMATCH';return result;}
  if(!ex.aggregate||!ref.aggregate){result.classification='NO_NUMERIC_AGGREGATE';return result;}
  const diffs=[];
  for(const a of ex.actions){
    const id=a.id;
    if(!(id in ex.aggregate)||!(id in ref.aggregate)){result.classification='ACTION_ID_MAPPING_MISMATCH';return result;}
    diffs.push({action:id,exact:Number(ex.aggregate[id]),reference:Number(ref.aggregate[id]),diff_pp:Math.abs(Number(ex.aggregate[id])-Number(ref.aggregate[id]))*100});
  }
  const max=Math.max(...diffs.map(x=>x.diff_pp));
  result.numeric={max_abs_diff_pp:max,actions:diffs};
  if(max<=ROUNDING_PP)result.classification='ROUNDING_ONLY';
  else if(max<=TOLERANCE_PP)result.classification='SOLVER_TOLERANCE';
  else result.classification='UNEXPLAINED_NUMERIC_DISAGREEMENT';
  return result;
}

const exactFiles=files(exactDir);const crossFiles=files(crossDir);
const exactNodes=exactFiles.map(p=>({file:p,node:readJson(p)}));
const profiles=crossFiles.map(p=>({file:p,profile:readJson(p)}));
const reports=[];
for(const {node:ex} of exactNodes){
  let candidates=[];
  for(const {profile} of profiles){for(const ref of profile.nodes??[]){if(nodeIdentity(ex)===`${profile.format}|${profile.effective_stack_bb}|${ref.hero_position}|${histKey(ref.history)}`)candidates.push(compareNode(ex,profile,ref));}}
  if(!candidates.length)reports.push({exact_id:ex.id??null,classification:'NO_COMPATIBLE_CURRENT_CROSSCHECK',reasons:[]});
  else reports.push(...candidates);
}

const fatal=reports.filter(r=>r.classification==='UNEXPLAINED_NUMERIC_DISAGREEMENT');
console.log(JSON.stringify({schema_version:'crosscheck-audit-report-v1',exact_nodes:exactNodes.length,crosscheck_profiles:profiles.length,reports},null,2));
if(fatal.length){console.error(`2026 cross-check audit FAILED: ${fatal.length} unexplained numeric disagreement(s).`);process.exit(1);}
console.error(`2026 cross-check audit PASS: ${exactNodes.length} exact node(s), ${profiles.length} current cross-check profile(s), ${fatal.length} unexplained numeric disagreements.`);
