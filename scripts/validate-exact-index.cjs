#!/usr/bin/env node

const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const Router=require('../ui/router.js');

const root=path.resolve(__dirname,'..');
const indexPath=path.join(root,'data','charts','exact-index.json');
const exactDir=path.join(root,'data','charts','exact');
const index=JSON.parse(fs.readFileSync(indexPath,'utf8'));
const errors=[];
function fail(x){errors.push(x);}
function filesRecursive(dir){if(!fs.existsSync(dir))return[];return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?filesRecursive(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
function sha256Text(text){return`sha256:${crypto.createHash('sha256').update(text,'utf8').digest('hex')}`;}

if(index.schema_version!=='exact-spin-runtime-index-v1')fail('exact index schema_version mismatch');
if(!index.nodes||typeof index.nodes!=='object'||Array.isArray(index.nodes))fail('exact index nodes must be object');
const indexedPaths=new Set();
for(const [key,entry] of Object.entries(index.nodes||{})){
  if(!entry||typeof entry!=='object'){fail(`${key}: invalid index entry`);continue;}
  if(typeof entry.path!=='string'||!entry.path.startsWith('data/charts/exact/')||!entry.path.endsWith('.json')){fail(`${key}: invalid exact path ${entry.path}`);continue;}
  if(typeof entry.artifact_id!=='string'||!entry.artifact_id.trim())fail(`${key}: artifact_id missing`);
  if(typeof entry.sha256!=='string'||!/^sha256:[0-9a-f]{64}$/.test(entry.sha256))fail(`${key}: invalid sha256`);
  if(typeof entry.captured_at!=='string'||!entry.captured_at.trim())fail(`${key}: captured_at missing`);
  if(indexedPaths.has(entry.path))fail(`${key}: duplicate indexed path ${entry.path}`);indexedPaths.add(entry.path);
  const full=path.join(root,entry.path);
  if(!fs.existsSync(full)){fail(`${key}: indexed file missing ${entry.path}`);continue;}
  const text=fs.readFileSync(full,'utf8');
  const actualSha=sha256Text(text);
  if(entry.sha256!==actualSha)fail(`${key}: checksum mismatch; index=${entry.sha256} actual=${actualSha}`);
  let node;try{node=JSON.parse(text);}catch(e){fail(`${key}: invalid JSON ${e.message}`);continue;}
  if(node.verification_status!=='VERIFIED_EXACT')fail(`${key}: runtime index points to non-VERIFIED_EXACT node`);
  if(node.id!==entry.artifact_id)fail(`${key}: artifact_id mismatch; index=${entry.artifact_id} node=${node.id}`);
  if(node.source?.captured_at!==entry.captured_at)fail(`${key}: captured_at mismatch; index=${entry.captured_at} node=${node.source?.captured_at}`);
  let expected;try{expected=Router.canonicalNodeIdFromExactNode(node);}catch(e){fail(`${key}: cannot derive canonical route: ${e.message}`);continue;}
  if(key!==expected)fail(`${key}: index key does not match node identity; expected ${expected}`);
}
for(const full of filesRecursive(exactDir)){
  const rel=path.relative(root,full).replaceAll('\\','/');
  if(!indexedPaths.has(rel))fail(`${rel}: VERIFIED_EXACT file is orphaned; every exact file must be explicitly runtime-indexed`);
}
if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact runtime index validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`Exact runtime index validation PASS: ${Object.keys(index.nodes||{}).length} indexed node(s), route + artifact_id + SHA-256 + capture provenance locked.`);
