(() => {
  'use strict';
  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
  const state = { user: null, csrfToken: null, authMode: 'login', authStatus: null, health: null, info: null, metrics: null, events: [], updatedAt: null, range: '24H', page: 'overview', loading: false };
  const iconPaths = {
    home: '<path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/>',
    cube: '<path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 7 9 5 9-5M3 7v10l9 5 9-5V7M12 12v10"/>',
    bot: '<rect x="4" y="7" width="16" height="13" rx="3"/><path d="M12 3v4M8 12h.01M16 12h.01M9 16h6M2 12h2m16 0h2"/>',
    sparkles: '<path d="m12 3 1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8L12 3Z"/><path d="m19 14 .9 2.1L22 17l-2.1.9L19 20l-.9-2.1L16 17l2.1-.9L19 14ZM5 3l.7 1.8L7.5 5.5l-1.8.7L5 8l-.7-1.8L2.5 5.5l1.8-.7L5 3Z"/>',
    box: '<path d="m12 2 9 5v10l-9 5-9-5V7l9-5Z"/><path d="m3 7 9 5 9-5M12 12v10"/>',
    chart: '<path d="M3 3v18h18"/><path d="m7 14 4-4 3 2 5-7"/>',
    database: '<ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v14c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="m19.4 15 .1.1 1.4 1.1-1.4 2.4-1.7-.6a8 8 0 0 1-1.5.9l-.3 1.8h-2.8l-.3-1.8a8 8 0 0 1-1.5-.9l-1.7.6-1.4-2.4L7.7 15a7 7 0 0 1 0-1.8l-1.4-1.1 1.4-2.4 1.7.6a8 8 0 0 1 1.5-.9l.3-1.8H14l.3 1.8a8 8 0 0 1 1.5.9l1.7-.6 1.4 2.4-1.4 1.1a7 7 0 0 1-.1 1.8Z"/>',
    search: '<circle cx="10.8" cy="10.8" r="7"/><path d="m16 16 5 5"/>',
    bell: '<path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M10 21h4"/>',
    'chevron-down': '<path d="m6 9 6 6 6-6"/>',
    menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
    logout: '<path d="M10 17l5-5-5-5M15 12H3"/><path d="M12 3h6a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3h-6"/>',
    server: '<rect x="3" y="3" width="18" height="7" rx="2"/><rect x="3" y="14" width="18" height="7" rx="2"/><path d="M7 6h.01M7 17h.01M11 6h6M11 17h6"/>',
    cpu: '<rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 9h6v6H9zM9 2v4m6-4v4M9 18v4m6-4v4M2 9h4m-4 6h4m12-6h4m-4 6h4"/>',
    memory: '<rect x="3" y="6" width="18" height="12" rx="2"/><path d="M7 3v3m5-3v3m5-3v3M7 18v3m5-3v3m5-3v3M7 10h2v4H7zm4 0h2v4h-2zm4 0h2v4h-2z"/>',
    'hard-drive': '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M7 15h10M8 10h.01M12 10h.01M16 10h.01"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    refresh: '<path d="M20 7v5h-5M4 17v-5h5"/><path d="M5.6 9A7 7 0 0 1 18 6l2 6M4 12l2 6a7 7 0 0 0 12.4-3"/>',
    list: '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
    layers: '<path d="m12 3 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5M3 16l9 5 9-5"/>',
    user: '<circle cx="12" cy="8" r="4"/><path d="M4 21v-2a8 8 0 0 1 16 0v2"/>',
    shield: '<path d="M12 22s8-4 8-11V5l-8-3-8 3v6c0 7 8 11 8 11Z"/><path d="m9 12 2 2 4-4"/>',
    activity: '<path d="M3 12h4l3-8 4 16 3-8h4"/>',
    check: '<path d="m5 12 4 4L19 6"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 7h.01"/>'
  };
  function icon(name, extra = '') { return `<svg class="${extra}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${iconPaths[name] || iconPaths.info}</svg>`; }
  function mountIcons(root = document) { $$('[data-icon]', root).forEach(el => { const name = el.dataset.icon; el.innerHTML = icon(name); }); }
  function toast(message, type = 'info') {
    const host = $('#toast-region'); const el = document.createElement('div'); el.className = `toast ${type}`; el.textContent = message; host.appendChild(el); window.setTimeout(() => el.remove(), 4200);
  }
  async function api(path, options = {}) {
    const headers = new Headers(options.headers || {});
    if (options.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
    const response = await fetch(path, { credentials: 'same-origin', cache: 'no-store', ...options, headers });
    const type = response.headers.get('content-type') || '';
    let payload = null;
    if (type.includes('application/json')) { try { payload = await response.json(); } catch (_) { payload = null; } }
    else { try { payload = await response.text(); } catch (_) { payload = null; } }
    if (!response.ok) {
      const message = payload && typeof payload === 'object' && (payload.error || payload.message) ? (payload.error || payload.message) : `Request failed (${response.status})`;
      const error = new Error(message); error.status = response.status; error.payload = payload; throw error;
    }
    return payload;
  }
  function safeString(value, fallback = '') { return typeof value === 'string' && value.trim() ? value.trim() : fallback; }
  function toNumber(value) { if (typeof value === 'number' && Number.isFinite(value)) return value; if (typeof value === 'string' && value.trim() !== '' && Number.isFinite(Number(value))) return Number(value); return null; }
  function numberAt(obj, keys) {
    if (!obj || typeof obj !== 'object') return null;
    for (const key of keys) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) { const v = toNumber(obj[key]); if (v !== null) return v; }
    }
    return null;
  }
  function valueAt(obj, paths) {
    for (const path of paths) { let current = obj; for (const part of path.split('.')) current = current && typeof current === 'object' ? current[part] : undefined; if (current !== undefined && current !== null) return current; }
    return null;
  }
  function percentFrom(obj, directKeys, usedKeys, totalKeys) {
    const direct = numberAt(obj, directKeys); if (direct !== null) return clamp(direct, 0, 100);
    const used = numberAt(obj, usedKeys), total = numberAt(obj, totalKeys); if (used !== null && total !== null && total > 0) return clamp(used / total * 100, 0, 100);
    return null;
  }
  function clamp(n, lo, hi) { return Math.max(lo, Math.min(hi, n)); }
  function findMetric(data, paths, directKeys, usedKeys = [], totalKeys = []) {
    for (const path of paths) { const object = valueAt(data, [path]); const scalar = toNumber(object); if (scalar !== null) return scalar; const n = numberAt(object, directKeys); if (n !== null) return n; const p = percentFrom(object, directKeys, usedKeys, totalKeys); if (p !== null) return p; }
    const allObjects = [data, valueAt(data, ['system']), valueAt(data, ['metrics']), valueAt(data, ['resources']), valueAt(data, ['system_metrics'])].filter(v => v && typeof v === 'object');
    for (const object of allObjects) { const p = percentFrom(object, directKeys, usedKeys, totalKeys); if (p !== null) return p; }
    return null;
  }
  function formatBytes(value) {
    const n = toNumber(value); if (n === null || n < 0) return null;
    const units = ['B', 'KB', 'MB', 'GB', 'TB']; let amount = n, index = 0;
    while (amount >= 1024 && index < units.length - 1) { amount /= 1024; index++; }
    if (index === 1 && amount >= 1024) { amount /= 1024; index++; }
    return `${amount >= 100 ? amount.toFixed(0) : amount >= 10 ? amount.toFixed(1) : amount.toFixed(2)} ${units[index]}`;
  }
  function setText(id, value) { const el = document.getElementById(id); if (el) el.textContent = value; }
  function formatPercent(value) { return value === null ? '—' : `${Math.round(value * 10) / 10}`; }
  function setProgress(id, value) { const el = document.getElementById(id); if (el) el.style.width = value === null ? '0%' : `${clamp(value, 0, 100)}%`; }
  function extractMetrics(data) {
    const cpu = findMetric(data, ['cpu', 'cpu_usage', 'cpuUsage'], ['cpu_percent','cpu_usage_percent','cpu_usage_pct','cpu_usage','cpuUsage','usage_percent','percent','usage','value'], ['used','used_bytes'], ['total','total_bytes']);
    const memory = findMetric(data, ['memory','mem','ram','memory_usage'], ['memory_percent','memory_usage_percent','memory_usage_pct','memory_usage','ram_percent','percent','usage_percent'], ['used_bytes','used','memory_used_bytes','used_memory'], ['total_bytes','total','memory_total_bytes','total_memory']);
    const disk = findMetric(data, ['disk','storage','disk_usage'], ['disk_percent','disk_usage_percent','disk_usage_pct','disk_usage','storage_percent','percent','usage_percent'], ['used_bytes','used','disk_used_bytes'], ['total_bytes','total','disk_total_bytes']);
    const memUsed = numberAt(data, ['memory_used_bytes','memory_used','mem_used','used_memory_bytes']) ?? numberAt(valueAt(data,['memory']) || {}, ['used_bytes','used','used_memory']);
    const memTotal = numberAt(data, ['memory_total_bytes','memory_total','mem_total','total_memory_bytes']) ?? numberAt(valueAt(data,['memory']) || {}, ['total_bytes','total']);
    const diskUsed = numberAt(data, ['disk_used_bytes','disk_used','storage_used_bytes']) ?? numberAt(valueAt(data,['disk']) || {}, ['used_bytes','used']);
    const diskTotal = numberAt(data, ['disk_total_bytes','disk_total','storage_total_bytes']) ?? numberAt(valueAt(data,['disk']) || {}, ['total_bytes','total']);
    const cpuCores = numberAt(data, ['cpu_cores','core_count','cores','logical_cores']);
    const uptime = valueAt(data, ['uptime_seconds','uptime_secs','uptime','uptime_seconds_total','system.uptime_seconds','server.uptime_seconds']);
    let uptimeSeconds = toNumber(uptime);
    if (uptime && typeof uptime === 'object') uptimeSeconds = numberAt(uptime,['seconds','total_seconds','secs']);
    if (typeof uptime === 'string' && uptimeSeconds === null) { const parsed = uptime.match(/(?:(\d+)d)?\s*(?:(\d+)h)?\s*(?:(\d+)m)?/i); if (parsed && parsed[0].trim()) uptimeSeconds = (+parsed[1]||0)*86400+(+parsed[2]||0)*3600+(+parsed[3]||0)*60; }
    return { cpu: cpu === null ? null : clamp(cpu,0,100), memory: memory === null ? null : clamp(memory,0,100), disk: disk === null ? null : clamp(disk,0,100), memUsed, memTotal, diskUsed, diskTotal, cpuCores, uptimeSeconds };
  }
  function formatUptime(seconds) { if (seconds === null || !Number.isFinite(seconds) || seconds < 0) return '—'; const s = Math.floor(seconds), d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60); if (d) return `${d}d ${h}h`; if (h) return `${h}h ${m}m`; return `${m}m`; }
  function showAuth(mode = 'login') {
    state.authMode = mode; $('#auth-screen').hidden = false; $('#app-shell').hidden = true;
    const firstAccount = Boolean(state.authStatus && state.authStatus.initialized === false);
    if (mode === 'register' && firstAccount) { setText('auth-title','Create your administrator account.'); setText('auth-description','This will be the first account for this NEXUS instance.'); setText('auth-submit','Create administrator ↗'); }
    else if (mode === 'register') { setText('auth-title','Create an account.'); setText('auth-description','Create a NEXUS account to access your workspace.'); setText('auth-submit','Create account ↗'); }
    else { setText('auth-title','Welcome back.'); setText('auth-description','Sign in to manage your infrastructure.'); setText('auth-submit','Sign in ↗'); }
    $('#auth-toggle').hidden = firstAccount && mode === 'register'; $('#auth-toggle').textContent = mode === 'register' ? 'Already have an account? Sign in' : 'Need an account? Create one';
    $('#auth-password').setAttribute('autocomplete', mode === 'register' ? 'new-password' : 'current-password'); $('#auth-error').hidden = true;
  }
  async function boot() {
    mountIcons(); updateClock(); window.setInterval(updateClock, 30000);
    try { state.authStatus = await api('/api/auth/status'); } catch (error) { state.authStatus = null; }
    try {
      const session = await api('/api/auth/me');
      acceptSession(session);
    } catch (error) {
      showAuth(state.authStatus && state.authStatus.initialized === false ? 'register' : 'login');
    }
    wireEvents();
  }
  function acceptSession(payload) {
    if (!payload || !payload.user) throw new Error('Authentication response did not include a user.');
    state.user = payload.user; state.csrfToken = payload.csrf_token || payload.csrfToken || null;
    $('#auth-screen').hidden = true; $('#app-shell').hidden = false;
    const displayName = safeString(state.user.display_name || state.user.name, safeString(state.user.email, 'NEXUS User').split('@')[0]);
    const role = safeString(state.user.role, 'user');
    ['sidebar-user','menu-user'].forEach(id => setText(id,displayName)); ['sidebar-role','menu-role'].forEach(id => setText(id,role.charAt(0).toUpperCase()+role.slice(1)));
    const initials = displayName.trim().split(/[\s._-]+/).slice(0,2).map(part=>part[0]).join('').toUpperCase() || 'N';
    ['sidebar-avatar','top-avatar'].forEach(id=>setText(id,initials));
    loadDashboard();
  }
  async function handleAuthSubmit(event) {
    event.preventDefault(); const form = event.currentTarget; const submit = $('#auth-submit'); const errorEl = $('#auth-error');
    const email = $('#auth-email').value.trim(); const password = $('#auth-password').value;
    if (!email || !password) return;
    if (state.authMode === 'register' && (new TextEncoder().encode(password).length < 12 || new TextEncoder().encode(password).length > 128)) { errorEl.textContent='Password must be between 12 and 128 bytes.'; errorEl.hidden=false; return; }
    submit.disabled = true; submit.textContent = state.authMode === 'register' ? 'Creating account…' : 'Signing in…'; errorEl.hidden = true;
    try { const result = await api(`/api/auth/${state.authMode}`, { method:'POST', body:JSON.stringify({email,password}) }); $('#auth-password').value=''; acceptSession(result); toast(state.authMode==='register'?'Account created successfully.':'Signed in successfully.','success'); }
    catch (error) { errorEl.textContent = error.message || 'Authentication failed. Please try again.'; errorEl.hidden=false; }
    finally { submit.disabled=false; submit.textContent = state.authMode==='register' ? 'Create account ↗' : 'Sign in ↗'; }
  }
  async function loadDashboard() {
    if (state.loading) return; state.loading=true; setText('last-updated','Refreshing…');
    const results = await Promise.allSettled([api('/api/health'),api('/api/info'),api('/api/metrics'),api('/api/events')]);
    if (results[0].status==='fulfilled') state.health=results[0].value;
    if (results[1].status==='fulfilled') state.info=results[1].value;
    if (results[2].status==='fulfilled') state.metrics=results[2].value;
    if (results[3].status==='fulfilled') state.events=normalizeEvents(results[3].value);
    const authFailure=results.some(r=>r.status==='rejected'&&r.reason&&r.reason.status===401);
    state.loading=false;
    if(authFailure){ state.user=null; showAuth('login'); toast('Your session expired. Please sign in again.','error'); return; }
    renderHealth(); renderMetrics(); renderServices(); renderEvents(); renderHistoryChart(); state.updatedAt=new Date(); setText('last-updated',`Updated ${state.updatedAt.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'})}`);
    const failures=results.filter(r=>r.status==='rejected'); if(failures.length) toast(`${failures.length} dashboard data request${failures.length===1?'':'s'} unavailable. Showing known data only.`,'error');
  }
  function renderHealth() {
    const status=safeString(state.health && state.health.status,'unknown').toLowerCase(); const healthy=status==='ok'||status==='healthy';
    const db=safeString(state.health && state.health.database,'unknown').toLowerCase(); const dbHealthy=db==='ok'||db==='healthy';
    setText('server-status-value',healthy?'Online':status==='unknown'?'Unknown':'Degraded'); setText('server-status-detail',`API ${healthy?'reachable':'not verified'} · Database ${dbHealthy?'healthy':db==='unknown'?'not reported':'not verified'}`);
    $('#server-status-dot').className=`status-dot ${healthy&&dbHealthy?'':'status-dot-muted'}`; $('#sidebar-health-dot').className=`status-dot ${healthy?'':'status-dot-muted'}`; $('#footer-status-dot').className=`status-dot ${healthy?'':'status-dot-muted'}`;
    setText('sidebar-health-title',healthy?'NEXUS Online':'Status unavailable'); setText('sidebar-health-subtitle',`API ${healthy?'reachable':'not verified'}`); setText('footer-status',healthy?'Phase 2 API online':'API status unavailable');
    const version=safeString(state.health&&state.health.version,safeString(state.info&&state.info.version,'Phase 2')); setText('footer-version',`NEXUS · ${version}`);
    $('#notification-dot').hidden=true; // Unread state is not provided by the current API.
  }
  function renderMetrics() {
    if (!state.metrics) { ['cpu-value','memory-value','disk-value','uptime-value'].forEach(id=>setText(id,'—')); ['cpu-detail','memory-detail','disk-detail','uptime-detail'].forEach(id=>setText(id,'Not reported by API')); return; }
    const m=extractMetrics(Object.assign({}, state.info || {}, state.metrics));
    setText('cpu-value',formatPercent(m.cpu));setText('memory-value',formatPercent(m.memory));setText('disk-value',formatPercent(m.disk));setText('uptime-value',formatUptime(m.uptimeSeconds));
    setText('cpu-detail',m.cpuCores!==null?`${m.cpuCores} logical cores reported`:'Current sample');
    setText('memory-detail',m.memUsed!==null&&m.memTotal!==null?`${formatBytes(m.memUsed)} / ${formatBytes(m.memTotal)}`:'Current sample');
    setText('disk-detail',m.diskUsed!==null&&m.diskTotal!==null?`${formatBytes(m.diskUsed)} / ${formatBytes(m.diskTotal)}`:'Current sample');
    setText('uptime-detail',m.uptimeSeconds!==null?'Reported by the API':'Not reported by API');
    setProgress('cpu-progress',m.cpu);setProgress('memory-progress',m.memory);setProgress('disk-progress',m.disk);
    $$('.metric-card .metric-note').forEach(el=>{if(el.textContent.includes('Unavailable'))el.textContent='Not reported by API';});
  }
  function normalizeEvents(payload) {
    let list=Array.isArray(payload)?payload:null;
    if(!list&&payload&&typeof payload==='object') for(const key of ['events','items','results','activity','data']) if(Array.isArray(payload[key])){list=payload[key];break;}
    if(!list)return [];
    return list.slice(0,40).map((item,index)=>({title:safeString(item.message||item.title||item.action||item.event,'System event'),detail:safeString(item.detail||item.description||item.metadata?.description,''),level:safeString(item.level||item.type||item.severity,'info').toLowerCase(),time:item.created_at||item.timestamp||item.time||null,id:item.id??index}));
  }
  function formatTime(value) {
    if(!value)return 'Time unavailable'; const date=typeof value==='number'?new Date(value<1000000000000?value*1000:value):new Date(value);
    return Number.isNaN(date.getTime())?'Time unavailable':date.toLocaleString([], {month:'short',day:'numeric',hour:'numeric',minute:'2-digit'});
  }
  function eventClass(level) { return /error|fail|critical/.test(level)?'warn':/warn/.test(level)?'warn':/login|auth/.test(level)?'neutral':''; }
  function eventIcon(level) { return /error|fail|critical/.test(level)?'info':/login|auth/.test(level)?'user':/backup/.test(level)?'database':/start|stop|service/.test(level)?'server':'activity'; }
  function eventMarkup(event) { return `<article class="activity-row"><span class="activity-pin" aria-hidden="true"></span><span class="activity-icon ${eventClass(event.level)}">${icon(eventIcon(`${event.level} ${event.title}`))}</span><span class="activity-copy"><strong>${escapeHTML(event.title)}</strong><small>${escapeHTML(event.detail||`Level: ${event.level}`)}</small></span><time class="activity-time">${escapeHTML(formatTime(event.time))}</time></article>`; }
  function escapeHTML(value) { return String(value??'').replace(/[&<>"']/g, c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])); }
  function renderEvents() {
    const host=$('#activity-list'); if(!state.events.length){host.innerHTML='<div class="list-empty"><strong>No activity recorded yet</strong>Events will appear here when the server reports them.</div>';$('#dialog-events-list').innerHTML='<div class="list-empty"><strong>No events available</strong>The API returned no activity records.</div>';return;}
    host.innerHTML=state.events.slice(0,5).map(eventMarkup).join(''); $('#dialog-events-list').innerHTML=state.events.map(eventMarkup).join('');
  }
  function renderServices() {
    const healthy=state.health&&['ok','healthy'].includes(safeString(state.health.status).toLowerCase());
    const rows=[{name:'NEXUS Phase 2',sub:'Authenticated API',status:healthy?'Running':'Unknown',kind:healthy?'ok':'muted',symbol:'server',symbolClass:healthy?'service-symbol-green':''}];
    const remoteChecks=state.info&&Array.isArray(state.info.services)?state.info.services:[];
    for(const service of remoteChecks){if(!service||typeof service!=='object')continue;const name=safeString(service.name||service.service);if(!name||/phase.?2/i.test(name))continue;const raw=safeString(service.status,'unknown').toLowerCase();const ok=['running','active','ok','healthy'].includes(raw);rows.push({name,sub:safeString(service.description||service.version,'Status supplied by API'),status:ok?'Running':raw==='planned'?'Planned':raw==='not_installed'?'Not installed':raw.charAt(0).toUpperCase()+raw.slice(1),kind:ok?'ok':raw==='planned'?'muted':'warn',symbol:'box',symbolClass:ok?'service-symbol-green':'service-symbol-muted'});}
    rows.push({name:'Bot Hosting Service',sub:'Bot runtime manager',status:'Planned',kind:'muted',symbol:'bot',symbolClass:'service-symbol-muted',planned:true});
    rows.push({name:'Hermes Agent',sub:'Existing installation not verified here',status:'Check required',kind:'warn',symbol:'sparkles',symbolClass:'service-symbol-muted',planned:true});
    $('#service-list').innerHTML=rows.slice(0,5).map(row=>`<div class="service-row"><span class="service-symbol ${row.symbolClass}">${icon(row.symbol)}</span><span class="service-copy"><strong>${escapeHTML(row.name)}</strong><small>${escapeHTML(row.sub)}</small></span><span class="service-state ${row.kind}">${escapeHTML(row.status)}</span><span class="service-chevron" aria-hidden="true">›</span></div>`).join('');
  }
  function historyArray(data) { if(!data||typeof data!=='object')return []; for(const p of ['history','timeseries','time_series','samples','points','history.samples','metrics.history']){const val=valueAt(data,[p]);if(Array.isArray(val)&&val.length>1)return val;}return []; }
  function renderHistoryChart() {
    const host=$('#resource-chart'); const points=historyArray(state.metrics);
    if(points.length<2){host.innerHTML=`<div class="chart-empty">${icon('chart')}<strong>No historical metrics available</strong><p>Current values are shown above. A time-series collector is required before this chart can display history.</p></div>`;mountIcons(host);return;}
    const get=(p,keys)=>numberAt(p,keys); const cpuKeys=['cpu','cpu_percent','cpu_usage_percent'], memKeys=['memory','memory_percent','memory_usage_percent'], diskKeys=['disk','disk_percent','disk_usage_percent'];
    const series=[{name:'CPU',color:'#fb8245',keys:cpuKeys},{name:'Memory',color:'#f4c65d',keys:memKeys},{name:'Disk',color:'#50dba6',keys:diskKeys}].map(s=>({...s,values:points.map(p=>get(p,s.keys))}));
    if(series.some(s=>s.values.every(v=>v===null))){host.innerHTML=`<div class="chart-empty">${icon('chart')}<strong>Historical fields not recognized</strong><p>The API returned samples, but not CPU, memory, and disk values in a supported format.</p></div>`;mountIcons(host);return;}
    const width=620,height=210,left=43,right=9,top=9,bottom=29,plotW=width-left-right,plotH=height-top-bottom;
    let grid='';for(let i=0;i<=4;i++){const y=top+plotH*i/4,percent=100-i*25;grid+=`<line x1="${left}" y1="${y}" x2="${width-right}" y2="${y}" stroke="#383431" stroke-width="1"/><text x="${left-8}" y="${y+4}" fill="#8d837d" text-anchor="end" font-size="10">${percent}%</text>`;}
    const paths=series.map(s=>{const v=s.values.filter(n=>n!==null);if(v.length<2)return '';const coords=s.values.map((n,i)=>n===null?null:[left+plotW*i/(s.values.length-1),top+plotH*(1-clamp(n,0,100)/100)]);let d='';let pen=false;coords.forEach(p=>{if(!p){pen=false;return;}d+=(pen?' L':' M')+p[0].toFixed(1)+' '+p[1].toFixed(1);pen=true;});return `<path d="${d}" fill="none" stroke="${s.color}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>`;}).join('');
    const labels=['Start','25%','50%','75%','Latest']; const xlabels=[0,.25,.5,.75,1].map((f,i)=>`<text x="${left+plotW*f}" y="${height-7}" fill="#8d837d" text-anchor="${i===0?'start':i===4?'end':'middle'}" font-size="10">${labels[i]}</text>`).join('');
    host.innerHTML=`<svg viewBox="0 0 ${width} ${height}" role="img" aria-label="Historical resource metrics chart">${grid}${paths}${xlabels}</svg>`;
  }
  function updateClock() { const now=new Date();setText('current-date',now.toLocaleDateString([], {weekday:'short',month:'short',day:'numeric',year:'numeric'}));setText('current-time',now.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'})); }
  function openSidebar(){ $('#app-shell').classList.add('sidebar-open'); $('#open-sidebar').setAttribute('aria-expanded','true'); }
  function closeSidebar(){ $('#app-shell').classList.remove('sidebar-open'); $('#open-sidebar').setAttribute('aria-expanded','false'); }
  const roadmap=[
    {phase:'Phase A',title:'Overview dashboard',status:'Current phase',current:true,description:'The reference-matched overview, responsive navigation, honest metrics, verified API health, events, and quick actions.',features:['Responsive sidebar and mobile layout','Auth-aware dashboard session','Live health and metrics where API fields exist','Empty states for unreported metrics/history','Event feed from the existing API']},
    {phase:'Phase B',title:'Bot hosting',status:'Planned',description:'Isolated, persistent Python and Node.js bot instances with real lifecycle control.',features:['Create bot wizard and project files','Runtime/dependency setup','Start, stop, restart and delete','Logs, environment variables, resource limits','Crash history and restart policies']},
    {phase:'Phase C',title:'Hermes Agent',status:'Planned',description:'A guided and restricted integration with the existing Hermes installation.',features:['Inspect current installation and service','Prerequisite/model configuration wizard','Secure credentials and permission boundaries','Health checks, logs, update and recovery']},
    {phase:'Phase D',title:'Applications',status:'Planned',description:'Manage only applications that have been verified through the backend.',features:['Inventory and details','Supported lifecycle operations','Logs, health checks and resource usage']},
    {phase:'Phase E',title:'Containers',status:'Planned',description:'Restricted container operations without exposing the Docker socket to workloads.',features:['Inventory and lifecycle control','Logs and resource use','Safe creation and cleanup']},
    {phase:'Phase F',title:'Monitoring & backups',status:'Planned',description:'Real measurements, operational alerts, and tested restore workflows.',features:['Metric collection and trends','Service alerts and restart events','Backup creation, integrity, retention and restore']},
    {phase:'Phase G',title:'Settings & security',status:'Planned',description:'Centralized account, configuration and audit experiences.',features:['Account and session settings','Access policies and resource limits','Security events and audit logs']}
  ];
  function renderRoadmap(){ $('#roadmap-content').innerHTML=roadmap.map(p=>`<article class="roadmap-card"><div class="roadmap-card-header"><h3>${escapeHTML(p.phase)} · ${escapeHTML(p.title)}</h3><span class="roadmap-status ${p.current?'current':''}">${escapeHTML(p.status)}</span></div><p>${escapeHTML(p.description)}</p><ul class="roadmap-list">${p.features.map(f=>`<li>${escapeHTML(f)}</li>`).join('')}</ul></article>`).join(''); }
  function renderPage(page){state.page=page;closeSidebar();$$('.nav-item').forEach(a=>a.classList.toggle('active',a.dataset.page===page));$('#overview-page').hidden=page!=='overview';$('#secondary-page').hidden=page==='overview';$('#secondary-page').classList.toggle('page-active',page!=='overview');if(page==='overview')return;
    const roadmapPage=['roadmap','bot-hosting','ai-agents','applications','containers','monitoring','backups','settings'].includes(page);
    if(!roadmapPage){renderPage('overview');return;}
    const map={'roadmap':['Feature roadmap','Roadmap status is a plan, not proof that these features are implemented.'],'applications':['Applications','The Applications module is planned; no lifecycle operations are exposed in this Phase A build.'],'bot-hosting':['Bot Hosting','Bot hosting is planned. Real runtime isolation, lifecycle controls, logs, and resource limits must be implemented before this section is enabled.'],'ai-agents':['AI Agents','The agent management workflow is planned. The existing Hermes installation is not modified by this dashboard.'],'containers':['Containers','Container management is planned. No Docker lifecycle commands are issued by this Phase A build.'],'monitoring':['Monitoring','Historical metrics and alerts require a verified collector. Current metrics are displayed only when reported by the backend.'],'backups':['Backups','Backup and restore operations are not yet implemented. No backup is claimed to exist until the backend verifies it.'],'settings':['Settings','Settings controls will be introduced when their server-side behavior and access controls are implemented.']};
    const info=map[page]||['Product roadmap','See the planned feature phases.'];setText('secondary-title',info[0]);setText('secondary-description',info[1]);renderRoadmap();
  }
  const searchItems=[{title:'Overview',sub:'Dashboard and server metrics',icon:'home',page:'overview'},{title:'Applications',sub:'Planned application management',icon:'cube',page:'applications'},{title:'Bot Hosting',sub:'Planned Python and Node.js hosting',icon:'bot',page:'bot-hosting'},{title:'AI Agents',sub:'Planned Hermes management',icon:'sparkles',page:'ai-agents'},{title:'Containers',sub:'Planned container management',icon:'box',page:'containers'},{title:'Monitoring',sub:'Metrics and alerts roadmap',icon:'chart',page:'monitoring'},{title:'Backups',sub:'Backup and restore roadmap',icon:'database',page:'backups'},{title:'Settings',sub:'Configuration roadmap',icon:'settings',page:'settings'},{title:'Product Roadmap',sub:'View the full feature plan',icon:'layers',page:'roadmap'},{title:'Refresh Overview',sub:'Fetch current health, metrics, and events',icon:'refresh',action:'refresh'},{title:'Recent Events',sub:'View server activity returned by the API',icon:'list',action:'events'}];
  function updateSearch(){const q=$('#command-search').value.toLowerCase().trim();const items=searchItems.filter(i=>(i.title+' '+i.sub).toLowerCase().includes(q));$('#search-results').innerHTML=items.length?items.map((item,i)=>`<button class="search-result" type="button" data-search-index="${searchItems.indexOf(item)}">${icon(item.icon)}<span>${escapeHTML(item.title)}<small>${escapeHTML(item.sub)}</small></span></button>`).join(''):'<div class="search-empty">No matching pages or actions.</div>';}
  function selectSearch(index){const item=searchItems[index];if(!item)return;$('#search-dialog').close();if(item.page)renderPage(item.page);if(item.action==='refresh')loadDashboard();if(item.action==='events')openDialog('events-dialog');}
  function openDialog(id){const el=document.getElementById(id);if(el&&typeof el.showModal==='function')el.showModal();}
  async function signOut(){try{if(state.csrfToken)await api('/api/auth/logout',{method:'POST',headers:{'x-csrf-token':state.csrfToken}});}catch(error){toast('Sign-out request failed; clearing this view.','error');}state.user=null;state.csrfToken=null;state.metrics=null;state.events=[];try{state.authStatus=await api('/api/auth/status');}catch(_){}showAuth('login');toast('Signed out.','success');}
  function wireEvents(){
    $('#auth-form').addEventListener('submit',handleAuthSubmit);
    $('#auth-toggle').addEventListener('click',()=>showAuth(state.authMode==='register'?'login':'register'));
    $('#open-sidebar').addEventListener('click',openSidebar);$('#close-sidebar').addEventListener('click',closeSidebar);$('#sidebar-scrim').addEventListener('click',closeSidebar);
    $$('.nav-item').forEach(a=>a.addEventListener('click',e=>{e.preventDefault();renderPage(a.dataset.page);}));
    $('#back-overview').addEventListener('click',()=>renderPage('overview'));
    $('#quick-refresh').addEventListener('click',()=>loadDashboard());$('#refresh-services').addEventListener('click',()=>loadDashboard());$('#refresh-events').addEventListener('click',async()=>{await loadDashboard();toast('Event list refreshed.','success');});
    $('#quick-events').addEventListener('click',()=>openDialog('events-dialog'));$('#quick-bots').addEventListener('click',()=>renderPage('bot-hosting'));$('#quick-roadmap').addEventListener('click',()=>renderPage('roadmap'));
    $('#server-health-button').addEventListener('click',()=>{loadDashboard();toast('Checking NEXUS health and reported metrics.');});$('#server-selector').addEventListener('click',()=>toast('Only the current local NEXUS instance is configured. Multi-server selection is not implemented.'));
    $('#notifications-button').addEventListener('click',()=>openDialog('events-dialog'));
    $('#profile-button').addEventListener('click',()=>{const menu=$('#profile-menu');menu.hidden=!menu.hidden;});$('#sign-out-bottom').addEventListener('click',signOut);$('#sign-out-top').addEventListener('click',signOut);
    $('#search-trigger').addEventListener('click',()=>{updateSearch();openDialog('search-dialog');$('#command-search').focus();});$('#command-search').addEventListener('input',updateSearch);$('#search-results').addEventListener('click',e=>{const b=e.target.closest('[data-search-index]');if(b)selectSearch(Number(b.dataset.searchIndex));});
    document.addEventListener('keydown',e=>{if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();updateSearch();openDialog('search-dialog');$('#command-search').focus();}if(e.key==='Escape')closeSidebar();if(e.key==='ArrowDown'&&$('#search-dialog').open){const buttons=$$('.search-result');const idx=buttons.indexOf(document.activeElement);buttons[Math.min(idx+1,buttons.length-1)]?.focus();}if(e.key==='ArrowUp'&&$('#search-dialog').open){const buttons=$$('.search-result');const idx=buttons.indexOf(document.activeElement);buttons[Math.max(idx-1,0)]?.focus();}if(e.key==='Enter'&&$('#search-dialog').open&&document.activeElement===$('#command-search')){$('.search-result')?.click();}});
    $$('[data-close-dialog]').forEach(b=>b.addEventListener('click',()=>document.getElementById(b.dataset.closeDialog).close()));$('#notice-ok').addEventListener('click',()=>$('#notice-dialog').close());
    $('#range-controls').addEventListener('click',e=>{const b=e.target.closest('[data-range]');if(!b)return;state.range=b.dataset.range;$$('#range-controls button').forEach(item=>{const selected=item===b;item.classList.toggle('selected',selected);item.setAttribute('aria-pressed',String(selected));});const data=state.metrics;const arr=historyArray(data);if(arr.length){toast(`Time range selection saved: ${state.range}. The API currently supplies ${arr.length} samples; server-side range filtering is not available.`, 'info');}else toast('Time range controls become active when historical samples are available.');});
    $('#server-selector').title='Current server';
  }
  document.addEventListener('DOMContentLoaded',boot);
})();
