import type { Config } from "tailwindcss";

export default {
  theme: {
    extend: {
      spacing: {
        1: "var(--space-1)",
        2: "var(--space-2)",
        3: "var(--space-3)",
        4: "var(--space-4)",
        6: "var(--space-6)",
        8: "var(--space-8)",
      },
      fontSize: {
        sm: "var(--font-size-1)",
        base: "var(--font-size-2)",
        lg: "var(--font-size-3)",
        xl: "var(--font-size-4)",
      },
      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
      },
      colors: {
        bg: "hsl(var(--color-bg) / <alpha-value>)",
        fg: "hsl(var(--color-fg) / <alpha-value>)",
        muted: "hsl(var(--color-muted) / <alpha-value>)",
        primary: "hsl(var(--color-primary) / <alpha-value>)",
        danger: "hsl(var(--color-danger) / <alpha-value>)",
      },
    },
  },
} satisfies Config;
