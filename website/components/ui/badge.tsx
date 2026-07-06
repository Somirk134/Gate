import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium tracking-normal transition-colors",
  {
    variants: {
      variant: {
        default:
          "border-primary/25 bg-primary/10 text-primary shadow-[0_0_28px_rgba(66,248,211,0.12)]",
        secondary: "border-white/10 bg-white/[0.07] text-white/76",
        amber: "border-amber-300/25 bg-amber-300/10 text-amber-100",
        rose: "border-rose-300/25 bg-rose-300/10 text-rose-100",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

function Badge({
  className,
  variant,
  ...props
}: React.ComponentProps<"span"> & VariantProps<typeof badgeVariants>) {
  return (
    <span
      data-slot="badge"
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
