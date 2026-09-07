const assert=require('assert');
const Grid=require('../ui/grid.js');

assert.deepStrictEqual(
  Grid.normalizeStrategy({fold:0.25,call:0.25,jam:0.5}).map(x=>x.action),
  ['fold','call','jam']
);

const parts=Grid.slices({fold:0.25,call:0.25,jam:0.5});
assert.strictEqual(parts[0].leftPct,0);
assert.strictEqual(parts[0].widthPct,25);
assert.strictEqual(parts[1].leftPct,25);
assert.strictEqual(parts[2].leftPct,50);
assert.strictEqual(parts[2].widthPct,50);

assert.throws(()=>Grid.normalizeStrategy({fold:0.4,jam:0.4}),/expected 1/);
assert.throws(()=>Grid.normalizeStrategy({fold:-0.1,jam:1.1}),/invalid frequency/);
assert.throws(()=>Grid.normalizeStrategy({fold:0.5,teleport:0.5}),/unknown action/);

Grid.validateChart({AA:{fold:0.2,jam:0.8},AKs:{call:1}},['AA','AKs']);
assert.throws(()=>Grid.validateChart({AA:{jam:1}},['AA','AKs']),/missing hand AKs/);

console.log('UI grid tests passed');
