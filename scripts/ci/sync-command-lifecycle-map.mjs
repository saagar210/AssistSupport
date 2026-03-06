import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { loadRegisteredCommands } from "./command-lifecycle-lib.mjs";

const OUTPUT_PATH = "contracts/tauri/v1/command-lifecycle.json";

const readExisting = () => {
  if (!existsSync(OUTPUT_PATH)) return null;
  return JSON.parse(readFileSync(OUTPUT_PATH, "utf8"));
};

const nowIso = new Date().toISOString();
const existing = readExisting();
const currentReleaseCycle = Number.isInteger(existing?.current_release_cycle)
  ? existing.current_release_cycle
  : 1;

const existingEntries = existing?.commands ?? {};
const commands = loadRegisteredCommands();

const nextEntries = {};
for (const command of commands) {
  const prev = existingEntries[command];
  if (prev && typeof prev === "object") {
    nextEntries[command] = prev;
  } else {
    nextEntries[command] = {
      status: "active",
    };
  }
}

const lifecycle = {
  schema_version: "command-lifecycle.v1",
  generated_at: nowIso,
  current_release_cycle: currentReleaseCycle,
  commands: Object.fromEntries(
    Object.entries(nextEntries).sort(([left], [right]) => left.localeCompare(right)),
  ),
};

mkdirSync("contracts/tauri/v1", { recursive: true });
writeFileSync(OUTPUT_PATH, `${JSON.stringify(lifecycle, null, 2)}\n`);

console.log(`Synced ${commands.length} command lifecycle entries to ${OUTPUT_PATH}`);
