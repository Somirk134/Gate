"use client";

import { motion } from "motion/react";

import { architectureNodes } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";

export function ArchitectureFlow() {
  return (
    <SectionShell
      id="architecture"
      eyebrow="Architecture"
      title="A clean path from local socket to public endpoint."
      description="Gate keeps the visual desktop shell, server control plane and TCP forwarding path separated so each layer can evolve without turning into a black box."
    >
      <div className="glass-panel relative overflow-hidden rounded-[20px] p-5 sm:p-6 lg:p-8">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_18%_20%,rgba(66,248,211,0.13),transparent_28%),radial-gradient(circle_at_88%_78%,rgba(251,191,36,0.11),transparent_28%)]" />
        <div className="relative grid gap-4 lg:grid-cols-3 lg:items-stretch">
          {architectureNodes.map((node, index) => {
            const Icon = node.icon;

            return (
              <motion.div
                key={node.title}
                className="relative rounded-[20px] border border-white/10 bg-white/[0.045] p-5 backdrop-blur-xl"
                initial={{ opacity: 0, y: 22 }}
                whileInView={{ opacity: 1, y: 0 }}
                whileHover={{ y: -6 }}
                transition={{ duration: 0.45, delay: index * 0.08 }}
                viewport={{ once: true }}
              >
                <div className="mb-5 flex items-center justify-between">
                  <div className="flex size-12 items-center justify-center rounded-[18px] bg-white/[0.07] text-primary ring-1 ring-white/10">
                    <Icon className="size-5" />
                  </div>
                  <span className="font-mono text-xs text-white/34">{node.eyebrow}</span>
                </div>
                <h3 className="text-xl font-semibold text-white">{node.title}</h3>
                <div className="mt-5 space-y-3">
                  {node.items.map((item) => (
                    <div key={item} className="flex items-center gap-3 rounded-2xl bg-white/[0.045] px-3 py-2.5 text-sm text-white/60">
                      <span className="size-1.5 rounded-full bg-primary" />
                      {item}
                    </div>
                  ))}
                </div>
              </motion.div>
            );
          })}
        </div>

        <div className="relative mt-8 hidden h-20 items-center lg:flex">
          <div className="absolute left-[15%] right-[15%] top-1/2 h-px -translate-y-1/2 bg-white/10" />
          <span className="packet top-1/2 -translate-y-1/2" style={{ "--packet-delay": "0s" } as React.CSSProperties} />
          <span className="packet top-1/2 -translate-y-1/2 bg-amber-200 shadow-[0_0_24px_rgba(251,191,36,0.72)]" style={{ "--packet-delay": "1.35s" } as React.CSSProperties} />
          <span className="packet top-1/2 -translate-y-1/2 bg-rose-200 shadow-[0_0_24px_rgba(244,114,182,0.72)]" style={{ "--packet-delay": "2.7s" } as React.CSSProperties} />
          <div className="mx-auto grid w-full max-w-3xl grid-cols-3 text-center font-mono text-xs text-white/32">
            <span>local bind</span>
            <span>authenticated tunnel</span>
            <span>public stream</span>
          </div>
        </div>
      </div>
    </SectionShell>
  );
}
