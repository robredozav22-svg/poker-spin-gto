#!/usr/bin/env node

const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const Router=require('../ui/router.js');

const root=path.resolve(__dirname,'..');
const rel=process.argv[2];
if(!rel){console.error('Usage: node scripts/build-exact-index-entry.cjs data/charts/exact/<node>.json');process.exit(2);}
if(!rel.startsWith('data/charts/exact/')||!rel.endsWith('.json')){throw new Error('input must be an exact chart JSON under data/charts/exact/');}
const full=path.join(root,rel);
const text=fs.readFileSync(full,'utf8');
const node=JSON.parse(text);
if(node.verification_status!=='VERIFIED_EXACT')throw new Error('index builder accepts only VERIFIED_EXACT nodes');
if(typeof node.id!=='string'||!node.id.trim())throw new Error('exact node id missing');
if(typeof node.source?.captured_at!=='string'||!node.source.captured_at.trim())throw new Error('exact source captured_at missing');
const key=Router.canonicalNodeIdFromExactNode(node);
const sha256=`sha256:${crypto.createHash('sha256').update(text,'utf8').digest('hex')}`;
const entry={path:rel,artifact_id:node.id,sha256,captured_at:node.source.captured_at,source_provider:node.source?.provider??null,source_node_id:node.source?.node_id??null};
console.log(JSON.stringify({key,entry},null,2));
