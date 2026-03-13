// clouder dashboard // shared utilities

const toastContainer = (() => {
  const el = document.createElement('div');
  el.className = 'toast-container';
  document.body.appendChild(el);
  return el;
})();

function toast(message, type = 'info', duration = 3000) {
  const el = document.createElement('div');
  el.className = `toast ${type}`;
  el.textContent = message;
  toastContainer.appendChild(el);

  requestAnimationFrame(() => {
    requestAnimationFrame(() => el.classList.add('show'));
  });

  setTimeout(() => {
    el.classList.remove('show');
    setTimeout(() => el.remove(), 300);
  }, duration);
}

async function apiFetch(method, url, body) {
  const opts = {
    method,
    headers: { 'Content-Type': 'application/json' },
  };
  if (body !== undefined) opts.body = JSON.stringify(body);

  const res = await fetch(url, opts);
  if (res.status === 401) {
    window.location.href = '/login';
    throw new Error('session expired');
  }
  return res;
}

// HTML escape utility - used across all templates
function escHtml(s) {
  if (s == null) return '';
  return String(s)
    .replace(/&/g, '&')
    .replace(/</g, '<')
    .replace(/>/g, '>')
    .replace(/"/g, '"');
}

// Timezone options - shared between about, profile, and reminders pages
const TIMEZONE_OPTIONS = [
  { value: 'UTC', label: 'UTC' },
  { value: 'Etc/GMT+12', label: 'GMT-12' }, { value: 'Etc/GMT+11', label: 'GMT-11' },
  { value: 'Etc/GMT+10', label: 'GMT-10' }, { value: 'Etc/GMT+9', label: 'GMT-9' },
  { value: 'Etc/GMT+8', label: 'GMT-8' }, { value: 'Etc/GMT+7', label: 'GMT-7' },
  { value: 'Etc/GMT+6', label: 'GMT-6' }, { value: 'Etc/GMT+5', label: 'GMT-5' },
  { value: 'Etc/GMT+4', label: 'GMT-4' }, { value: 'Etc/GMT+3', label: 'GMT-3' },
  { value: 'Etc/GMT+2', label: 'GMT-2' }, { value: 'Etc/GMT+1', label: 'GMT-1' },
  { value: 'Etc/GMT-1', label: 'GMT+1' }, { value: 'Etc/GMT-2', label: 'GMT+2' },
  { value: 'Etc/GMT-3', label: 'GMT+3' }, { value: 'Asia/Tehran', label: 'GMT+3:30' },
  { value: 'Etc/GMT-4', label: 'GMT+4' }, { value: 'Asia/Kabul', label: 'GMT+4:30' },
  { value: 'Etc/GMT-5', label: 'GMT+5' }, { value: 'Asia/Kolkata', label: 'GMT+5:30' },
  { value: 'Asia/Kathmandu', label: 'GMT+5:45' }, { value: 'Etc/GMT-6', label: 'GMT+6' },
  { value: 'Asia/Yangon', label: 'GMT+6:30' }, { value: 'Etc/GMT-7', label: 'GMT+7' },
  { value: 'Etc/GMT-8', label: 'GMT+8' }, { value: 'Etc/GMT-9', label: 'GMT+9' },
  { value: 'Australia/Darwin', label: 'GMT+9:30' }, { value: 'Etc/GMT-10', label: 'GMT+10' },
  { value: 'Etc/GMT-11', label: 'GMT+11' }, { value: 'Etc/GMT-12', label: 'GMT+12' },
  { value: 'Etc/GMT-13', label: 'GMT+13' },
];

// Populate timezone select element with options
function populateTimezoneSelect(selectEl, selectedValue = 'UTC') {
  if (!selectEl) return;
  selectEl.innerHTML = TIMEZONE_OPTIONS.map(tz =>
    `<option value="${escHtml(tz.value)}"${tz.value === selectedValue ? ' selected' : ''}>${escHtml(tz.label)}</option>`
  ).join('');
}

// Populate timezone datalist element
function populateTimezoneDatalist(datalistEl) {
  if (!datalistEl) return;
  datalistEl.innerHTML = TIMEZONE_OPTIONS.map(tz =>
    `<option value="${escHtml(tz.value)}">${escHtml(tz.label)}</option>`
  ).join('');
}

// Color conversion utility - hex to integer
function colorToInt(hex) {
  return parseInt(hex.replace('#', ''), 16);
}

// Format number with locale
function fmtNumber(n) {
  if (n == null) return '?';
  return Number(n).toLocaleString();
}

// tab switching
document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    const group = tab.closest('.tabs').dataset.group;
    document.querySelectorAll(`.tab[data-group="${group}"], [data-tab-group="${group}"]`).forEach(el => {
      el.classList.remove('active');
    });
    tab.classList.add('active');
    const target = tab.dataset.tab;
    const panel = document.querySelector(`[data-tab-group="${group}"][data-tab-id="${target}"]`);
    if (panel) panel.classList.add('active');
  });
});

console.log('%cclouder', 'font-size: 18px; color: #E07B53; font-weight: bold; font-family: monospace;');
console.log('%c// dashboard loaded', 'font-size: 11px; color: #6c7086; font-family: monospace;');
