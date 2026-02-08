//! Stable embedded Outcome command surface for host runtimes.
//!
//! Host projects (such as `MemoryKernel`) should embed Outcome behavior through:
//! - [`run_cli`] for full parsed CLI execution.
//! - [`run_outcome_with_db`] for direct `OutcomeCommand` execution against a DB path.
//! - [`run_outcome`] for execution against an existing [`SqliteOutcomeStore`].
//!
//! These entrypoints are the supported v1 embed API and are version-frozen by
//! `/Users/d/Projects/OutcomeMemory/docs/v1-contract-freeze.md`.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use memory_kernel_core::MemoryId;
use memory_kernel_outcome_core::{
    format_rfc3339, now_utc, parse_rfc3339_utc, GateDecision, MemoryKey, OutcomeEventInput,
    OutcomeEventType, RetrievalMode, Severity,
};
use memory_kernel_outcome_store_sqlite::{
    parse_memory_key, BenchmarkConfig, BenchmarkReport, BenchmarkThresholds, ProjectorCheck,
    ProjectorIssueSeverity, ProjectorStaleKey, ProjectorStatus, SqliteOutcomeStore,
};
use ulid::Ulid;

#[derive(Debug, Parser)]
#[command(name = "mk")]
#[command(about = "Memory Kernel Outcome CLI")]
pub struct Cli {
    #[arg(long, default_value = "./memory_kernel.sqlite3")]
    db: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Outcome {
        #[command(subcommand)]
        command: Box<OutcomeCommand>,
    },
}

#[derive(Debug, Subcommand)]
pub enum OutcomeCommand {
    Log(LogArgs),
    Manual {
        #[command(subcommand)]
        command: Box<ManualCommand>,
    },
    System {
        #[command(subcommand)]
        command: Box<SystemCommand>,
    },
    Trust {
        #[command(subcommand)]
        command: Box<TrustCommand>,
    },
    Replay(ReplayArgs),
    Benchmark {
        #[command(subcommand)]
        command: Box<BenchmarkCommand>,
    },
    Projector {
        #[command(subcommand)]
        command: Box<ProjectorCommand>,
    },
    Gate {
        #[command(subcommand)]
        command: Box<GateCommand>,
    },
    Events {
        #[command(subcommand)]
        command: Box<EventsCommand>,
    },
}

#[derive(Debug, Args)]
pub struct LogArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    event: LogEventArg,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long)]
    edited: bool,
    #[arg(long)]
    escalated: bool,
    #[arg(long)]
    severity: Option<SeverityArg>,
    #[arg(long)]
    occurred_at: Option<String>,
    #[arg(long, default_value_t = 1)]
    ruleset_version: u32,
    #[arg(long, default_value = "{}")]
    payload_json: String,
}

#[derive(Debug, Subcommand)]
pub enum ManualCommand {
    SetConfidence(ManualSetConfidenceArgs),
    Promote(ManualSimpleArgs),
    Retire(ManualSimpleArgs),
}

#[derive(Debug, Args)]
pub struct ManualSetConfidenceArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    value: f32,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    override_cap: bool,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long)]
    occurred_at: Option<String>,
    #[arg(long, default_value_t = 1)]
    ruleset_version: u32,
    #[arg(long, default_value = "{}")]
    payload_json: String,
}

#[derive(Debug, Args)]
pub struct ManualSimpleArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long)]
    occurred_at: Option<String>,
    #[arg(long, default_value_t = 1)]
    ruleset_version: u32,
    #[arg(long, default_value = "{}")]
    payload_json: String,
}

#[derive(Debug, Subcommand)]
pub enum SystemCommand {
    Contradiction(SystemContradictionArgs),
    Inherit(SystemInheritArgs),
}

#[derive(Debug, Args)]
pub struct SystemContradictionArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long)]
    occurred_at: Option<String>,
    #[arg(long)]
    escalated: bool,
    #[arg(long)]
    severity: Option<SeverityArg>,
    #[arg(long, default_value_t = 1)]
    ruleset_version: u32,
    #[arg(long, default_value = "{}")]
    payload_json: String,
}

#[derive(Debug, Args)]
pub struct SystemInheritArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    source_confidence: f32,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long)]
    occurred_at: Option<String>,
    #[arg(long, default_value_t = 1)]
    ruleset_version: u32,
    #[arg(long, default_value = "{}")]
    payload_json: String,
}

#[derive(Debug, Subcommand)]
pub enum TrustCommand {
    Show(TrustShowArgs),
}

#[derive(Debug, Args)]
pub struct TrustShowArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    as_of: Option<String>,
}

#[derive(Debug, Args)]
pub struct ReplayArgs {
    #[arg(long)]
    from_event_seq: Option<i64>,
}

#[derive(Debug, Subcommand)]
pub enum BenchmarkCommand {
    Run(BenchmarkRunArgs),
}

#[derive(Debug, Args)]
pub struct BenchmarkRunArgs {
    #[arg(long = "volume")]
    volumes: Vec<usize>,
    #[arg(long, default_value_t = 3)]
    repetitions: usize,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    append_p95_max_ms: Option<f64>,
    #[arg(long)]
    replay_p95_max_ms: Option<f64>,
    #[arg(long)]
    gate_p95_max_ms: Option<f64>,
}

#[derive(Debug, Subcommand)]
pub enum ProjectorCommand {
    Status(ProjectorStatusArgs),
    Check(ProjectorCheckArgs),
    StaleKeys(ProjectorStaleKeysArgs),
}

#[derive(Debug, Args)]
pub struct ProjectorStatusArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct ProjectorCheckArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct ProjectorStaleKeysArgs {
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
pub enum GateCommand {
    Preview(GatePreviewArgs),
}

#[derive(Debug, Args)]
pub struct GatePreviewArgs {
    #[arg(long)]
    mode: GateModeArg,
    #[arg(long)]
    as_of: String,
    #[arg(long)]
    context_id: Option<String>,
    #[arg(long = "candidate")]
    candidates: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
pub enum EventsCommand {
    List(EventsListArgs),
}

#[derive(Debug, Args)]
pub struct EventsListArgs {
    #[arg(long)]
    memory_id: String,
    #[arg(long)]
    version: u32,
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogEventArg {
    Success,
    Failure,
    Ignored,
    Unknown,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SeverityArg {
    Low,
    Med,
    High,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GateModeArg {
    Safe,
    Exploration,
}

/// Executes the parsed top-level CLI command graph.
///
/// # Errors
/// Returns an error when command parsing dependencies, migration, or command
/// execution fails.
pub fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Outcome { command } => match *command {
            OutcomeCommand::Benchmark { command } => run_benchmark(*command),
            outcome_command => {
                let mut store = SqliteOutcomeStore::open(&cli.db)?;
                store.migrate()?;
                run_outcome(outcome_command, &mut store)
            }
        },
    }
}

/// Executes a parsed Outcome command using the provided `SQLite` DB path.
///
/// # Errors
/// Returns an error when store open/migrate fails or the requested command fails.
pub fn run_outcome_with_db(db_path: &std::path::Path, command: OutcomeCommand) -> Result<()> {
    match command {
        OutcomeCommand::Benchmark { command } => run_benchmark(*command),
        outcome_command => {
            let mut store = SqliteOutcomeStore::open(db_path)?;
            store.migrate()?;
            run_outcome(outcome_command, &mut store)
        }
    }
}

/// Executes a parsed Outcome command against an existing store handle.
///
/// # Errors
/// Returns an error when command validation, persistence, replay, or retrieval
/// operations fail.
pub fn run_outcome(command: OutcomeCommand, store: &mut SqliteOutcomeStore) -> Result<()> {
    match command {
        OutcomeCommand::Log(args) => {
            let payload = parse_payload_json(&args.payload_json)?;
            let input = OutcomeEventInput {
                event_id: None,
                ruleset_version: args.ruleset_version,
                memory_id: parse_memory_id(&args.memory_id)?,
                version: args.version,
                event_type: match args.event {
                    LogEventArg::Success => OutcomeEventType::Success,
                    LogEventArg::Failure => OutcomeEventType::Failure,
                    LogEventArg::Ignored => OutcomeEventType::Ignored,
                    LogEventArg::Unknown => OutcomeEventType::Unknown,
                },
                occurred_at: parse_optional_utc(args.occurred_at.as_deref())?,
                writer: args.writer,
                justification: args.justification,
                context_id: args.context_id,
                edited: args.edited,
                escalated: args.escalated,
                severity: args.severity.map(map_severity),
                manual_confidence: None,
                override_cap: false,
                payload_json: payload,
            };

            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
        OutcomeCommand::Manual { command } => run_manual(*command, store),
        OutcomeCommand::System { command } => run_system(*command, store),
        OutcomeCommand::Trust { command } => run_trust(*command, store),
        OutcomeCommand::Replay(args) => {
            let report = store.replay(args.from_event_seq)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        OutcomeCommand::Benchmark { .. } => Err(anyhow!(
            "internal dispatch error: benchmark should be handled before store initialization"
        )),
        OutcomeCommand::Projector { command } => run_projector(*command, store),
        OutcomeCommand::Gate { command } => run_gate(*command, store),
        OutcomeCommand::Events { command } => run_events(*command, store),
    }
}

fn run_manual(command: ManualCommand, store: &mut SqliteOutcomeStore) -> Result<()> {
    match command {
        ManualCommand::SetConfidence(args) => {
            let payload = parse_payload_json(&args.payload_json)?;
            let input = OutcomeEventInput {
                event_id: None,
                ruleset_version: args.ruleset_version,
                memory_id: parse_memory_id(&args.memory_id)?,
                version: args.version,
                event_type: OutcomeEventType::ManualSetConfidence,
                occurred_at: parse_optional_utc(args.occurred_at.as_deref())?,
                writer: args.writer,
                justification: args.justification,
                context_id: args.context_id,
                edited: false,
                escalated: false,
                severity: None,
                manual_confidence: Some(args.value),
                override_cap: args.override_cap,
                payload_json: payload,
            };

            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
        ManualCommand::Promote(args) => {
            let input = from_manual_simple(args, OutcomeEventType::ManualPromote)?;
            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
        ManualCommand::Retire(args) => {
            let input = from_manual_simple(args, OutcomeEventType::ManualRetire)?;
            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
    }
}

fn run_system(command: SystemCommand, store: &mut SqliteOutcomeStore) -> Result<()> {
    match command {
        SystemCommand::Contradiction(args) => {
            let payload = parse_payload_json(&args.payload_json)?;
            let input = OutcomeEventInput {
                event_id: None,
                ruleset_version: args.ruleset_version,
                memory_id: parse_memory_id(&args.memory_id)?,
                version: args.version,
                event_type: OutcomeEventType::AuthoritativeContradiction,
                occurred_at: parse_optional_utc(args.occurred_at.as_deref())?,
                writer: args.writer,
                justification: args.justification,
                context_id: args.context_id,
                edited: false,
                escalated: args.escalated,
                severity: args.severity.map(map_severity),
                manual_confidence: None,
                override_cap: false,
                payload_json: payload,
            };
            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
        SystemCommand::Inherit(args) => {
            let payload = parse_payload_json(&args.payload_json)?;
            let input = OutcomeEventInput {
                event_id: None,
                ruleset_version: args.ruleset_version,
                memory_id: parse_memory_id(&args.memory_id)?,
                version: args.version,
                event_type: OutcomeEventType::Inherited,
                occurred_at: parse_optional_utc(args.occurred_at.as_deref())?,
                writer: args.writer,
                justification: args.justification,
                context_id: args.context_id,
                edited: false,
                escalated: false,
                severity: None,
                manual_confidence: Some(args.source_confidence),
                override_cap: false,
                payload_json: payload,
            };
            let event = store.append_event(&input)?;
            println!("{}", serde_json::to_string_pretty(&event)?);
            Ok(())
        }
    }
}

fn run_trust(command: TrustCommand, store: &SqliteOutcomeStore) -> Result<()> {
    match command {
        TrustCommand::Show(args) => {
            let memory_id = parse_memory_id(&args.memory_id)?;
            let as_of = match args.as_of {
                Some(raw) => Some(
                    parse_rfc3339_utc(&raw)
                        .map_err(|err| anyhow!("invalid --as-of value: {err}"))?,
                ),
                None => None,
            };

            let Some(trust) = store.get_memory_trust(memory_id, args.version, as_of)? else {
                return Err(anyhow!(
                    "trust snapshot not found for {}:{}",
                    args.memory_id,
                    args.version
                ));
            };

            println!("{}", serde_json::to_string_pretty(&trust)?);
            Ok(())
        }
    }
}

fn run_gate(command: GateCommand, store: &SqliteOutcomeStore) -> Result<()> {
    match command {
        GateCommand::Preview(args) => {
            if args.candidates.is_empty() {
                return Err(anyhow!(
                    "at least one --candidate <memory_id:version> is required"
                ));
            }

            let as_of = parse_rfc3339_utc(&args.as_of)
                .map_err(|err| anyhow!("invalid --as-of value: {err}"))?;
            let mode = match args.mode {
                GateModeArg::Safe => RetrievalMode::Safe,
                GateModeArg::Exploration => RetrievalMode::Exploration,
            };

            let candidates = args
                .candidates
                .iter()
                .map(|raw| parse_memory_key(raw))
                .collect::<Result<Vec<_>>>()?;

            let decisions =
                store.gate_preview(mode, as_of, args.context_id.as_deref(), &candidates)?;

            if args.json {
                let payload = build_gate_preview_json_payload(
                    mode,
                    as_of,
                    args.context_id.as_deref(),
                    &candidates,
                    &decisions,
                )?;
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                print_gate_table(mode, &candidates, &decisions);
            }
            Ok(())
        }
    }
}

fn run_events(command: EventsCommand, store: &SqliteOutcomeStore) -> Result<()> {
    match command {
        EventsCommand::List(args) => {
            let memory_id = parse_memory_id(&args.memory_id)?;
            let events = store.list_events_for_key(memory_id, args.version, args.limit)?;
            println!("{}", serde_json::to_string_pretty(&events)?);
            Ok(())
        }
    }
}

/// Runs the benchmark command group and optional threshold enforcement.
///
/// # Errors
/// Returns an error when argument combinations are invalid, benchmark execution
/// fails, artifact write fails, or thresholds are violated.
pub fn run_benchmark(command: BenchmarkCommand) -> Result<()> {
    match command {
        BenchmarkCommand::Run(args) => {
            let volumes = if args.volumes.is_empty() {
                vec![100, 500, 2_000]
            } else {
                args.volumes
            };

            let thresholds = match (
                args.append_p95_max_ms,
                args.replay_p95_max_ms,
                args.gate_p95_max_ms,
            ) {
                (Some(append), Some(replay), Some(gate)) => Some(BenchmarkThresholds {
                    append_p95_ms_max: append,
                    replay_p95_ms_max: replay,
                    gate_p95_ms_max: gate,
                }),
                (None, None, None) => None,
                _ => {
                    return Err(anyhow!(
                        "benchmark thresholds require all of --append-p95-max-ms, --replay-p95-max-ms, --gate-p95-max-ms"
                    ))
                }
            };

            let config = BenchmarkConfig {
                volumes,
                repetitions: args.repetitions,
            };

            // Benchmark runner uses isolated temporary sqlite files and does not depend on --db.
            let benchmark_runner = SqliteOutcomeStore::open(std::path::Path::new(":memory:"))?;
            let report = benchmark_runner.run_benchmark(&config, thresholds)?;

            if let Some(path) = args.output {
                let serialized = serde_json::to_string_pretty(&report)?;
                std::fs::write(&path, serialized).with_context(|| {
                    format!("failed writing benchmark report to {}", path.display())
                })?;
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_benchmark_report(&report);
            }

            if !report.within_thresholds {
                return Err(anyhow!(
                    "benchmark thresholds violated: {}",
                    report.violations.join("; ")
                ));
            }
            Ok(())
        }
    }
}

fn run_projector(command: ProjectorCommand, store: &SqliteOutcomeStore) -> Result<()> {
    match command {
        ProjectorCommand::Status(args) => {
            let status = store.projector_status()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                print_projector_status(&status);
            }
            Ok(())
        }
        ProjectorCommand::Check(args) => {
            let check = store.projector_check()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&check)?);
            } else {
                print_projector_check(&check);
            }

            if !check.healthy {
                return Err(anyhow!(
                    "projector consistency check failed: {}",
                    check
                        .issues
                        .iter()
                        .map(|item| format!("{}:{}", item.code, item.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }

            Ok(())
        }
        ProjectorCommand::StaleKeys(args) => {
            let stale_keys = store.projector_stale_keys(args.limit)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&stale_keys)?);
            } else {
                print_projector_stale_keys(&stale_keys);
            }
            Ok(())
        }
    }
}

fn from_manual_simple(
    args: ManualSimpleArgs,
    event_type: OutcomeEventType,
) -> Result<OutcomeEventInput> {
    Ok(OutcomeEventInput {
        event_id: None,
        ruleset_version: args.ruleset_version,
        memory_id: parse_memory_id(&args.memory_id)?,
        version: args.version,
        event_type,
        occurred_at: parse_optional_utc(args.occurred_at.as_deref())?,
        writer: args.writer,
        justification: args.justification,
        context_id: args.context_id,
        edited: false,
        escalated: false,
        severity: None,
        manual_confidence: None,
        override_cap: false,
        payload_json: parse_payload_json(&args.payload_json)?,
    })
}

fn parse_payload_json(raw: &str) -> Result<serde_json::Value> {
    serde_json::from_str(raw).with_context(|| format!("payload_json must be valid JSON: {raw}"))
}

fn parse_optional_utc(raw: Option<&str>) -> Result<time::OffsetDateTime> {
    match raw {
        Some(value) => parse_rfc3339_utc(value).map_err(|err| anyhow!("invalid timestamp: {err}")),
        None => Ok(now_utc()),
    }
}

fn parse_memory_id(raw: &str) -> Result<MemoryId> {
    let parsed = Ulid::from_string(raw).with_context(|| format!("invalid ULID: {raw}"))?;
    Ok(MemoryId(parsed))
}

fn map_severity(value: SeverityArg) -> Severity {
    match value {
        SeverityArg::Low => Severity::Low,
        SeverityArg::Med => Severity::Med,
        SeverityArg::High => Severity::High,
    }
}

fn print_gate_table(mode: RetrievalMode, candidates: &[MemoryKey], decisions: &[GateDecision]) {
    println!("mode: {mode:?}");
    println!(
        "{:<32} {:<7} {:<8} {:<10} {:<7} reasons",
        "memory_id", "version", "include", "confidence", "capped"
    );
    println!("{}", "-".repeat(110));

    for (candidate, decision) in candidates.iter().zip(decisions) {
        println!(
            "{:<32} {:<7} {:<8} {:<10.3} {:<7} {}",
            candidate.memory_id,
            candidate.version,
            if decision.include { "yes" } else { "no" },
            decision.confidence_effective,
            if decision.capped { "yes" } else { "no" },
            decision.reason_codes.join(",")
        );
    }
}

fn print_projector_status(status: &ProjectorStatus) {
    println!(
        "contract={} projector={} ruleset={} projected_event_seq={} latest_event_seq={} lag_events={} lag_delta={}",
        status.contract_version,
        status.projector_name,
        status.ruleset_version,
        status.projected_event_seq,
        status.latest_event_seq,
        status.lag_events,
        status.lag_delta_events
    );
    println!(
        "tracked_keys={} trust_rows={} stale_trust_rows={} keys_with_events_no_trust_row={} trust_rows_without_events={} max_stale_seq_gap={} updated_at={}",
        status.tracked_keys,
        status.trust_rows,
        status.stale_trust_rows,
        status.keys_with_events_no_trust_row,
        status.trust_rows_without_events,
        status.max_stale_seq_gap,
        status.updated_at.as_deref().unwrap_or("n/a")
    );
}

fn print_projector_check(check: &ProjectorCheck) {
    println!("contract={}", check.contract_version);
    print_projector_status(&check.status);
    println!("healthy={}", if check.healthy { "yes" } else { "no" });
    if !check.issues.is_empty() {
        let formatted = check
            .issues
            .iter()
            .map(|item| {
                let severity = match item.severity {
                    ProjectorIssueSeverity::Warning => "warning",
                    ProjectorIssueSeverity::Error => "error",
                };
                format!("{severity}:{}:{}", item.code, item.message)
            })
            .collect::<Vec<_>>()
            .join(" | ");
        println!("issues={formatted}");
        println!("hint=run `mk outcome projector stale-keys --json` for affected keys");
    }
}

fn print_projector_stale_keys(stale_keys: &[ProjectorStaleKey]) {
    println!(
        "{:<32} {:<7} {:<14} projected_event_seq",
        "memory_id", "version", "max_event_seq"
    );
    println!("{}", "-".repeat(80));
    for item in stale_keys {
        println!(
            "{:<32} {:<7} {:<14} {}",
            item.memory_id,
            item.version,
            item.max_event_seq,
            item.projected_event_seq
                .map_or_else(|| "none".to_string(), |value| value.to_string())
        );
    }
}

fn print_benchmark_report(report: &BenchmarkReport) {
    println!(
        "contract={} generated_at={} repetitions={} within_thresholds={}",
        report.contract_version,
        report.generated_at,
        report.repetitions,
        if report.within_thresholds {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "{:<10} {:<12} {:<12} {:<12} {:<12} {:<12} {:<12}",
        "events", "append_p50", "append_p95", "replay_p50", "replay_p95", "gate_p50", "gate_p95"
    );
    println!("{}", "-".repeat(90));
    for item in &report.volumes {
        println!(
            "{:<10} {:<12.3} {:<12.3} {:<12.3} {:<12.3} {:<12.3} {:<12.3}",
            item.event_count,
            item.append_p50_ms,
            item.append_p95_ms,
            item.replay_p50_ms,
            item.replay_p95_ms,
            item.gate_p50_ms,
            item.gate_p95_ms
        );
    }

    if !report.violations.is_empty() {
        println!("violations={}", report.violations.join(" | "));
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GatePreviewJsonPayload {
    contract_version: String,
    mode: RetrievalMode,
    as_of: String,
    context_id: Option<String>,
    candidates: Vec<MemoryKey>,
    decisions: Vec<GateDecision>,
}

fn build_gate_preview_json_payload(
    mode: RetrievalMode,
    as_of: time::OffsetDateTime,
    context_id: Option<&str>,
    candidates: &[MemoryKey],
    decisions: &[GateDecision],
) -> Result<GatePreviewJsonPayload> {
    Ok(GatePreviewJsonPayload {
        contract_version: "gate_preview.v1".to_string(),
        mode,
        as_of: format_rfc3339(as_of).map_err(|err| anyhow!(err.to_string()))?,
        context_id: context_id.map(str::to_string),
        candidates: candidates.to_vec(),
        decisions: decisions.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines, clippy::manual_let_else)]

    use super::*;
    use rusqlite::Connection;
    use serde_json::json;
    use std::fs;

    fn must<T>(result: Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("test failure: {err}"),
        }
    }

    #[test]
    fn parse_payload_accepts_valid_json() {
        let value = must(parse_payload_json(r#"{"key":"value"}"#));
        assert_eq!(value["key"], json!("value"));
    }

    #[test]
    fn parse_payload_rejects_invalid_json() {
        let value = parse_payload_json("{");
        assert!(value.is_err());
    }

    #[test]
    fn parse_optional_utc_rejects_non_utc() {
        let value = parse_optional_utc(Some("2026-02-07T12:00:00+02:00"));
        assert!(value.is_err());
    }

    fn execute_cli(args: Vec<String>) -> Result<()> {
        let cli = Cli::try_parse_from(args)?;
        run_cli(cli)
    }

    fn fixture_memory_id() -> MemoryId {
        let parsed = match Ulid::from_string("01J0SQQP7M70P6Y3R4T8D8G8M2") {
            Ok(value) => value,
            Err(err) => panic!("invalid fixture ULID: {err}"),
        };
        MemoryId(parsed)
    }

    #[test]
    fn gate_json_contract_is_stable_v1() {
        let memory_id = fixture_memory_id();
        let candidate = MemoryKey {
            memory_id,
            version: 1,
        };
        let decision = GateDecision {
            memory_id,
            version: 1,
            include: true,
            confidence_effective: 0.5,
            trust_status: memory_kernel_outcome_core::TrustStatus::Validated,
            capped: false,
            reason_codes: vec!["included.safe.validated_threshold".to_string()],
        };
        let as_of =
            must(parse_rfc3339_utc("2026-02-07T12:00:00Z").map_err(|err| anyhow!(err.to_string())));

        let payload = must(build_gate_preview_json_payload(
            RetrievalMode::Safe,
            as_of,
            Some("ctx-1"),
            &[candidate],
            &[decision],
        ));

        let value = must(serde_json::to_value(payload).map_err(Into::into));
        assert_eq!(
            value,
            json!({
                "contract_version": "gate_preview.v1",
                "mode": "safe",
                "as_of": "2026-02-07T12:00:00Z",
                "context_id": "ctx-1",
                "candidates": [
                    {
                        "memory_id": "01J0SQQP7M70P6Y3R4T8D8G8M2",
                        "version": 1
                    }
                ],
                "decisions": [
                    {
                        "memory_id": "01J0SQQP7M70P6Y3R4T8D8G8M2",
                        "version": 1,
                        "include": true,
                        "confidence_effective": 0.5,
                        "trust_status": "validated",
                        "capped": false,
                        "reason_codes": ["included.safe.validated_threshold"]
                    }
                ]
            })
        );
    }

    #[test]
    fn stable_embed_api_host_path_stays_operational() {
        let db_path =
            std::env::temp_dir().join(format!("outcome-embed-host-{}.sqlite3", Ulid::new()));
        let db_path_str = match db_path.to_str() {
            Some(value) => value.to_string(),
            None => panic!("temp db path must be valid UTF-8"),
        };
        let memory_id = fixture_memory_id();

        let setup_conn = must(Connection::open(&db_path).map_err(Into::into));
        must(
            memory_kernel_outcome_store_sqlite::seed_minimal_memory_record(
                &setup_conn,
                memory_id,
                1,
            ),
        );

        must(run_outcome_with_db(
            &db_path,
            OutcomeCommand::Log(LogArgs {
                memory_id: memory_id.to_string(),
                version: 1,
                event: LogEventArg::Success,
                writer: "tester".to_string(),
                justification: "embed api regression".to_string(),
                context_id: Some("ctx-embed".to_string()),
                edited: false,
                escalated: false,
                severity: None,
                occurred_at: Some("2026-02-07T12:00:00Z".to_string()),
                ruleset_version: 1,
                payload_json: "{}".to_string(),
            }),
        ));

        let mut store = must(SqliteOutcomeStore::open(&db_path));
        must(store.migrate());
        must(run_outcome(
            OutcomeCommand::Replay(ReplayArgs {
                from_event_seq: None,
            }),
            &mut store,
        ));

        let cli = match Cli::try_parse_from(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str,
            "outcome".to_string(),
            "projector".to_string(),
            "status".to_string(),
            "--json".to_string(),
        ]) {
            Ok(value) => value,
            Err(err) => panic!("failed to parse cli args for embed API regression test: {err}"),
        };
        must(run_cli(cli));

        must(run_benchmark(BenchmarkCommand::Run(BenchmarkRunArgs {
            volumes: vec![10],
            repetitions: 1,
            output: None,
            json: true,
            append_p95_max_ms: Some(5_000.0),
            replay_p95_max_ms: Some(5_000.0),
            gate_p95_max_ms: Some(5_000.0),
        })));

        let _ = fs::remove_file(&db_path);
    }

    #[test]
    fn cli_end_to_end_log_replay_show_and_gate_preview() {
        let db_path = std::env::temp_dir().join(format!("outcome-cli-e2e-{}.sqlite3", Ulid::new()));
        let db_path_str = match db_path.to_str() {
            Some(value) => value.to_string(),
            None => panic!("temp db path must be valid UTF-8"),
        };

        let memory_id = fixture_memory_id();
        let setup_conn = must(Connection::open(&db_path).map_err(Into::into));
        must(
            memory_kernel_outcome_store_sqlite::seed_minimal_memory_record(
                &setup_conn,
                memory_id,
                1,
            ),
        );

        for _ in 0..3 {
            must(execute_cli(vec![
                "mk".to_string(),
                "--db".to_string(),
                db_path_str.clone(),
                "outcome".to_string(),
                "log".to_string(),
                "--memory-id".to_string(),
                memory_id.to_string(),
                "--version".to_string(),
                "1".to_string(),
                "--event".to_string(),
                "success".to_string(),
                "--writer".to_string(),
                "tester".to_string(),
                "--justification".to_string(),
                "fixture".to_string(),
                "--occurred-at".to_string(),
                "2026-02-07T12:00:00Z".to_string(),
            ]));
        }

        must(execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "projector".to_string(),
            "status".to_string(),
            "--json".to_string(),
        ]));
        let check_before_replay = execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "projector".to_string(),
            "check".to_string(),
            "--json".to_string(),
        ]);
        assert!(check_before_replay.is_err());

        must(execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "replay".to_string(),
        ]));
        must(execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "projector".to_string(),
            "check".to_string(),
            "--json".to_string(),
        ]));

        must(execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "trust".to_string(),
            "show".to_string(),
            "--memory-id".to_string(),
            memory_id.to_string(),
            "--version".to_string(),
            "1".to_string(),
            "--as-of".to_string(),
            "2026-02-07T12:00:00Z".to_string(),
        ]));

        must(execute_cli(vec![
            "mk".to_string(),
            "--db".to_string(),
            db_path_str.clone(),
            "outcome".to_string(),
            "gate".to_string(),
            "preview".to_string(),
            "--mode".to_string(),
            "safe".to_string(),
            "--as-of".to_string(),
            "2026-02-07T12:00:00Z".to_string(),
            "--context-id".to_string(),
            "ctx-1".to_string(),
            "--candidate".to_string(),
            format!("{memory_id}:1"),
            "--json".to_string(),
        ]));

        let store = must(SqliteOutcomeStore::open(&db_path));
        must(store.migrate());
        let trust = match must(store.get_memory_trust(memory_id, 1, None)) {
            Some(value) => value,
            None => panic!("missing trust snapshot after replay"),
        };
        assert_eq!(
            trust.trust_status,
            memory_kernel_outcome_core::TrustStatus::Validated
        );

        let decisions = must(store.gate_preview(
            RetrievalMode::Safe,
            must(parse_rfc3339_utc("2026-02-07T12:00:00Z").map_err(|err| anyhow!(err.to_string()))),
            Some("ctx-1"),
            &[MemoryKey {
                memory_id,
                version: 1,
            }],
        ));
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].include);

        let _ = fs::remove_file(&db_path);
    }
}
