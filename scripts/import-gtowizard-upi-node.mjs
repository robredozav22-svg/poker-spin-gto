#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';

const RANKS='23456789TJQKA';
const SUITS='cdhs';
const TYPES=new Set(['FOLD','CHECK','LIMP','CALL','RAISE','JAM']);
const TOL=1e-8;
const SYMMETRY_TOL=1e-7;

function card(rank,suit){return `${rank}${suit}`;}
const DECK=[];for(const r of RANKS)for(const s of SUITS)DECK.push(card(r,s));
const RANK_VALUE=Object.fromEntries([...RANKS].map((r,i)=>[r,i]));
const SUIT_VALUE=Object.fromEntries([...SUITS].map((s,i)=>[s,i]));
function cardIndex(c){return RANK_VALUE[c[0]]*4+SUIT_VALUE[c[1]];}
function comboKey(a,b){if(a===b)throw new Error(`duplicate card combo ${a}${b}`);return cardIndex(a)<cardIndex(b)?`${a}${b}`:`${b}${a}`;}
function classOfCombo(key){
  const a=key.slice(0,2),b=key.slice(2,4),ra=a[0],rb=b[0];
  if(ra===rb)return ra+rb;
  const hi=RANK_VALUE[ra]>RANK_VALUE[rb]?ra:rb,lo=hi===ra?rb:ra;
  return `${hi}${lo}${a[1]===b[1]?'s':'o'}`;
}
function allCombos(){const out=[];for(let i=0;i<52;i++)for(let j=i+1;j<52;j++)out.push(comboKey(DECK[i],DECK[j]));return out;}
const COMBOS=allCombos();
if(COMBOS.length!==1326||new Set(COMBOS).size!==1326)throw new Error('internal combo universe invalid');

function parseWeight(raw){
  const s=raw.trim();
  const pct=s.endsWith('%');
  const n=Number(pct?s.slice(0,-1):s);
  if(!Number.isFinite(n))throw new Error(`invalid weight ${raw}`);
  const w=pct?n/100:n;
  if(w<0||w>1+TOL)throw new Error(`weight out of [0,1]: ${raw}`);
  return Math.max(0,Math.min(1,w));
}

export function parseExplicitUpi(text,label='range'){
  if(typeof text!=='string')throw new Error(`${label}: UPI must be string`);
  const map=new Map();
  const chunks=text.split(/[\n,;]+/).map(x=>x.trim()).filter(Boolean);
  for(const chunk of chunks){
    const m=chunk.match(/^([2-9TJQKA][cdhs])([2-9TJQKA][cdhs])(?:\s*:\s*([^\s]+))?$/i);
    if(!m)throw new Error(`${label}: unsupported/non-explicit UPI token '${chunk}'. Exact importer requires explicit two-card combos such as KcQc: 0.5`);
    const a=`${m[1][0].toUpperCase()}${m[1][1].toLowerCase()}`,b=`${m[2][0].toUpperCase()}${m[2][1].toLowerCase()}`;
    const key=comboKey(a,b),weight=m[3]===undefined?1:parseWeight(m[3]);
    if(map.has(key))throw new Error(`${label}: duplicate combo ${key}`);
    map.set(key,weight);
  }
  return map;
}

function validateCapture(c){
  if(c.schema_version!=='gtowizard-upi-capture-v1')throw new Error('capture schema_version must be gtowizard-upi-capture-v1');
  for(const k of ['id','format','payout_profile','hero_position'])if(typeof c[k]!=='string'||!c[k].trim())throw new Error(`capture.${k} missing`);
  if(!Number.isFinite(c.effective_stack_bb)||c.effective_stack_bb<=0)throw new Error('capture.effective_stack_bb invalid');
  if(!Array.isArray(c.history))throw new Error('capture.history must be array');
  if(!c.source||c.source.provider!=='GTO Wizard')throw new Error('capture.source.provider must be GTO Wizard');
  for(const k of ['solution_family','node_id'])if(typeof c.source[k]!=='string'||!c.source[k].trim())throw new Error(`capture.source.${k} missing`);
  if(typeof c.full_range_upi!=='string')throw new Error('capture.full_range_upi missing');
  if(!Array.isArray(c.actions)||!c.actions.length)throw new Error('capture.actions[] missing');
  const ids=new Set();
  for(const a of c.actions){
    if(typeof a.id!=='string'||!a.id.trim()||ids.has(a.id))throw new Error(`invalid/duplicate action id ${a.id}`);ids.add(a.id);
    if(!TYPES.has(a.type))throw new Error(`invalid action type ${a.type}`);
    if(['RAISE','JAM'].includes(a.type)&&(!Number.isFinite(a.to_bb)||a.to_bb<=0))throw new Error(`${a.id} requires to_bb`);
    if(typeof a.range_upi!=='string')throw new Error(`${a.id}.range_upi missing`);
  }
}

function maxDelta(values){if(values.length<2)return 0;return Math.max(...values)-Math.min(...values);}

export function importCapture(capture){
  validateCapture(capture);
  const full=parseExplicitUpi(capture.full_range_upi,'full_range');
  const actionMaps=new Map(capture.actions.map(a=>[a.id,parseExplicitUpi(a.range_upi,a.id)]));
  const byClass=new Map();
  let reachCombos=0;
  const aggregateMass=Object.fromEntries(capture.actions.map(a=>[a.id,0]));

  for(const combo of COMBOS){
    const fw=full.get(combo)??0;
    let sum=0;const aw={};
    for(const a of capture.actions){const w=actionMaps.get(a.id).get(combo)??0;if(w>fw+TOL)throw new Error(`${combo}: action ${a.id} weight ${w} exceeds full range weight ${fw}`);aw[a.id]=w;sum+=w;}
    if(Math.abs(sum-fw)>TOL)throw new Error(`${combo}: action-range weights sum ${sum}, full node range weight ${fw}; capture is not lossless/action-complete`);
    reachCombos+=fw;
    for(const a of capture.actions)aggregateMass[a.id]+=aw[a.id];
    const hc=classOfCombo(combo);
    if(!byClass.has(hc))byClass.set(hc,[]);
    byClass.get(hc).push({combo,fw,aw});
  }

  if(reachCombos<=TOL)throw new Error('captured node has zero reach mass');
  const hands={};
  for(const [hc,rows] of byClass){
    const fullWeights=rows.map(r=>r.fw);
    if(maxDelta(fullWeights)>SYMMETRY_TOL)throw new Error(`${hc}: preflop suit symmetry violation in full range, delta=${maxDelta(fullWeights)}`);
    for(const a of capture.actions){const conditional=rows.filter(r=>r.fw>TOL).map(r=>r.aw[a.id]/r.fw);if(maxDelta(conditional)>SYMMETRY_TOL)throw new Error(`${hc}: preflop suit symmetry violation for ${a.id}, delta=${maxDelta(conditional)}`);}
    const rw=fullWeights.reduce((x,y)=>x+y,0)/rows.length;
    if(rw<=TOL){hands[hc]={range_weight:0,strategy:null};continue;}
    const denom=rows.reduce((n,r)=>n+r.fw,0);
    const strategy={};let ss=0;
    for(const a of capture.actions){const p=rows.reduce((n,r)=>n+r.aw[a.id],0)/denom;strategy[a.id]=p;ss+=p;}
    if(Math.abs(ss-1)>TOL)throw new Error(`${hc}: derived class strategy sums ${ss}`);
    hands[hc]={range_weight:rw,strategy};
  }
  if(Object.keys(hands).length!==169)throw new Error(`derived ${Object.keys(hands).length} classes, expected 169`);

  const aggregate={};for(const a of capture.actions)aggregate[a.id]=aggregateMass[a.id]/reachCombos;
  return {
    schema_version:'exact-spin-node-v1',
    id:capture.id,
    verification_status:'VERIFIED_EXACT',
    format:capture.format,
    payout_profile:capture.payout_profile,
    effective_stack_bb:capture.effective_stack_bb,
    hero_position:capture.hero_position,
    history:capture.history,
    actions:capture.actions.map(({range_upi,...a})=>a),
    source:{
      provider:'GTO Wizard',
      solution_family:capture.source.solution_family,
      node_id:capture.source.node_id,
      extraction_method:'GTOWIZARD_UPI_FULL_PLUS_ACTION_RANGES',
      interpolated:false,
      default_filled:false,
      captured_at:capture.source.captured_at??null
    },
    reach_combos:reachCombos,
    aggregate,
    hands
  };
}

function explicitRange(weights){return [...weights.entries()].filter(([,w])=>w>TOL).map(([c,w])=>`${c}: ${w}`).join(',');}
function selfTest(){
  const full=new Map(),fold=new Map(),raise=new Map();
  for(const c of COMBOS){full.set(c,1);fold.set(c,0.4);raise.set(c,0.6);}
  const capture={schema_version:'gtowizard-upi-capture-v1',id:'selftest',format:'SPIN_3MAX',payout_profile:'TEST',effective_stack_bb:15,hero_position:'BTN',history:[],source:{provider:'GTO Wizard',solution_family:'SELFTEST',node_id:'SELFTEST'},full_range_upi:explicitRange(full),actions:[{id:'FOLD',type:'FOLD',range_upi:explicitRange(fold)},{id:'RAISE_TO_2BB',type:'RAISE',to_bb:2,range_upi:explicitRange(raise)}]};
  const out=importCapture(capture);
  if(Object.keys(out.hands).length!==169||Math.abs(out.aggregate.FOLD-0.4)>TOL||Math.abs(out.aggregate.RAISE_TO_2BB-0.6)>TOL)throw new Error('self-test aggregate mismatch');
  if(Math.abs(out.hands.AKs.strategy.RAISE_TO_2BB-0.6)>TOL)throw new Error('self-test AKs mismatch');
  const broken=structuredClone(capture);broken.actions[0].range_upi='AcAd: 0.1';
  let rejected=false;try{importCapture(broken);}catch{rejected=true;}if(!rejected)throw new Error('self-test failed to reject incomplete action capture');
  console.log('GTO Wizard UPI importer self-test PASS: 1326 combos -> 169 classes; action/full conservation and no-default guards active.');
}

if(import.meta.url===`file://${process.argv[1]}`){
  if(process.argv.includes('--self-test'))selfTest();
  else{
    const input=process.argv[2],output=process.argv[3];
    if(!input||!output){console.error('Usage: node scripts/import-gtowizard-upi-node.mjs <capture.json> <output.json> | --self-test');process.exit(2);}
    const capture=JSON.parse(fs.readFileSync(input,'utf8'));
    const node=importCapture(capture);
    fs.mkdirSync(path.dirname(output),{recursive:true});
    fs.writeFileSync(output,JSON.stringify(node,null,2)+'\n');
    console.log(`Imported ${capture.id} -> ${output}; reach_combos=${node.reach_combos.toFixed(6)}.`);
  }
}
