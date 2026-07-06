"use client";

import { ArchitectureFlow } from "@/components/landing/architecture-flow";
import { CrossPlatform } from "@/components/landing/cross-platform";
import { CTAFooter } from "@/components/landing/cta-footer";
import { FAQ } from "@/components/landing/faq";
import { FeatureBento } from "@/components/landing/feature-bento";
import { Hero } from "@/components/landing/hero";
import { InteractiveDemo } from "@/components/landing/interactive-demo";
import { Navigation } from "@/components/landing/navigation";
import { OpenSource } from "@/components/landing/open-source";
import { Performance } from "@/components/landing/performance";
import { Roadmap } from "@/components/landing/roadmap";
import { SmoothScroll } from "@/components/landing/smooth-scroll";

export function LandingPage() {
  return (
    <div className="site-shell min-h-screen">
      <SmoothScroll />
      <Navigation />
      <main>
        <Hero />
        <FeatureBento />
        <InteractiveDemo />
        <ArchitectureFlow />
        <Performance />
        <CrossPlatform />
        <OpenSource />
        <Roadmap />
        <FAQ />
        <CTAFooter />
      </main>
    </div>
  );
}
