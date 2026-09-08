(function(global){
  const INDEX_URL='data/charts/exact-index.json';
  let indexPromise=null;
  const cache=new Map();

  function loadIndex(){
    if(!indexPromise){
      indexPromise=fetch(INDEX_URL,{cache:'no-store'}).then(r=>{
        if(!r.ok)throw new Error(`exact index HTTP ${r.status}`);
        return r.json();
      }).then(doc=>{
        if(doc?.schema_version!=='exact-spin-runtime-index-v1'||!doc.nodes||typeof doc.nodes!=='object')throw new Error('invalid exact runtime index');
        return doc;
      });
    }
    return indexPromise;
  }

  function getCached(key){return cache.get(key)||null;}
  function hasIndexedNode(key){return loadIndex().then(i=>Object.prototype.hasOwnProperty.call(i.nodes,key));}
  function loadNode(key){
    if(cache.has(key))return Promise.resolve(cache.get(key));
    return loadIndex().then(index=>{
      const entry=index.nodes[key];
      if(!entry)return null;
      if(typeof entry.path!=='string'||!entry.path.startsWith('data/charts/exact/')||!entry.path.endsWith('.json'))throw new Error(`invalid exact node path for ${key}`);
      return fetch(entry.path,{cache:'no-store'}).then(r=>{
        if(!r.ok)throw new Error(`exact node HTTP ${r.status}: ${entry.path}`);
        return r.json();
      }).then(node=>{
        if(node?.verification_status!=='VERIFIED_EXACT')throw new Error(`runtime refused non-VERIFIED_EXACT node ${key}`);
        cache.set(key,node);
        return node;
      });
    });
  }
  function reset(){indexPromise=null;cache.clear();}

  const api={loadIndex,loadNode,getCached,hasIndexedNode,reset};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  global.SpinsExactStore=api;
})(typeof window!=='undefined'?window:globalThis);
