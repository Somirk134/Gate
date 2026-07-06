/* ==================================================================
   Gate Design System — Icon Registry
   ------------------------------------------------------------------
   统一使用 Lucide Icons。本文件集中注册项目用到的图标名，
   页面/组件只通过字符串名称引用，禁止散落内联 SVG。

   用法：
     <GIcon name="dashboard" :size="18" />
   ------------------------------------------------------------------
   命名规范：与 Lucide 原生 kebab-case 一致，便于查阅。
   ================================================================== */

import {
  LayoutDashboard, FolderKanban, Server, ScrollText, Settings, Info,
  ChevronLeft, ChevronRight, ChevronDown, ChevronUp,
  Search, Plus, Minus, X, Check, CheckCircle2, AlertTriangle, AlertCircle,
  Info as InfoIcon, Bell, Menu, PanelLeftClose, PanelLeftOpen, PanelRightClose, PanelRightOpen,
  RefreshCw, Power, Play, Square, Pause, Circle, CircleDot, Loader2,
  ArrowLeft, ArrowRight, ArrowUp, ArrowDown, ArrowUpDown, ArrowRightLeft,
  Copy, Clipboard, ClipboardCheck, Trash2, Edit3, Pencil, Save,
  Eye, EyeOff, Lock, Unlock, Key, Shield, ShieldCheck,
  Globe, Wifi, WifiOff, Activity, Zap, Link, Link2, Unlink, Network,
  Cloud, CloudUpload, CloudDownload, Upload, Download,
  Terminal, Code2, FileText, FileCode, Cpu, HardDrive, Database,
  Clock, Calendar, Timer, Gauge, Signal, SignalHigh, SignalLow,
  Filter, SlidersHorizontal, MoreHorizontal, MoreVertical, EllipsisVertical,
  User, Users, UserCircle, LogOut, LogIn, UserPlus,
  Home, Star, StarOff, Bookmark, Flag, Tag, Pin, PinOff,
  ExternalLink, Maximize2, Minimize2, Expand, Shrink,
  Sun, Moon, Monitor, Palette, Languages, Type,
  CircleHelp, HelpCircle, MessageSquare, Send, Mail,
  List, Grid2x2, LayoutGrid, Columns2, Rows2, Table2,
  TrendingUp, TrendingDown, ChartBar, ChartLine, ChartPie,
  Inbox, Package, Boxes, Box, Layers, GitBranch, GitCommitHorizontal,
  Plug, PlugZap, Router, Cable, Antenna, Radio,
  Rocket, Sparkles, Github, History, MemoryStick, CircuitBoard, CreditCard,
  type LucideIcon,
} from "lucide-vue-next"

/** 图标名 → 组件 映射（单一注册点） */
export const iconRegistry: Record<string, LucideIcon> = {
  /* ── 导航 ── */
  dashboard: LayoutDashboard,
  projects: FolderKanban,
  servers: Server,
  logs: ScrollText,
  settings: Settings,
  about: Info,
  home: Home,

  /* ── 方向 ── */
  "chevron-left": ChevronLeft,
  "chevron-right": ChevronRight,
  "chevron-down": ChevronDown,
  "chevron-up": ChevronUp,
  "arrow-left": ArrowLeft,
  "arrow-right": ArrowRight,
  "arrow-up": ArrowUp,
  "arrow-down": ArrowDown,
  "arrow-up-down": ArrowUpDown,
  "arrow-right-left": ArrowRightLeft,

  /* ── 操作 ── */
  search: Search,
  plus: Plus,
  minus: Minus,
  close: X,
  check: Check,
  "check-circle": CheckCircle2,
  copy: Copy,
  clipboard: Clipboard,
  "clipboard-check": ClipboardCheck,
  trash: Trash2,
  edit: Edit3,
  pencil: Pencil,
  save: Save,
  filter: Filter,
  sliders: SlidersHorizontal,
  "more-horizontal": MoreHorizontal,
  "more-vertical": MoreVertical,
  "ellipsis-vertical": EllipsisVertical,
  refresh: RefreshCw,
  expand: Expand,
  shrink: Shrink,
  maximize: Maximize2,
  minimize: Minimize2,

  /* ── 状态 ── */
  "alert-triangle": AlertTriangle,
  "alert-circle": AlertCircle,
  "info-circle": InfoIcon,
  bell: Bell,
  power: Power,
  play: Play,
  stop: Square,
  pause: Pause,
  circle: Circle,
  "circle-dot": CircleDot,
  loader: Loader2,
  spinner: Loader2,

  /* ── 面板控制 ── */
  menu: Menu,
  "panel-left-close": PanelLeftClose,
  "panel-left-open": PanelLeftOpen,
  "panel-right-close": PanelRightClose,
  "panel-right-open": PanelRightOpen,

  /* ── 连接 / 网络 ── */
  globe: Globe,
  wifi: Wifi,
  "wifi-off": WifiOff,
  activity: Activity,
  zap: Zap,
  link: Link,
  "link-2": Link2,
  unlink: Unlink,
  network: Network,
  plug: Plug,
  "plug-zap": PlugZap,
  router: Router,
  cable: Cable,
  antenna: Antenna,
  radio: Radio,
  signal: Signal,
  "signal-high": SignalHigh,
  "signal-low": SignalLow,

  /* ── 云 / 传输 ── */
  cloud: Cloud,
  "cloud-upload": CloudUpload,
  "cloud-download": CloudDownload,
  upload: Upload,
  download: Download,

  /* ── 开发 ── */
  terminal: Terminal,
  code: Code2,
  "file-text": FileText,
  "file-code": FileCode,
  cpu: Cpu,
  "hard-drive": HardDrive,
  database: Database,

  /* ── 安全 ── */
  lock: Lock,
  unlock: Unlock,
  key: Key,
  shield: Shield,
  "shield-check": ShieldCheck,
  eye: Eye,
  "eye-off": EyeOff,

  /* ── 时间 / 指标 ── */
  clock: Clock,
  calendar: Calendar,
  timer: Timer,
  gauge: Gauge,

  /* ── 用户 ── */
  user: User,
  users: Users,
  "user-circle": UserCircle,
  "log-in": LogIn,
  "log-out": LogOut,
  "user-plus": UserPlus,

  /* ── 标记 ── */
  star: Star,
  "star-off": StarOff,
  bookmark: Bookmark,
  flag: Flag,
  tag: Tag,
  pin: Pin,
  "pin-off": PinOff,

  /* ── 外链 ── */
  "external-link": ExternalLink,

  /* ── 主题 / 语言 ── */
  sun: Sun,
  moon: Moon,
  monitor: Monitor,
  palette: Palette,
  languages: Languages,
  type: Type,

  /* ── 帮助 ── */
  help: HelpCircle,
  "circle-help": CircleHelp,
  message: MessageSquare,
  send: Send,
  mail: Mail,

  /* ── 视图 ── */
  list: List,
  grid: Grid2x2,
  "layout-grid": LayoutGrid,
  columns: Columns2,
  rows: Rows2,
  table: Table2,

  /* ── 图表 ── */
  "trending-up": TrendingUp,
  "trending-down": TrendingDown,
  "chart-bar": ChartBar,
  "chart-line": ChartLine,
  "chart-pie": ChartPie,

  /* ── 容器/包 ── */
  inbox: Inbox,
  package: Package,
  box: Box,
  boxes: Boxes,
  layers: Layers,
  "git-branch": GitBranch,
  "git-commit": GitCommitHorizontal,

  /* ── Dashboard 扩展 ── */
  rocket: Rocket,
  sparkles: Sparkles,
  github: Github,
  history: History,
  "memory-stick": MemoryStick,
  "circuit-board": CircuitBoard,
  "credit-card": CreditCard,
}

/** 根据名称取图标组件，找不到时回退到 info */
export function resolveIcon(name: string): LucideIcon {
  return iconRegistry[name] ?? Info
}

export type IconName = keyof typeof iconRegistry
