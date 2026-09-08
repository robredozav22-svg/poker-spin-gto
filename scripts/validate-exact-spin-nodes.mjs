#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const exactDir=path.join(root,'data','charts','exact');
const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
const POSITIONS=new Set(['BTN','SB','BB']);
const TYPES=new Set(['FOLD','CHECK','LIMP','CALL','RAISE','JAM']);
const FORBIDDEN_METHODS=new Set(['INTERPOLATED','SCREENSHOT_COLOR_RECONSTRUCTION','DEFAULT_FILL','INFERRED_RANGE']);
const errors=[];

function fail(s){errors.push(s);}
function hands169(){const out=[];for(let i=0;i<13;i++)for(let j=0;j<13;j++){if(i===j)out.push(RANKS[i]+RANKS[j]);else if(i<j)out.push(RANKS[i]+RANKS[j]+'s');else out.push(RANKS[j]+RANKS[i]+'o');}return out;}
const HANDS=hands169(),HAND_SET=new Set(HANDS);
function combos(h){return h.length===2?6:h.endsWith('s')?4:12;}
if(HANDS.length!==169||HAND_SET.size!==169)fail('internal 169-hand universe invalid');
if(HANDS.reduce((n,h)=>n+combos(h),0)!==1326)fail('internal combo universe must be 1326');

function filesRecursive(dir){if(!fs.existsSync(dir))return[];return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?filesRecursive(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
function finite(x){return typeof x==='number'&&Number.isFinite(x);}
function validateAction(action,ctx){
  if(!action||typeof action!=='object'||Array.isArray(action))return fail(`${ctx}: action must be object`);
  if(typeof action.id!=='string'||!action.id.trim())fail(`${ctx}: action.id missing`);
  if(!TYPES.has(action.type))fail(`${ctx}: invalid action.type ${action.type}`);
  if(['RAISE','JAM'].includes(action.type)){
    if(!finite(action.to_bb)||action.to_bb<=0)fail(`${ctx}: ${action.type} requires positive to_bb`);
  }else if(action.to_bb!==undefined){fail(`${ctx}: ${action.type} must not carry to_bb`);}
}
function validateHistory(row,ctx){
  if(!row||typeof row!=='object'||Array.isArray(row))return fail(`${ctx}: history row must be object`);
  if(!POSITIONS.has(row.actor))fail(`${ctx}: invalid/missing actor`);
  if(!TYPES.has(row.type))fail(`${ctx}: invalid/missing type`);
  if(['RAISE','JAM'].includes(row.type)&&(!finite(row.to_bb)||row.to_bb<=0))fail(`${ctx}: ${row.type} history requires to_bb`);
}
function validateNode(doc,file){
  const ctx=path.relative(root,file);
  if(doc.schema_version!=='exact-spin-node-v1')fail(`${ctx}: schema_version must be exact-spin-node-v1`);
  if(doc.verification_status!=='VERIFIED_EXACT')fail(`${ctx}: exact directory requires verification_status=VERIFIED_EXACT`);
  if(doc.format!=='SPIN_3MAX'&&doc.format!=='SPIN_HU')fail(`${ctx}: invalid format`);
  if(typeof doc.payout_profile!=='string'||!doc.payout_profile.trim())fail(`${ctx}: payout_profile missing`);
  if(!finite(doc.effective_stack_bb)||doc.effective_stack_bb<=0)fail(`${ctx}: effective_stack_bb must be positive`);
  if(!POSITIONS.has(doc.hero_position))fail(`${ctx}: hero_position invalid/missing`);
  if(!Array.isArray(doc.history))fail(`${ctx}: history must be an array`);else doc.history.forEach((r,i)=>validateHistory(r,`${ctx}:history[${i}]`));
  if(!doc.source||typeof doc.source!=='object')fail(`${ctx}: source object missing`);else{
    for(const k of ['provider','solution_family','node_id','extraction_method'])if(typeof doc.source[k]!=='string'||!doc.source[k].trim())fail(`${ctx}: source.${k} missing`);
    if(FORBIDDEN_METHODS.has(doc.source.extraction_method))fail(`${ctx}: forbidden extraction_method ${doc.source.extraction_method}`);
    if(doc.source.interpolated===true)fail(`${ctx}: interpolation is forbidden`);
    if(doc.source.default_filled===true)fail(`${ctx}: default-filled hands are forbidden`);
  }
  if(!Array.isArray(doc.actions)||doc.actions.length<1)fail(`${ctx}: actions[] missing`);
  const actionIds=new Set();
  for(let i=0;i<(doc.actions??[]).length;i++){const a=doc.actions[i];validateAction(a,`${ctx}:actions[${i}]`);if(a?.id){if(actionIds.has(a.id))fail(`${ctx}: duplicate action id ${a.id}`);actionIds.add(a.id);}}
  if(!doc.hands||typeof doc.hands!=='object'||Array.isArray(doc.hands))return fail(`${ctx}: hands object missing`);
  const keys=Object.keys(doc.hands);
  if(keys.length!==169)fail(`${ctx}: expected exactly 169 hands, got ${keys.length}`);
  for(const h of keys)if(!HAND_SET.has(h))fail(`${ctx}: unknown hand ${h}`);
  for(const h of HANDS)if(!(h in doc.hands))fail(`${ctx}: missing hand ${h}; no default fold is allowed`);

  const aggregate={};for(const id of actionIds)aggregate[id]=0;
  for(const h of HANDS){
    const cell=doc.hands[h];
    if(!cell||typeof cell!=='object'||Array.isArray(cell)){fail(`${ctx}:${h} cell must be object`);continue;}
    const entries=Object.entries(cell);
    if(!entries.length){fail(`${ctx}:${h} empty cell`);continue;}
    let sum=0;
    for(const [id,p] of entries){if(!actionIds.has(id)){fail(`${ctx}:${h} undeclared action ${id}`);continue;}if(!finite(p)||p<0||p>1){fail(`${ctx}:${h} invalid probability ${id}=${p}`);continue;}sum+=p;aggregate[id]+=p*combos(h);}
    if(Math.abs(sum-1)>1e-9)fail(`${ctx}:${h} probabilities sum ${sum}, expected 1`);
  }
  for(const id of Object.keys(aggregate))aggregate[id]/=1326;
  if(doc.aggregate!==undefined){
    if(!doc.aggregate||typeof doc.aggregate!=='object'||Array.isArray(doc.aggregate))fail(`${ctx}: aggregate must be object`);else{
      for(const id of actionIds){if(!finite(doc.aggregate[id]))fail(`${ctx}: aggregate missing ${id}`);else if(Math.abs(doc.aggregate[id]-aggregate[id])>1e-8)fail(`${ctx}: aggregate ${id}=${doc.aggregate[id]} disagrees with matrix ${aggregate[id]}`);}
      for(const id of Object.keys(doc.aggregate))if(!actionIds.has(id))fail(`${ctx}: aggregate contains undeclared action ${id}`);
    }
  }
}

const files=filesRecursive(exactDir);
for(const file of files){try{validateNode(JSON.parse(fs.readFileSync(file,'utf8')),file);}catch(e){fail(`${path.relative(root,file)}: invalid JSON (${e.message})`);}}
if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact Spin node validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`Exact Spin node validation PASS: ${files.length} exact node file(s); strict 169-hand/no-default/interpolation guard active.`);
