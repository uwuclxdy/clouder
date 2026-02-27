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
