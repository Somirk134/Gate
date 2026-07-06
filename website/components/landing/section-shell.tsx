"use client";

import type { ReactNode } from "react";
import { motion } from "motion/react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type SectionShellProps = {
  id: string;
  eyebrow: string;
  title: string;
  description?: string;
  children: ReactNode;
  className?: string;
};

export function SectionShell({
  id,
  eyebrow,
  title,
  description,
  children,
  className,
}: SectionShellProps) {
  return (
    <motion.section
      id={id}
      className={cn("relative mx-auto w-full max-w-7xl px-5 py-20 sm:px-6 lg:px-8 lg:py-28", className)}
      initial={{ opacity: 0, y: 28 }}
      whileInView={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.65, ease: [0.22, 1, 0.36, 1] }}
      viewport={{ once: true, margin: "-120px" }}
    >
      <div className="mx-auto mb-10 max-w-3xl text-center lg:mb-14">
        <Badge variant="secondary" className="mb-5">
          {eyebrow}
        </Badge>
        <h2 className="text-balance text-3xl font-semibold tracking-normal text-white sm:text-4xl lg:text-5xl">
          {title}
        </h2>
        {description ? (
          <p className="mx-auto mt-5 max-w-2xl text-pretty text-base leading-8 text-white/58 sm:text-lg">
            {description}
          </p>
        ) : null}
      </div>
      {children}
    </motion.section>
  );
}
