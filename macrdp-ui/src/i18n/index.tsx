import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type Lang = "en" | "zh";

const en: Record<string, string> = {
  // ── Navigation ───────────────────────────────────────────
  "nav.group.service": "Service",
  "nav.group.config": "Configuration",
  "nav.group.diagnostics": "Diagnostics",
  "nav.dashboard": "Dashboard",
  "nav.settings": "Settings",
  "nav.permissions": "Permissions",
  "nav.logs": "Logs",
  "nav.statistics": "Statistics",
  "nav.about": "About",

  // ── Theme ────────────────────────────────────────────────
  "theme.system": "System",
  "theme.light": "Light",
  "theme.dark": "Dark",

  // ── Server status ────────────────────────────────────────
  "status.running": "Running",
  "status.starting": "Starting…",
  "status.stopped": "Stopped",
  "status.error": "Error",
  "status.working": "Working…",
  "status.stop": "Stop",
  "status.start": "Start",

  // ── Common ───────────────────────────────────────────────
  "common.waiting": "Waiting for data…",
  "common.loading": "Loading…",

  // ── Dashboard ────────────────────────────────────────────
  "dashboard.throughput": "Network throughput (Mbps)",
  "dashboard.encodeLatency": "Encode latency (ms)",
  "dashboard.encodeShort": "Encode (ms)",
  "dashboard.connections": "Connections",
  "dashboard.activeConnections": "Active connections",
  "dashboard.noActiveConnections": "No active connections",

  // ── Statistics ───────────────────────────────────────────
  "stats.title": "Statistics",
  "stats.days7": "7 days",
  "stats.days30": "30 days",
  "stats.totalConnections": "Total connections",
  "stats.totalTraffic": "Total traffic",
  "stats.avgSession": "Avg. session",
  "stats.trafficTrend": "Traffic trend",
  "stats.noTrafficData": "No traffic data",
  "stats.noHistory": "No connection history",
  "stats.user": "User",
  "stats.time": "Time",
  "stats.duration": "Duration",
  "stats.traffic": "Traffic",
  "stats.page": "Page {n}",
  "stats.prev": "Previous",
  "stats.next": "Next",

  // ── Permissions ──────────────────────────────────────────
  "perm.title": "Permissions",
  "perm.screenRecording": "Screen Recording",
  "perm.accessibility": "Accessibility",
  "perm.microphone": "Microphone",
  "perm.allGranted": "All permissions granted",
  "perm.granted": "Granted",
  "perm.notGranted": "Not granted",
  "perm.openSettings": "Open System Settings",

  // ── Logs ─────────────────────────────────────────────────
  "logs.title": "Logs",
  "logs.all": "All",
  "logs.error": "Error",
  "logs.warn": "Warn",
  "logs.info": "Info",
  "logs.debug": "Debug",
  "logs.live": "Live",
  "logs.paused": "Paused",
  "logs.search": "Search logs…",
  "logs.noMatch": "No matching logs",
  "logs.none": "No logs yet",

  // ── About ────────────────────────────────────────────────
  "about.checking": "Checking…",
  "about.latest": "Up to date",
  "about.available": "New version {v}",
  "about.checkFailed": "Check failed",
  "about.check": "Check for updates",
  "about.version": "Version {v}",
  "about.download": "Download",

  // ── Settings ─────────────────────────────────────────────
  "settings.loading": "Loading configuration…",
  "settings.restartBanner": "Some changes take effect after restarting the server",
  "settings.cat.rdp": "RDP Server",
  "settings.cat.network": "Network",
  "settings.cat.security": "Security",
  "settings.cat.display": "Display",
  "settings.cat.audio": "Audio",
  "settings.cat.general": "General",
  "settings.encoder": "Encoder",
  "settings.encoder.software": "Software",
  "settings.encoder.hardware": "Hardware",
  "settings.bitrate": "Bitrate",
  "settings.frameRate": "Frame rate",
  "settings.chromaMode": "Chroma mode",
  "settings.port": "Port",
  "settings.bindAddress": "Bind address",
  "settings.maxConnections": "Max connections",
  "settings.idleTimeout": "Idle timeout (s)",
  "settings.username": "Username",
  "settings.password": "Password",
  "settings.resolution": "Resolution",
  "settings.resolutionHint": "Applies to new connections; Auto follows the display",
  "settings.showCursor": "Show cursor",
  "settings.showCursorHint": "Render the system cursor in the captured image",
  "settings.audioUnavailable": "Audio settings are not available yet",
  "settings.logLevel": "Log level",
  "settings.autostart": "Launch at login",
  "settings.autostartHint": "Start the server when you log in",
  "settings.theme": "Theme",
  "settings.language": "Language",
  "settings.restartRequired": "Restart required",

  // ── Tray popover ─────────────────────────────────────────
  "popover.quitConfirm": "Quit macrdp?",
  "popover.settings": "Settings",
  "popover.quit": "Quit",
  "popover.encode": "Encode",
  "popover.network": "Network",
  "popover.traffic": "Traffic",
  "popover.connections": "Connections ({n})",
  "popover.noConnections": "No connections",
  "popover.more": "+{n} more",
  "popover.mainWindow": "Main window",
};

const zh: Record<string, string> = {
  "nav.group.service": "服务",
  "nav.group.config": "配置",
  "nav.group.diagnostics": "诊断",
  "nav.dashboard": "控制台",
  "nav.settings": "设置",
  "nav.permissions": "权限",
  "nav.logs": "日志",
  "nav.statistics": "统计",
  "nav.about": "关于",

  "theme.system": "跟随系统",
  "theme.light": "浅色模式",
  "theme.dark": "深色模式",

  "status.running": "运行中",
  "status.starting": "启动中...",
  "status.stopped": "已停止",
  "status.error": "错误",
  "status.working": "处理中...",
  "status.stop": "停止",
  "status.start": "启动",

  "common.waiting": "等待数据...",
  "common.loading": "加载中...",

  "dashboard.throughput": "网络速度 (Mbps)",
  "dashboard.encodeLatency": "编码延迟 (ms)",
  "dashboard.encodeShort": "ms 编码",
  "dashboard.connections": "连接",
  "dashboard.activeConnections": "活跃连接",
  "dashboard.noActiveConnections": "暂无活跃连接",

  "stats.title": "统计",
  "stats.days7": "7天",
  "stats.days30": "30天",
  "stats.totalConnections": "总连接数",
  "stats.totalTraffic": "总流量",
  "stats.avgSession": "平均会话时长",
  "stats.trafficTrend": "流量趋势",
  "stats.noTrafficData": "暂无流量数据",
  "stats.noHistory": "暂无连接记录",
  "stats.user": "用户",
  "stats.time": "时间",
  "stats.duration": "时长",
  "stats.traffic": "流量",
  "stats.page": "第 {n} 页",
  "stats.prev": "上一页",
  "stats.next": "下一页",

  "perm.title": "权限",
  "perm.screenRecording": "屏幕录制",
  "perm.accessibility": "辅助功能",
  "perm.microphone": "麦克风",
  "perm.allGranted": "所有权限已就绪",
  "perm.granted": "已授权",
  "perm.notGranted": "未授权",
  "perm.openSettings": "前往系统设置",

  "logs.title": "日志",
  "logs.all": "全部",
  "logs.error": "Error",
  "logs.warn": "Warn",
  "logs.info": "Info",
  "logs.debug": "Debug",
  "logs.live": "实时",
  "logs.paused": "已暂停",
  "logs.search": "搜索日志...",
  "logs.noMatch": "未找到匹配的日志",
  "logs.none": "暂无日志记录",

  "about.checking": "检查中...",
  "about.latest": "已是最新",
  "about.available": "新版本 {v}",
  "about.checkFailed": "检查失败",
  "about.check": "检查更新",
  "about.version": "版本 {v}",
  "about.download": "前往下载",

  "settings.loading": "加载配置中...",
  "settings.restartBanner": "部分配置需重启服务后生效",
  "settings.cat.rdp": "RDP 服务",
  "settings.cat.network": "网络",
  "settings.cat.security": "安全",
  "settings.cat.display": "显示",
  "settings.cat.audio": "音频",
  "settings.cat.general": "通用",
  "settings.encoder": "编码器",
  "settings.encoder.software": "软件编码",
  "settings.encoder.hardware": "硬件加速",
  "settings.bitrate": "比特率",
  "settings.frameRate": "帧率",
  "settings.chromaMode": "色度模式",
  "settings.port": "端口",
  "settings.bindAddress": "绑定地址",
  "settings.maxConnections": "最大连接数",
  "settings.idleTimeout": "空闲超时 (秒)",
  "settings.username": "用户名",
  "settings.password": "密码",
  "settings.resolution": "分辨率",
  "settings.resolutionHint": "新连接生效，Auto 跟随屏幕",
  "settings.showCursor": "显示光标",
  "settings.showCursorHint": "在采集画面中渲染系统光标",
  "settings.audioUnavailable": "音频配置尚未开放",
  "settings.logLevel": "日志级别",
  "settings.autostart": "开机自启",
  "settings.autostartHint": "登录时自动启动服务",
  "settings.theme": "主题",
  "settings.language": "语言",
  "settings.restartRequired": "重启服务生效",

  "popover.quitConfirm": "确定退出 macrdp？",
  "popover.settings": "设置",
  "popover.quit": "退出",
  "popover.encode": "编码",
  "popover.network": "网络",
  "popover.traffic": "流量",
  "popover.connections": "连接 ({n})",
  "popover.noConnections": "暂无连接",
  "popover.more": "+{n} 更多",
  "popover.mainWindow": "主窗口",
};

const messages: Record<Lang, Record<string, string>> = { en, zh };

const STORAGE_KEY = "macrdp.lang";

export type Translate = (
  key: string,
  vars?: Record<string, string | number>,
) => string;

interface I18nValue {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: Translate;
  /** BCP-47 locale for date/number formatting. */
  locale: string;
}

const I18nContext = createContext<I18nValue>({
  lang: "en",
  setLang: () => {},
  t: (key) => en[key] ?? key,
  locale: "en-US",
});

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(() => {
    if (typeof window === "undefined") return "en";
    try {
      const saved = window.localStorage.getItem(STORAGE_KEY);
      if (saved === "zh" || saved === "en") return saved;
    } catch {
      /* localStorage unavailable */
    }
    return "en";
  });

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    try {
      window.localStorage.setItem(STORAGE_KEY, next);
    } catch {
      /* ignore */
    }
  }, []);

  const value = useMemo<I18nValue>(
    () => ({
      lang,
      setLang,
      locale: lang === "zh" ? "zh-CN" : "en-US",
      t: (key, vars) => {
        let text = messages[lang][key] ?? en[key] ?? key;
        if (vars) {
          for (const [name, replacement] of Object.entries(vars)) {
            text = text.split(`{${name}}`).join(String(replacement));
          }
        }
        return text;
      },
    }),
    [lang, setLang],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  return useContext(I18nContext);
}
