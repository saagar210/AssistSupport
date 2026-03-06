import { execSync } from "node:child_process";

const commands = [
  "pnpm perf:bundle",
  "pnpm perf:build",
  "pnpm perf:memory",
  "pnpm perf:runtime",
  "pnpm perf:dora",
  "pnpm perf:summary",
  "pnpm perf:scorecard",
];

for (const command of commands) {
  console.log(`>>> ${command}`);
  execSync(command, { stdio: "inherit" });
}

console.log("Baseline capture workflow completed.");
