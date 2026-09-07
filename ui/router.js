(function(global){
  function cloneHistory(history){return (history||[]).map(x=>({pos:x.pos,action:x.action}));}

  function normalizeMode(hu){return hu?'HU':'3MAX';}

  function canonicalNodeId({stack,history=[],hu=false,payout='WTA'}){
    const mode=normalizeMode(hu);
    const actionPath=history.length
      ? history.map(x=>`${x.pos}_${String(x.action).toUpperCase().replaceAll(' ','_')}`).join('/')
      : 'ROOT';
    return `${mode}/${payout}/${Number(stack)}/${actionPath}`;
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
    return {
      ...route,
      history:[...cloneHistory((route.history||[]).filter(x=>x.pos!==pos)),{pos,action}],
      future:[]
    };
  }

  function back(route){
    const history=cloneHistory(route.history||[]);
    if(!history.length)return {...route};
    const popped=history.pop();
    return {...route,history,future:[popped,...(route.future||[])]};
  }

  function forward(route){
    const future=route.future||[];
    if(!future.length)return {...route};
    const [first,...rest]=future;
    return {...route,history:[...cloneHistory(route.history||[]),{...first}],future:rest};
  }

  const api={cloneHistory,canonicalNodeId,legacyReferenceKey,nextPosition,appendAction,back,forward};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  global.SpinsRouter=api;
})(typeof window!=='undefined'?window:globalThis);
