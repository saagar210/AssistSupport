import { readFileSync } from "node:fs";

const requiredFixtures = [
  "contracts/tauri/v1/generate_with_context_result.json",
  "contracts/tauri/v1/hybrid_search_response.json",
  "contracts/tauri/v1/search_api_health_status.json",
  "contracts/search-api/v1/search_response.json",
  "contracts/search-api/v1/feedback_response.json",
  "contracts/search-api/v1/stats_response.json",
  "contracts/search-api/v1/health_response.json",
];

for (const fixturePath of requiredFixtures) {
  const raw = readFileSync(fixturePath, "utf8");
  try {
    JSON.parse(raw);
  } catch (error) {
    throw new Error(`Invalid JSON fixture at ${fixturePath}: ${error}`);
  }
}

console.log(`Contract fixture check passed (${requiredFixtures.length} files).`);
