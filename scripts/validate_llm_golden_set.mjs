import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const defaultEvidencePath = 'docs/revamp/evidence/LLM_GOLDEN_SET_BASELINE_2026-02-08.json';

function fail(message) {
  console.error(`LLM golden-set validation failed: ${message}`);
  process.exit(1);
}

function readJson(relPath) {
  const absPath = path.join(root, relPath);
  if (!fs.existsSync(absPath)) {
    fail(`missing evidence file: ${relPath}`);
  }
  try {
    return JSON.parse(fs.readFileSync(absPath, 'utf8'));
  } catch (error) {
    fail(`invalid JSON in ${relPath}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function requireNumber(value, field) {
  if (typeof value !== 'number' || Number.isNaN(value)) {
    fail(`field "${field}" must be a valid number`);
  }
}

function requireString(value, field) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    fail(`field "${field}" must be a non-empty string`);
  }
}

function ensureRange(value, field, min, max) {
  if (value < min || value > max) {
    fail(`field "${field}" must be in range [${min}, ${max}] but was ${value}`);
  }
}

function main() {
  const evidencePath = process.env.ASSISTSUPPORT_GOLDEN_SET_PATH ?? defaultEvidencePath;
  const minScore = Number(process.env.ASSISTSUPPORT_GOLDEN_SET_MIN_SCORE ?? 80);
  const maxHallucinationRate = Number(process.env.ASSISTSUPPORT_GOLDEN_SET_MAX_HALLUCINATION_RATE ?? 0.03);
  const maxPolicyViolationRate = Number(process.env.ASSISTSUPPORT_GOLDEN_SET_MAX_POLICY_VIOLATION_RATE ?? 0.01);
  const maxCriticalFailures = Number(process.env.ASSISTSUPPORT_GOLDEN_SET_MAX_CRITICAL_FAILURES ?? 0);

  const payload = readJson(evidencePath);

  requireString(payload.evaluated_at, 'evaluated_at');
  requireNumber(payload.sample_count, 'sample_count');
  requireNumber(payload.overall_score, 'overall_score');
  requireNumber(payload.hallucination_rate, 'hallucination_rate');
  requireNumber(payload.policy_violation_rate, 'policy_violation_rate');
  requireNumber(payload.critical_failures, 'critical_failures');

  ensureRange(payload.sample_count, 'sample_count', 1, Number.MAX_SAFE_INTEGER);
  ensureRange(payload.overall_score, 'overall_score', 0, 100);
  ensureRange(payload.hallucination_rate, 'hallucination_rate', 0, 1);
  ensureRange(payload.policy_violation_rate, 'policy_violation_rate', 0, 1);
  ensureRange(payload.critical_failures, 'critical_failures', 0, Number.MAX_SAFE_INTEGER);

  if (payload.overall_score < minScore) {
    fail(`overall_score ${payload.overall_score} is below minimum ${minScore}`);
  }
  if (payload.hallucination_rate > maxHallucinationRate) {
    fail(`hallucination_rate ${payload.hallucination_rate} exceeds maximum ${maxHallucinationRate}`);
  }
  if (payload.policy_violation_rate > maxPolicyViolationRate) {
    fail(`policy_violation_rate ${payload.policy_violation_rate} exceeds maximum ${maxPolicyViolationRate}`);
  }
  if (payload.critical_failures > maxCriticalFailures) {
    fail(`critical_failures ${payload.critical_failures} exceeds maximum ${maxCriticalFailures}`);
  }

  console.log(
    `LLM golden-set validation passed (score=${payload.overall_score}, hallucination_rate=${payload.hallucination_rate}, policy_violation_rate=${payload.policy_violation_rate}, critical_failures=${payload.critical_failures}).`,
  );
}

main();

