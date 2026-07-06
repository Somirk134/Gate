"use client";

import { useState } from "react";
import Image from "next/image";
import { Download, Github, Menu, X } from "lucide-react";
import { motion } from "motion/react";

import { Button } from "@/components/ui/button";
import { navItems, product } from "@/components/landing/data";
import { cn } from "@/lib/utils";

export function Navigation() {
  const [open, setOpen] = useState(false);

  return (
    <header className="fixed inset-x-0 top-0 z-50 px-4 pt-4">
      <motion.nav
        className="glass-panel mx-auto flex h-16 max-w-7xl items-center justify-between rounded-[20px] px-4 sm:px-5"
        initial={{ opacity: 0, y: -18 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.55, ease: [0.22, 1, 0.36, 1] }}
      >
        <a href="#top" className="flex items-center gap-3" aria-label="Gate home">
          <span className="relative flex size-9 overflow-hidden rounded-2xl shadow-[0_0_34px_rgba(66,248,211,0.18)]">
            <Image src="/gate-mark.svg" alt="" width={36} height={36} priority />
          </span>
          <span className="text-sm font-semibold text-white">{product.name}</span>
        </a>

        <div className="hidden items-center gap-1 lg:flex">
          {navItems.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="rounded-full px-3.5 py-2 text-sm text-white/58 transition hover:bg-white/[0.07] hover:text-white"
            >
              {item.label}
            </a>
          ))}
        </div>

        <div className="hidden items-center gap-2 md:flex">
          <Button variant="ghost" size="sm" asChild>
            <a href={product.repoUrl} target="_blank" rel="noreferrer">
              <Github />
              GitHub
            </a>
          </Button>
          <Button size="sm" asChild>
            <a href="#cta">
              <Download />
              Get started
            </a>
          </Button>
        </div>

        <Button
          variant="ghost"
          size="icon"
          className="md:hidden"
          aria-label={open ? "Close navigation" : "Open navigation"}
          onClick={() => setOpen((value) => !value)}
        >
          {open ? <X /> : <Menu />}
        </Button>
      </motion.nav>

      <div
        className={cn(
          "glass-panel mx-auto mt-3 max-w-7xl rounded-[20px] p-3 transition md:hidden",
          open ? "pointer-events-auto translate-y-0 opacity-100" : "pointer-events-none -translate-y-3 opacity-0",
        )}
      >
        <div className="grid gap-1">
          {navItems.map((item) => (
            <a
              key={item.href}
              href={item.href}
              onClick={() => setOpen(false)}
              className="rounded-2xl px-4 py-3 text-sm text-white/70 transition hover:bg-white/[0.08] hover:text-white"
            >
              {item.label}
            </a>
          ))}
        </div>
        <div className="mt-3 grid grid-cols-2 gap-2">
          <Button variant="secondary" size="sm" asChild>
            <a href={product.repoUrl} target="_blank" rel="noreferrer">
              <Github />
              GitHub
            </a>
          </Button>
          <Button size="sm" asChild>
            <a href="#cta">
              <Download />
              Start
            </a>
          </Button>
        </div>
      </div>
    </header>
  );
}
