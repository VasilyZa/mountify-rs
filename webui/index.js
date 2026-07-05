import { exec, toast } from 'kernelsu-alt';
import { t, getLang, setLang, getLanguages, initLanguage } from './language.js';

const MODDIR = '/data/adb/modules/mountify';
const CFG = '/data/adb/mountify/config.sh';

let config = {};

const META = {
    mountify_mounts: { tab: 'general', opts: [0, 1, 2], key: 'mountify_mounts' },
    FAKE_MOUNT_NAME: { tab: 'general', key: 'FAKE_MOUNT_NAME' },
    test_decoy_mount: { tab: 'general', opts: [0, 1], key: 'test_decoy_mount' },
    mountify_stop_start: { tab: 'general', opts: [0, 1], key: 'mountify_stop_start' },
    FS_TYPE_ALIAS: { tab: 'general', opts: ['overlay'], key: 'FS_TYPE_ALIAS' },
    MOUNT_DEVICE_NAME: { tab: 'general', opts: ['overlay', 'KSU', 'APatch', 'magisk'], key: 'MOUNT_DEVICE_NAME' },
    mountify_custom_umount: { tab: 'general', opts: [0, 1, 2], key: 'mountify_custom_umount' },
    mountify_expert_mode: { tab: 'general', opts: [0, 1], key: 'mountify_expert_mode', advanced: true },
    use_ext4_sparse: { tab: 'ext4', opts: [0, 1], key: 'use_ext4_sparse' },
    spoof_sparse: { tab: 'ext4', opts: [0, 1], key: 'spoof_sparse' },
    FAKE_APEX_NAME: { tab: 'ext4', opts: ['com.android.mntservice'], key: 'FAKE_APEX_NAME' },
    sparse_size: { tab: 'ext4', opts: ['512', '1024', '2048', '4096'], key: 'sparse_size' },
    enable_lkm_nuke: { tab: 'ext4', opts: [0, 1], key: 'enable_lkm_nuke' },
    lkm_filename: { tab: 'ext4', opts: [
        'nuke-android12-5.10.ko', 'nuke-android13-5.10.ko', 'nuke-android13-5.15.ko',
        'nuke-android14-5.15.ko', 'nuke-android14-6.1.ko', 'nuke-android15-6.6.ko',
        'nuke-android16-6.12.ko', 'nuke-android-4.14.ko'
    ], key: 'lkm_filename' },
};

function lang(key) { return t(key); }

async function loadConfig() {
    const r = await exec(`cat ${CFG} 2>/dev/null || echo "#fail"`);
    if (r.errno !== 0 || r.stdout.includes('#fail')) { toast(lang('toast.config_not_found')); return {}; }
    const cfg = {};
    for (const line of r.stdout.split('\n')) {
        const s = line.trim();
        if (!s || s.startsWith('#') || !s.includes('=')) continue;
        const idx = s.indexOf('=');
        let k = s.slice(0, idx).trim(), v = s.slice(idx + 1).trim();
        if (v.startsWith('"') && v.endsWith('"')) v = v.slice(1, -1);
        cfg[k] = v;
    }
    return cfg;
}

async function saveConfig() {
    const lines = Object.entries(config).map(([k, v]) => {
        const isStr = isNaN(v) || (typeof v === 'string' && !/^\d+$/.test(v));
        return isStr ? `${k}="${v}"` : `${k}=${v}`;
    });
    const r = await exec(`cat > ${CFG} << 'EOF'\n${lines.join('\n')}\nEOF`);
    if (r.errno !== 0) toast(lang('toast.save_failed'));
}

function updateStatus() {
    const modeMap = { '0': lang('stat.off'), '1': lang('stat.manual'), '2': lang('stat.auto') };
    const fsMap = { '1': lang('stat.ext4') };
    document.getElementById('stat-mode').textContent = modeMap[config.mountify_mounts] || '--';
    document.getElementById('stat-fstype').textContent = config.use_ext4_sparse === '1' ? lang('stat.ext4') : config.spoof_sparse === '1' ? lang('stat.apex') : lang('stat.tmpfs');
    document.getElementById('stat-modules').textContent = config.mountify_mounts === '0' ? lang('stat.na') : lang('stat.active');
}

function applyI18n() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.dataset.i18n;
        const text = lang(key);
        if (text !== key) el.textContent = text;
    });
    // tab labels need span inside
    document.querySelectorAll('.tab span[data-i18n]').forEach(el => {
        el.textContent = lang(el.dataset.i18n);
    });
}

function render() {
    for (const tab of ['general', 'ext4', 'advanced']) {
        const el = document.getElementById(`${tab}-group`);
        if (el) el.innerHTML = '';
    }

    for (const [key, meta] of Object.entries(META)) {
        const group = document.getElementById(`${meta.tab}-group`);
        if (!group) continue;

        const row = document.createElement('div');
        row.className = 'config-row';
        if (meta.advanced) row.dataset.advanced = 'true';

        const label = document.createElement('div');
        label.className = 'label';
        label.textContent = key;
        const hint = document.createElement('span');
        hint.className = 'hint';
        hint.textContent = '?';
        const descKey = `desc.${meta.key}`;
        hint.title = lang(descKey);
        hint.onclick = (e) => { e.stopPropagation(); showDesc(key, lang(descKey)); };
        label.appendChild(hint);

        const ctrl = document.createElement('div');
        ctrl.style.cssText = 'display:flex;align-items:center;gap:8px';

        const val = String(config[key] ?? '');

        if (meta.opts) {
            const isBool = meta.opts.length === 2 && meta.opts.every(o => o === 0 || o === 1);
            if (isBool) {
                const cb = document.createElement('input');
                cb.type = 'checkbox'; cb.className = 'switch';
                cb.checked = val === '1';
                cb.onchange = async () => { config[key] = cb.checked ? '1' : '0'; await saveConfig(); updateStatus(); };
                ctrl.appendChild(cb);
            } else if (meta.opts.length <= 12) {
                const sel = document.createElement('select');
                meta.opts.forEach(o => {
                    const opt = document.createElement('option');
                    opt.value = String(o); opt.textContent = String(o);
                    if (String(o) === val) opt.selected = true;
                    sel.appendChild(opt);
                });
                sel.onchange = async () => { config[key] = sel.value; await saveConfig(); updateStatus(); };
                ctrl.appendChild(sel);
            }
        } else {
            const inp = document.createElement('input');
            inp.type = 'text'; inp.value = val;
            inp.onchange = async () => { config[key] = inp.value; await saveConfig(); };
            ctrl.appendChild(inp);
        }

        row.appendChild(label); row.appendChild(ctrl);
        group.appendChild(row);
    }

    updateStatus();
    updateMountedModules();
    applyI18n();
    toggleAdvanced();
}

function showDesc(title, body) {
    document.getElementById('modal-title').textContent = title;
    document.getElementById('modal-body').innerHTML = body.replace(/\n/g, '<br>');
    document.getElementById('desc-modal').classList.add('active');
}

function toggleAdvanced() {
    const show = document.getElementById('advanced-toggle')?.checked;
    document.querySelectorAll('[data-advanced="true"]').forEach(el => el.style.display = show ? '' : 'none');
}

async function updateMountedModules() {
    const el = document.getElementById('mounted-list');
    if (!el) return;
    const r = await exec(`cat /data/adb/mountify/mounted_modules 2>/dev/null || echo ""`);
    const modules = r.stdout.trim().split('\n').filter(Boolean);
    if (modules.length === 0) {
        el.textContent = config.mountify_mounts === '0' ? lang('stat.na') : lang('mounted.none');
        el.style.color = 'var(--fg-dim)';
    } else {
        el.innerHTML = modules.map(m => `<div style="display:flex;align-items:center;gap:8px;padding:2px 0"><span style="color:var(--accent)">&#9656;</span> ${m}</div>`).join('');
        el.style.color = 'var(--fg)';
    }
}

// --- Init ---
function initTabs() {
    document.querySelectorAll('.tab').forEach(t => {
        t.addEventListener('click', () => {
            document.querySelectorAll('.tab').forEach(x => x.classList.remove('active'));
            document.querySelectorAll('.panel').forEach(x => x.classList.remove('active'));
            t.classList.add('active');
            document.getElementById(`panel-${t.dataset.tab}`).classList.add('active');
        });
    });
}

function initModals() {
    document.querySelectorAll('.modal-overlay').forEach(m => {
        m.addEventListener('click', e => { if (e.target === m || e.target.closest('.modal-close')) m.classList.remove('active'); });
    });
}

function initSwitches() {
    document.getElementById('advanced-toggle').addEventListener('change', toggleAdvanced);
    document.getElementById('update-toggle').addEventListener('change', async function () {
        const s = this.checked ? 'updateJson' : 'updateLink';
        await exec(`sed -i "s/updateLink\\|updateJson/${s}/g" ${MODDIR}/module.prop`);
    });
    exec(`grep -q "^updateJson=" ${MODDIR}/module.prop`).then(r => {
        document.getElementById('update-toggle').checked = r.errno === 0;
    });
}

function initReboot() {
    const show = () => document.getElementById('reboot-modal').classList.add('active');
    const hide = () => document.getElementById('reboot-modal').classList.remove('active');
    document.getElementById('reboot').addEventListener('click', show);
    document.getElementById('reboot-cancel').addEventListener('click', hide);
    document.getElementById('reboot-confirm').addEventListener('click', () => {
        exec('/system/bin/reboot'); hide();
    });
}

function initLang() {
    const list = document.getElementById('lang-list');
    const modal = document.getElementById('lang-modal');

    document.getElementById('language').addEventListener('click', () => {
        list.innerHTML = getLanguages().map(l =>
            `<div class="module-item" style="cursor:pointer" data-code="${l.code}">${l.native} <span style="color:var(--fg-dim);margin-left:auto;font-size:11px">${l.name}</span></div>`
        ).join('');
        list.querySelectorAll('.module-item').forEach(el => {
            el.addEventListener('click', () => {
                setLang(el.dataset.code);
                modal.classList.remove('active');
                render();
            });
        });
        modal.classList.add('active');
    });
}

function initModulePicker() {
    const list = document.getElementById('module-list');

    const obs = setInterval(() => {
        if (document.querySelector('.config-row')) {
            clearInterval(obs);
            const btn = document.createElement('button');
            btn.className = 'btn primary';
            btn.textContent = lang('modules.select');
            btn.style.cssText = 'padding:6px 12px;font-size:11px';

            const updateBtn = () => {
                const target = [...document.querySelectorAll('.config-row')]
                    .find(r => r.querySelector('.label')?.textContent?.startsWith('mountify_mounts'));
                if (target && config.mountify_mounts === '1') {
                    if (!target.contains(btn)) target.querySelector(':scope > :last-child')?.appendChild(btn);
                } else {
                    btn.remove();
                }
            };

            btn.onclick = async () => {
                const modR = await exec(
                    `d=/data/adb/modules; for m in $(ls $d); do ` +
                    `[ -d "$d/$m/system" ] && ! [ -f "$d/$m/system/etc/hosts" ] && echo "$m"; done`
                );
                const selR = await exec(`cat /data/adb/mountify/modules.txt 2>/dev/null`);
                const mods = modR.stdout.trim().split('\n').filter(Boolean);
                const sel = selR.stdout.trim().split('\n').map(s => s.trim()).filter(Boolean);

                list.innerHTML = mods.map(m =>
                    `<div class="module-item"><input type="checkbox" value="${m}" ${sel.includes(m) ? 'checked' : ''}> ${m}</div>`
                ).join('');
                document.getElementById('module-modal').classList.add('active');
            };

            document.getElementById('module-save').onclick = () => {
                const ckd = [...list.querySelectorAll('input:checked')].map(c => c.value);
                exec(`echo "${ckd.join('\n')}" > /data/adb/mountify/modules.txt`);
                document.getElementById('module-modal').classList.remove('active');
            };

            const origRender = render;
            render = function() { origRender(); setTimeout(updateBtn, 50); };
            document.addEventListener('change', e => {
                if (e.target.closest('.config-row')?.querySelector('.label')?.textContent?.startsWith('mountify_mounts')) {
                    setTimeout(updateBtn, 100);
                }
            });
        }
    }, 500);
}

document.addEventListener('DOMContentLoaded', async () => {
    initLanguage();
    config = await loadConfig();
    initTabs();
    initModals();
    initSwitches();
    initReboot();
    initLang();
    render();
    initModulePicker();
    const v = await exec(`grep "^version=" ${MODDIR}/module.prop | cut -d= -f2`);
    if (v.errno === 0) {
        document.getElementById('version').textContent = v.stdout.trim();
        document.getElementById('about-version').textContent = v.stdout.trim();
    }
});
