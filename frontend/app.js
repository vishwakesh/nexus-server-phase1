'use strict';

const $ = (id) => document.getElementById(id);
const formatBytes = (bytes) => {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) { value /= 1024; index += 1; }
  return `${value.toFixed(index >= 3 ? 2 : 1)} ${units[index]}`;
};
const formatUptime = (seconds) => {
  if (!Number.isFinite(seconds) || seconds < 0) return '—';
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
};
const setText = (id, value) => { const el = $(id); if (el) el.textContent = String(value); };
const setMeter = (id, percent) => {
  const el = $(id);
  if (!el) return;
  const number = Number(percent);
  el.style.width = `${Number.isFinite(number) ? Math.max(0, Math.min(100, number)) : 0}%`;
};
const safeDate = (value) => {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? 'Time unavailable' : date.toLocaleString();
};

async function fetchJson(path) {
  const response = await fetch(path, { headers: { Accept: 'application/json' }, cache: 'no-store' });
  if (!response.ok) throw new Error(`${path} returned HTTP ${response.status}`);
  return response.json();
}

function renderEvents(events) {
  const list = $('eventsList');
  if (!Array.isArray(events) || events.length === 0) {
    list.innerHTML = '<div class="empty-state"><strong>No events recorded</strong><small>New NEXUS events will appear here.</small></div>';
    setText('eventCount', '0 EVENTS');
    return;
  }
  setText('eventCount', `${events.length} EVENTS`);
  list.replaceChildren();
  for (const event of events) {
    const row = document.createElement('div');
    row.className = 'event-row';
    const mark = document.createElement('span');
    mark.className = `event-mark ${event.level === 'error' ? 'error' : event.level === 'warn' ? 'warn' : ''}`;
    mark.textContent = event.level === 'error' ? '!' : event.level === 'warn' ? '!' : '✓';
    const main = document.createElement('div');
    main.className = 'event-main';
    const message = document.createElement('strong');
    message.textContent = typeof event.message === 'string' ? event.message : 'Unnamed event';
    const time = document.createElement('small');
    time.textContent = safeDate(event.created_at);
    main.append(message, time);
    row.append(mark, main);
    list.append(row);
  }
}

async function refreshDashboard() {
  const button = $('refreshButton');
  button.disabled = true;
  button.textContent = '…';
  try {
    const [health, info, metrics, events] = await Promise.all([
      fetchJson('/api/health'), fetchJson('/api/info'), fetchJson('/api/metrics'), fetchJson('/api/events')
    ]);
    const healthy = health.status === 'ok' && health.database === 'ok';
    setText('apiStatus', healthy ? 'All systems operational' : 'Service degraded');
    setText('lastUpdated', `Updated ${new Date().toLocaleTimeString()}`);
    setText('serviceApi', healthy ? 'Operational' : 'Degraded');
    setText('serviceDb', health.database === 'ok' ? 'Connected' : 'Unavailable');
    setText('footerVersion', `v${info.version}`);

    const cpu = Number(metrics.cpu_percent);
    setText('cpuValue', Number.isFinite(cpu) ? cpu.toFixed(1) : '—');
    setText('cpuCaption', Number.isFinite(cpu) ? (cpu > 85 ? 'High load' : 'Current load') : 'Unavailable');
    setMeter('cpuBar', cpu);

    const ram = metrics.memory || {};
    setText('ramValue', Number.isFinite(Number(ram.used_percent)) ? Number(ram.used_percent).toFixed(1) : '—');
    setText('ramDetail', `${formatBytes(Number(ram.used_bytes))} used of ${formatBytes(Number(ram.total_bytes))}`);
    setMeter('ramBar', ram.used_percent);

    const disk = metrics.disk;
    if (disk) {
      setText('diskValue', Number(disk.used_percent).toFixed(1));
      setText('diskDetail', `${formatBytes(Number(disk.available_bytes))} available`);
      setText('diskCaption', `${disk.mount_point || 'Root'} volume`);
      setMeter('diskBar', disk.used_percent);
    } else {
      setText('diskValue', '—'); setText('diskDetail', 'Metrics unavailable'); setText('diskCaption', 'Root volume'); setMeter('diskBar', 0);
    }
    setText('uptimeValue', formatUptime(Number(metrics.system_uptime_seconds)));
    renderEvents(events);
  } catch (error) {
    setText('apiStatus', 'API unavailable');
    setText('lastUpdated', 'Could not fetch dashboard data');
    setText('serviceApi', 'Unavailable');
    setText('serviceDb', 'Unknown');
    const list = $('eventsList');
    list.replaceChildren();
    const message = document.createElement('div');
    message.className = 'empty-state';
    const title = document.createElement('strong');
    title.textContent = 'NEXUS API is unreachable';
    const detail = document.createElement('small');
    detail.textContent = 'Start the server and refresh this page.';
    message.append(title, detail);
    list.append(message);
    setText('eventCount', '— EVENTS');
    console.error('NEXUS refresh failed:', error);
  } finally {
    button.disabled = false;
    button.textContent = '↻';
  }
}

let toastTimer;
function toast(text) {
  const target = $('toast');
  target.textContent = text;
  target.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => target.classList.remove('show'), 2400);
}

document.querySelectorAll('[data-future]').forEach((button) => {
  button.addEventListener('click', () => toast(`${button.dataset.future} will be added in a later NEXUS phase.`));
});
$('refreshButton').addEventListener('click', refreshDashboard);
const sidebar = $('sidebar');
const menuToggle = $('menuToggle');
menuToggle.addEventListener('click', () => {
  const isOpen = sidebar.classList.toggle('open');
  menuToggle.setAttribute('aria-expanded', String(isOpen));
});
document.addEventListener('click', (event) => {
  if (window.innerWidth <= 760 && sidebar.classList.contains('open') && !sidebar.contains(event.target) && !menuToggle.contains(event.target)) {
    sidebar.classList.remove('open');
    menuToggle.setAttribute('aria-expanded', 'false');
  }
});

refreshDashboard();
