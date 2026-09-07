(function(global){
  const ACTION_CLASSES={
    fold:'fold',call:'call',raise:'raise',jam:'jam',limp:'limp',check:'check'
  };

  function normalizeStrategy(strategy,tolerance=1e-6){
    if(!strategy||typeof strategy!=='object')throw new Error('strategy object required');
    const entries=Object.entries(strategy).filter(([,v])=>Number(v)>0);
    if(!entries.length)throw new Error('strategy has no positive actions');
    let sum=0;
    for(const [action,value] of entries){
      const v=Number(value);
      if(!Number.isFinite(v)||v<0||v>1)throw new Error(`invalid frequency for ${action}`);
      if(!ACTION_CLASSES[action])throw new Error(`unknown action ${action}`);
      sum+=v;
    }
    if(Math.abs(sum-1)>tolerance)throw new Error(`strategy frequencies sum to ${sum}, expected 1`);
    return entries.map(([action,value])=>({action,value:Number(value),className:ACTION_CLASSES[action]}));
  }

  function slices(strategy){
    const normalized=normalizeStrategy(strategy);
    let left=0;
    return normalized.map(item=>{
      const width=item.value*100;
      const out={...item,leftPct:left,widthPct:width};
      left+=width;
      return out;
    });
  }

  function renderCell(cell,label,strategy){
    cell.innerHTML='';
    cell.classList.remove('unverified');
    for(const slice of slices(strategy)){
      const el=document.createElement('span');
      el.className=`slice action-${slice.className}`;
      el.style.left=`${slice.leftPct}%`;
      el.style.width=`${slice.widthPct}%`;
      el.dataset.action=slice.action;
      el.dataset.frequency=String(slice.value);
      cell.appendChild(el);
    }
    const text=document.createElement('span');text.className='label';text.textContent=label;cell.appendChild(text);
  }

  function validateChart(chart,handNames){
    if(!chart||typeof chart!=='object')throw new Error('chart object required');
    for(const hand of handNames){
      if(!chart[hand])throw new Error(`missing hand ${hand}`);
      normalizeStrategy(chart[hand]);
    }
    const extras=Object.keys(chart).filter(h=>!handNames.includes(h));
    if(extras.length)throw new Error(`unexpected hand ${extras[0]}`);
    return true;
  }

  const api={normalizeStrategy,slices,renderCell,validateChart,ACTION_CLASSES};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  global.SpinsGrid=api;
})(typeof window!=='undefined'?window:globalThis);
