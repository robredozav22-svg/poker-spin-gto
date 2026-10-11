#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const dir=path.join(root,'data','crosschecks','2026');
const errors=[];
const allowedFormats=new Set(['SPIN_3MAX','SPIN_HU']);
function fail(x){errors.push(x);}
function files(d){return !fs.existsSync(d)?[]:fs.readdirSync(d,{withFileTypes:true}).flatMap(e=>{const p=path.join(d,e.name);return e.isDirectory()?files(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
function finite(x){return typeof x==='number'&&Number.isFinite(x);}
function year2026(v){const t=Date.parse(v??'');return Number.isFinite(t)&&new Date(t).getUTCFullYear()===2026;}
function unresolved(v){return typeof v==='string'&&v.startsWith('UNRESOLVED');}

for(const file of files(dir)){
  const rel=path.relative(root,file);let p;
  try{p=JSON.parse(fs.readFileSync(file,'utf8'));}catch(e){fail(`${rel}: invalid JSON ${e.message}`);continue;}
  if(p.schema_version!=='spin-crosscheck-profile-v1')fail(`${rel}: unsupported schema_version`);
  if(!year2026(p.captured_at))fail(`${rel}: captured_at must be valid 2026 timestamp`);
  if(!allowedFormats.has(p.format))fail(`${rel}: invalid format ${p.format}`);
  if(!finite(p.effective_stack_bb)||p.effective_stack_bb<=0)fail(`${rel}: effective_stack_bb invalid`);
  if(typeof p.payout_profile!=='string'||!p.payout_profile.trim())fail(`${rel}: payout_profile missing`);
  if(typeof p.ante_profile!=='string'||!p.ante_profile.trim())fail(`${rel}: ante_profile missing`);
  if(p.promotion_use!=='CROSSCHECK_ONLY')fail(`${rel}: external public profile must remain CROSSCHECK_ONLY`);
  const s=p.source;
  if(!s||typeof s!=='object')fail(`${rel}: source missing`);else{
    if(typeof s.provider!=='string'||!s.provider.trim())fail(`${rel}: source.provider missing`);
    if(typeof s.role!=='string'||!s.role.trim())fail(`${rel}: source.role missing`);
    if(!Array.isArray(s.urls)||!s.urls.length)fail(`${rel}: source.urls missing`);
    if(s.provider==='PreflopRanges.app'){
      if(s.usage_policy!=='LIMITED_PUBLIC_SPOT_CHECKS_ONLY_NO_SCRAPING_OR_BULK_EXTRACTION')fail(`${rel}: PreflopRanges usage_policy must prohibit scraping/bulk extraction`);
      if(!year2026(s.terms_checked_at))fail(`${rel}: PreflopRanges terms_checked_at must be a valid 2026 timestamp`);
    }
  }
  if(!Array.isArray(p.nodes)||!p.nodes.length){fail(`${rel}: nodes[] missing`);continue;}
  const nodeIds=new Set();
  for(const node of p.nodes){
    const ctx=`${rel}:${node?.id??'UNKNOWN'}`;
    if(typeof node?.id!=='string'||!node.id.trim())fail(`${ctx}: id missing`);else if(nodeIds.has(node.id))fail(`${ctx}: duplicate id`);else nodeIds.add(node.id);
    if(!['BTN','SB','BB'].includes(node?.hero_position))fail(`${ctx}: invalid hero_position`);
    if(!Array.isArray(node?.history))fail(`${ctx}: history must be array`);
    if(!Array.isArray(node?.actions)||!node.actions.length){fail(`${ctx}: actions missing`);continue;}
    const actionIds=new Set();
    for(const a of node.actions){if(typeof a.id!=='string'||!a.id.trim())fail(`${ctx}: action id missing`);else if(actionIds.has(a.id))fail(`${ctx}: duplicate action ${a.id}`);else actionIds.add(a.id);if(['RAISE','JAM'].includes(a.type)&&(!finite(a.to_bb)||a.to_bb<=0))fail(`${ctx}: ${a.id} requires to_bb`);}
    if(!node.aggregate||typeof node.aggregate!=='object'){fail(`${ctx}: aggregate missing`);continue;}
    let sum=0;
    for(const id of actionIds){const v=node.aggregate[id];if(!finite(v)||v<0||v>1)fail(`${ctx}: aggregate ${id} invalid`);else sum+=v;}
    for(const id of Object.keys(node.aggregate))if(!actionIds.has(id))fail(`${ctx}: aggregate contains undeclared action ${id}`);
    if(Math.abs(sum-1)>0.005)fail(`${ctx}: aggregate sum ${sum} outside rounding tolerance`);
    if(node.hand_spot_checks!==undefined){
      for(const [hand,spot] of Object.entries(node.hand_spot_checks)){
        let handSum=0;
        for(const id of actionIds){const v=spot[id]??0;if(!finite(v)||v<0||v>1)fail(`${ctx}:${hand}: spot ${id} invalid`);else handSum+=v;}
        for(const [id,v] of Object.entries(spot))if(!['source_surface','precision'].includes(id)&&!actionIds.has(id))fail(`${ctx}:${hand}: undeclared spot action ${id}`);
        if(Math.abs(handSum-1)>0.011)fail(`${ctx}:${hand}: hand spot frequencies sum ${handSum}`);
        if(typeof spot.source_surface!=='string'||!spot.source_surface.trim())fail(`${ctx}:${hand}: source_surface missing`);
        if(typeof spot.precision!=='string'||!spot.precision.trim())fail(`${ctx}:${hand}: precision missing`);
      }
    }
  }
  if((unresolved(p.payout_profile)||unresolved(p.ante_profile))&&p.promotion_use!=='CROSSCHECK_ONLY')fail(`${rel}: unresolved profile cannot be promotion-capable`);
}

if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`2026 cross-check profile validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`2026 cross-check profile validation PASS: ${files(dir).length} profile file(s); provenance, rounding and source-usage guards active.`);
