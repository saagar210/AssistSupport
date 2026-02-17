import fs from "node:fs";

const candidateRoots = [
  "apps",
  "packages",
  "services",
  "libs",
  "src",
  "src-tauri",
  "search-api",
  "tests",
];
const discoveredScopes = candidateRoots
  .filter((dir) => fs.existsSync(dir))
  .flatMap((dir) =>
    fs
      .readdirSync(dir, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name),
  );

const scopes = [
  ...new Set(["repo", "deps", "ci", "release", "perf", "ui", "db", "docs", ...discoveredScopes]),
];

export default {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "refactor", "perf", "docs", "test", "build", "ci", "chore", "revert"],
    ],
    "scope-enum": [2, "always", scopes],
    "header-max-length": [2, "always", 72],
    "subject-empty": [2, "never"],
    "subject-full-stop": [2, "never", "."],
  },
};
