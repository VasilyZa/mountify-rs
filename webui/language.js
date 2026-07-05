const LANGUAGES = {
    'en': { name: 'English', native: 'English' },
    'zh-CN': { name: 'Chinese (Simplified)', native: '简体中文' },
};

const TRANSLATIONS = {
    'en': {
        'tab.general': 'General',
        'tab.ext4': 'Ext4',
        'tab.advanced': 'Advanced',
        'tab.about': 'About',
        'stat.mode': 'MODE',
        'stat.fstype': 'FSTYPE',
        'stat.modules': 'MODULES',
        'stat.na': 'N/A',
        'stat.active': 'ACTIVE',
        'stat.off': 'OFF',
        'stat.manual': 'MANUAL',
        'stat.auto': 'AUTO',
        'stat.tmpfs': 'TMPFS',
        'stat.ext4': 'EXT4',
        'stat.apex': 'APEX',
        'desc_advanced': 'Show advanced options',
        'desc_update': 'Check for updates',
        'reboot.title': 'Reboot',
        'reboot.msg': 'Reboot now?',
        'reboot.confirm': 'Reboot',
        'reboot.cancel': 'Cancel',
        'modules.title': 'Select Modules',
        'modules.save': 'Save',
        'modules.select': 'SELECT',
        'toast.config_not_found': 'Config not found',
        'toast.save_failed': 'Save failed',
        'toast.lang_soon': 'More languages coming soon',
        'lang.title': 'Select Language',
        'placeholder': 'Language',

        'desc.mountify_mounts': '0 = disable\n1 = manual (use modules.txt)\n2 = auto (mount all with system folder)',
        'desc.FAKE_MOUNT_NAME': 'Mount folder name under /mnt/vendor.\nDefault: mountify',
        'desc.test_decoy_mount': 'Test for decoy mount detection (tmpfs mode only).\nUses blank system folders as decoy targets.',
        'desc.mountify_stop_start': 'Restart Android at service stage.\nWorkaround for racey modules (GPU, bootanim).',
        'desc.FS_TYPE_ALIAS': 'Custom overlayfs driver alias.\nOnly useful if kernel registered another alias.',
        'desc.MOUNT_DEVICE_NAME': 'Device name for zygisk unmount.\nAllows NeoZygisk/NoHello/etc to hide mounts.',
        'desc.mountify_custom_umount': '0 = disable\n1 = susfs4ksu try_umount\n2 = ksud kernel umount (KSU 22106+)',
        'desc.mountify_expert_mode': 'Disables safety checks.\nWARNING: you can bootloop!',
        'desc.use_ext4_sparse': 'Force ext4 sparse mode even if tmpfs xattr works.',
        'desc.spoof_sparse': 'Spoof sparse mount as an Android apex service mount.\nDisables LKM nuke.',
        'desc.FAKE_APEX_NAME': 'Apex name used when spoof_sparse=1.',
        'desc.sparse_size': 'Sparse image size in MB.\nDefault: 2048',
        'desc.enable_lkm_nuke': 'Load LKM to unregister ext4 sysfs.\nHides ext4 nodes from /proc/fs.',
        'desc.lkm_filename': 'Select LKM filename matching your kernel.',
        'about.desc': 'A Rust rewrite of mountify. Globally mounted modules via OverlayFS.',
        'about.author': 'Author',
        'about.thanks': 'Special Thanks',
        'about.thanks.orig': 'backslashxx — original mountify author',
        'about.thanks.tester': 'dyzihnieg — first beta tester',
    },
    'zh-CN': {
        'tab.general': '常规',
        'tab.ext4': 'Ext4',
        'tab.advanced': '高级',
        'tab.about': '关于',
        'stat.mode': '模式',
        'stat.fstype': '文件系统',
        'stat.modules': '模块',
        'stat.na': '无',
        'stat.active': '运行中',
        'stat.off': '关闭',
        'stat.manual': '手动',
        'stat.auto': '自动',
        'stat.tmpfs': 'TMPFS',
        'stat.ext4': 'EXT4',
        'stat.apex': 'APEX',
        'desc_advanced': '显示高级选项',
        'desc_update': '检查更新',
        'reboot.title': '重启',
        'reboot.msg': '立即重启设备？',
        'reboot.confirm': '重启',
        'reboot.cancel': '取消',
        'modules.title': '选择模块',
        'modules.save': '保存',
        'modules.select': '选择',
        'toast.config_not_found': '未找到配置文件',
        'toast.save_failed': '保存失败',
        'toast.lang_soon': '更多语言即将支持',
        'lang.title': '选择语言',
        'placeholder': '语言',

        'desc.mountify_mounts': '0 = 禁用\n1 = 手动模式（使用 modules.txt）\n2 = 自动模式（挂载所有含 system 文件夹的模块）',
        'desc.FAKE_MOUNT_NAME': '挂载文件夹名称（位于 /mnt/vendor 下）。\n默认：mountify',
        'desc.test_decoy_mount': '测试诱饵挂载检测（仅 tmpfs 模式）。\n使用空白系统文件夹作为诱饵目标。',
        'desc.mountify_stop_start': '在 service 阶段重启 Android。\n解决竞争性模块问题（GPU、开机动画等）。',
        'desc.FS_TYPE_ALIAS': '自定义 overlayfs 驱动别名。\n仅当内核注册了其他别名时有用。',
        'desc.MOUNT_DEVICE_NAME': 'zygisk 卸载的设备名称。\n允许 NeoZygisk/NoHello 等隐藏挂载。',
        'desc.mountify_custom_umount': '0 = 禁用\n1 = susfs4ksu try_umount\n2 = ksud 内核卸载（KSU 22106+）',
        'desc.mountify_expert_mode': '禁用安全检查。\n警告：可能导致启动循环！',
        'desc.use_ext4_sparse': '强制使用 ext4 sparse 模式，即使 tmpfs xattr 可用。',
        'desc.spoof_sparse': '将 sparse 挂载伪装为 Android apex 服务挂载。\n禁用 LKM nuke。',
        'desc.FAKE_APEX_NAME': 'spoof_sparse=1 时使用的 apex 名称。',
        'desc.sparse_size': 'sparse 镜像大小（MB）。\n默认：2048',
        'desc.enable_lkm_nuke': '加载 LKM 以注销 ext4 sysfs。\n从 /proc/fs 隐藏 ext4 节点。',
        'desc.lkm_filename': '选择与内核匹配的 LKM 文件名。',
        'about.desc': '用 Rust 重写的 mountify，基于 OverlayFS 全局挂载模块。',
        'about.author': '作者',
        'about.thanks': '特别鸣谢',
        'about.thanks.orig': 'backslashxx — 原始 mountify 作者',
        'about.thanks.tester': 'dyzihnieg — 第一位内测用户',
    },
};

let currentLang = 'en';

export function t(key) {
    const lang = TRANSLATIONS[currentLang];
    return lang?.[key] ?? TRANSLATIONS['en'][key] ?? key;
}

export function getLang() { return currentLang; }
export function setLang(code) {
    if (TRANSLATIONS[code]) {
        currentLang = code;
        localStorage.setItem('mountify_lang', code);
    }
}

export function getLanguages() {
    return Object.entries(LANGUAGES).map(([code, info]) => ({ code, ...info }));
}

export function initLanguage() {
    const saved = localStorage.getItem('mountify_lang');
    if (saved && TRANSLATIONS[saved]) currentLang = saved;
}
