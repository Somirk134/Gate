"use client";

import { ArrowUpRight, Check, Terminal } from "lucide-react";
import { motion } from "motion/react";

import { Button } from "@/components/ui/button";
import { openSourceItems, product } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";

export function OpenSource() {
  return (
    <SectionShell
      id="open-source"
      eyebrow="Open Source"
      title="Transparent internals for people who like to own the stack."
      description="Gate is shaped as a Rust workspace with modular crates, desktop UI boundaries and roadmap docs that make contribution paths visible."
    >
      <div className="grid gap-4 lg:grid-cols-[0.95fr_1.05fr]">
        <motion.div
          className="glass-panel rounded-[20px] p-6"
          whileHover={{ y: -6 }}
          transition={{ type: "spring", stiffness: 160, damping: 22 }}
        >
          <div className="grid gap-3 sm:grid-cols-2">
            {openSourceItems.map((item) => {
              const Icon = item.icon;

              return (
                <div key={item.label} className="flex items-center gap-3 rounded-[18px] border border-white/10 bg-white/[0.045] px-4 py-4">
                  <span className="flex size-10 items-center justify-center rounded-[15px] bg-primary/10 text-primary">
                    <Icon className="size-5" />
                  </span>
                  <span className="text-sm font-medium text-white/78">{item.label}</span>
                </div>
              );
            })}
          </div>
          <div className="mt-6 rounded-[20px] border border-white/10 bg-[#05070b]/80 p-5">
            {["crates/engine", "crates/protocol", "crates/domain", "client/src-tauri", "docs/10-roadmap"].map((path) => (
              <div key={path} className="flex items-center gap-3 border-b border-white/8 py-3 text-sm text-white/58 last:border-b-0">
                <Check className="size-4 text-primary" />
                <span className="font-mono">{path}</span>
              </div>
            ))}
          </div>
        </motion.div>

        <motion.div
          className="glass-panel flex min-h-[420px] flex-col justify-between overflow-hidden rounded-[20px]"
          whileHover={{ y: -6 }}
          transition={{ type: "spring", stiffness: 160, damping: 22 }}
        >
          <div className="border-b border-white/10 bg-white/[0.035] px-5 py-4">
            <div className="flex items-center gap-2 text-sm font-medium text-white">
              <Terminal className="size-4 text-primary" />
              Quick start
            </div>
          </div>
          <pre className="overflow-x-auto p-6 text-sm leading-8 text-white/64">
            <code>{`git clone ${product.repoUrl}
cd gate
cargo test --workspace
cd client
npm install
npm run tauri dev`}</code>
          </pre>
          <div className="border-t border-white/10 p-5">
            <Button asChild>
              <a href={product.repoUrl} target="_blank" rel="noreferrer">
                Star the project
                <ArrowUpRight />
              </a>
            </Button>
          </div>
        </motion.div>
      </div>
    </SectionShell>
  );
}
