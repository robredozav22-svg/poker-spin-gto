#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {importCapture as importV2} from './import-gtowizard-upi-node.mjs';

const POSITIONS_3MAX=['BTN','SB','BB'];

function validateSeatStacks(capture){
  if(capture.format==='SPIN_3MAX'){
    const s=capture.seat_stacks_bb;
    if(!s||typeof s!=='object'||Array.isArray(s))throw new Error('SPIN_3MAX capture.seat_stacks_bb object required');
    for(const pos of POSITIONS_3MAX){const v=Number(s[pos]);if(!Number.isFinite(v)||v<=0)throw new Error(`capture.seat_stacks_bb.${pos} must be positive`);}
    const extra=Object.keys(s).filter(k=>!POSITIONS_3MAX.includes(k));if(extra.length)throw new Error(`unexpected 3max seat stack key ${extra[0]}`);
    return {BTN:Number(s.BTN),SB:Number(s.SB),BB:Number(s.BB)};
  }
  if(capture.format==='SPIN_HU'){
    const s=capture.seat_stacks_bb;
    if(!s||typeof s!=='object'||Array.isArray(s))throw new Error('SPIN_HU capture.seat_stacks_bb object required');
    const keys=Object.keys(s);if(keys.length!==2)throw new Error('SPIN_HU seat_stacks_bb must contain exactly two seats');
    for(const k of keys){const v=Number(s[k]);if(!Number.isFinite(v)||v<=0)throw new Error(`capture.seat_stacks_bb.${k} must be positive`);}
    return Object.fromEntries(keys.sort().map(k=>[k,Number(s[k])]));
  }
  throw new Error(`unsupported format for seat-stack-aware import: ${capture.format}`);
}

function classifyStacks(capture,seatStacks){
  const values=Object.values(seatStacks);const symmetric=values.every(v=>Math.abs(v-values[0])<1e-12);
  if(symmetric&&Math.abs(values[0]-Number(capture.effective_stack_bb))>1e-12)throw new Error(`symmetric seat stacks ${values[0]}bb disagree with effective_stack_bb ${capture.effective_stack_bb}`);
  return symmetric?'SYMMETRIC':'ASYMMETRIC';
}

export function importCapture(capture){
  const seatStacks=validateSeatStacks(capture);
  const stackProfile=classifyStacks(capture,seatStacks);
  const node=importV2(capture);
  node.seat_stacks_bb=seatStacks;
  node.stack_profile=stackProfile;
  node.import_audit={...node.import_audit,importer:'import-gtowizard-upi-node-v3',seat_stack_identity:'PASS',stack_profile:stackProfile};
  return node;
}

function selfTest(){
  const ranks=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];const hands=[];for(let i=0;i<13;i++)for(let j=0;j<13;j++)hands.push(i===j?ranks[i]+ranks[j]:(i<j?ranks[i]+ranks[j]+'s':ranks[j]+ranks[i]+'o'));
  const weighted=p=>hands.map(h=>`${h}: ${p}`).join(',');
  const base={schema_version:'gtowizard-upi-capture-v1',id:'v3-selftest',format:'SPIN_3MAX',payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',effective_stack_bb:15,seat_stacks_bb:{BTN:15,SB:15,BB:15},hero_position:'BTN',history:[],source:{provider:'GTO Wizard',solution_family:'Research',node_id:'V3_SELFTEST',captured_at:'2026-09-08T00:00:00Z'},full_range_upi:weighted(1),actions:[{id:'FOLD',type:'FOLD',range_upi:weighted(0.6)},{id:'RAISE_TO_2BB',type:'RAISE',to_bb:2,range_upi:weighted(0.3)},{id:'JAM_TO_15BB',type:'JAM',to_bb:15,range_upi:weighted(0.1)}]};
  const out=importCapture(base);if(out.stack_profile!=='SYMMETRIC'||out.seat_stacks_bb.BTN!==15||out.import_audit.seat_stack_identity!=='PASS')throw new Error('symmetric seat-stack provenance failed');
  let missing=false;try{const x=structuredClone(base);delete x.seat_stacks_bb;importCapture(x);}catch{missing=true;}if(!missing)throw new Error('missing seat stacks must fail');
  let mismatch=false;try{const x=structuredClone(base);x.seat_stacks_bb.BTN=14;importCapture(x);}catch{mismatch=true;}if(!mismatch)throw new Error('symmetric/effective mismatch must fail');
  const asymmetric=structuredClone(base);asymmetric.id='v3-asym';asymmetric.seat_stacks_bb={BTN:5,SB:20,BB:20};asymmetric.effective_stack_bb=5;const a=importCapture(asymmetric);if(a.stack_profile!=='ASYMMETRIC')throw new Error('asymmetric stack profile not preserved');
  console.log('GTO Wizard importer v3 self-test PASS: proven v2 range core + mandatory seat-stack identity; symmetric/asymmetric profiles separated.');
}

if(import.meta.url===`file://${process.argv[1]}`){
  if(process.argv.includes('--self-test'))selfTest();
  else{
    const input=process.argv[2],output=process.argv[3];if(!input||!output){console.error('Usage: node scripts/import-gtowizard-upi-node-v3.mjs <capture.json> <output.json> | --self-test');process.exit(2);}
    const capture=JSON.parse(fs.readFileSync(input,'utf8'));const node=importCapture(capture);fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,JSON.stringify(node,null,2)+'\n');console.log(`Imported ${capture.id} via v3 -> ${output}; stack_profile=${node.stack_profile} capture=${node.import_audit.capture_sha256}.`);
  }
}
