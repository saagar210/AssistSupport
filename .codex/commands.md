# AssistSupport .codex command map

| Action | Command | Source |
| --- | --- | --- |
| setup deps | `pnpm install --frozen-lockfile` | `.github/workflows/ci.yml` |
| lint | `pnpm run typecheck` | `.github/workflows/ci.yml`, `package.json` |
| test | `pnpm run test:ci` | `package.json` |
| build | `pnpm run build` | `package.json` |
| lean dev | `pnpm run dev:lean` | `README.md`, `package.json` |
