const assert=require('assert');
const Router=require('../ui/router.js');

const root={stack:15,history:[],future:[],hu:false};
assert.strictEqual(Router.canonicalNodeId(root),'3MAX/WTA/15/ROOT');
assert.strictEqual(Router.nextPosition(root),'BTN');

const a=Router.appendAction(root,'BTN','Raise 2');
assert.strictEqual(Router.canonicalNodeId(a),'3MAX/WTA/15/BTN_RAISE_2');
assert.strictEqual(Router.nextPosition(a),'SB');
assert.deepStrictEqual(root.history,[],'appendAction must not mutate previous history');

const b=Router.appendAction(a,'SB','Call');
assert.strictEqual(Router.canonicalNodeId(b),'3MAX/WTA/15/BTN_RAISE_2/SB_CALL');
assert.strictEqual(Router.nextPosition(b),'BB');

const back=Router.back(b);
assert.strictEqual(Router.canonicalNodeId(back),'3MAX/WTA/15/BTN_RAISE_2');
assert.strictEqual(back.future[0].pos,'SB');
assert.strictEqual(back.future[0].action,'Call');

const forward=Router.forward(back);
assert.strictEqual(Router.canonicalNodeId(forward),'3MAX/WTA/15/BTN_RAISE_2/SB_CALL');
assert.deepStrictEqual(forward.future,[]);

const hu={stack:2,history:[],future:[],hu:true};
assert.strictEqual(Router.canonicalNodeId(hu),'HU/WTA/2/ROOT');
assert.strictEqual(Router.legacyReferenceKey(hu),'2|HU');
assert.strictEqual(Router.nextPosition(hu),'BTN');

console.log('UI router tests passed');
