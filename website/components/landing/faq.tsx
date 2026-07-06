"use client";

import { faqs } from "@/components/landing/data";
import { SectionShell } from "@/components/landing/section-shell";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion";

export function FAQ() {
  return (
    <SectionShell
      id="faq"
      eyebrow="FAQ"
      title="Answers before you clone the repo."
      description="The short version: Gate is self-hosted, desktop-first and currently early-stage."
    >
      <div className="glass-panel mx-auto max-w-3xl rounded-[20px] px-5 sm:px-7">
        <Accordion type="single" collapsible defaultValue="item-0">
          {faqs.map((faq, index) => (
            <AccordionItem key={faq.question} value={`item-${index}`}>
              <AccordionTrigger>{faq.question}</AccordionTrigger>
              <AccordionContent>{faq.answer}</AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </div>
    </SectionShell>
  );
}
