import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";

const WINDOW_DAYS = Number.parseInt(process.env.DORA_WINDOW_DAYS ?? "28", 10);
const DEPLOYMENT_RECORDS_PATH =
  process.env.DORA_DEPLOYMENT_RECORDS_PATH ?? "artifacts/deployments/deployment-records.json";

const exec = (command) =>
  execSync(command, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    maxBuffer: 10 * 1024 * 1024,
  }).trim();

const parseGitLog = (raw) =>
  raw
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [authorIso, committerIso, subject] = line.split("|");
      const authorMs = Date.parse(authorIso ?? "");
      const committerMs = Date.parse(committerIso ?? "");
      return {
        authorIso,
        committerIso,
        subject: subject ?? "",
        authorMs: Number.isFinite(authorMs) ? authorMs : null,
        committerMs: Number.isFinite(committerMs) ? committerMs : null,
      };
    });

const mean = (values) =>
  values.length === 0 ? null : values.reduce((sum, value) => sum + value, 0) / values.length;

const parseIsoMs = (value) => {
  const parsed = Date.parse(value ?? "");
  return Number.isFinite(parsed) ? parsed : null;
};

const toBool = (value) => {
  if (typeof value === "boolean") return value;
  if (typeof value === "number") return value !== 0;
  if (typeof value !== "string") return false;
  const normalized = value.trim().toLowerCase();
  return ["1", "true", "yes", "failed", "failure", "error"].includes(normalized);
};

const readDeploymentRecords = () => {
  if (!existsSync(DEPLOYMENT_RECORDS_PATH)) return [];
  try {
    const raw = JSON.parse(readFileSync(DEPLOYMENT_RECORDS_PATH, "utf8"));
    if (Array.isArray(raw)) return raw;
    if (raw && Array.isArray(raw.records)) return raw.records;
    return [];
  } catch {
    return [];
  }
};

const withinWindow = (timestampMs) => {
  if (!Number.isFinite(timestampMs)) return false;
  const now = Date.now();
  const windowMs = WINDOW_DAYS * 24 * 60 * 60 * 1000;
  return timestampMs >= now - windowMs && timestampMs <= now;
};

const deploymentRecords = readDeploymentRecords()
  .map((record) => {
    const deployedMs =
      parseIsoMs(record?.deployed_at)
      ?? parseIsoMs(record?.completed_at)
      ?? parseIsoMs(record?.created_at);
    const failedAtMs = parseIsoMs(record?.failed_at) ?? deployedMs;
    const recoveredAtMs =
      parseIsoMs(record?.recovered_at)
      ?? parseIsoMs(record?.restored_at)
      ?? parseIsoMs(record?.resolved_at);
    const explicitLeadTime = Number.parseFloat(
      String(record?.lead_time_hours ?? record?.leadTimeHours ?? ""),
    );
    const explicitMttr = Number.parseFloat(String(record?.mttr_hours ?? record?.mttrHours ?? ""));
    const status = String(record?.status ?? record?.outcome ?? "").trim().toLowerCase();
    const failed =
      toBool(record?.failed)
      || toBool(record?.is_failed)
      || status === "failed"
      || status === "failure"
      || status === "error";

    return {
      deployedMs,
      failedAtMs,
      recoveredAtMs,
      failed,
      leadTimeHours: Number.isFinite(explicitLeadTime) ? explicitLeadTime : null,
      mttrHours: Number.isFinite(explicitMttr) ? explicitMttr : null,
    };
  })
  .filter((record) => withinWindow(record.deployedMs));

const commitsRaw = exec(
  `git log --since="${WINDOW_DAYS} days ago" --first-parent --pretty=format:'%aI|%cI|%s'`,
);
const commits = parseGitLog(commitsRaw);
const totalCommits = commits.length;

const leadTimeSamples = commits
  .map((commit) => {
    if (!Number.isFinite(commit.authorMs) || !Number.isFinite(commit.committerMs)) return null;
    const delta = (commit.committerMs - commit.authorMs) / 3_600_000;
    return delta >= 0 ? delta : 0;
  })
  .filter((value) => Number.isFinite(value));

const revertIndices = commits
  .map((commit, index) => ({ commit, index }))
  .filter(({ commit }) => /^revert\b/i.test(commit.subject));

const mttrSamples = [];
for (const { index } of revertIndices) {
  const revertedAt = commits[index].committerMs;
  if (!Number.isFinite(revertedAt)) continue;
  const recovery = commits
    .slice(index + 1)
    .find((candidate) => !/^revert\b/i.test(candidate.subject) && Number.isFinite(candidate.committerMs));
  if (!recovery) continue;
  const deltaHours = (recovery.committerMs - revertedAt) / 3_600_000;
  if (deltaHours >= 0) {
    mttrSamples.push(deltaHours);
  }
}

const envLeadTime = process.env.DORA_LEAD_TIME_HOURS
  ? Number.parseFloat(process.env.DORA_LEAD_TIME_HOURS)
  : null;
const envDeployFrequency = process.env.DORA_DEPLOY_FREQUENCY_PER_WEEK
  ? Number.parseFloat(process.env.DORA_DEPLOY_FREQUENCY_PER_WEEK)
  : null;
const envChangeFailRate = process.env.DORA_CHANGE_FAIL_RATE
  ? Number.parseFloat(process.env.DORA_CHANGE_FAIL_RATE)
  : null;
const envMttr = process.env.DORA_MTTR_HOURS ? Number.parseFloat(process.env.DORA_MTTR_HOURS) : null;
const recordLeadTime = mean(
  deploymentRecords
    .map((record) => record.leadTimeHours)
    .filter((value) => Number.isFinite(value)),
);
const recordDeploymentFrequency =
  deploymentRecords.length === 0 ? null : deploymentRecords.length / (WINDOW_DAYS / 7);
const recordChangeFailRate =
  deploymentRecords.length === 0
    ? null
    : deploymentRecords.filter((record) => record.failed).length / deploymentRecords.length;
const recordMttr = mean(
  deploymentRecords
    .map((record) => {
      if (Number.isFinite(record.mttrHours)) return record.mttrHours;
      if (!record.failed) return null;
      if (!Number.isFinite(record.failedAtMs) || !Number.isFinite(record.recoveredAtMs)) return null;
      const deltaHours = (record.recoveredAtMs - record.failedAtMs) / 3_600_000;
      return deltaHours >= 0 ? deltaHours : null;
    })
    .filter((value) => Number.isFinite(value)),
);

const sourceParts = ["git-first-parent-history"];
if (deploymentRecords.length > 0) {
  sourceParts.push("deployment-records");
}
if (
  Number.isFinite(envLeadTime)
  || Number.isFinite(envDeployFrequency)
  || Number.isFinite(envChangeFailRate)
  || Number.isFinite(envMttr)
) {
  sourceParts.push("ci-env-overrides");
}

const dora = {
  generatedAt: new Date().toISOString(),
  window_days: WINDOW_DAYS,
  source: sourceParts.join("+"),
  deployment_records_path: DEPLOYMENT_RECORDS_PATH,
  lead_time_hours: Number.isFinite(envLeadTime)
    ? envLeadTime
    : Number.isFinite(recordLeadTime)
      ? recordLeadTime
      : mean(leadTimeSamples),
  deployment_frequency_per_week: Number.isFinite(envDeployFrequency)
    ? envDeployFrequency
    : Number.isFinite(recordDeploymentFrequency)
      ? recordDeploymentFrequency
      : totalCommits / (WINDOW_DAYS / 7),
  change_fail_rate: Number.isFinite(envChangeFailRate)
    ? envChangeFailRate
    : Number.isFinite(recordChangeFailRate)
      ? recordChangeFailRate
      : totalCommits === 0
        ? null
        : revertIndices.length / totalCommits,
  mttr_hours: Number.isFinite(envMttr)
    ? envMttr
    : Number.isFinite(recordMttr)
      ? recordMttr
      : mean(mttrSamples),
  samples: {
    commit_count: totalCommits,
    revert_count: revertIndices.length,
    lead_time_sample_count: leadTimeSamples.length,
    mttr_sample_count: mttrSamples.length,
    deployment_record_count: deploymentRecords.length,
    deployment_failed_count: deploymentRecords.filter((record) => record.failed).length,
  },
};

mkdirSync(".perf-results", { recursive: true });
writeFileSync(".perf-results/dora.json", `${JSON.stringify(dora, null, 2)}\n`);

console.log("DORA snapshot captured at .perf-results/dora.json");
