const RANKS=['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
const STACKS=[2,4,6,8,10,12,15,17,20,23,25];
const RECENT_KEY='spins-v45-recent';
const FAVORITE_KEY='spins-v45-favorites';
const Router=window.SpinsRouter;

const REFERENCE_NODES={
  '15|':{hero:'BTN',actions:['Fold','Raise 2','All In 15'],legend:[['Fold',67.22,'fold'],['Raise 2',25.41,'raise'],['All In 15',7.36,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2':{hero:'SB',actions:['Fold','Call','All In 15'],legend:[['Fold',78.31,'fold'],['Call',2.41,'call'],['All In 15',19.27,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Raise 2>SB:Call':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',59.11,'fold'],['Call',22.19,'call'],['All In 15',18.70,'jam']],status:'SCREEN_CROSSCHECK'},
  '15|BTN:Fold':{hero:'SB',actions:['Fold','Call','Raise 3','All In 15'],legend:null,status:'TREE_REFERENCE_ONLY'},
  '15|BTN:Fold>SB:Raise 3':{hero:'BB',actions:['Fold','Call','All In 15'],legend:[['Fold',47.12,'fold'],['Call',33.85,'call'],['All In 15',19.01,'jam']],status:'SCREEN_CROSSCHECK'},
  '2|HU':{hero:'BTN',actions:['Fold','Call','All In 2'],legend:[['Fold',57.16,'fold'],['Call',0.03,'call'],['All In 2',42.82,'jam']],status:'HU_SCREEN_REFERENCE'}
};

let state={stack:15,history:[],future:[],hu:false,recentOpen:false,favoriteOpen:false,mode:'REVIEW'};

function handName(row,col){if(row===col)return RANKS[row]+RANKS[col];if(row<col)return RANKS[row]+RANKS[col]+'s';return RANKS[col]+RANKS[row]+'o';}
function legacyKey(){return Router.legacyReferenceKey(state);}
function canonicalKey(){return Router.canonicalNodeId(state);}
function currentNode(){return REFERENCE_NODES[legacyKey()]||null;}
function seats(){return state.hu?['BTN','SB']:['BTN','SB','BB'];}
function selectedAction(pos){const h=state.history.find(x=>x.pos===pos);return h?h.action:null;}

function compactStatus(raw){
  if(raw==='VERIFIED_EXACT')return ['EXACT','exact'];
  if(raw==='SCREEN_CROSSCHECK'||raw==='HU_SCREEN_REFERENCE')return ['CROSS-CHECK','cross'];
  if(raw==='SOLVER_APPROX'||raw==='TREE_REFERENCE_ONLY')return ['APPROX','approx'];
  return ['MISSING','missing'];
}

function readList(key){try{return JSON.parse(localStorage.getItem(key)||'[]');}catch{return [];}}
function writeList(key,items,max=12){localStorage.setItem(key,JSON.stringify(items.slice(0,max)));}
function routeEntry(){return {stack:state.stack,hu:state.hu,history:Router.cloneHistory(state.history),key:canonicalKey(),ts:Date.now()};}
function rememberCurrent(){if(!state.history.length)return;const e=routeEntry();writeList(RECENT_KEY,[e,...readList(RECENT_KEY).filter(x=>x.key!==e.key)],8);}
function labelRoute(item){const prefix=`${item.hu?'HU':'3M'} ${item.stack}BB`;const hist=item.history.length?item.history.map(x=>`${x.pos} ${x.action}`).join(' / '):'First in';return `${prefix} · ${hist}`;}
function isFavorite(){return readList(FAVORITE_KEY).some(x=>x.key===canonicalKey());}
function toggleFavorite(){
  const key=canonicalKey();let items=readList(FAVORITE_KEY);
  if(items.some(x=>x.key===key))items=items.filter(x=>x.key!==key);else items=[routeEntry(),...items];
  writeList(FAVORITE_KEY,items,20);renderNav();renderFavorites();
}

function restoreRoute(item){state={...state,stack:item.stack,hu:item.hu,history:Router.cloneHistory(item.history),future:[],recentOpen:false,favoriteOpen:false};render();}

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

function renderTree(node){
  const tree=document.getElementById('tree');tree.innerHTML='';
  seats().forEach(pos=>{
    const card=document.createElement('section');card.className='seat'+(node&&node.hero===pos?' active':'');
    const head=document.createElement('div');head.className='seat-head';head.innerHTML=`<span>${pos}</span><span>${state.stack}</span>`;card.appendChild(head);
    const actions=document.createElement('div');actions.className='seat-actions';
    const chosen=selectedAction(pos);let opts=[];
    if(chosen)opts=[chosen];else if(node&&node.hero===pos)opts=node.actions;else if(!node&&pos===Router.nextPosition(state))opts=legalFallback(pos);else opts=['—'];
    opts.forEach(a=>{const b=document.createElement('button');b.className='action-btn'+(chosen===a?' selected':'');b.textContent=a;if(a==='—'||chosen)b.disabled=true;else b.onclick=()=>advance(pos,a);actions.appendChild(b);});
    card.appendChild(actions);tree.appendChild(card);
  });
}

function advance(pos,action){state=Router.appendAction(state,pos,action);rememberCurrent();render();}
function goBack(){state=Router.back(state);render();}
function goForward(){state=Router.forward(state);if(state.history.length)rememberCurrent();render();}
function reset(){state={...state,history:[],future:[]};render();}

function renderListPanel(id,key,open){
  const panel=document.getElementById(id);panel.innerHTML='';panel.classList.toggle('hidden',!open);if(!open)return;
  const items=readList(key);if(!items.length){panel.textContent='Список пока пуст.';return;}
  items.forEach(item=>{const b=document.createElement('button');b.className='recent-item';b.textContent=labelRoute(item);b.onclick=()=>restoreRoute(item);panel.appendChild(b);});
}
function renderRecent(){renderListPanel('recentPanel',RECENT_KEY,state.recentOpen);}
function renderFavorites(){renderListPanel('favoritePanel',FAVORITE_KEY,state.favoriteOpen);}

function renderStatus(node){
  const el=document.getElementById('status');const hist=state.history.length?state.history.map(x=>`${x.pos} ${x.action}`).join(' → '):'First in';
  const raw=node?node.status:'NO_VERIFIED_NODE';const [label,cls]=compactStatus(raw);
  el.innerHTML=`<div class="status-main"><strong>${state.hu?'HU':'3-MAX'} · EFF ${state.stack} BB</strong><span class="source-badge ${cls}">${label}</span></div><div>${hist}</div><div class="status-raw">${canonicalKey()} · ${raw}</div>`;
}

function renderTrain(node){
  const p=document.getElementById('trainPrompt');
  const active=state.mode==='TRAIN';p.classList.toggle('hidden',!active);if(!active)return;
  if(!node||node.status!=='VERIFIED_EXACT'){
    p.innerHTML='<strong>TRAIN заблокирован для этого узла.</strong><br>Нужны VERIFIED_EXACT hand frequencies; CROSS-CHECK/APPROX не используются как ответы тренажёра.';
    return;
  }
  p.textContent='TRAIN ready';
}

function renderMode(){
  document.getElementById('reviewMode').classList.toggle('active',state.mode==='REVIEW');
  document.getElementById('trainMode').classList.toggle('active',state.mode==='TRAIN');
  document.getElementById('modeLabel').textContent=`${state.mode} · V45 prototype`;
}

function renderNav(){
  document.getElementById('backBtn').disabled=!state.history.length;
  document.getElementById('forwardBtn').disabled=!state.future.length;
  const fav=document.getElementById('favoriteBtn');fav.textContent=isFavorite()?'★ FAVORITE':'☆ FAVORITE';fav.classList.toggle('active',isFavorite());
}

function render(){renderMode();renderStacks();renderGrid();const node=currentNode();renderLegend(node);renderTree(node);renderStatus(node);renderTrain(node);renderNav();renderRecent();renderFavorites();}

document.getElementById('reset').onclick=reset;
document.getElementById('backBtn').onclick=goBack;
document.getElementById('forwardBtn').onclick=goForward;
document.getElementById('recentToggle').onclick=()=>{state={...state,recentOpen:!state.recentOpen,favoriteOpen:false};renderRecent();renderFavorites();};
document.getElementById('favoriteBtn').onclick=toggleFavorite;
document.getElementById('reviewMode').onclick=()=>{state={...state,mode:'REVIEW'};render();};
document.getElementById('trainMode').onclick=()=>{state={...state,mode:'TRAIN'};render();};
document.getElementById('huToggle').onclick=()=>{const hu=!state.hu;state={...state,hu,history:[],future:[],stack:hu?2:15};render();};
document.getElementById('favoritePanel').onclick=()=>{};
render();