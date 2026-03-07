import * as React from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  description?: string;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, description, id, ...props }, ref) => {
    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label htmlFor={id} className="text-xs font-medium text-[var(--text-secondary)]">
            {label}
          </label>
        )}
        <input
          id={id}
          ref={ref}
          className={cn(
            "h-9 w-full rounded-lg px-3 text-sm",
            "bg-[var(--glass-control-bg)] border border-[var(--glass-border)]",
            "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
            "focus:outline-none focus:border-[rgba(139,92,246,0.5)] focus:bg-[var(--glass-control-hover)]",
            "transition-all duration-150",
            className,
          )}
          {...props}
        />
        {description && (
          <p className="text-[10px] text-[var(--text-tertiary)]">{description}</p>
        )}
      </div>
    );
  },
);

Input.displayName = "Input";

export { Input };
