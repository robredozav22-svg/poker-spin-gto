#!/usr/bin/env node

const fs=require('fs');
const path=require('path');
const Router=require('../ui/router.js');

const root=path.resolve(__dirname,'..');
const indexPath=path.join(root,'data','charts','exact-index.json');
const exactDir=path.join(root,'data','charts','exact');
const index=JSON.parse(fs.readFileSync(indexPath,'utf8'));
const errors=[];
function fail(x){errors.push(x);}
function filesRecursive(dir){if(!fs.existsSync(dir))return[];return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{const p=path.join(dir,e.name);return e.isDirectory()?filesRecursive(p):(e.isFile()&&e.name.endsWith('.json')?[p]:[]);});}
if(index.schema_version!=='exact-spin-runtime-index-v1')fail('exact index schema_version mismatch');
if(!index.nodes||typeof index.nodes!=='object'||Array.isArray(index.nodes))fail('exact index nodes must be object');
const indexedPaths=new Set();
for(const [key,entry] of Object.entries(index.nodes||{})){
  if(!entry||typeof entry!=='object'||typeof entry.path!=='string'){fail(`${key}: invalid index entry`);continue;}
  if(!entry.path.startsWith('data/charts/exact/')||!entry.path.endsWith('.json')){fail(`${key}: invalid exact path ${entry.path}`);continue;}
  if(indexedPaths.has(entry.path))fail(`${key}: duplicate indexed path ${entry.path}`);indexedPaths.add(entry.path);
  const full=path.join(root,entry.path);
  if(!fs.existsSync(full)){fail(`${key}: indexed file missing ${entry.path}`);continue;}
  let node;try{node=JSON.parse(fs.readFileSync(full,'utf8'));}catch(e){fail(`${key}: invalid JSON ${e.message}`);continue;}
  if(node.verification_status!=='VERIFIED_EXACT')fail(`${key}: runtime index points to non-VERIFIED_EXACT node`);
  let expected;try{expected=Router.canonicalNodeIdFromExactNode(node);}catch(e){fail(`${key}: cannot derive canonical route: ${e.message}`);continue;}
  if(key!==expected)fail(`${key}: index key does not match node identity; expected ${expected}`);
}
for(const full of filesRecursive(exactDir)){
  const rel=path.relative(root,full).replaceAll('\\','/');
  if(!indexedPaths.has(rel))fail(`${rel}: VERIFIED_EXACT file is orphaned; every exact file must be explicitly runtime-indexed`);
}
if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact runtime index validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`Exact runtime index validation PASS: ${Object.keys(index.nodes||{}).length} indexed node(s), canonical route identity locked.`);
