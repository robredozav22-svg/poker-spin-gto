#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

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
function classOfCombo(key){const a=key.slice(0,2),b=key.slice(2,4),ra=a[0],rb=b[0];if(ra===rb)return ra+rb;const hi=RANK_VALUE[ra]>RANK_VALUE[rb]?ra:rb,lo=hi===ra?rb:ra;return`${hi}${lo}${a[1]===b[1]?'s':'o'}`;}
function allCombos(){const out=[];for(let i=0;i<52;i++)for(let j=i+1;j<52;j++)out.push(comboKey(DECK[i],DECK[j]));return out;}
const COMBOS=allCombos();if(COMBOS.length!==1326||new Set(COMBOS).size!==1326)throw new Error('internal combo universe invalid');

function parseWeight(raw){const s=raw.trim(),pct=s.endsWith('%'),n=Number(pct?s.slice(0,-1):s);if(!Number.isFinite(n))throw new Error(`invalid weight ${raw}`);const w=pct?n/100:n;if(w<0||w>1+TOL)throw new Error(`weight out of [0,1]: ${raw}`);return Math.max(0,Math.min(1,w));}
function normalizeClass(raw){const t=raw.trim();const m=t.match(/^([2-9TJQKA])([2-9TJQKA])([so])?$/i);if(!m)return null;const a=m[1].toUpperCase(),b=m[2].toUpperCase(),kind=m[3]?.toLowerCase();if(a===b){if(kind)throw new Error(`pair class must not have suitedness suffix: ${raw}`);return a+a;}if(!kind)throw new Error(`non-pair class requires s/o suffix: ${raw}`);const hi=RANK_VALUE[a]>RANK_VALUE[b]?a:b,lo=hi===a?b:a;return`${hi}${lo}${kind}`;}
function combosForClass(h){const a=h[0],b=h[1];if(a===b){const out=[];for(let i=0;i<4;i++)for(let j=i+1;j<4;j++)out.push(comboKey(card(a,SUITS[i]),card(a,SUITS[j])));return out;}const suited=h.endsWith('s'),out=[];if(suited){for(const s of SUITS)out.push(comboKey(card(a,s),card(b,s)));}else{for(const sa of SUITS)for(const sb of SUITS)if(sa!==sb)out.push(comboKey(card(a,sa),card(b,sb)));}return out;}

export function parseUpi(text,label='range'){
  if(typeof text!=='string')throw new Error(`${label}: UPI must be string`);
  const map=new Map();const chunks=text.split(/[\n,;]+/).map(x=>x.trim()).filter(Boolean);
  for(const chunk of chunks){
    let keys=[],weight=1;
    const physical=chunk.match(/^([2-9TJQKA][cdhs])([2-9TJQKA][cdhs])(?:\s*:\s*([^\s]+))?$/i);
    if(physical){const a=`${physical[1][0].toUpperCase()}${physical[1][1].toLowerCase()}`,b=`${physical[2][0].toUpperCase()}${physical[2][1].toLowerCase()}`;keys=[comboKey(a,b)];if(physical[3]!==undefined)weight=parseWeight(physical[3]);}
    else{
      const weighted=chunk.match(/^([^:\s]+)(?:\s*:\s*([^\s]+))?$/);if(!weighted)throw new Error(`${label}: unsupported UPI token '${chunk}'`);
      const hc=normalizeClass(weighted[1]);if(!hc)throw new Error(`${label}: unsupported/non-exact UPI token '${chunk}'. Use physical combos or exact 169 classes; shorthand +/ranges are rejected.`);
      keys=combosForClass(hc);if(weighted[2]!==undefined)weight=parseWeight(weighted[2]);
    }
    for(const key of keys){if(map.has(key))throw new Error(`${label}: duplicate/overlapping combo ${key}`);map.set(key,weight);}
  }
  return map;
}
export const parseExplicitUpi=parseUpi;

function canonicalJson(value){if(Array.isArray(value))return`[${value.map(canonicalJson).join(',')}]`;if(value&&typeof value==='object'){return`{${Object.keys(value).sort().map(k=>`${JSON.stringify(k)}:${canonicalJson(value[k])}`).join(',')}}`;}return JSON.stringify(value);}
function captureSha256(capture){return`sha256:${crypto.createHash('sha256').update(canonicalJson(capture),'utf8').digest('hex')}`;}

function validateCapture(c){
  if(c.schema_version!=='gtowizard-upi-capture-v1')throw new Error('capture schema_version must be gtowizard-upi-capture-v1');
  for(const k of ['id','format','payout_profile','ante_profile','hero_position'])if(typeof c[k]!=='string'||!c[k].trim())throw new Error(`capture.${k} missing`);
  if(!Number.isFinite(c.effective_stack_bb)||c.effective_stack_bb<=0)throw new Error('capture.effective_stack_bb invalid');
  if(!Array.isArray(c.history))throw new Error('capture.history must be array');
  if(!c.source||c.source.provider!=='GTO Wizard')throw new Error('capture.source.provider must be GTO Wizard');
  for(const k of ['solution_family','node_id','captured_at'])if(typeof c.source[k]!=='string'||!c.source[k].trim())throw new Error(`capture.source.${k} missing`);
  const captured=Date.parse(c.source.captured_at);if(!Number.isFinite(captured)||new Date(captured).getUTCFullYear()!==2026)throw new Error('capture.source.captured_at must be a valid live-2026 timestamp');
  if(typeof c.full_range_upi!=='string')throw new Error('capture.full_range_upi missing');
  if(!Array.isArray(c.actions)||!c.actions.length)throw new Error('capture.actions[] missing');
  const ids=new Set();for(const a of c.actions){if(typeof a.id!=='string'||!a.id.trim()||ids.has(a.id))throw new Error(`invalid/duplicate action id ${a.id}`);ids.add(a.id);if(!TYPES.has(a.type))throw new Error(`invalid action type ${a.type}`);if(['RAISE','JAM'].includes(a.type)&&(!Number.isFinite(a.to_bb)||a.to_bb<=0))throw new Error(`${a.id} requires to_bb`);if(typeof a.range_upi!=='string')throw new Error(`${a.id}.range_upi missing`);}
}
function maxDelta(values){if(values.length<2)return 0;return Math.max(...values)-Math.min(...values);}

export function importCapture(capture){
  validateCapture(capture);
  const captureHash=captureSha256(capture);
  const full=parseUpi(capture.full_range_upi,'full_range');
  const actionMaps=new Map(capture.actions.map(a=>[a.id,parseUpi(a.range_upi,a.id)]));
  const byClass=new Map();let reachCombos=0;const aggregateMass=Object.fromEntries(capture.actions.map(a=>[a.id,0]));
  for(const combo of COMBOS){const fw=full.get(combo)??0;let sum=0;const aw={};for(const a of capture.actions){const w=actionMaps.get(a.id).get(combo)??0;if(w>fw+TOL)throw new Error(`${combo}: action ${a.id} weight ${w} exceeds full range weight ${fw}`);aw[a.id]=w;sum+=w;}if(Math.abs(sum-fw)>TOL)throw new Error(`${combo}: action-range weights sum ${sum}, full node range weight ${fw}; capture is not lossless/action-complete`);reachCombos+=fw;for(const a of capture.actions)aggregateMass[a.id]+=aw[a.id];const hc=classOfCombo(combo);if(!byClass.has(hc))byClass.set(hc,[]);byClass.get(hc).push({combo,fw,aw});}
  if(reachCombos<=TOL)throw new Error('captured node has zero reach mass');
  const hands={};
  for(const [hc,rows] of byClass){const fullWeights=rows.map(r=>r.fw);if(maxDelta(fullWeights)>SYMMETRY_TOL)throw new Error(`${hc}: preflop suit symmetry violation in full range, delta=${maxDelta(fullWeights)}`);for(const a of capture.actions){const conditional=rows.filter(r=>r.fw>TOL).map(r=>r.aw[a.id]/r.fw);if(maxDelta(conditional)>SYMMETRY_TOL)throw new Error(`${hc}: preflop suit symmetry violation for ${a.id}, delta=${maxDelta(conditional)}`);}const rw=fullWeights.reduce((x,y)=>x+y,0)/rows.length;if(rw<=TOL){hands[hc]={range_weight:0,strategy:null};continue;}const denom=rows.reduce((n,r)=>n+r.fw,0);const strategy={};let ss=0;for(const a of capture.actions){const p=rows.reduce((n,r)=>n+r.aw[a.id],0)/denom;strategy[a.id]=p;ss+=p;}if(Math.abs(ss-1)>TOL)throw new Error(`${hc}: derived class strategy sums ${ss}`);hands[hc]={range_weight:rw,strategy};}
  if(Object.keys(hands).length!==169)throw new Error(`derived ${Object.keys(hands).length} classes, expected 169`);
  const aggregate={};for(const a of capture.actions)aggregate[a.id]=aggregateMass[a.id]/reachCombos;
  const nonzero=m=>[...m.values()].filter(w=>w>TOL).length;
  return {
    schema_version:'exact-spin-node-v1',id:capture.id,verification_status:'VERIFIED_EXACT',format:capture.format,payout_profile:capture.payout_profile,ante_profile:capture.ante_profile,effective_stack_bb:capture.effective_stack_bb,hero_position:capture.hero_position,history:capture.history,actions:capture.actions.map(({range_upi,...a})=>a),
    source:{provider:'GTO Wizard',solution_family:capture.source.solution_family,node_id:capture.source.node_id,extraction_method:'GTOWIZARD_UPI_FULL_PLUS_ACTION_RANGES',interpolated:false,default_filled:false,captured_at:capture.source.captured_at},
    import_audit:{importer:'import-gtowizard-upi-node-v2',capture_sha256:captureHash,upi_parser:'PHYSICAL_OR_EXACT_169_CLASS_V1',physical_combos_checked:1326,hand_classes_derived:169,action_conservation:'PASS',suit_symmetry:'PASS',zero_reach_policy:'EXPLICIT_NULL',full_range_nonzero_physical_combos:nonzero(full),action_nonzero_physical_combos:Object.fromEntries(capture.actions.map(a=>[a.id,nonzero(actionMaps.get(a.id))])),probability_tolerance:TOL,symmetry_tolerance:SYMMETRY_TOL},
    reach_combos:reachCombos,aggregate,hands
  };
}

function explicitRange(weights){return[...weights.entries()].filter(([,w])=>w>TOL).map(([c,w])=>`${c}: ${w}`).join(',');}
function selfTest(){
  if(parseUpi('AA: 0.5').size!==6||parseUpi('AKs: 0.25').size!==4||parseUpi('AKo: 0.75').size!==12)throw new Error('169-class UPI expansion failed');
  let shorthandRejected=false;try{parseUpi('22+');}catch{shorthandRejected=true;}if(!shorthandRejected)throw new Error('shorthand must be rejected');
  const full=new Map(),fold=new Map(),raise=new Map();for(const c of COMBOS){full.set(c,1);fold.set(c,0.4);raise.set(c,0.6);}
  const capture={schema_version:'gtowizard-upi-capture-v1',id:'selftest',format:'SPIN_3MAX',payout_profile:'TEST',ante_profile:'NONE',effective_stack_bb:15,hero_position:'BTN',history:[],source:{provider:'GTO Wizard',solution_family:'SELFTEST',node_id:'SELFTEST',captured_at:'2026-09-08T00:00:00Z'},full_range_upi:explicitRange(full),actions:[{id:'FOLD',type:'FOLD',range_upi:explicitRange(fold)},{id:'RAISE_TO_2BB',type:'RAISE',to_bb:2,range_upi:explicitRange(raise)}]};
  const out=importCapture(capture);if(Object.keys(out.hands).length!==169||Math.abs(out.aggregate.FOLD-0.4)>TOL||Math.abs(out.aggregate.RAISE_TO_2BB-0.6)>TOL)throw new Error('self-test aggregate mismatch');if(Math.abs(out.hands.AKs.strategy.RAISE_TO_2BB-0.6)>TOL)throw new Error('self-test AKs mismatch');if(out.import_audit.physical_combos_checked!==1326||out.import_audit.action_conservation!=='PASS'||!/^sha256:[0-9a-f]{64}$/.test(out.import_audit.capture_sha256))throw new Error('self-test import audit missing');
  const broken=structuredClone(capture);broken.actions[0].range_upi='AcAd: 0.1';let rejected=false;try{importCapture(broken);}catch{rejected=true;}if(!rejected)throw new Error('self-test failed to reject incomplete action capture');
  console.log('GTO Wizard UPI importer self-test PASS: exact class/physical syntax -> 1326 combos -> 169 classes; provenance/conservation/no-default guards active.');
}

if(import.meta.url===`file://${process.argv[1]}`){if(process.argv.includes('--self-test'))selfTest();else{const input=process.argv[2],output=process.argv[3];if(!input||!output){console.error('Usage: node scripts/import-gtowizard-upi-node.mjs <capture.json> <output.json> | --self-test');process.exit(2);}const capture=JSON.parse(fs.readFileSync(input,'utf8'));const node=importCapture(capture);fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,JSON.stringify(node,null,2)+'\n');console.log(`Imported ${capture.id} -> ${output}; reach_combos=${node.reach_combos.toFixed(6)} capture=${node.import_audit.capture_sha256}.`);}}
