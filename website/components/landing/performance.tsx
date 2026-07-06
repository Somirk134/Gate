"use client";

import { useEffect, useRef, useState } from "react";
import { animate, motion, useInView } from "motion/react";

import { Badge } from "@/components/ui/badge";
import { metrics } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";

export function Performance() {
  return (
    <SectionShell
      id="performance"
      eyebrow="Performance"
      title="Numbers shown as targets, not marketing mythology."
      description="Gate V1 prioritizes correctness, stable lifecycle and a clean data-plane boundary. The public benchmark lanes are already documented for future pressure testing."
    >
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {metrics.map((metric, index) => {
          const Icon = metric.icon;

          return (
            <motion.article
              key={metric.label}
              className="glass-panel min-h-[220px] rounded-[20px] p-5"
              initial={{ opacity: 0, y: 22 }}
              whileInView={{ opacity: 1, y: 0 }}
              whileHover={{ y: -7 }}
              transition={{ duration: 0.45, delay: index * 0.06 }}
              viewport={{ once: true }}
            >
              <div className="mb-8 flex items-center justify-between">
                <div className="flex size-11 items-center justify-center rounded-[16px] bg-primary/10 text-primary ring-1 ring-primary/18">
                  <Icon className="size-5" />
                </div>
                <Badge variant="secondary">target</Badge>
              </div>
              <div className="font-mono text-4xl font-semibold text-white">
                <AnimatedNumber value={metric.value} suffix={metric.suffix} />
              </div>
              <p className="mt-4 text-sm leading-6 text-white/54">{metric.label}</p>
            </motion.article>
          );
        })}
      </div>
    </SectionShell>
  );
}

function AnimatedNumber({ value, suffix }: { value: number; suffix: string }) {
  const ref = useRef<HTMLSpanElement | null>(null);
  const isInView = useInView(ref, { once: true, margin: "-80px" });
  const [display, setDisplay] = useState(0);

  useEffect(() => {
    if (!isInView) {
      return;
    }

    const controls = animate(0, value, {
      duration: 1.8,
      ease: [0.22, 1, 0.36, 1],
      onUpdate: (latest) => setDisplay(latest),
    });

    return () => controls.stop();
  }, [isInView, value]);

  return (
    <span ref={ref}>
      {Math.round(display).toLocaleString("en-US")}
      {suffix}
    </span>
  );
}
