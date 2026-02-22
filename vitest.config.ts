import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    environment: "node",
    reporters: ["default"],
    coverage: {
      provider: "v8",
      reportsDirectory: "coverage/frontend",
      reporter: ["text-summary", "lcov"],
    },
  },
});
