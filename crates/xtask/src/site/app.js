// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

// Filtering of the rule catalog: free text, category and severity.

const state = {
  query: '',
  category: 'All',
  severity: 'All'
};

function chip(label, group, key) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'chip';
  button.textContent = label;
  button.dataset.group = group;
  button.dataset.value = key;
  button.setAttribute('aria-pressed', key === 'All' ? 'true' : 'false');
  button.addEventListener('click', () => {
    state[group] = key;
    for (const other of document.querySelectorAll(`.chip[data-group="${group}"]`)) {
      other.setAttribute('aria-pressed', other.dataset.value === key ? 'true' : 'false');
    }
    apply();
  });
  return button;
}

function buildChips() {
  const categories = document.getElementById('categories');
  const severities = document.getElementById('severities');
  if (!categories || !severities) {
    return;
  }
  categories.append(chip('All categories', 'category', 'All'));
  for (const name of [...new Set([...document.querySelectorAll('.group')].map((group) => group.dataset.category))]) {
    categories.append(chip(name, 'category', name));
  }
  severities.append(chip('All severities', 'severity', 'All'));
  for (const name of [...new Set([...document.querySelectorAll('.rule')].map((rule) => rule.dataset.severity))]) {
    severities.append(chip(name, 'severity', name));
  }
}

function apply() {
  const query = state.query.trim().toLowerCase();
  let visible = 0;
  for (const rule of document.querySelectorAll('.rule')) {
    const haystack = `${rule.dataset.id} ${rule.dataset.search}`.toLowerCase();
    const matchesQuery = query === '' || haystack.includes(query);
    const matchesCategory = state.category === 'All' || rule.dataset.category === state.category;
    const matchesSeverity = state.severity === 'All' || rule.dataset.severity === state.severity;
    const shown = matchesQuery && matchesCategory && matchesSeverity;
    rule.hidden = !shown;
    if (shown) {
      visible += 1;
    }
  }
  for (const group of document.querySelectorAll('.group')) {
    const any = group.querySelectorAll('.rule:not([hidden])').length > 0;
    group.hidden = !any;
  }
  const count = document.getElementById('count');
  if (count) {
    count.textContent = `${visible} of ${document.querySelectorAll('.rule').length} rules`;
  }
}

function boot() {
  const search = document.getElementById('search');
  if (search) {
    search.addEventListener('input', () => {
      state.query = search.value;
      apply();
    });
  }
  buildChips();
  apply();
}

boot();
