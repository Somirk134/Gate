"use client";

import { motion } from "motion/react";

import { Badge } from "@/components/ui/badge";
import { roadmap } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";

export function Roadmap() {
  return (
    <SectionShell
      id="roadmap"
      eyebrow="Roadmap"
      title="Early, honest and built in public."
      description="The project is still pre-alpha. The roadmap keeps the homepage ambitious without pretending the production release already exists."
    >
      <div className="relative">
        <div className="absolute bottom-0 left-4 top-0 hidden w-px bg-white/10 md:block" />
        <div className="grid gap-4">
          {roadmap.map((item, index) => (
            <motion.article
              key={`${item.version}-${item.title}`}
              className="glass-panel relative rounded-[20px] p-5 md:ml-12"
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              whileHover={{ y: -5 }}
              transition={{ duration: 0.45, delay: index * 0.05 }}
              viewport={{ once: true }}
            >
              <span className="absolute -left-[3.75rem] top-7 hidden size-8 rounded-full border border-primary/30 bg-[#07100e] shadow-[0_0_28px_rgba(66,248,211,0.22)] md:block" />
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-3">
                  <Badge variant={item.status === "Done" ? "default" : item.status === "In progress" ? "amber" : "secondary"}>
                    {item.version}
                  </Badge>
                  <span className="font-mono text-xs text-white/38">{item.date}</span>
                </div>
                <span className="text-sm text-white/46">{item.status}</span>
              </div>
              <h3 className="mt-5 text-xl font-semibold text-white">{item.title}</h3>
              <p className="mt-3 text-sm leading-7 text-white/56">{item.description}</p>
            </motion.article>
          ))}
        </div>
      </div>
    </SectionShell>
  );
}
