const assert=require('assert');
const Store=require('../ui/exact-store.js');

(async()=>{
  const hash=await Store.sha256Text('abc');
  assert.strictEqual(hash,'sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  const good={path:'data/charts/exact/node.json',artifact_id:'node-v1',sha256:hash,captured_at:'2026-09-08T00:00:00Z'};
  assert.strictEqual(Store.validateIndexEntry('key',good),good);
  assert.throws(()=>Store.validateIndexEntry('key',{...good,sha256:'sha256:bad'}),/invalid exact sha256/);
  assert.throws(()=>Store.validateIndexEntry('key',{...good,path:'data/reference.json'}),/invalid exact node path/);
  assert.throws(()=>Store.validateIndexEntry('key',{...good,artifact_id:''}),/missing exact artifact_id/);
  assert.throws(()=>Store.validateIndexEntry('key',{...good,captured_at:''}),/missing exact captured_at/);
  console.log('Exact runtime store integrity tests passed');
})().catch(err=>{console.error(err);process.exit(1);});
