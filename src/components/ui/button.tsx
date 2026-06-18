import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium transition-all duration-150 disabled:pointer-events-none disabled:opacity-40 select-none",
  {
    variants: {
      variant: {
        default:
          "bg-[var(--glass-control-bg)] border border-[var(--glass-border)] text-[var(--text-primary)] hover:bg-[var(--glass-control-hover)] active:bg-[var(--glass-control-active)]",
        ghost:
          "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--glass-control-bg)]",
        glass:
          "bg-[var(--glass-control-bg)] border border-[var(--glass-border)] text-[var(--text-primary)] hover:bg-[var(--glass-control-hover)] active:bg-[var(--glass-control-active)]",
        accent:
          "bg-[var(--accent-subtle)] border border-[rgba(139,92,246,0.25)] text-[var(--accent)] hover:bg-[var(--accent-subtle-hover)]",
        icon: "bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--glass-control-bg)] rounded-lg",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-7 px-3 text-xs",
        lg: "h-11 px-6",
        icon: "h-8 w-8",
        "icon-sm": "h-6 w-6",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
    );
  },
);

Button.displayName = "Button";

export { Button, buttonVariants };
