import { existsSync } from "node:fs";
import { execSync } from "node:child_process";

const failures = [];
const warnings = [];

const requiredFiles = [
  "package.json",
  "pnpm-lock.yaml",
  "src-tauri/Cargo.toml",
  "search-api/requirements-test.txt",
  "search-api/smoke_search_api.py",
];

for (const file of requiredFiles) {
  if (!existsSync(file)) {
    failures.push(`Missing required file: ${file}`);
  }
}

const nodeMajor = Number.parseInt(process.versions.node.split(".")[0] ?? "", 10);
if (!Number.isFinite(nodeMajor) || nodeMajor < 20) {
  failures.push(`Node.js 20+ is required (detected ${process.versions.node}).`);
}

try {
  execSync("cargo --version", { stdio: "ignore" });
} catch {
  warnings.push("cargo is not available; Rust checks will fail until Rust is installed.");
}

try {
  execSync("python3 --version", { stdio: "ignore" });
} catch {
  warnings.push("python3 is not available; search-api checks will fail until Python is installed.");
}

if (warnings.length > 0) {
  console.warn("Preflight warnings:");
  for (const warning of warnings) {
    console.warn(`- ${warning}`);
  }
}

if (failures.length > 0) {
  console.error("Preflight failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("Workstation preflight passed.");
