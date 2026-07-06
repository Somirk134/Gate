"use client";

import Image from "next/image";
import { ArrowRight, Github, Sparkles } from "lucide-react";
import { motion } from "motion/react";

import { Button } from "@/components/ui/button";
import { navItems, product } from "@/components/landing/data";

export function CTAFooter() {
  return (
    <>
      <section id="cta" className="mx-auto max-w-7xl px-5 py-20 sm:px-6 lg:px-8 lg:py-28">
        <motion.div
          className="glass-panel relative overflow-hidden rounded-[20px] px-6 py-14 text-center sm:px-10 lg:px-20 lg:py-20"
          initial={{ opacity: 0, y: 28 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.65, ease: [0.22, 1, 0.36, 1] }}
          viewport={{ once: true }}
        >
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(66,248,211,0.18),transparent_38%),radial-gradient(circle_at_70%_100%,rgba(251,191,36,0.12),transparent_34%)]" />
          <div className="relative mx-auto max-w-3xl">
            <div className="mx-auto mb-6 flex size-14 items-center justify-center rounded-[20px] bg-primary/12 text-primary ring-1 ring-primary/25">
              <Sparkles className="size-6" />
            </div>
            <h2 className="text-balance text-4xl font-semibold tracking-normal text-white sm:text-5xl">
              Build the tunnel experience developers actually want to use.
            </h2>
            <p className="mx-auto mt-5 max-w-2xl text-lg leading-8 text-white/60">
              Clone Gate, follow the roadmap, and help shape a visual, self-hosted TCP tunneling stack for makers and small teams.
            </p>
            <div className="mt-9 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <Button size="lg" asChild>
                <a href={product.repoUrl} target="_blank" rel="noreferrer">
                  <Github />
                  Open GitHub
                </a>
              </Button>
              <Button size="lg" variant="secondary" asChild>
                <a href="#top">
                  Back to top
                  <ArrowRight />
                </a>
              </Button>
            </div>
          </div>
        </motion.div>
      </section>

      <footer className="border-t border-white/10 px-5 py-10 sm:px-6 lg:px-8">
        <div className="mx-auto flex max-w-7xl flex-col gap-8 md:flex-row md:items-center md:justify-between">
          <a href="#top" className="flex items-center gap-3">
            <Image src="/gate-mark.svg" alt="" width={34} height={34} />
            <div>
              <div className="text-sm font-semibold text-white">{product.name}</div>
              <div className="text-xs text-white/40">{product.tagline}</div>
            </div>
          </a>
          <div className="flex flex-wrap gap-x-5 gap-y-3 text-sm text-white/48">
            {navItems.map((item) => (
              <a key={item.href} href={item.href} className="transition hover:text-white">
                {item.label}
              </a>
            ))}
            <a href={product.repoUrl} target="_blank" rel="noreferrer" className="transition hover:text-white">
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </>
  );
}
