#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const file=path.join(root,'data','chart-source-policy-2026.json');
const p=JSON.parse(fs.readFileSync(file,'utf8'));
const errors=[];const fail=x=>errors.push(x);

if(p.schema_version!=='chart-source-policy-2026-v2')fail('source policy schema_version must be chart-source-policy-2026-v2');
if(p.policy_year!==2026)fail('source policy year must be 2026');
if(p.goal!=='MAXIMUM_CURRENT_ACCURACY_NOT_SOURCE_IMITATION')fail('source policy goal changed unexpectedly');
const primary=p.primary_baseline||{};
if(primary.provider!=='GTO Wizard')fail('primary baseline provider must remain GTO Wizard');
if(JSON.stringify(primary.preferred_solution_family_order)!==JSON.stringify(['Research','General','Simple']))fail('preferred current GTO Wizard family order must be Research -> General -> Simple');
for(const stale of ['Legacy','Basic'])if(!(primary.disallowed_as_new_primary||[]).includes(stale))fail(`${stale} must remain disallowed as new primary`);
if(primary.capture_requirement!=='LIVE_NODE_CAPTURE_IN_2026')fail('live 2026 capture requirement missing');
if(primary.required_export_method!=='FULL_NODE_UPI_PLUS_EVERY_ACTION_UPI')fail('lossless UPI export requirement missing');
const gates=p.promotion_gates||{};
for(const k of ['require_exact_node_identity','require_exact_action_history','require_exact_action_sizes','require_169_hand_classes','require_explicit_zero_reach','require_no_interpolation','require_no_default_fold','require_live_2026_source_capture','require_internal_probability_conservation','require_independent_crosscheck','require_tree_compatibility_before_numeric_comparison','require_disagreement_classification','require_source_usage_compliance'])if(gates[k]!==true)fail(`promotion gate ${k} must remain true`);
for(const type of ['COMPATIBLE_EXTERNAL_SOURCE','INDEPENDENT_INTERNAL_SOLVER_PROOF'])if(!(gates.allowed_independent_crosscheck_types||[]).includes(type))fail(`promotion proof type ${type} missing`);
const pre=(p.independent_cross_checks||[]).find(x=>x.provider==='PreflopRanges.app');
if(!pre)fail('PreflopRanges cross-check policy missing');else{
  if(pre.allowed_for_exact_promotion!==false)fail('PreflopRanges must remain ineligible as direct exact promotion source');
  if(pre.usage_policy!=='NO_SCRAPING_NO_BULK_EXTRACTION_NO_RATE_LIMIT_BYPASS')fail('PreflopRanges no-scraping/no-bulk policy missing');
  if(!String(pre.profile_status||'').includes('UNRESOLVED'))fail('PreflopRanges unresolved profile status must remain explicit');
}
const internal=(p.independent_cross_checks||[]).find(x=>x.provider==='independent internal solver proof');
if(!internal||internal.allowed_for_exact_promotion!==true)fail('independent internal solver proof must remain an allowed proof path');
const hard=(p.hard_rules||[]).join('\n');
for(const phrase of ['Never average strategies from different trees','do not scrape','Self-generated solver output remains ResearchOnly'])if(!hard.toLowerCase().includes(phrase.toLowerCase()))fail(`hard rule missing phrase: ${phrase}`);

if(errors.length){for(const e of errors)console.error(`ERROR: ${e}`);console.error(`2026 source policy validation FAILED: ${errors.length} error(s).`);process.exit(1);}
console.log('2026 source policy validation PASS: freshness, no-interpolation, source compliance and independent-proof invariants locked.');
