const assert=require('assert');
const Router=require('../ui/router.js');

const root={stack:15,history:[],future:[],hu:false};
assert.strictEqual(Router.canonicalNodeId(root),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/ROOT');
assert.strictEqual(Router.nextPosition(root),'BTN');

const a=Router.appendAction(root,'BTN','Raise 2');
assert.strictEqual(Router.canonicalNodeId(a),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2');
assert.strictEqual(Router.nextPosition(a),'SB');
assert.deepStrictEqual(root.history,[],'appendAction must not mutate previous history');

const b=Router.appendAction(a,'SB','Call');
assert.strictEqual(Router.canonicalNodeId(b),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2/SB_CALL');
assert.strictEqual(Router.nextPosition(b),'BB');

const back=Router.back(b);
assert.strictEqual(Router.canonicalNodeId(back),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2');
assert.strictEqual(back.future[0].pos,'SB');
assert.strictEqual(back.future[0].action,'Call');
const forward=Router.forward(back);
assert.strictEqual(Router.canonicalNodeId(forward),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2/SB_CALL');

const anteRoot={...root,ante_profile:'SPIN_PLUS_ANTE_0_2BB'};
assert.strictEqual(Router.canonicalNodeId(anteRoot),'3MAX/WTA_CHIPEV_BASELINE/SPIN_PLUS_ANTE_0_2BB/15/ROOT');
assert.notStrictEqual(Router.canonicalNodeId(anteRoot),Router.canonicalNodeId(root));

const hu={stack:2,history:[],future:[],hu:true};
assert.strictEqual(Router.canonicalNodeId(hu),'HU/WTA_CHIPEV_BASELINE/NONE/2/ROOT');
assert.strictEqual(Router.legacyReferenceKey(hu),'2|HU');

const exactRoot={format:'SPIN_3MAX',payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',effective_stack_bb:15,history:[]};
assert.strictEqual(Router.canonicalNodeIdFromExactNode(exactRoot),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/ROOT');
const exactResponse={format:'SPIN_3MAX',payout_profile:'WTA_CHIPEV_BASELINE',ante_profile:'NONE',effective_stack_bb:15,history:[{actor:'BTN',type:'RAISE',to_bb:2},{actor:'SB',type:'JAM',to_bb:15}]};
assert.strictEqual(Router.canonicalNodeIdFromExactNode(exactResponse),'3MAX/WTA_CHIPEV_BASELINE/NONE/15/BTN_RAISE_2/SB_ALL_IN_15');
assert.throws(()=>Router.canonicalNodeIdFromExactNode({...exactRoot,ante_profile:''}),/ante_profile required/);
assert.strictEqual(Router.exactActionLabel({type:'LIMP'}),'Limp');
assert.throws(()=>Router.exactActionLabel({type:'RAISE'}),/requires to_bb/);
console.log('UI router tests passed');
