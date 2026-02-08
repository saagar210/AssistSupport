interface BadgeProps {
  children: React.ReactNode;
  variant?: "default" | "success" | "warning" | "error" | "info";
  size?: "sm" | "md";
}

const VARIANT_CLASSES: Record<string, string> = {
  default: "bg-muted text-foreground",
  success: "bg-success/10 text-success border-success/30",
  warning: "bg-warning/10 text-warning border-warning/30",
  error: "bg-destructive/10 text-destructive border-destructive/30",
  info: "bg-accent/10 text-accent border-accent/30",
};

const SIZE_CLASSES: Record<string, string> = {
  sm: "px-1.5 py-0.5 text-[10px]",
  md: "px-2 py-0.5 text-xs",
};

export function Badge({
  children,
  variant = "default",
  size = "md",
}: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded-full border font-medium ${VARIANT_CLASSES[variant]} ${SIZE_CLASSES[size]}`}
      data-testid="badge"
    >
      {children}
    </span>
  );
}
