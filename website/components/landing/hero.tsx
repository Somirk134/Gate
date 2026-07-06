"use client";

import { ArrowRight, Circle, Github, Play, ShieldCheck, Zap } from "lucide-react";
import { motion } from "motion/react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { product } from "@/components/landing/data";
import { cn } from "@/lib/utils";

const tunnelNodes = [
  { label: "Local app", meta: "127.0.0.1:5173", tone: "cyan" },
  { label: "Gate server", meta: "tokio relay", tone: "amber" },
  { label: "Public edge", meta: "gate.dev:443", tone: "rose" },
] as const;

export function Hero() {
  return (
    <section id="top" className="relative overflow-hidden px-5 pb-14 pt-32 sm:px-6 lg:px-8 lg:pb-20 lg:pt-36">
      <div className="absolute left-1/2 top-24 -z-10 h-[34rem] w-[34rem] -translate-x-1/2 rounded-full bg-primary/10 blur-3xl" />
      <div className="absolute right-[8%] top-40 -z-10 h-64 w-64 rounded-full bg-amber-300/10 blur-3xl" />

      <div className="mx-auto max-w-7xl">
        <motion.div
          className="mx-auto max-w-4xl text-center"
          initial={{ opacity: 0, y: 26 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.72, ease: [0.22, 1, 0.36, 1] }}
        >
          <Badge className="mb-6">
            <span className="size-1.5 rounded-full bg-primary shadow-[0_0_18px_rgba(66,248,211,0.9)]" />
            Pre-alpha · Rust + Tauri · Self-hosted
          </Badge>
          <h1 className="text-gradient text-balance text-6xl font-semibold tracking-normal sm:text-7xl lg:text-8xl">
            {product.name}
          </h1>
          <p className="mx-auto mt-6 max-w-3xl text-pretty text-lg leading-8 text-white/64 sm:text-xl">
            {product.description}
          </p>
          <div className="mt-9 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Button size="lg" asChild>
              <a href="#demo">
                <Play />
                Explore the demo
              </a>
            </Button>
            <Button size="lg" variant="secondary" asChild>
              <a href={product.repoUrl} target="_blank" rel="noreferrer">
                <Github />
                View on GitHub
                <ArrowRight />
              </a>
            </Button>
          </div>
        </motion.div>

        <motion.div
          className="mx-auto mt-14 max-w-6xl"
          initial={{ opacity: 0, y: 36, scale: 0.98 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ duration: 0.85, delay: 0.18, ease: [0.22, 1, 0.36, 1] }}
        >
          <DashboardHero />
        </motion.div>
      </div>
    </section>
  );
}

function DashboardHero() {
  return (
    <motion.div
      className="glass-panel relative overflow-hidden rounded-[20px] p-3 sm:p-4"
      whileHover={{ y: -6 }}
      transition={{ type: "spring", stiffness: 180, damping: 22 }}
    >
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(66,248,211,0.18),transparent_42%),radial-gradient(circle_at_100%_80%,rgba(251,191,36,0.10),transparent_26%)]" />
      <div className="relative overflow-hidden rounded-[16px] border border-white/10 bg-[#070a0f]/82">
        <div className="flex h-12 items-center justify-between border-b border-white/10 px-4">
          <div className="flex items-center gap-2">
            <span className="size-3 rounded-full bg-rose-400/85" />
            <span className="size-3 rounded-full bg-amber-300/85" />
            <span className="size-3 rounded-full bg-emerald-300/85" />
          </div>
          <div className="hidden items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-white/56 sm:flex">
            <Circle className="size-2 fill-primary text-primary" />
            gate-server · online
          </div>
          <div className="font-mono text-xs text-white/34">v0.2 preview</div>
        </div>

        <div className="grid gap-0 lg:grid-cols-[220px_minmax(0,1fr)]">
          <aside className="hidden border-r border-white/10 bg-white/[0.025] p-4 lg:block">
            <div className="mb-6 text-xs font-medium uppercase tracking-[0.18em] text-white/32">Workspace</div>
            {["Dashboard", "Tunnels", "Servers", "Logs"].map((item, index) => (
              <div
                key={item}
                className={cn(
                  "mb-2 rounded-2xl px-3 py-2.5 text-sm transition",
                  index === 1 ? "bg-white/[0.09] text-white" : "text-white/48",
                )}
              >
                {item}
              </div>
            ))}
            <div className="mt-8 rounded-[20px] border border-primary/20 bg-primary/10 p-4">
              <div className="mb-2 flex items-center gap-2 text-sm font-medium text-primary">
                <ShieldCheck className="size-4" />
                Guarded
              </div>
              <p className="text-xs leading-5 text-white/50">TLS rotation and token boundaries ready for the security milestone.</p>
            </div>
          </aside>

          <div className="p-4 sm:p-5 lg:p-6">
            <div className="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm text-white/44">Active tunnel</p>
                <h2 className="mt-1 text-xl font-semibold text-white sm:text-2xl">Local Web Preview</h2>
              </div>
              <div className="flex items-center gap-2 rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-2 text-sm text-emerald-100">
                <Zap className="size-4" />
                42.8 MB forwarded
              </div>
            </div>

            <div className="relative rounded-[20px] border border-white/10 bg-white/[0.035] p-4 sm:p-6">
              <div className="absolute inset-0 rounded-[20px] bg-[radial-gradient(circle_at_20%_20%,rgba(66,248,211,0.14),transparent_28%),radial-gradient(circle_at_80%_65%,rgba(244,114,182,0.12),transparent_28%)]" />
              <div className="relative grid items-center gap-7 lg:grid-cols-[1fr_1.1fr_1fr]">
                {tunnelNodes.map((node, index) => (
                  <TunnelNode key={node.label} {...node} index={index} />
                ))}
              </div>
              <div className="relative mt-8 h-2 rounded-full bg-white/[0.06]">
                <div className="flow-line h-full rounded-full bg-primary/20" />
                <span className="packet top-1/2 -translate-y-1/2" style={{ "--packet-delay": "0s" } as React.CSSProperties} />
                <span className="packet top-1/2 -translate-y-1/2 bg-amber-200 shadow-[0_0_24px_rgba(251,191,36,0.72)]" style={{ "--packet-delay": "1.35s" } as React.CSSProperties} />
              </div>
            </div>

            <div className="mt-4 grid gap-3 sm:grid-cols-3">
              {[
                ["Latency", "18 ms"],
                ["Sessions", "12 active"],
                ["Health", "99.98%"],
              ].map(([label, value]) => (
                <div key={label} className="glass-subtle rounded-[20px] px-4 py-3">
                  <div className="text-xs text-white/42">{label}</div>
                  <div className="mt-1 text-lg font-semibold text-white">{value}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function TunnelNode({
  label,
  meta,
  tone,
  index,
}: {
  label: string;
  meta: string;
  tone: "cyan" | "amber" | "rose";
  index: number;
}) {
  const toneClass = {
    cyan: "bg-cyan-300/16 text-cyan-100 ring-cyan-300/24",
    amber: "bg-amber-300/16 text-amber-100 ring-amber-300/24",
    rose: "bg-rose-300/16 text-rose-100 ring-rose-300/24",
  }[tone];

  return (
    <div className="relative text-center">
      <div className="orbital-ring absolute left-1/2 top-1/2 size-32 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-60" />
      <motion.div
        className={cn("relative mx-auto flex size-24 items-center justify-center rounded-[20px] ring-1", toneClass)}
        animate={{ y: [0, -7, 0] }}
        transition={{ duration: 4, delay: index * 0.45, repeat: Infinity, ease: "easeInOut" }}
      >
        <span className="font-mono text-xl font-semibold">0{index + 1}</span>
      </motion.div>
      <div className="mt-4 text-sm font-medium text-white">{label}</div>
      <div className="mt-1 font-mono text-xs text-white/42">{meta}</div>
    </div>
  );
}
