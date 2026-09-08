#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const exactDir=path.join(root,'data','charts','exact');
const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
const POSITIONS=new Set(['BTN','SB','BB']);
const TYPES=new Set(['FOLD','CHECK','LIMP','CALL','RAISE','JAM']);
const FORBIDDEN_METHODS=new Set(['INTERPOLATED','SCREENSHOT_COLOR_RECONSTRUCTION','DEFAULT_FILL','INFERRED_RANGE']);
const STALE_PRIMARY_FAMILIES=new Set(['LEGACY','BASIC']);
const errors=[];
const fail=s=>errors.push(s);
function hands169(){const out=[];for(let i=0;i<13;i++)for(let j=0;j<13;j++)out.push(i===j?RANKS[i]+RANKS[j]:(i<j?RANKS[i]+RANKS[j]+'s':RANKS[j]+RANKS[i]+'o'));return out;}
const HANDS=hands169(),HAND_SET=new Set(HANDS);
const combos=h=>h.length===2?6:h.endsWith('s')?4:12;
if(HANDS.length!==169||HAND_SET.size!==169||HANDS.reduce((n,h)=>n+combos(h),0)!==1326)fail('internal hand/combo universe invalid');
function filesRecursive(dir){if(!fs.existsSync(dir))return[];return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?filesRecursive(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
const finite=x=>typeof x==='number'&&Number.isFinite(x);
function validateAction(a,ctx){if(!a||typeof a!=='object'||Array.isArray(a))return fail(`${ctx}: action must be object`);if(typeof a.id!=='string'||!a.id.trim())fail(`${ctx}: action.id missing`);if(!TYPES.has(a.type))fail(`${ctx}: invalid action.type ${a.type}`);if(['RAISE','JAM'].includes(a.type)){if(!finite(a.to_bb)||a.to_bb<=0)fail(`${ctx}: ${a.type} requires positive to_bb`);}else if(a.to_bb!==undefined)fail(`${ctx}: ${a.type} must not carry to_bb`);}
function validateHistory(r,ctx){if(!r||typeof r!=='object'||Array.isArray(r))return fail(`${ctx}: history row must be object`);if(!POSITIONS.has(r.actor))fail(`${ctx}: invalid/missing actor`);if(!TYPES.has(r.type))fail(`${ctx}: invalid/missing type`);if(['RAISE','JAM'].includes(r.type)&&(!finite(r.to_bb)||r.to_bb<=0))fail(`${ctx}: ${r.type} history requires to_bb`);}
function validateSource(s,ctx){
  if(!s||typeof s!=='object')return fail(`${ctx}: source object missing`);
  for(const k of ['provider','solution_family','node_id','extraction_method','captured_at'])if(typeof s[k]!=='string'||!s[k].trim())fail(`${ctx}: source.${k} missing`);
  if(FORBIDDEN_METHODS.has(s.extraction_method))fail(`${ctx}: forbidden extraction_method ${s.extraction_method}`);
  if(s.interpolated===true)fail(`${ctx}: interpolation is forbidden`);if(s.default_filled===true)fail(`${ctx}: default-filled hands are forbidden`);
  const captured=Date.parse(s.captured_at||'');if(!Number.isFinite(captured))fail(`${ctx}: source.captured_at invalid`);else if(new Date(captured).getUTCFullYear()!==2026)fail(`${ctx}: VERIFIED_EXACT requires live 2026 capture`);
  if(s.provider==='GTO Wizard'&&STALE_PRIMARY_FAMILIES.has(String(s.solution_family).trim().toUpperCase()))fail(`${ctx}: GTO Wizard ${s.solution_family} cannot be new 2026 exact primary`);
}
function validateImportAudit(doc,ctx){
  if(doc.source?.extraction_method!=='GTOWIZARD_UPI_FULL_PLUS_ACTION_RANGES')return;
  const a=doc.import_audit;if(!a||typeof a!=='object')return fail(`${ctx}: GTO Wizard UPI exact node requires import_audit`);
  if(a.importer!=='import-gtowizard-upi-node-v2')fail(`${ctx}: import_audit.importer must be v2`);
  if(typeof a.capture_sha256!=='string'||!/^sha256:[0-9a-f]{64}$/.test(a.capture_sha256))fail(`${ctx}: import_audit.capture_sha256 invalid`);
  if(a.physical_combos_checked!==1326)fail(`${ctx}: import audit must check exactly 1326 physical combos`);
  if(a.hand_classes_derived!==169)fail(`${ctx}: import audit must derive exactly 169 hand classes`);
  if(a.action_conservation!=='PASS')fail(`${ctx}: UPI action conservation must PASS`);
  if(a.suit_symmetry!=='PASS')fail(`${ctx}: preflop suit symmetry must PASS`);
  if(a.zero_reach_policy!=='EXPLICIT_NULL')fail(`${ctx}: zero reach policy must be EXPLICIT_NULL`);
  if(!finite(a.probability_tolerance)||a.probability_tolerance<=0)fail(`${ctx}: probability tolerance invalid`);
  if(!finite(a.symmetry_tolerance)||a.symmetry_tolerance<=0)fail(`${ctx}: symmetry tolerance invalid`);
}
function validateNode(doc,file){
  const ctx=path.relative(root,file);
  if(doc.schema_version!=='exact-spin-node-v1')fail(`${ctx}: schema_version must be exact-spin-node-v1`);
  if(doc.verification_status!=='VERIFIED_EXACT')fail(`${ctx}: exact directory requires VERIFIED_EXACT`);
  if(doc.format!=='SPIN_3MAX'&&doc.format!=='SPIN_HU')fail(`${ctx}: invalid format`);
  if(typeof doc.payout_profile!=='string'||!doc.payout_profile.trim())fail(`${ctx}: payout_profile missing`);
  if(typeof doc.ante_profile!=='string'||!doc.ante_profile.trim())fail(`${ctx}: ante_profile missing`);
  if(!finite(doc.effective_stack_bb)||doc.effective_stack_bb<=0)fail(`${ctx}: effective_stack_bb must be positive`);
  if(!POSITIONS.has(doc.hero_position))fail(`${ctx}: hero_position invalid/missing`);
  if(!Array.isArray(doc.history))fail(`${ctx}: history must be array`);else doc.history.forEach((r,i)=>validateHistory(r,`${ctx}:history[${i}]`));
  validateSource(doc.source,ctx);validateImportAudit(doc,ctx);
  if(!Array.isArray(doc.actions)||!doc.actions.length)fail(`${ctx}: actions[] missing`);
  const actionIds=new Set();for(let i=0;i<(doc.actions??[]).length;i++){const a=doc.actions[i];validateAction(a,`${ctx}:actions[${i}]`);if(a?.id){if(actionIds.has(a.id))fail(`${ctx}: duplicate action id ${a.id}`);actionIds.add(a.id);}}
  if(!doc.hands||typeof doc.hands!=='object'||Array.isArray(doc.hands))return fail(`${ctx}: hands object missing`);
  const keys=Object.keys(doc.hands);if(keys.length!==169)fail(`${ctx}: expected exactly 169 hand classes, got ${keys.length}`);for(const h of keys)if(!HAND_SET.has(h))fail(`${ctx}: unknown hand ${h}`);for(const h of HANDS)if(!(h in doc.hands))fail(`${ctx}: missing hand ${h}; no default action allowed`);
  const mass=Object.fromEntries([...actionIds].map(id=>[id,0]));let reach=0;
  for(const h of HANDS){const c=doc.hands[h];if(!c||typeof c!=='object'||Array.isArray(c)){fail(`${ctx}:${h} invalid cell`);continue;}const rw=c.range_weight;if(!finite(rw)||rw<0||rw>1){fail(`${ctx}:${h} range_weight invalid`);continue;}if(rw<=1e-12){if(c.strategy!==null)fail(`${ctx}:${h} zero-reach must use strategy:null`);continue;}reach+=rw*combos(h);if(!c.strategy||typeof c.strategy!=='object'||Array.isArray(c.strategy)){fail(`${ctx}:${h} reachable hand requires strategy`);continue;}let sum=0;for(const [id,p] of Object.entries(c.strategy)){if(!actionIds.has(id)){fail(`${ctx}:${h} undeclared action ${id}`);continue;}if(!finite(p)||p<0||p>1){fail(`${ctx}:${h} invalid probability ${id}=${p}`);continue;}sum+=p;mass[id]+=p*rw*combos(h);}if(Math.abs(sum-1)>1e-9)fail(`${ctx}:${h} strategy sums ${sum}`);}
  if(reach<=1e-12)fail(`${ctx}: node has zero reach mass`);
  const aggregate=Object.fromEntries([...actionIds].map(id=>[id,mass[id]/reach]));
  if(doc.reach_combos!==undefined&&(!finite(doc.reach_combos)||Math.abs(doc.reach_combos-reach)>1e-8))fail(`${ctx}: reach_combos mismatch`);
  if(doc.aggregate!==undefined){if(!doc.aggregate||typeof doc.aggregate!=='object'||Array.isArray(doc.aggregate))fail(`${ctx}: aggregate must be object`);else{for(const id of actionIds){if(!finite(doc.aggregate[id]))fail(`${ctx}: aggregate missing ${id}`);else if(Math.abs(doc.aggregate[id]-aggregate[id])>1e-8)fail(`${ctx}: aggregate ${id} disagrees with matrix`);}for(const id of Object.keys(doc.aggregate))if(!actionIds.has(id))fail(`${ctx}: aggregate undeclared action ${id}`);}}
}
const files=filesRecursive(exactDir);for(const file of files){try{validateNode(JSON.parse(fs.readFileSync(file,'utf8')),file);}catch(e){fail(`${path.relative(root,file)}: invalid JSON (${e.message})`);}}
if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact Spin node validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`Exact Spin node validation PASS: ${files.length} exact node file(s); live-2026 + payout/ante identity + importer provenance + 169/zero-reach guards active.`);
