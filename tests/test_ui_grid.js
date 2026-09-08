const assert=require('assert');
const Grid=require('../ui/grid.js');

assert.deepStrictEqual(Grid.normalizeStrategy({fold:0.25,call:0.25,jam:0.5}).map(x=>x.action),['fold','call','jam']);
assert.strictEqual(Grid.actionClass('RAISE_TO_2BB'),'raise');
assert.strictEqual(Grid.actionClass('JAM_TO_15BB'),'jam');
assert.strictEqual(Grid.actionClass('FOLD'),'fold');

const parts=Grid.slices({FOLD:0.25,CALL:0.25,JAM_TO_15BB:0.5});
assert.strictEqual(parts[0].leftPct,0);assert.strictEqual(parts[0].widthPct,25);assert.strictEqual(parts[1].leftPct,25);assert.strictEqual(parts[2].leftPct,50);assert.strictEqual(parts[2].widthPct,50);
assert.throws(()=>Grid.normalizeStrategy({FOLD:0.4,JAM_TO_15BB:0.4}),/expected 1/);
assert.throws(()=>Grid.normalizeStrategy({FOLD:-0.1,JAM_TO_15BB:1.1}),/invalid frequency/);
assert.throws(()=>Grid.normalizeStrategy({FOLD:0.5,TELEPORT:0.5}),/unknown action/);

Grid.validateExactHands({AA:{range_weight:1,strategy:{FOLD:0.2,JAM_TO_15BB:0.8}},AKs:{range_weight:0,strategy:null}},['AA','AKs']);
assert.throws(()=>Grid.validateExactHands({AA:{range_weight:1,strategy:{JAM_TO_15BB:1}},AKs:{range_weight:0,strategy:{FOLD:1}}},['AA','AKs']),/strategy:null/);
assert.throws(()=>Grid.validateExactHands({AA:{range_weight:1,strategy:{JAM_TO_15BB:1}}},['AA','AKs']),/missing hand AKs/);

console.log('UI exact-grid tests passed');
