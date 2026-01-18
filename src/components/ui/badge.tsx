import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-md border px-1.5 py-0.5 text-[10px] font-semibold transition-colors focus:outline-none",
  {
    variants: {
      variant: {
        default: "border-transparent bg-red-500 text-white",
        secondary: "border-transparent bg-slate-100 text-slate-600",
        destructive: "border-transparent bg-red-50 text-red-600 border-red-200",
        outline: "text-slate-600 border-slate-200 bg-white",
        success: "border-transparent bg-emerald-50 text-emerald-600 border-emerald-200",
        warning: "border-transparent bg-amber-50 text-amber-600 border-amber-200",
        live: "border-transparent bg-red-500 text-white font-bold shadow-sm shadow-red-500/30",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
