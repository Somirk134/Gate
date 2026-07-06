"use client";

import { Download } from "lucide-react";
import { motion } from "motion/react";

import { Button } from "@/components/ui/button";
import { platforms } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";

export function CrossPlatform() {
  return (
    <SectionShell
      id="platforms"
      eyebrow="Cross Platform"
      title="A native-feeling client, wherever developers work."
      description="Gate pairs a Tauri desktop client with self-hosted server deployment targets so personal labs and small teams can share the same workflow."
    >
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {platforms.map((platform, index) => {
          const Icon = platform.icon;

          return (
            <motion.div
              key={platform.name}
              className="glass-panel rounded-[20px] p-5"
              initial={{ opacity: 0, y: 22 }}
              whileInView={{ opacity: 1, y: 0 }}
              whileHover={{ y: -7 }}
              transition={{ duration: 0.45, delay: index * 0.06 }}
              viewport={{ once: true }}
            >
              <div className="mb-8 flex size-12 items-center justify-center rounded-[18px] bg-white/[0.07] text-primary ring-1 ring-white/10">
                <Icon className="size-5" />
              </div>
              <h3 className="text-lg font-semibold text-white">{platform.name}</h3>
              <p className="mt-3 min-h-20 text-sm leading-6 text-white/54">{platform.description}</p>
              <Button className="mt-6 w-full" variant="secondary" size="sm" asChild>
                <a href="#cta">
                  <Download />
                  Builds soon
                </a>
              </Button>
            </motion.div>
          );
        })}
      </div>
    </SectionShell>
  );
}
