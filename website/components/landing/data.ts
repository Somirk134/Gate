import type { LucideIcon } from "lucide-react";
import {
  Activity,
  Braces,
  Cable,
  Code2,
  Cpu,
  Database,
  Github,
  Globe2,
  HardDrive,
  LayoutDashboard,
  Monitor,
  Network,
  Package,
  RadioTower,
  Rocket,
  Server,
  ShieldCheck,
  Sparkles,
  Terminal,
  Waypoints,
  Zap,
} from "lucide-react";

export const product = {
  name: "Gate",
  tagline: "Visual TCP tunneling for self-hosted builders.",
  description:
    "Gate brings a modern desktop experience to TCP intranet traversal: Rust data-plane, Tauri shell, live tunnel observability and a self-hosted server you control.",
  repoUrl: "https://github.com/your-org/gate",
};

export const navItems = [
  { label: "Features", href: "#features" },
  { label: "Demo", href: "#demo" },
  { label: "Architecture", href: "#architecture" },
  { label: "Roadmap", href: "#roadmap" },
  { label: "FAQ", href: "#faq" },
] as const;

export type Feature = {
  title: string;
  description: string;
  icon: LucideIcon;
  accent: "cyan" | "amber" | "rose" | "violet" | "emerald";
  className?: string;
};

export const features: Feature[] = [
  {
    title: "Desktop-first tunnel control",
    description:
      "Create, inspect and pause tunnels from a focused Tauri desktop app instead of editing scattered command-line configs.",
    icon: LayoutDashboard,
    accent: "cyan",
    className: "md:col-span-2",
  },
  {
    title: "Rust data-plane",
    description:
      "Tokio streams, lifecycle boundaries and atomic traffic counters keep the forwarding core small and predictable.",
    icon: Cpu,
    accent: "amber",
  },
  {
    title: "Self-hosted by design",
    description:
      "Run the server on your own VPS, home lab or team machine. No mandatory cloud relay or hosted control plane.",
    icon: Server,
    accent: "emerald",
  },
  {
    title: "Live observability",
    description:
      "Sessions, health, traffic and logs are modelled as first-class views so developers can debug the path quickly.",
    icon: Activity,
    accent: "rose",
    className: "md:col-span-2",
  },
  {
    title: "Protocol roadmap",
    description:
      "TCP today, with planned UDP, HTTP, WebSocket and protocol-aware pipeline stages as the project matures.",
    icon: Cable,
    accent: "violet",
  },
];

export const demoPresets = [
  {
    name: "Local Web",
    port: "5173",
    publicUrl: "gate.dev:443",
    protocol: "TCP",
    traffic: "42.8 MB",
    latency: "18 ms",
    status: "Online",
    logs: [
      "bound local 127.0.0.1:5173",
      "handshake accepted by gate-server",
      "tls session rotated",
      "forwarded 128 frames in 12s",
    ],
  },
  {
    name: "SSH Lab",
    port: "22",
    publicUrl: "lab.gate.dev:2201",
    protocol: "TCP",
    traffic: "9.4 MB",
    latency: "24 ms",
    status: "Guarded",
    logs: [
      "policy: key-only access",
      "new visitor from 203.0.113.42",
      "idle session reclaimed",
      "audit event persisted",
    ],
  },
  {
    name: "Game Night",
    port: "25565",
    publicUrl: "play.gate.dev:25565",
    protocol: "TCP",
    traffic: "318 MB",
    latency: "31 ms",
    status: "Busy",
    logs: [
      "peer window expanded",
      "traffic spike normalized",
      "8 active visitor sessions",
      "health probe ok",
    ],
  },
] as const;

export const architectureNodes = [
  {
    title: "Gate Client",
    eyebrow: "Tauri desktop",
    icon: Monitor,
    items: ["Visual shell", "IPC commands", "Local tunnel binding"],
  },
  {
    title: "Gate Server",
    eyebrow: "Axum + Tokio",
    icon: RadioTower,
    items: ["Auth boundary", "Tunnel registry", "TCP forwarding"],
  },
  {
    title: "Public Visitor",
    eyebrow: "Any TCP client",
    icon: Globe2,
    items: ["Stable endpoint", "Session routing", "Response stream"],
  },
] as const;

export const metrics = [
  {
    value: 10000,
    suffix: "+",
    label: "planned concurrent TCP benchmark scenario",
    icon: Zap,
  },
  {
    value: 4,
    suffix: " B",
    label: "length-prefixed protocol frame header",
    icon: Braces,
  },
  {
    value: 0,
    suffix: "",
    label: "mandatory hosted cloud services",
    icon: ShieldCheck,
  },
  {
    value: 6,
    suffix: "",
    label: "clean runtime domains in the forwarding path",
    icon: Network,
  },
] as const;

export const platforms = [
  {
    name: "macOS",
    description: "Polished tray workflow and native window shell through Tauri 2.",
    icon: Monitor,
  },
  {
    name: "Windows",
    description: "Developer-friendly desktop app for local services, labs and teams.",
    icon: HardDrive,
  },
  {
    name: "Linux",
    description: "Lightweight GUI and server binaries for self-hosted environments.",
    icon: Terminal,
  },
  {
    name: "VPS / Homelab",
    description: "Deploy the server wherever your routing boundary should live.",
    icon: Database,
  },
] as const;

export const openSourceItems = [
  { label: "MIT licensed", icon: Github },
  { label: "Rust workspace", icon: Code2 },
  { label: "Open roadmap", icon: Rocket },
  { label: "Modular crates", icon: Package },
] as const;

export const roadmap = [
  {
    version: "0.1",
    date: "Q3 2026",
    title: "Project foundation",
    status: "Done",
    description: "Workspace scaffolding, architecture direction and core boundaries.",
  },
  {
    version: "0.2",
    date: "Q3 2026",
    title: "Core protocol and server tunneling",
    status: "In progress",
    description: "Protocol framing, TCP forwarding path, lifecycle and session registry.",
  },
  {
    version: "0.3",
    date: "Q4 2026",
    title: "Client GUI and IPC",
    status: "Planned",
    description: "Tauri commands, dashboard, tunnels, connections and project views.",
  },
  {
    version: "0.4",
    date: "Q4 2026",
    title: "Authentication and security",
    status: "Planned",
    description: "JWT, TLS, token management, rate limiting and hardened defaults.",
  },
  {
    version: "1.0",
    date: "Q2 2027",
    title: "Production release",
    status: "Target",
    description: "Packaging, deployment docs, benchmarks and stable release process.",
  },
] as const;

export const faqs = [
  {
    question: "Is Gate a hosted tunneling service?",
    answer:
      "No. Gate is designed around self-hosting. You run the server on infrastructure you control, then connect the desktop client to expose local TCP services.",
  },
  {
    question: "How is Gate different from CLI-only tunnel tools?",
    answer:
      "The core difference is the visual workflow. Gate keeps tunnel creation, state, traffic, logs and health visible in one desktop experience while preserving a small Rust forwarding core.",
  },
  {
    question: "Which protocols are supported?",
    answer:
      "The current implementation centers on TCP. UDP, HTTP, WebSocket and protocol-aware pipeline stages are part of the documented future path.",
  },
  {
    question: "Can teams use it?",
    answer:
      "Yes, the product model includes projects, shared servers, tunnel organization and observability features that make it suitable for small teams and open-source communities.",
  },
  {
    question: "Is the project production ready?",
    answer:
      "Not yet. The repository is in early development, so the website presents the roadmap clearly and treats performance numbers as engineering targets or benchmark lanes.",
  },
] as const;

export const accentClasses: Record<Feature["accent"], string> = {
  cyan: "from-cyan-300/24 to-cyan-300/5 text-cyan-100",
  amber: "from-amber-300/24 to-amber-300/5 text-amber-100",
  rose: "from-rose-300/24 to-rose-300/5 text-rose-100",
  violet: "from-violet-300/24 to-violet-300/5 text-violet-100",
  emerald: "from-emerald-300/24 to-emerald-300/5 text-emerald-100",
};

export const sectionBadges = {
  features: Sparkles,
  architecture: Waypoints,
};
