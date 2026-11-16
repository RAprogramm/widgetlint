// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

// Playground logic: a debounced live check through the wasm pipeline.
//
// Every text that comes from the widget is inserted with textContent, never
// as markup, so widget content can never inject HTML or instructions into the
// page.

import init, { check_agent, rules_json, version_json } from './pkg/wl_wasm.js';

const SAMPLES = {
  broken: `<div class="card">
  <h1>Tap me</h1>
  <input type="text" placeholder="your name">
  <script>
    fetch("https://example.com/data");
  </script>
</div>`,
  clean: `<div class="card">
  <style>
    .card { color: #101828; font: 16px system-ui; }
  </style>
  <h1>Tap me</h1>
  <button class="cta">Send</button>
  <script>
    const button = document.querySelector(".cta");
    button.addEventListener("click", () => {
      Telegram.WebApp.sendData("tapped");
    });
  </script>
</div>`,
  heavy: `<div class="card">
  <style>
${"    .pad { padding: 8px; }\n".repeat(120)}
  </style>
  <p>${"padding makes the compressed size grow. ".repeat(80)}</p>
</div>`,
  css: `<div class="card">
  <style>
    .card { position: sticky; top: 0; transform: rotate(3deg); }
  </style>
  <p>Unsupported features</p>
</div>`
};

const THEME_ORDER = ['auto', 'light', 'dark'];

const elements = {
  editor: document.getElementById('editor'),
  gutter: document.getElementById('gutter'),
  failOn: document.getElementById('fail-on'),
  sample: document.getElementById('sample'),
  openFile: document.getElementById('open-file'),
  fileInput: document.getElementById('file-input'),
  check: document.getElementById('check'),
  copyReport: document.getElementById('copy-report'),
  verdict: document.getElementById('verdict'),
  reason: document.getElementById('reason'),
  budget: document.getElementById('budget'),
  budgetText: document.getElementById('budget-text'),
  barZlib: document.getElementById('bar-zlib'),
  barBudget: document.getElementById('bar-budget'),
  barLimit: document.getElementById('bar-limit'),
  legendZlib: document.getElementById('legend-zlib'),
  legendBudget: document.getElementById('legend-budget'),
  legendLimit: document.getElementById('legend-limit'),
  action: document.getElementById('action'),
  findings: document.getElementById('findings'),
  findingsCount: document.getElementById('findings-count'),
  emptyState: document.getElementById('empty-state'),
  caveatsBox: document.getElementById('caveats-box'),
  caveats: document.getElementById('caveats'),
  version: document.getElementById('version'),
  ruleset: document.getElementById('ruleset'),
  factRules: document.getElementById('fact-rules'),
  themeToggle: document.getElementById('theme-toggle'),
  themeLabel: document.getElementById('theme-label'),
  drawer: document.getElementById('rule-drawer'),
  drawerTitle: document.getElementById('drawer-title'),
  drawerBody: document.getElementById('drawer-body'),
  drawerClose: document.getElementById('drawer-close'),
  toast: document.getElementById('toast')
};

let rules = new Map();
let lastReport = null;
let timer = null;
let toastTimer = null;

function formatBytes(value) {
  if (value < 1024) {
    return `${value} B`;
  }
  return `${(value / 1024).toFixed(1)} KiB`;
}

function text(tag, className, content) {
  const node = document.createElement(tag);
  if (className) {
    node.className = className;
  }
  if (content !== undefined) {
    node.textContent = content;
  }
  return node;
}

function showToast(message) {
  elements.toast.textContent = message;
  elements.toast.hidden = false;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    elements.toast.hidden = true;
  }, 2200);
}

function renderGutter() {
  const lines = elements.editor.value.split('\n').length;
  const numbers = [];
  for (let index = 1; index <= lines; index += 1) {
    numbers.push(index);
  }
  elements.gutter.textContent = numbers.join('\n');
  elements.gutter.scrollTop = elements.editor.scrollTop;
}

function severityClass(severity) {
  if (severity === 'error') {
    return 'error';
  }
  if (severity === 'warn') {
    return 'warn';
  }
  return 'info';
}

function renderVerdict(report) {
  const passed = report.verdict === 'pass';
  elements.verdict.className = `verdict verdict--${passed ? 'pass' : 'fail'}`;
  elements.verdict.replaceChildren();
  const icon = text('span', 'verdict__icon', passed ? '✓' : '!');
  icon.setAttribute('aria-hidden', 'true');
  elements.verdict.append(icon, text('span', '', passed
    ? 'Looks good — no blocking findings'
    : 'This widget will not work as written'));
  elements.reason.textContent = report.reason;
}

function renderBudget(report) {
  const budget = report.budget;
  const scale = Math.max(budget.limit, budget.zlib9, 1);
  const percent = (value) => `${Math.min(100, (value / scale) * 100)}%`;
  elements.budget.hidden = false;
  elements.barZlib.style.width = percent(budget.zlib9);
  elements.barZlib.classList.toggle('meter__fill--over', budget.over_by > 0);
  elements.barBudget.style.left = percent(budget.target);
  elements.barLimit.style.left = percent(budget.limit);
  elements.legendZlib.textContent = formatBytes(budget.zlib9);
  elements.legendBudget.textContent = formatBytes(budget.target);
  elements.legendLimit.textContent = formatBytes(budget.limit);
  elements.budgetText.textContent = budget.over_by > 0
    ? `${formatBytes(budget.over_by)} over the limit`
    : `${formatBytes(budget.limit - budget.zlib9)} of headroom`;
}

function renderFindings(report) {
  elements.findings.replaceChildren();
  elements.findingsCount.textContent = String(report.findings.length);
  elements.emptyState.hidden = report.findings.length > 0;
  for (const finding of report.findings) {
    const item = text('li', `finding finding--${severityClass(finding.sev)}`);
    const head = text('div', 'finding__head');
    head.append(text('span', `pill pill--${severityClass(finding.sev)}`, finding.sev));
    const id = text('button', 'finding__id', finding.id);
    id.type = 'button';
    id.addEventListener('click', () => openRule(finding.id));
    head.append(id);
    if (finding.where) {
      head.append(text('span', 'pill pill--muted', `line ${finding.where.line}`));
    }
    if (finding.repeat > 1) {
      head.append(text('span', 'pill pill--muted', `×${finding.repeat}`));
    }
    item.append(head);
    item.append(text('p', 'finding__message', finding.msg));
    if (finding.fix && finding.fix.hint) {
      item.append(text('p', 'finding__fix', finding.fix.hint));
    }
    const meta = text('div', 'finding__meta');
    meta.append(text('span', `pill pill--${finding.conf}`, finding.conf));
    meta.append(text('span', '', finding.why));
    item.append(meta);
    elements.findings.append(item);
  }
}

function renderCaveats(report) {
  elements.caveats.replaceChildren();
  elements.caveatsBox.hidden = !report.caveats || report.caveats.length === 0;
  for (const caveat of report.caveats || []) {
    elements.caveats.append(text('li', '', caveat));
  }
}

function render(report) {
  lastReport = report;
  renderVerdict(report);
  renderBudget(report);
  renderFindings(report);
  renderCaveats(report);
  elements.action.hidden = !report.next_action;
  elements.action.textContent = report.next_action;
  const errors = report.counts.error;
  const warnings = report.counts.warn;
  elements.check.textContent = 'Check widget';
  elements.check.setAttribute(
    'aria-label',
    `Check widget: ${errors} errors, ${warnings} warnings`
  );
}

function reportError(message) {
  elements.verdict.className = 'verdict verdict--fail';
  elements.verdict.textContent = 'The checker could not read this widget';
  elements.reason.textContent = message;
  elements.findings.replaceChildren();
  elements.findingsCount.textContent = '0';
  elements.emptyState.hidden = true;
  elements.action.hidden = true;
}

function check() {
  const source = elements.editor.value;
  if (source.trim() === '') {
    elements.verdict.className = 'verdict verdict--pending';
    elements.verdict.textContent = 'Nothing to check yet';
    elements.reason.textContent = 'Paste a widget on the left.';
    elements.findings.replaceChildren();
    elements.findingsCount.textContent = '0';
    elements.emptyState.hidden = false;
    elements.action.hidden = true;
    elements.budget.hidden = true;
    lastReport = null;
    return;
  }
  try {
    render(JSON.parse(check_agent(source, elements.failOn.value)));
  } catch (error) {
    reportError(String(error));
  }
}

function scheduleCheck() {
  window.clearTimeout(timer);
  timer = window.setTimeout(check, 220);
}

function openRule(ruleId) {
  const rule = rules.get(ruleId);
  elements.drawerBody.replaceChildren();
  if (!rule) {
    elements.drawerTitle.textContent = ruleId;
    elements.drawerBody.append(text('p', '', 'No metadata for this rule.'));
  } else {
    elements.drawerTitle.textContent = `${rule.id} — ${rule.title}`;
    const list = text('dl');
    const add = (label, value) => {
      list.append(text('dt', '', label));
      list.append(text('dd', '', value));
    };
    add('Severity', rule.severity);
    add('Confidence', rule.confidence);
    if (rule.help) {
      add('How to fix', rule.help);
    }
    const link = text('a', '', 'Telegram widget specification');
    link.href = rule.spec;
    link.rel = 'noreferrer';
    list.append(text('dt', '', 'Specification'));
    const holder = text('dd');
    holder.append(link);
    list.append(holder);
    elements.drawerBody.append(list);
  }
  elements.drawer.hidden = false;
  elements.drawerClose.focus();
}

function closeRule() {
  elements.drawer.hidden = true;
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  elements.themeLabel.textContent = theme[0].toUpperCase() + theme.slice(1);
  window.localStorage.setItem('widgetlint-theme', theme);
}

function toggleTheme() {
  const current = document.documentElement.dataset.theme || 'auto';
  const next = THEME_ORDER[(THEME_ORDER.indexOf(current) + 1) % THEME_ORDER.length];
  applyTheme(next);
}

function installTabHandling() {
  elements.editor.addEventListener('keydown', (event) => {
    if (event.key !== 'Tab') {
      return;
    }
    event.preventDefault();
    const start = elements.editor.selectionStart;
    const end = elements.editor.selectionEnd;
    elements.editor.setRangeText('  ', start, end, 'end');
    scheduleCheck();
  });
}

function installShortcuts() {
  document.addEventListener('keydown', (event) => {
    const meta = event.ctrlKey || event.metaKey;
    if (meta && event.key === 'Enter') {
      event.preventDefault();
      check();
    }
    if (meta && event.key === 'k') {
      event.preventDefault();
      elements.editor.focus();
    }
    if (event.key === 'Escape' && !elements.drawer.hidden) {
      closeRule();
    }
  });
}

function installFileLoading() {
  elements.openFile.addEventListener('click', () => elements.fileInput.click());
  elements.fileInput.addEventListener('change', () => {
    const file = elements.fileInput.files && elements.fileInput.files[0];
    if (file) {
      readFile(file);
    }
  });
  const drop = (event) => {
    event.preventDefault();
    const file = event.dataTransfer && event.dataTransfer.files[0];
    if (file) {
      readFile(file);
    }
  };
  elements.editor.addEventListener('dragover', drop);
  elements.editor.addEventListener('drop', drop);
}

function readFile(file) {
  const reader = new FileReader();
  reader.onload = () => {
    elements.editor.value = String(reader.result || '');
    renderGutter();
    check();
  };
  reader.readAsText(file);
}

function installCopy() {
  elements.copyReport.addEventListener('click', async () => {
    if (!lastReport) {
      showToast('Nothing to copy yet');
      return;
    }
    try {
      await navigator.clipboard.writeText(JSON.stringify(lastReport, null, 2));
      showToast('Report copied as JSON');
    } catch (error) {
      showToast(`Copy failed: ${error}`);
    }
  });
}

function installSamples() {
  elements.sample.addEventListener('change', () => {
    const key = elements.sample.value;
    if (!key || !SAMPLES[key]) {
      return;
    }
    elements.editor.value = SAMPLES[key];
    elements.sample.value = '';
    renderGutter();
    check();
  });
}

async function boot() {
  applyTheme(window.localStorage.getItem('widgetlint-theme') || 'auto');
  elements.themeToggle.addEventListener('click', toggleTheme);
  elements.drawerClose.addEventListener('click', closeRule);
  elements.drawer.addEventListener('click', (event) => {
    if (event.target === elements.drawer) {
      closeRule();
    }
  });
  elements.editor.addEventListener('input', () => {
    renderGutter();
    scheduleCheck();
  });
  elements.editor.addEventListener('scroll', () => {
    elements.gutter.scrollTop = elements.editor.scrollTop;
  });
  elements.failOn.addEventListener('change', check);
  elements.check.addEventListener('click', check);
  installTabHandling();
  installShortcuts();
  installFileLoading();
  installCopy();
  installSamples();
  renderGutter();

  try {
    await init();
  } catch (error) {
    reportError(`The WebAssembly module failed to load: ${error}`);
    return;
  }

  const identity = JSON.parse(version_json());
  elements.version.textContent = `v${identity.version}`;
  elements.ruleset.textContent = `ruleset ${identity.ruleset.slice(-8)}`;
  elements.ruleset.title = identity.ruleset;
  elements.factRules.textContent = `${identity.rules} rules · profile ${identity.profile}`;

  try {
    rules = new Map(JSON.parse(rules_json()).map((rule) => [rule.id, rule]));
  } catch (error) {
    rules = new Map();
  }

  elements.editor.value = SAMPLES.broken;
  renderGutter();
  check();
}

boot();
