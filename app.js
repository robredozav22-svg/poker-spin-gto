const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
const STACKS=[2,4,6,8,10,12,15,17,20,23,25];
const RECENT_KEY='spins-v45-recent';

const REFERENCE_NODES={
  '15|':{hero:'BTN',actions:['Fold','Raise 2','All In 15'],legend:[['Fold',67.22,'fold'],['Raise 2',25.41,'raise'],['All In 15',7.36,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2':{hero:'SB',actions:['Fold','Call','All In 15'],legend:[['Fold',78.31,'fold'],['Call',2.41,'call'],['All In 15',19.27,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2>SB:Call':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',59.11,'fold'],['Call',22.19,'call'],['All In 15',18.70,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Fold':{hero:'SB',actions:['Fold','Call','Raise 3','All In 15'],legend:null,status:'TREE_REFERENCE_ONLY'},
  '15|BTN:Fold>SB:Raise 3':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',47.12,'fold'],['Call',33.85,'call'],['All In 15',19.01,'jam']],status:'SCREEN_CROSSCHECK'},
  '2|HU':{hero:'BTN',actions:['Fold','Call','All In 2'],legend:[['Fold',57.16,'fold'],['Call',0.03,'call'],['All In 2',42.82,'jam']],status:'HU_SCREEN_REFERENCE'}
};

let state={stack:15,history:[],future:[],hu:false,recentOpen:false};

function cloneHistory(history){return history.map(x=>({pos:x.pos,action:x.action}));}
function handName(row,col){if(row===col)return RANKS[row]+RANKS[col];if(row<col)return RANKS[row]+RANKS[col]+'s';return RANKS[col]+RANKS[row]+'o';}
function nodeKeyFor(stack,history,hu){if(hu&&stack===2)return '2|HU';return `${stack}|${history.map(x=>x.pos+':'+x.action).join('>')}`;}
function nodeKey(){return nodeKeyFor(state.stack,state.history,state.hu);}
function currentNode(){return REFERENCE_NODES[nodeKey()]||null;}

function compactStatus(raw){
  if(raw==='VERIFIED_EXACT')return ['EXACT','exact'];
  if(raw==='SCREEN_CROSSCHECK'||raw==='HU_SCREEN_REFERENCE')return ['CROSS-CHECK','cross'];
  if(raw==='SOLVER_APPROX'||raw==='TREE_REFERENCE_ONLY')return ['APPROX','approx'];
  return ['MISSING','missing'];
}

function readRecents(){try{return JSON.parse(localStorage.getItem(RECENT_KEY)||'[]');}catch{return [];}}
function writeRecents(items){localStorage.setItem(RECENT_KEY,JSON.stringify(items.slice(0,8)));}
function rememberCurrent(){
  if(state.history.length===0)return;
  const entry={stack:state.stack,hu:state.hu,history:cloneHistory(state.history),key:nodeKey(),ts:Date.now()};
  const recents=readRecents().filter(x=>x.key!==entry.key);
  writeRecents([entry,...recents]);
}
function labelRoute(item){const prefix=`${item.hu?'HU':'3M'} ${item.stack}BB`;const hist=item.history.length?item.history.map(x=>`${x.pos} ${x.action}`).join(' / '):'First in';return `${prefix} · ${hist}`;}

function renderStacks(){
  const el=document.getElementById('stacks');el.innerHTML='';
  STACKS.forEach(s=>{const b=document.createElement('button');b.className='stack-btn'+(s===state.stack?' active':'');b.textContent=s;b.onclick=()=>{state={...state,stack:s,history:[],future:[],hu:false};render();};el.appendChild(b);});
}

function renderGrid(){
  const grid=document.getElementById('grid');grid.innerHTML='';
  for(let r=0;r<13;r++)for(let c=0;c<13;c++){
    const cell=document.createElement('div');cell.className='hand unverified';
    const label=document.createElement('span');label.className='label';label.textContent=handName(r,c);cell.appendChild(label);grid.appendChild(cell);
  }
  const warning=document.getElementById('gridWarning');
  warning.textContent='Нет VERIFIED_EXACT hand frequencies для этого узла. Приблизительные границы не показываются.';
  warning.classList.remove('hidden');
}

function renderLegend(node){
  const el=document.getElementById('legend');el.innerHTML='';
  if(!node||!node.legend){el.textContent='Нет подтверждённых aggregate frequencies для этого узла.';return;}
  node.legend.forEach(([name,pct,cls])=>{const d=document.createElement('div');d.innerHTML=`<span class="dot ${cls}"></span>${name} (${pct.toFixed(2)}%)`;el.appendChild(d);});
}

function legalFallback(pos){if(pos==='BTN')return ['Fold','Raise 2','All In '+state.stack];if(pos==='SB')return ['Fold','Call','Raise 3','All In '+state.stack];return ['Fold','Call','All In '+state.stack];}
function seats(){return state.hu?['BTN','SB']:['BTN','SB','BB'];}
function selectedAction(pos){const h=state.history.find(x=>x.pos===pos);return h?h.action:null;}
function nextPosition(){for(const p of seats())if(!state.history.some(x=>x.pos===p))return p;return null;}

function renderTree(node){
  const tree=document.getElementById('tree');tree.innerHTML='';
  seats().forEach(pos=>{
    const card=document.createElement('section');card.className='seat'+(node&&node.hero===pos?' active':'');
    const head=document.createElement('div');head.className='seat-head';head.innerHTML=`<span>${pos}</span><span>${state.stack}</span>`;card.appendChild(head);
    const actions=document.createElement('div');actions.className='seat-actions';
    const chosen=selectedAction(pos);let opts=[];
    if(chosen)opts=[chosen];else if(node&&node.hero===pos)opts=node.actions;else if(!node&&pos===nextPosition())opts=legalFallback(pos);else opts=['—'];
    opts.forEach(a=>{const b=document.createElement('button');b.className='action-btn'+(chosen===a?' selected':'');b.textContent=a;if(a==='—'||chosen)b.disabled=true;else b.onclick=()=>advance(pos,a);actions.appendChild(b);});
    card.appendChild(actions);tree.appendChild(card);
  });
}

function advance(pos,action){
  const next=cloneHistory(state.history.filter(x=>x.pos!==pos));next.push({pos,action});
  state={...state,history:next,future:[]};rememberCurrent();render();
}
function goBack(){
  if(!state.history.length)return;
  const next=cloneHistory(state.history);const popped=next.pop();state={...state,history:next,future:[popped,...state.future]};render();
}
function goForward(){
  if(!state.future.length)return;
  const [first,...rest]=state.future;state={...state,history:[...cloneHistory(state.history),{...first}],future:rest};rememberCurrent();render();
}
function reset(){state={...state,history:[],future:[]};render();}
function restoreRecent(item){state={...state,stack:item.stack,hu:item.hu,history:cloneHistory(item.history),future:[],recentOpen:false};render();}

function renderRecent(){
  const panel=document.getElementById('recentPanel');panel.innerHTML='';panel.classList.toggle('hidden',!state.recentOpen);
  if(!state.recentOpen)return;
  const recents=readRecents();
  if(!recents.length){panel.textContent='Недавних узлов пока нет.';return;}
  recents.forEach(item=>{const b=document.createElement('button');b.className='recent-item';b.textContent=labelRoute(item);b.onclick=()=>restoreRecent(item);panel.appendChild(b);});
}

function renderStatus(node){
  const el=document.getElementById('status');const hist=state.history.length?state.history.map(x=>`${x.pos} ${x.action}`).join(' → '):'First in';
  const raw=node?node.status:'NO_VERIFIED_NODE';const [label,cls]=compactStatus(raw);
  el.innerHTML=`<div class="status-main"><strong>${state.hu?'HU':'3-MAX'} · EFF ${state.stack} BB</strong><span class="source-badge ${cls}">${label}</span></div><div>${hist}</div><div class="status-raw">${raw}</div>`;
}

function renderNav(){document.getElementById('backBtn').disabled=!state.history.length;document.getElementById('forwardBtn').disabled=!state.future.length;}
function render(){renderStacks();renderGrid();const node=currentNode();renderLegend(node);renderTree(node);renderStatus(node);renderNav();renderRecent();}

document.getElementById('reset').onclick=reset;
document.getElementById('backBtn').onclick=goBack;
document.getElementById('forwardBtn').onclick=goForward;
document.getElementById('recentToggle').onclick=()=>{state={...state,recentOpen:!state.recentOpen};renderRecent();};
document.getElementById('huToggle').onclick=()=>{const hu=!state.hu;state={...state,hu,history:[],future:[],stack:hu?2:15};render();};
render();