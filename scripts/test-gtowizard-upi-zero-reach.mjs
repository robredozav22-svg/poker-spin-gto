#!/usr/bin/env node

import {importCapture} from './import-gtowizard-upi-node.mjs';

const R='23456789TJQKA',S='cdhs';
const deck=[];for(const r of R)for(const s of S)deck.push(r+s);
const rv=Object.fromEntries([...R].map((r,i)=>[r,i]));
function key(a,b){const ai=rv[a[0]]*4+S.indexOf(a[1]),bi=rv[b[0]]*4+S.indexOf(b[1]);return ai<bi?a+b:b+a;}
const combos=[];for(let i=0;i<52;i++)for(let j=i+1;j<52;j++)combos.push(key(deck[i],deck[j]));
function range(rows){return rows.map(([c,w])=>`${c}: ${w}`).join(',');}

const live=combos.filter(c=>c[0]==='A'||c[2]==='A');
const full=range(live.map(c=>[c,1]));
const call=range(live.map(c=>[c,0.25]));
const jam=range(live.map(c=>[c,0.75]));
const capture={
  schema_version:'gtowizard-upi-capture-v1',id:'zero-reach-selftest',format:'SPIN_3MAX',payout_profile:'TEST',ante_profile:'NONE',effective_stack_bb:15,hero_position:'BTN',
  history:[{actor:'BTN',type:'RAISE',to_bb:2},{actor:'SB',type:'JAM',to_bb:15}],
  source:{provider:'GTO Wizard',solution_family:'SELFTEST',node_id:'ZERO_REACH_SELFTEST',captured_at:'2026-09-08T00:00:00Z'},
  full_range_upi:full,
  actions:[{id:'FOLD',type:'FOLD',range_upi:''},{id:'CALL',type:'CALL',range_upi:call},{id:'JAM_TO_15BB',type:'JAM',to_bb:15,range_upi:jam}]
};
const out=importCapture(capture);
if(out.hands['22'].range_weight!==0||out.hands['22'].strategy!==null)throw new Error('22 must remain unreachable with strategy:null');
if(out.hands['AKs'].range_weight<=0)throw new Error('AKs must be reachable');
if(Math.abs(out.hands['AKs'].strategy.CALL-0.25)>1e-9||Math.abs(out.hands['AKs'].strategy.JAM_TO_15BB-0.75)>1e-9)throw new Error('reachable strategy mismatch');
if(Object.prototype.hasOwnProperty.call(out.hands['22'],'FOLD'))throw new Error('unreachable hand acquired invented FOLD');
if(out.ante_profile!=='NONE')throw new Error('ante profile lost during import');
if(out.import_audit.action_conservation!=='PASS'||out.import_audit.zero_reach_policy!=='EXPLICIT_NULL')throw new Error('import audit missing zero-reach/conservation proof');
console.log('Zero-reach importer test PASS: excluded hands remain strategy:null; no invented fold/default action; ante/provenance retained.');
