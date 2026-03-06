import { readFileSync } from "node:fs";
import { loadRegisteredCommands } from "./command-lifecycle-lib.mjs";

const MAP_PATH = "contracts/tauri/v1/command-lifecycle.json";
const ALLOWED_STATUSES = new Set(["active", "deprecated", "alias_of"]);

const lifecycle = JSON.parse(readFileSync(MAP_PATH, "utf8"));
const commands = lifecycle?.commands;

if (lifecycle?.schema_version !== "command-lifecycle.v1") {
  throw new Error(`Invalid schema_version in ${MAP_PATH}; expected command-lifecycle.v1`);
}

if (!Number.isInteger(lifecycle?.current_release_cycle)) {
  throw new Error(`Invalid current_release_cycle in ${MAP_PATH}; expected integer`);
}

if (!commands || typeof commands !== "object" || Array.isArray(commands)) {
  throw new Error(`Invalid commands object in ${MAP_PATH}`);
}

const currentReleaseCycle = lifecycle.current_release_cycle;
const registered = new Set(loadRegisteredCommands());
const declared = new Set(Object.keys(commands));
const errors = [];

for (const command of registered) {
  if (!declared.has(command)) {
    errors.push(`Missing lifecycle entry for registered command: ${command}`);
  }
}

for (const command of declared) {
  if (!registered.has(command)) {
    errors.push(`Lifecycle entry exists for non-registered command: ${command}`);
  }
}

for (const [command, entry] of Object.entries(commands)) {
  if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
    errors.push(`Command ${command} has non-object lifecycle entry`);
    continue;
  }

  const status = entry.status;
  if (!ALLOWED_STATUSES.has(status)) {
    errors.push(
      `Command ${command} has invalid status '${status}'. Allowed: active|deprecated|alias_of`,
    );
    continue;
  }

  if (status === "active") continue;

  const removalCycle = entry.remove_after_release_cycle;
  if (!Number.isInteger(removalCycle)) {
    errors.push(
      `Command ${command} (${status}) must include integer remove_after_release_cycle`,
    );
    continue;
  }

  if (removalCycle < currentReleaseCycle + 2) {
    errors.push(
      `Command ${command} (${status}) has remove_after_release_cycle=${removalCycle}; must be >= current_release_cycle + 2 (${currentReleaseCycle + 2})`,
    );
  }

  if (status === "alias_of") {
    if (typeof entry.alias_of !== "string" || entry.alias_of.trim().length === 0) {
      errors.push(`Command ${command} (alias_of) must include alias_of target`);
    } else if (!registered.has(entry.alias_of)) {
      errors.push(
        `Command ${command} (alias_of) references unknown target '${entry.alias_of}'`,
      );
    } else if (entry.alias_of === command) {
      errors.push(`Command ${command} (alias_of) cannot alias itself`);
    }
  }
}

if (errors.length > 0) {
  const detail = errors.map((error) => `- ${error}`).join("\n");
  throw new Error(`Command lifecycle policy validation failed:\n${detail}`);
}

console.log(
  `Command lifecycle policy valid: ${registered.size} commands mapped (current_release_cycle=${currentReleaseCycle})`,
);
