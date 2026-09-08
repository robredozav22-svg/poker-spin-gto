#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';

const require=createRequire(import.meta.url);
const Router=require('../ui/router.js');
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const indexPath=path.join(root,'data','charts','exact-index.json');
const index=JSON.parse(fs.readFileSync(indexPath,'utf8'));
const errors=[];
const fail=x=>errors.push(x);

if(index.schema_version!=='exact-spin-runtime-index-v1')fail('exact-index schema_version invalid');
if(!index.nodes||typeof index.nodes!=='object'||Array.isArray(index.nodes))fail('exact-index nodes must be object');

const seenPaths=new Set();
for(const [routeKey,entry] of Object.entries(index.nodes??{})){
  if(!entry||typeof entry!=='object'||Array.isArray(entry)){fail(`${routeKey}: entry must be object`);continue;}
  if(typeof entry.path!=='string'||!entry.path.startsWith('data/charts/exact/')||!entry.path.endsWith('.json')){fail(`${routeKey}: invalid exact path`);continue;}
  if(seenPaths.has(entry.path))fail(`${routeKey}: duplicate exact path ${entry.path}`);seenPaths.add(entry.path);
  const abs=path.join(root,entry.path);
  if(!fs.existsSync(abs)){fail(`${routeKey}: indexed file missing ${entry.path}`);continue;}
  let node;try{node=JSON.parse(fs.readFileSync(abs,'utf8'));}catch(e){fail(`${routeKey}: invalid JSON ${e.message}`);continue;}
  if(node.verification_status!=='VERIFIED_EXACT')fail(`${routeKey}: indexed node must be VERIFIED_EXACT`);
  if(typeof node.id!=='string'||!node.id.trim())fail(`${routeKey}: indexed node id missing`);
  let derived;try{derived=Router.canonicalNodeIdFromExactNode(node);}catch(e){fail(`${routeKey}: cannot derive canonical route: ${e.message}`);continue;}
  if(derived!==routeKey)fail(`${routeKey}: route mismatch; canonical node route is ${derived}`);
  if(entry.node_id!==undefined&&entry.node_id!==node.id)fail(`${routeKey}: entry.node_id ${entry.node_id} != file node.id ${node.id}`);
}

// Non-vacuous route identity self-test even while production exact index is empty.
const fixture={format:'SPIN_3MAX',effective_stack_bb:15,payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',history:[{actor:'BTN',type:'RAISE',to_bb:2},{actor:'SB',type:'JAM',to_bb:15}]};
const expected='3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2/SB_ALL_IN_15';
const got=Router.canonicalNodeIdFromExactNode(fixture);
if(got!==expected)fail(`canonical route fixture mismatch: got ${got}, expected ${expected}`);
const changed=structuredClone(fixture);changed.history[0].to_bb=2.2;
if(Router.canonicalNodeIdFromExactNode(changed)===got)fail('canonical route failed to distinguish raise sizing 2.0 vs 2.2bb');

if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`Exact runtime index validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log(`Exact runtime index validation PASS: ${Object.keys(index.nodes??{}).length} indexed node(s); canonical route/path/sizing guard active.`);
