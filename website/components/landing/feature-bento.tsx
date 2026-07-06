"use client";

import { motion } from "motion/react";

import { features, accentClasses } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";
import { cn } from "@/lib/utils";

export function FeatureBento() {
  return (
    <SectionShell
      id="features"
      eyebrow="Feature Bento"
      title="A tunnel tool that feels closer to a native control room."
      description="Gate is designed around visual feedback, small sharp primitives and a forwarding core that stays understandable."
    >
      <div className="grid gap-4 md:grid-cols-3">
        {features.map((feature, index) => {
          const Icon = feature.icon;

          return (
            <motion.article
              key={feature.title}
              className={cn(
                "glass-panel group relative min-h-[260px] overflow-hidden rounded-[20px] p-6",
                feature.className,
              )}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              whileHover={{ y: -8, scale: 1.01 }}
              transition={{ duration: 0.45, delay: index * 0.05, ease: [0.22, 1, 0.36, 1] }}
              viewport={{ once: true }}
            >
              <div className={cn("absolute inset-x-0 top-0 h-40 bg-gradient-to-b opacity-80 blur-2xl", accentClasses[feature.accent])} />
              <div className="relative flex h-full flex-col justify-between">
                <div className="flex items-center justify-between">
                  <div className={cn("flex size-12 items-center justify-center rounded-[18px] bg-gradient-to-br ring-1 ring-white/10", accentClasses[feature.accent])}>
                    <Icon className="size-5" />
                  </div>
                  <span className="font-mono text-xs text-white/30">0{index + 1}</span>
                </div>

                <div className="mt-12">
                  <h3 className="text-xl font-semibold text-white">{feature.title}</h3>
                  <p className="mt-4 max-w-xl text-sm leading-7 text-white/56">{feature.description}</p>
                </div>
              </div>
            </motion.article>
          );
        })}
      </div>
    </SectionShell>
  );
}
