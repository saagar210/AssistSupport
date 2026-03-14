#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(
  path.dirname(new URL(import.meta.url).pathname),
  "..",
  "..",
);
const writeMode = process.argv.includes("--write");

const packageJsonPath = path.join(repoRoot, "package.json");
const tauriConfigPath = path.join(repoRoot, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(repoRoot, "src-tauri", "Cargo.toml");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const targetVersion = packageJson.version;

if (typeof targetVersion !== "string" || targetVersion.trim().length === 0) {
  throw new Error("package.json must contain a non-empty version");
}

const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));
const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");

const currentTauriVersion = tauriConfig.version;
const cargoVersionMatch = cargoToml.match(/^version = "([^"]+)"/m);

if (!cargoVersionMatch) {
  throw new Error("Could not find the package version in src-tauri/Cargo.toml");
}

const currentCargoVersion = cargoVersionMatch[1];

if (writeMode) {
  tauriConfig.version = targetVersion;
  fs.writeFileSync(
    tauriConfigPath,
    `${JSON.stringify(tauriConfig, null, 2)}\n`,
  );

  const updatedCargoToml = cargoToml.replace(
    /^version = "([^"]+)"/m,
    `version = "${targetVersion}"`,
  );
  fs.writeFileSync(cargoTomlPath, updatedCargoToml);

  console.log(`Synced Tauri and Cargo versions to ${targetVersion}`);
  process.exit(0);
}

const mismatches = [];

if (currentTauriVersion !== targetVersion) {
  mismatches.push(`src-tauri/tauri.conf.json=${currentTauriVersion}`);
}

if (currentCargoVersion !== targetVersion) {
  mismatches.push(`src-tauri/Cargo.toml=${currentCargoVersion}`);
}

if (mismatches.length > 0) {
  console.error(
    `Version mismatch: package.json=${targetVersion}, ${mismatches.join(", ")}. Run "pnpm version:sync" after bumping package.json.`,
  );
  process.exit(1);
}

console.log(`Version parity OK: ${targetVersion}`);
