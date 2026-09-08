(function(global){
  const INDEX_URL='data/charts/exact-index.json';
  const PROMOTION_URL='data/charts/exact-promotion-manifest.json';
  let indexPromise=null,promotionPromise=null;
  const cache=new Map();

  async function sha256Text(text){
    if(!global.crypto?.subtle)throw new Error('WebCrypto SHA-256 unavailable; exact charts fail closed');
    const bytes=new TextEncoder().encode(text);
    const digest=await global.crypto.subtle.digest('SHA-256',bytes);
    const hex=[...new Uint8Array(digest)].map(b=>b.toString(16).padStart(2,'0')).join('');
    return`sha256:${hex}`;
  }

  function validateIndexEntry(key,entry){
    if(!entry||typeof entry!=='object')throw new Error(`invalid exact index entry for ${key}`);
    if(typeof entry.path!=='string'||!entry.path.startsWith('data/charts/exact/')||!entry.path.endsWith('.json'))throw new Error(`invalid exact node path for ${key}`);
    if(typeof entry.artifact_id!=='string'||!entry.artifact_id.trim())throw new Error(`missing exact artifact_id for ${key}`);
    if(typeof entry.sha256!=='string'||!/^sha256:[0-9a-f]{64}$/.test(entry.sha256))throw new Error(`invalid exact sha256 for ${key}`);
    if(typeof entry.captured_at!=='string'||!entry.captured_at.trim())throw new Error(`missing exact captured_at for ${key}`);
    return entry;
  }

  function validatePromotionProof(key,entry,p){
    if(!p||typeof p!=='object')throw new Error(`missing promotion proof for ${key}`);
    if(p.status!=='PROMOTED_2026_EXACT')throw new Error(`runtime refused unpromoted exact node ${key}`);
    if(p.artifact_id!==entry.artifact_id||p.path!==entry.path||p.sha256!==entry.sha256||p.captured_at!==entry.captured_at)throw new Error(`promotion/index provenance mismatch for ${key}`);
    const c=p.checks||{};
    for(const name of ['live_2026_source','exact_node_schema','upi_action_conservation','zero_reach_guard','suit_symmetry','aggregate_recompute','canonical_route','artifact_sha256'])if(c[name]!=='PASS')throw new Error(`promotion check ${name} not PASS for ${key}`);
    if(c.independent_crosscheck!=='PASS_COMPATIBLE'||c.unexplained_numeric_disagreement!==false)throw new Error(`independent promotion cross-check not passed for ${key}`);
    const x=p.crosscheck||{};
    if(x.compatibility_resolved!==true||!['ROUNDING_ONLY','SOLVER_TOLERANCE','NONE'].includes(x.classification))throw new Error(`runtime cross-check classification not promotable for ${key}`);
    return p;
  }

  function loadIndex(){
    if(!indexPromise)indexPromise=fetch(INDEX_URL,{cache:'no-store'}).then(r=>{if(!r.ok)throw new Error(`exact index HTTP ${r.status}`);return r.json();}).then(doc=>{if(doc?.schema_version!=='exact-spin-runtime-index-v1'||!doc.nodes||typeof doc.nodes!=='object'||Array.isArray(doc.nodes))throw new Error('invalid exact runtime index');for(const [key,entry] of Object.entries(doc.nodes))validateIndexEntry(key,entry);return doc;});
    return indexPromise;
  }
  function loadPromotion(){
    if(!promotionPromise)promotionPromise=fetch(PROMOTION_URL,{cache:'no-store'}).then(r=>{if(!r.ok)throw new Error(`promotion manifest HTTP ${r.status}`);return r.json();}).then(doc=>{if(doc?.schema_version!=='exact-spin-promotion-manifest-v1'||!doc.nodes||typeof doc.nodes!=='object'||Array.isArray(doc.nodes))throw new Error('invalid exact promotion manifest');return doc;});
    return promotionPromise;
  }

  function getCached(key){return cache.get(key)||null;}
  function hasIndexedNode(key){return loadIndex().then(i=>Object.prototype.hasOwnProperty.call(i.nodes,key));}
  async function loadNode(key){
    if(cache.has(key))return cache.get(key);
    const [index,promotion]=await Promise.all([loadIndex(),loadPromotion()]);
    const rawEntry=index.nodes[key];if(!rawEntry)return null;
    const entry=validateIndexEntry(key,rawEntry);
    validatePromotionProof(key,entry,promotion.nodes[entry.artifact_id]);
    const response=await fetch(entry.path,{cache:'no-store'});if(!response.ok)throw new Error(`exact node HTTP ${response.status}: ${entry.path}`);
    const text=await response.text();const actual=await sha256Text(text);if(actual!==entry.sha256)throw new Error(`exact checksum mismatch for ${key}`);
    let node;try{node=JSON.parse(text);}catch(e){throw new Error(`invalid exact node JSON for ${key}: ${e.message}`);}
    if(node?.verification_status!=='VERIFIED_EXACT')throw new Error(`runtime refused non-VERIFIED_EXACT node ${key}`);
    if(node.id!==entry.artifact_id)throw new Error(`exact artifact_id mismatch for ${key}`);
    if(node.source?.captured_at!==entry.captured_at)throw new Error(`exact captured_at mismatch for ${key}`);
    cache.set(key,node);return node;
  }
  function reset(){indexPromise=null;promotionPromise=null;cache.clear();}

  const api={INDEX_URL,PROMOTION_URL,sha256Text,validateIndexEntry,validatePromotionProof,loadIndex,loadPromotion,loadNode,getCached,hasIndexedNode,reset};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  global.SpinsExactStore=api;
})(typeof window!=='undefined'?window:globalThis);
