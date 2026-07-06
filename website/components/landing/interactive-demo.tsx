"use client";

import { useMemo, useState } from "react";
import { Activity, ArrowUpRight, Circle, Copy, Pause, Play, Plus, Radio, Settings2, ShieldCheck } from "lucide-react";
import { motion } from "motion/react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { demoPresets } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";
import { cn } from "@/lib/utils";

export function InteractiveDemo() {
  const [selected, setSelected] = useState(0);
  const [running, setRunning] = useState(true);
  const [secure, setSecure] = useState(true);
  const demo = demoPresets[selected];

  const timeline = useMemo(
    () => (running ? demo.logs : ["tunnel paused by operator", "visitor sockets drained", "local listener released"]),
    [demo.logs, running],
  );

  return (
    <SectionShell
      id="demo"
      eyebrow="Interactive Demo"
      title="Operate tunnels like a modern desktop app."
      description="The homepage demo mirrors the intended Gate workflow: select a tunnel, inspect the public endpoint, watch health and adjust safety controls."
    >
      <motion.div
        className="glass-panel overflow-hidden rounded-[20px]"
        whileHover={{ y: -4 }}
        transition={{ type: "spring", stiffness: 160, damping: 22 }}
      >
        <div className="flex flex-col border-b border-white/10 bg-white/[0.035] px-4 py-4 lg:flex-row lg:items-center lg:justify-between lg:px-5">
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-[16px] bg-primary/12 text-primary ring-1 ring-primary/20">
              <Radio className="size-5" />
            </div>
            <div>
              <h3 className="text-base font-semibold text-white">Tunnels</h3>
              <p className="text-xs text-white/44">Live local-to-public routes</p>
            </div>
          </div>
          <div className="mt-4 flex flex-wrap items-center gap-2 lg:mt-0">
            {demoPresets.map((preset, index) => (
              <button
                key={preset.name}
                type="button"
                onClick={() => setSelected(index)}
                className={cn(
                  "rounded-full px-4 py-2 text-sm transition hover:bg-white/[0.1] hover:text-white",
                  selected === index ? "bg-white/[0.12] text-white" : "text-white/48",
                )}
              >
                {preset.name}
              </button>
            ))}
          </div>
        </div>

        <div className="grid lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="p-4 sm:p-6">
            <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <Badge variant={running ? "default" : "secondary"}>
                    <Circle className={cn("size-2", running ? "fill-primary text-primary" : "fill-white/30 text-white/30")} />
                    {running ? demo.status : "Paused"}
                  </Badge>
                  <Badge variant={secure ? "amber" : "secondary"}>
                    <ShieldCheck className="size-3.5" />
                    {secure ? "TLS + token" : "Local only"}
                  </Badge>
                </div>
                <h3 className="mt-4 text-2xl font-semibold text-white">{demo.name}</h3>
                <p className="mt-2 font-mono text-sm text-white/46">
                  127.0.0.1:{demo.port} <span className="text-white/22">→</span> {demo.publicUrl}
                </p>
              </div>
              <div className="flex gap-2">
                <Button variant="secondary" size="icon" aria-label="Copy endpoint">
                  <Copy />
                </Button>
                <Button variant="secondary" size="icon" aria-label="Open tunnel settings">
                  <Settings2 />
                </Button>
                <Button size="icon" aria-label={running ? "Pause tunnel" : "Resume tunnel"} onClick={() => setRunning((value) => !value)}>
                  {running ? <Pause /> : <Play />}
                </Button>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-3">
              {[
                ["Protocol", demo.protocol],
                ["Traffic", demo.traffic],
                ["Latency", demo.latency],
              ].map(([label, value]) => (
                <div key={label} className="glass-subtle rounded-[20px] p-4">
                  <div className="text-xs text-white/38">{label}</div>
                  <div className="mt-2 text-lg font-semibold text-white">{value}</div>
                </div>
              ))}
            </div>

            <div className="mt-4 rounded-[20px] border border-white/10 bg-[#05070b]/80 p-4">
              <div className="mb-4 flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-medium text-white">
                  <Activity className="size-4 text-primary" />
                  Traffic stream
                </div>
                <div className="text-xs text-white/34">last 60 seconds</div>
              </div>
              <div className="flex h-36 items-end gap-2">
                {Array.from({ length: 26 }, (_, index) => (
                  <motion.span
                    key={index}
                    className="flex-1 rounded-t-full bg-gradient-to-t from-primary/20 via-primary/55 to-amber-200/80"
                    animate={{ height: running ? [`${28 + ((index * 17) % 54)}%`, `${36 + ((index * 23) % 60)}%`, `${28 + ((index * 17) % 54)}%`] : "18%" }}
                    transition={{ duration: 2.8, delay: index * 0.045, repeat: Infinity, ease: "easeInOut" }}
                  />
                ))}
              </div>
            </div>
          </div>

          <aside className="border-t border-white/10 bg-white/[0.025] p-4 sm:p-6 lg:border-l lg:border-t-0">
            <div className="mb-4 flex items-center justify-between">
              <h4 className="text-sm font-semibold text-white">Session policy</h4>
              <button
                type="button"
                onClick={() => setSecure((value) => !value)}
                className={cn(
                  "flex h-7 w-12 items-center rounded-full border p-1 transition",
                  secure ? "border-primary/40 bg-primary/22" : "border-white/12 bg-white/[0.06]",
                )}
                aria-label="Toggle secure mode"
              >
                <span className={cn("size-5 rounded-full bg-white transition", secure ? "translate-x-5" : "translate-x-0")} />
              </button>
            </div>
            <div className="space-y-3">
              {["Require token", "Rotate session keys", "Rate-limit visitors"].map((item, index) => (
                <div key={item} className="flex items-center justify-between rounded-[16px] border border-white/10 bg-white/[0.04] px-3 py-3">
                  <span className="text-sm text-white/68">{item}</span>
                  <span className={cn("size-2 rounded-full", secure || index === 2 ? "bg-primary shadow-[0_0_16px_rgba(66,248,211,0.75)]" : "bg-white/22")} />
                </div>
              ))}
            </div>

            <div className="mt-5 rounded-[20px] border border-white/10 bg-[#05070b]/80 p-4">
              <div className="mb-3 flex items-center justify-between">
                <span className="text-sm font-medium text-white">Event log</span>
                <Plus className="size-4 text-white/35" />
              </div>
              <div className="space-y-3">
                {timeline.map((line) => (
                  <div key={line} className="flex gap-3 font-mono text-xs text-white/48">
                    <span className="mt-1 size-1.5 rounded-full bg-primary" />
                    <span>{line}</span>
                  </div>
                ))}
              </div>
            </div>

            <Button variant="outline" className="mt-5 w-full" asChild>
              <a href="#architecture">
                See architecture
                <ArrowUpRight />
              </a>
            </Button>
          </aside>
        </div>
      </motion.div>
    </SectionShell>
  );
}
