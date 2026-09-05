const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
const STACKS=[2,4,6,8,10,12,15,17,20,23,25];

const REFERENCE_NODES={
  '15|':{hero:'BTN',actions:['Fold','Raise 2','All In 15'],legend:[['Fold',67.22,'fold'],['Raise 2',25.41,'raise'],['All In 15',7.36,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2':{hero:'SB',actions:['Fold','Call','All In 15'],legend:[['Fold',78.31,'fold'],['Call',2.41,'call'],['All In 15',19.27,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2>SB:Call':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',59.11,'fold'],['Call',22.19,'call'],['All In 15',18.70,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Fold':{hero:'SB',actions:['Fold','Call','Raise 3','All In 15'],legend:null,status:'TREE_REFERENCE_ONLY'},
  '15|BTN:Fold>SB:Raise 3':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',47.12,'fold'],['Call',33.85,'call'],['All In 15',19.01,'jam']],status:'SCREEN_CROSSCHECK'},
  '2|HU':{hero:'BTN',actions:['Fold','Call','All In 2'],legend:[['Fold',57.16,'fold'],['Call',0.03,'call'],['All In 2',42.82,'jam']],status:'HU_SCREEN_REFERENCE'}
};

let state={stack:15,history:[],hu:false};

function handName(row,col){
  if(row===col)return RANKS[row]+RANKS[col];
  if(row<col)return RANKS[row]+RANKS[col]+'s';
  return RANKS[col]+RANKS[row]+'o';
}

function nodeKey(){
  if(state.hu&&state.stack===2)return '2|HU';
  return `${state.stack}|${state.history.map(x=>x.pos+':'+x.action).join('>')}`;
}

function currentNode(){return REFERENCE_NODES[nodeKey()]||null;}

function renderStacks(){
  const el=document.getElementById('stacks');el.innerHTML='';
  STACKS.forEach(s=>{const b=document.createElement('button');b.className='stack-btn'+(s===state.stack?' active':'');b.textContent=s;b.onclick=()=>{state={stack:s,history:[],hu:false};render();};el.appendChild(b);});
}

function renderGrid(){
  const grid=document.getElementById('grid');grid.innerHTML='';
  for(let r=0;r<13;r++)for(let c=0;c<13;c++){
    const cell=document.createElement('div');cell.className='hand unverified';
    const label=document.createElement('span');label.className='label';label.textContent=handName(r,c);cell.appendChild(label);grid.appendChild(cell);
  }
  const warning=document.getElementById('gridWarning');
  warning.textContent='Индивидуальные частоты рук ещё не загружены из VERIFIED_EXACT solver-источника. Показывать приблизительные границы запрещено.';
  warning.classList.remove('hidden');
}

function renderLegend(node){
  const el=document.getElementById('legend');el.innerHTML='';
  if(!node||!node.legend){el.textContent='Нет подтверждённых aggregate frequencies для этого узла.';return;}
  node.legend.forEach(([name,pct,cls])=>{const d=document.createElement('div');d.innerHTML=`<span class="dot ${cls}"></span>${name} (${pct.toFixed(2)}%)`;el.appendChild(d);});
}

function legalFallback(pos){
  if(pos==='BTN')return ['Fold','Raise 2','All In '+state.stack];
  if(pos==='SB')return ['Fold','Call','Raise 3','All In '+state.stack];
  return ['Fold','Call','All In '+state.stack];
}

function seats(){return state.hu?['BTN','SB']:['BTN','SB','BB'];}

function selectedAction(pos){const h=state.history.find(x=>x.pos===pos);return h?h.action:null;}

function renderTree(node){
  const tree=document.getElementById('tree');tree.innerHTML='';
  const order=seats();
  order.forEach(pos=>{
    const card=document.createElement('section');card.className='seat'+(node&&node.hero===pos?' active':'');
    const head=document.createElement('div');head.className='seat-head';head.innerHTML=`<span>${pos}</span><span>${state.stack}</span>`;card.appendChild(head);
    const actions=document.createElement('div');actions.className='seat-actions';
    const chosen=selectedAction(pos);
    let opts=[];
    if(chosen) opts=[chosen];
    else if(node&&node.hero===pos) opts=node.actions;
    else if(!node&&pos===nextPosition()) opts=legalFallback(pos);
    else opts=['—'];
    opts.forEach(a=>{const b=document.createElement('button');b.className='action-btn'+(chosen===a?' selected':'');b.textContent=a;if(a==='—'||chosen)b.disabled=true;else b.onclick=()=>advance(pos,a);actions.appendChild(b);});
    card.appendChild(actions);tree.appendChild(card);
  });
}

function nextPosition(){
  const order=seats();
  for(const p of order)if(!state.history.some(x=>x.pos===p))return p;
  return null;
}

function advance(pos,action){
  state.history=state.history.filter(x=>x.pos!==pos);
  state.history.push({pos,action});
  if(action.startsWith('All In')){
    // Keep tree on next unresolved responder; no invented strategy is inserted.
  }
  render();
}

function reset(){state.history=[];render();}

function renderStatus(node){
  const el=document.getElementById('status');
  const hist=state.history.length?state.history.map(x=>`${x.pos} ${x.action}`).join(' → '):'First in';
  const src=node?node.status:'NO_VERIFIED_NODE';
  el.innerHTML=`<strong>${state.hu?'HU':'3-MAX'} · EFF ${state.stack} BB</strong> · ${hist}<br>Chart status: <strong>${src}</strong>`;
}

function render(){
  renderStacks();renderGrid();const node=currentNode();renderLegend(node);renderTree(node);renderStatus(node);
}

document.getElementById('reset').onclick=reset;
document.getElementById('huToggle').onclick=()=>{state.hu=!state.hu;state.history=[];if(state.hu&&state.stack!==2)state.stack=2;render();};
render();