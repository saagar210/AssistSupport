import { readFileSync } from "node:fs";

const GENERATE_HANDLER_MARKER = "tauri::generate_handler![";

const isIdentifierChar = (char) => /[A-Za-z0-9_:]/.test(char);

export const extractGenerateHandlerBody = (source) => {
  const start = source.indexOf(GENERATE_HANDLER_MARKER);
  if (start === -1) {
    throw new Error(`Missing ${GENERATE_HANDLER_MARKER} in src-tauri/src/lib.rs`);
  }

  const bytes = source;
  let idx = start + GENERATE_HANDLER_MARKER.length;
  let depth = 1;

  while (idx < bytes.length) {
    const char = bytes[idx];
    if (char === "[") {
      depth += 1;
    } else if (char === "]") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start + GENERATE_HANDLER_MARKER.length, idx);
      }
    }
    idx += 1;
  }

  throw new Error("Unterminated tauri::generate_handler![] list");
};

export const extractRegisteredCommands = (handlerBody) => {
  const commands = new Set();
  let cursor = 0;

  while (cursor < handlerBody.length) {
    const found = handlerBody.indexOf("commands::", cursor);
    if (found === -1) break;

    let end = found;
    while (end < handlerBody.length && isIdentifierChar(handlerBody[end])) {
      end += 1;
    }

    const candidate = handlerBody.slice(found, end);
    if ((candidate.match(/::/g) ?? []).length >= 2) {
      commands.add(candidate);
    }
    cursor = end;
  }

  return [...commands].sort((a, b) => a.localeCompare(b));
};

export const loadRegisteredCommands = (libPath = "src-tauri/src/lib.rs") => {
  const libSource = readFileSync(libPath, "utf8");
  const handlerBody = extractGenerateHandlerBody(libSource);
  return extractRegisteredCommands(handlerBody);
};
