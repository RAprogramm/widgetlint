// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

// Playground logic: debounced live check through the wasm pipeline. All report
// text is inserted with textContent, never as HTML, so widget content can
// never inject markup or instructions into the page.

import init, { check_agent, version_json } from './pkg/wl_wasm.js';

const source = document.getElementById('source');
const failOn = document.getElementById('fail-on');
const verdictEl = document.getElementById('verdict');
const reasonEl = document.getElementById('reason');
const budgetEl = document.getElementById('budget');
const barFill = document.getElementById('bar-fill');
const budgetText = document.getElementById('budget-text');
const findingsEl = document.getElementById('findings');
const moreEl = document.getElementById('more');
const actionEl = document.getElementById('action');
const caveatsBox = document.getElementById('caveats-box');
const caveatsEl = document.getElementById('caveats');
const sizeEl = document.getElementById('size');
const versionEl = document.getElementById('version');

const ready = init();

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function renderFinding(finding) {
  const li = el('li', finding.sev);
  const head = el('div', 'finding-head');
  head.append(el('span', 'rule', finding.id));
  head.append(el('span', 'sev', finding.sev));
  if (finding.repeat > 1) head.append(el('span', 'repeat', `×${finding.repeat}`));
  if (finding.where) {
    head.append(el('span', 'where', `${finding.where.region} ${finding.where.line}:${finding.where.col}`));
  }
  li.append(head);
  li.append(el('p', 'msg', finding.msg));
  if (finding.fix && finding.fix.hint) li.append(el('p', 'hint', `help: ${finding.fix.hint}`));
  return li;
}

function render(report) {
  verdictEl.textContent = report.verdict.toUpperCase();
  verdictEl.className = `verdict ${report.verdict}`;
  reasonEl.textContent = report.reason;
  budgetEl.hidden = false;
  const pct = Math.min(100, (report.budget.zlib9 / report.budget.limit) * 100);
  barFill.style.width = `${pct}%`;
  barFill.className = report.budget.over_by > 0 ? 'over' : '';
  budgetText.textContent =
    `${report.budget.zlib9} B / ${report.budget.limit} B zlib-9 ` +
    `(budget ${report.budget.target} B)` +
    (report.budget.over_by > 0 ? ` — over by ${report.budget.over_by} B` : '');
  findingsEl.textContent = '';
  for (const finding of report.findings) findingsEl.append(renderFinding(finding));
  moreEl.textContent = report.truncated > 0
    ? `+ ${report.truncated} more finding(s) hidden; run the CLI for the full list`
    : '';
  actionEl.hidden = false;
  actionEl.textContent = report.next_action;
  caveatsEl.textContent = '';
  for (const caveat of report.caveats) caveatsEl.append(el('li', '', caveat));
  caveatsBox.hidden = report.caveats.length === 0;
  sizeEl.textContent = `${report.budget.zlib9} B zlib-9`;
}

function renderError(message) {
  verdictEl.textContent = 'INPUT ERROR';
  verdictEl.className = 'verdict error';
  reasonEl.textContent = message;
  budgetEl.hidden = true;
  findingsEl.textContent = '';
  moreEl.textContent = '';
  actionEl.hidden = true;
  caveatsBox.hidden = true;
  sizeEl.textContent = '';
}

let timer;
function schedule() {
  clearTimeout(timer);
  timer = setTimeout(run, 250);
}

async function run() {
  await ready;
  if (source.value.trim() === '') {
    verdictEl.textContent = 'waiting for input';
    verdictEl.className = 'verdict';
    reasonEl.textContent = '';
    return;
  }
  try {
    render(JSON.parse(check_agent(source.value, failOn.value)));
  } catch (error) {
    renderError(String(error.message ?? error));
  }
}

source.addEventListener('input', schedule);
failOn.addEventListener('change', run);

ready.then(() => {
  try {
    versionEl.textContent = JSON.parse(version_json()).version;
  } catch {
    versionEl.textContent = '';
  }
  run();
});
