(function(global){
  const DEFAULT_PAYOUT_PROFILE='WTA_CHIPEV_BASELINE';

  function cloneHistory(history){return (history||[]).map(x=>({pos:x.pos,action:x.action}));}
  function normalizeMode(hu){return hu?'HU':'3MAX';}
  function normalizeToken(value){return String(value).trim().toUpperCase().replaceAll(' ','_').replaceAll('.','_');}

  function canonicalNodeId({stack,history=[],hu=false,payout_profile,payout}){
    const mode=normalizeMode(hu);
    const profile=normalizeToken(payout_profile||payout||DEFAULT_PAYOUT_PROFILE);
    const actionPath=history.length
      ? history.map(x=>`${x.pos}_${normalizeToken(x.action)}`).join('/')
      : 'ROOT';
    return `${mode}/${profile}/${Number(stack)}/${actionPath}`;
  }

  function exactActionLabel(action){
    const type=String(action?.type||'').toUpperCase();
    if(type==='FOLD')return'Fold';
    if(type==='CALL')return'Call';
    if(type==='CHECK')return'Check';
    if(type==='LIMP')return'Limp';
    if(type==='RAISE'){
      if(!Number.isFinite(Number(action.to_bb)))throw new Error('RAISE exact history requires to_bb');
      return`Raise ${Number(action.to_bb)}`;
    }
    if(type==='JAM'){
      if(!Number.isFinite(Number(action.to_bb)))throw new Error('JAM exact history requires to_bb');
      return`All In ${Number(action.to_bb)}`;
    }
    throw new Error(`unsupported exact history action ${type}`);
  }

  function canonicalNodeIdFromExactNode(node){
    if(!node||typeof node!=='object')throw new Error('exact node object required');
    const hu=node.format==='SPIN_HU';
    if(node.format!=='SPIN_3MAX'&&!hu)throw new Error(`unsupported exact format ${node.format}`);
    if(!Number.isFinite(Number(node.effective_stack_bb)))throw new Error('exact node effective_stack_bb required');
    const history=(node.history||[]).map(x=>({pos:x.actor,action:exactActionLabel(x)}));
    return canonicalNodeId({stack:node.effective_stack_bb,history,hu,payout_profile:node.payout_profile});
  }

  function legacyReferenceKey({stack,history=[],hu=false}){
    if(hu&&Number(stack)===2)return '2|HU';
    return `${Number(stack)}|${history.map(x=>x.pos+':'+x.action).join('>')}`;
  }

  function nextPosition({history=[],hu=false}){
    const seats=hu?['BTN','SB']:['BTN','SB','BB'];
    return seats.find(p=>!history.some(x=>x.pos===p))||null;
  }

  function appendAction(route,pos,action){
    return {...route,history:[...cloneHistory((route.history||[]).filter(x=>x.pos!==pos)),{pos,action}],future:[]};
  }
  function back(route){const history=cloneHistory(route.history||[]);if(!history.length)return {...route};const popped=history.pop();return {...route,history,future:[popped,...(route.future||[])]};}
  function forward(route){const future=route.future||[];if(!future.length)return {...route};const [first,...rest]=future;return {...route,history:[...cloneHistory(route.history||[]),{...first}],future:rest};}

  const api={DEFAULT_PAYOUT_PROFILE,cloneHistory,canonicalNodeId,canonicalNodeIdFromExactNode,exactActionLabel,legacyReferenceKey,nextPosition,appendAction,back,forward};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  global.SpinsRouter=api;
})(typeof window!=='undefined'?window:globalThis);
