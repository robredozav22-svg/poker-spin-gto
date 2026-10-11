#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const dataDir = path.join(root, 'data');
const anchorsPath = path.join(dataDir, 'source-verified-anchors.json');
const manifestPath = path.join(dataDir, 'chart-manifest.json');

const ERRORS = [];
const WARNINGS = [];
const ALLOWED_ACTIONS = new Set(['FOLD', 'CHECK', 'LIMP', 'CALL', 'RAISE', 'JAM']);
const RANKS = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'];

function fail(msg) { ERRORS.push(msg); }
function warn(msg) { WARNINGS.push(msg); }
function readJson(p) { return JSON.parse(fs.readFileSync(p, 'utf8')); }

function all169Hands() {
  const out = [];
  for (let i = 0; i < RANKS.length; i++) {
    for (let j = 0; j < RANKS.length; j++) {
      if (i === j) out.push(RANKS[i] + RANKS[j]);
      else if (i < j) out.push(RANKS[i] + RANKS[j] + 's');
      else out.push(RANKS[j] + RANKS[i] + 'o');
    }
  }
  return out;
}

const HANDS = all169Hands();
const HAND_SET = new Set(HANDS);
if (HANDS.length !== 169 || HAND_SET.size !== 169) fail(`hand universe is not exactly 169: ${HANDS.length}/${HAND_SET.size}`);

function handCombos(hand) {
  if (hand.length === 2) return 6;
  if (hand.endsWith('s')) return 4;
  if (hand.endsWith('o')) return 12;
  throw new Error(`invalid hand: ${hand}`);
}

const TOTAL_COMBOS = HANDS.reduce((n, h) => n + handCombos(h), 0);
if (TOTAL_COMBOS !== 1326) fail(`starting combo universe must equal 1326, got ${TOTAL_COMBOS}`);

function validateAnchors() {
  if (!fs.existsSync(anchorsPath)) return fail('missing data/source-verified-anchors.json');
  const a = readJson(anchorsPath);
  if (a.total_starting_combos !== 1326) fail('anchor total_starting_combos must be 1326');
  const seen = new Set();
  for (const row of a.rfi_3max ?? []) {
    const key = `${row.position}:${row.stack_bb}`;
    if (seen.has(key)) fail(`duplicate RFI anchor ${key}`);
    seen.add(key);
    const entries = Object.entries(row.action_combos ?? {});
    if (!entries.length) fail(`RFI anchor ${key} has no action totals`);
    let sum = 0;
    for (const [action, combos] of entries) {
      if (!ALLOWED_ACTIONS.has(action)) fail(`RFI anchor ${key} has invalid action ${action}`);
      if (!Number.isFinite(combos) || combos < 0) fail(`RFI anchor ${key} has invalid combo count ${action}=${combos}`);
      sum += combos;
    }
    if (Math.abs(sum - 1326) > 1e-9) fail(`RFI anchor ${key} action combo sum=${sum}, expected 1326`);
  }
  const expected = ['BTN:8','BTN:10','BTN:15','BTN:20','BTN:25','SB:8','SB:10','SB:15','SB:20','SB:25'];
  for (const key of expected) if (!seen.has(key)) fail(`missing audited RFI anchor ${key}`);

  for (const row of a.public_call_vs_jam ?? []) {
    if (!row.node || !Number.isFinite(row.stack_bb)) fail('call-vs-jam row missing node/stack');
    if (!row.source) fail(`call-vs-jam ${row.node}:${row.stack_bb} missing source`);
    if (row.grade !== 'A') warn(`public explicit call row ${row.node}:${row.stack_bb} grade is ${row.grade}, expected A`);
    if (!Number.isFinite(row.combos) || row.combos < 0 || row.combos > 1326) fail(`invalid call combos ${row.node}:${row.stack_bb}`);
    const pct = row.combos / 1326 * 100;
    if (Math.abs(pct - row.pct) > 0.06) fail(`call pct mismatch ${row.node}:${row.stack_bb}: combos imply ${pct.toFixed(2)}%, stored ${row.pct}%`);
  }
}

function validateManifest() {
  if (!fs.existsSync(manifestPath)) return fail('missing data/chart-manifest.json');
  const m = readJson(manifestPath);
  const p = m.policy ?? {};
  if (p.allow_interpolation !== false) fail('policy.allow_interpolation must be false');
  if (p.allow_unverified_ranges_in_app !== false) fail('policy.allow_unverified_ranges_in_app must be false');
  if (p.require_previous_action_context !== true) fail('policy.require_previous_action_context must be true');
  if (m.existing_main_data_status !== 'INVALID_FOR_STRATEGY') fail('existing main data must remain marked INVALID_FOR_STRATEGY until replaced and re-audited');
}

function normalizeFreqCell(cell, file, hand) {
  if (typeof cell === 'string') {
    if (!ALLOWED_ACTIONS.has(cell)) fail(`${file}:${hand} invalid action ${cell}`);
    return { [cell]: 100 };
  }
  if (!cell || typeof cell !== 'object' || Array.isArray(cell)) {
    fail(`${file}:${hand} cell must be action string or frequency object`);
    return null;
  }
  const actions = cell.actions && typeof cell.actions === 'object' ? cell.actions : cell;
  let sum = 0;
  for (const [action, freq] of Object.entries(actions)) {
    if (['source','grade','note','status','source_stack_bb','source_vector','history','sizing'].includes(action)) continue;
    if (!ALLOWED_ACTIONS.has(action)) { fail(`${file}:${hand} invalid action ${action}`); continue; }
    if (!Number.isFinite(freq) || freq < 0 || freq > 100) fail(`${file}:${hand} invalid frequency ${action}=${freq}`);
    else sum += freq;
  }
  if (Math.abs(sum - 100) > 0.11) fail(`${file}:${hand} action frequencies sum to ${sum}, expected 100`);
  return actions;
}

function validateDominance(chart, file) {
  for (const hi of RANKS) {
    for (const lo of RANKS) {
      if (hi === lo) continue;
      const i = RANKS.indexOf(hi), j = RANKS.indexOf(lo);
      if (i >= j) continue;
      const s = `${hi}${lo}s`, o = `${hi}${lo}o`;
      const so = chart[s], oo = chart[o];
      if (!so || !oo) continue;
      const sf = normalizeFreqCell(so, file, s);
      const of = normalizeFreqCell(oo, file, o);
      if (!sf || !of) continue;
      const offCall = Number(of.CALL ?? 0);
      const suitedCall = Number(sf.CALL ?? 0);
      if (offCall > 0.001 && suitedCall < 0.001) fail(`${file}: suited-over-offsuit CALL violation: ${o} CALL=${offCall}, ${s} CALL=${suitedCall}`);
    }
  }
}

function validateChartFile(filePath) {
  const file = path.relative(root, filePath);
  let doc;
  try { doc = readJson(filePath); } catch (e) { return fail(`${file}: invalid JSON (${e.message})`); }
  const charts = Array.isArray(doc) ? doc : (doc.charts ?? []);
  if (!Array.isArray(charts)) return fail(`${file}: expected charts[]`);
  for (const chartDoc of charts) {
    const id = chartDoc.id ?? '<missing-id>';
    const status = chartDoc.status ?? chartDoc.grade ?? 'UNKNOWN';
    const chart = chartDoc.hands ?? chartDoc.chart;
    if (!chart || typeof chart !== 'object' || Array.isArray(chart)) { fail(`${file}:${id} missing hands/chart object`); continue; }
    const keys = Object.keys(chart);
    for (const hand of keys) if (!HAND_SET.has(hand)) fail(`${file}:${id} unknown hand key ${hand}`);
    if (status === 'VERIFIED_EXACT' || status === 'A') {
      if (keys.length !== 169) fail(`${file}:${id} exact chart has ${keys.length} hand classes, expected 169`);
      if (!chartDoc.source) fail(`${file}:${id} exact chart missing source`);
      if (!chartDoc.history && !chartDoc.node) fail(`${file}:${id} exact chart missing previous-action context/node`);
      if (chartDoc.asymmetric === true && !chartDoc.source_vector) fail(`${file}:${id} asymmetric exact chart missing source_vector`);
    }
    for (const [hand, cell] of Object.entries(chart)) normalizeFreqCell(cell, `${file}:${id}`, hand);
    validateDominance(chart, `${file}:${id}`);
  }
}

function scanChartFiles() {
  if (!fs.existsSync(dataDir)) return;
  const files = fs.readdirSync(dataDir)
    .filter(f => f.endsWith('.json'))
    .filter(f => !['chart-manifest.json','source-verified-anchors.json'].includes(f));
  for (const file of files) validateChartFile(path.join(dataDir, file));
}

validateAnchors();
validateManifest();
scanChartFiles();

for (const w of WARNINGS) console.warn(`WARN: ${w}`);
if (ERRORS.length) {
  for (const e of ERRORS) console.error(`ERROR: ${e}`);
  console.error(`\nChart validation FAILED: ${ERRORS.length} error(s), ${WARNINGS.length} warning(s).`);
  process.exit(1);
}
console.log(`Chart validation PASS: 169 hand classes / 1326 combos; ${WARNINGS.length} warning(s).`);
