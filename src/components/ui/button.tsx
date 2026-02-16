import * as React from "react";

type ButtonVariant = "primary" | "secondary" | "danger";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
}

export function Button({ variant = "primary", size = "md", loading = false, className = "", ...props }: ButtonProps) {
  return (
    <button
      data-variant={variant}
      data-size={size}
      data-loading={loading ? "true" : "false"}
      className={className}
      disabled={loading || props.disabled}
      {...props}
    />
  );
}
