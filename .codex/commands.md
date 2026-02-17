# AssistSupport .codex command map

| Action                  | Command                                             | Source                                     |
| ----------------------- | --------------------------------------------------- | ------------------------------------------ |
| setup deps              | `pnpm install --frozen-lockfile`                    | `.github/workflows/ci.yml`                 |
| lint                    | `pnpm run typecheck`                                | `.github/workflows/ci.yml`, `package.json` |
| test                    | `pnpm run test:ci`                                  | `package.json`                             |
| build                   | `pnpm run build`                                    | `package.json`                             |
| lean dev                | `pnpm run dev:lean`                                 | `README.md`, `package.json`                |
| create branch from task | `pnpm run git:branch:create -- "task summary" feat` | `package.json`                             |
| git guardrails          | `pnpm run git:guard:all`                            | `package.json`                             |
| propose commit message  | `pnpm run git:commit:propose`                       | `package.json`                             |
| perf bundle             | `pnpm run perf:bundle`                              | `package.json`                             |
| perf build              | `pnpm run perf:build`                               | `package.json`                             |
| perf assets             | `pnpm run perf:assets`                              | `package.json`                             |
