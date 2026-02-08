import { forwardRef } from "react";

interface TextInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  icon?: React.ReactNode;
  error?: boolean;
}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  function TextInput({ icon, error, className = "", ...props }, ref) {
    return (
      <div className="relative">
        {icon && (
          <div className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
            {icon}
          </div>
        )}
        <input
          ref={ref}
          className={`h-9 w-full rounded-lg border bg-background px-3 text-sm text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:ring-1 focus:ring-accent ${
            icon ? "pl-9" : ""
          } ${
            error
              ? "border-destructive focus:ring-destructive"
              : "border-border"
          } ${className}`}
          {...props}
        />
      </div>
    );
  },
);
