#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const targets=JSON.parse(fs.readFileSync(path.join(root,'data','chart-coverage-targets.json'),'utf8'));
const exactIndex=JSON.parse(fs.readFileSync(path.join(root,'data','charts','exact-index.json'),'utf8'));
const crossDir=path.join(root,'data','crosschecks','2026');
const crossFiles=fs.existsSync(crossDir)?fs.readdirSync(crossDir).filter(x=>x.endsWith('.json')):[];
const crossStacks=new Set(crossFiles.map(f=>JSON.parse(fs.readFileSync(path.join(crossDir,f),'utf8')).effective_stack_bb));
const exactKeys=Object.keys(exactIndex.nodes??{});

const rows=targets.opening_nodes.map(t=>{
  const exactForStack=exactKeys.filter(k=>k.startsWith(`SPIN_3MAX|${t.stack_bb}|`));
  return {
    stack_bb:t.stack_bb,
    positions:t.positions,
    exact_runtime_nodes:exactForStack.length,
    current_2026_crosscheck:crossStacks.has(t.stack_bb),
    target_status:t.status,
    interpolation_allowed:false
  };
});

const summary={
  schema_version:'chart-coverage-report-v1',
  exact_runtime_nodes_total:exactKeys.length,
  stacks_with_current_2026_crosscheck:[...crossStacks].sort((a,b)=>a-b),
  stacks_without_current_2026_crosscheck:targets.ui_effective_stacks_bb.filter(s=>!crossStacks.has(s)),
  rows,
  p0_response_registry:'data/charts/15bb-node-registry.json'
};
console.log(JSON.stringify(summary,null,2));
if(targets.no_interpolation!==true)throw new Error('coverage policy must forbid interpolation');
console.error(`Coverage report PASS: exact runtime nodes=${exactKeys.length}; cross-check stacks=${summary.stacks_with_current_2026_crosscheck.join(',')||'none'}; interpolation=FORBIDDEN.`);
