use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use memory_kernel_outcome_core::RetrievalMode;
use multi_agent_center_domain::{
    ContextPackageEnvelope, NormalizedWorkflow, NormalizedWorkflowEnvelope, RunId,
};
use multi_agent_center_orchestrator::{
    AllowAllTrustGateSource, ApiMemoryKernelContextSource, DefaultHumanGateDecider,
    HumanGateDecider, HumanGateRequest, HumanGateResponse, NoopProposedWriteApplier, Orchestrator,
    OutcomeMemoryTrustGateSource, RunConfig, StaticContextPackageSource,
};
use multi_agent_center_trace_core::TraceStore;
use multi_agent_center_trace_sqlite::SqliteTraceStore;
use multi_agent_center_workflow::load_workflow_from_path;
use serde_json::json;
use time::OffsetDateTime;
use ulid::Ulid;

#[derive(Debug, Parser)]
#[command(name = "multi-agent-center")]
#[command(about = "Controlled agent orchestration with SQLite audit traces")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
    Trace(TraceArgs),
    Replay(ReplayArgs),
    Export(ExportArgs),
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long)]
    workflow: PathBuf,
    #[arg(long)]
    trace_db: PathBuf,
    #[arg(long)]
    memory_db: Option<PathBuf>,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long)]
    as_of: Option<String>,
    #[arg(long)]
    external_correlation_id: Option<String>,
    #[arg(long, default_value_t = false)]
    non_interactive: bool,
    #[arg(long)]
    trust_db: Option<PathBuf>,
    #[arg(long, default_value = "safe")]
    trust_mode: String,
    #[arg(long, default_value_t = false)]
    apply_proposed_writes: bool,
}

#[derive(Debug, Args)]
struct ReplayArgs {
    #[arg(long)]
    trace_db: PathBuf,
    #[arg(long)]
    run_id: String,
    #[arg(long, default_value_t = false)]
    rerun_provider: bool,
}

#[derive(Debug, Args)]
struct ExportArgs {
    #[arg(long)]
    trace_db: PathBuf,
    #[arg(long)]
    run_id: String,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct TraceArgs {
    #[command(subcommand)]
    command: TraceSubcommand,
}

#[derive(Debug, Subcommand)]
enum TraceSubcommand {
    Runs {
        #[arg(long)]
        trace_db: PathBuf,
    },
    Events {
        #[arg(long)]
        trace_db: PathBuf,
        #[arg(long)]
        run_id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_command(args),
        Commands::Trace(args) => trace_command(args),
        Commands::Replay(args) => replay_command(&args),
        Commands::Export(args) => export_command(&args),
    }
}

fn run_command(args: RunArgs) -> Result<()> {
    let workflow = load_workflow_from_path(&args.workflow)?;
    let trace_store = SqliteTraceStore::open(&args.trace_db)?;
    trace_store.migrate()?;

    let context_source = StaticContextPackageSource::default();
    let human_gate = CliHumanGateDecider;
    let write_applier = NoopProposedWriteApplier;
    let memory_db_opt = args.memory_db.clone();
    let trust_db_opt = args.trust_db.clone();

    let run_id = args.run_id.as_deref().map(parse_run_id).transpose()?;

    let as_of = args.as_of.as_deref().map(parse_rfc3339).transpose()?;

    let trust_mode = parse_retrieval_mode(&args.trust_mode)?;

    let config = RunConfig {
        run_id,
        as_of,
        replay_of_run_id: None,
        external_correlation_id: args.external_correlation_id,
        non_interactive: args.non_interactive,
        cli_args_json: json!({
            "workflow": args.workflow,
            "trace_db": args.trace_db,
            "memory_db": memory_db_opt,
            "non_interactive": args.non_interactive,
            "trust_mode": args.trust_mode,
            "trust_db": trust_db_opt,
            "apply_proposed_writes": args.apply_proposed_writes,
        }),
        engine_version: "multi-agent-center.v0".to_string(),
        apply_proposed_writes: args.apply_proposed_writes,
    };

    let summary = if let Some(memory_db) = memory_db_opt.as_ref() {
        let context_source = ApiMemoryKernelContextSource::new(memory_db);
        if let Some(trust_db) = trust_db_opt.as_ref() {
            let trust_source = OutcomeMemoryTrustGateSource::new(trust_db, trust_mode);
            Orchestrator::new(
                &trace_store,
                &context_source,
                &trust_source,
                &human_gate,
                &write_applier,
            )
            .execute_workflow(&workflow, config)?
        } else {
            let trust_source = AllowAllTrustGateSource;
            Orchestrator::new(
                &trace_store,
                &context_source,
                &trust_source,
                &human_gate,
                &write_applier,
            )
            .execute_workflow(&workflow, config)?
        }
    } else if let Some(trust_db) = trust_db_opt.as_ref() {
        let trust_source = OutcomeMemoryTrustGateSource::new(trust_db, trust_mode);
        Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(&workflow, config)?
    } else {
        let trust_source = AllowAllTrustGateSource;
        Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(&workflow, config)?
    };

    println!(
        "run_id={} status={} steps_total={} steps_succeeded={} steps_failed_or_rejected={}",
        summary.run_id,
        format_run_status(&summary.status),
        summary.steps_total,
        summary.steps_succeeded,
        summary.steps_failed_or_rejected
    );

    Ok(())
}

fn trace_command(args: TraceArgs) -> Result<()> {
    match args.command {
        TraceSubcommand::Runs { trace_db } => {
            let trace_store = SqliteTraceStore::open(&trace_db)?;
            let runs = trace_store.list_runs()?;
            for run in runs {
                println!("{}", serde_json::to_string(&run)?);
            }
        }
        TraceSubcommand::Events { trace_db, run_id } => {
            let trace_store = SqliteTraceStore::open(&trace_db)?;
            let run_id = parse_run_id(&run_id)?;
            let events = trace_store.list_events_for_run(run_id)?;
            for event in events {
                println!("{}", serde_json::to_string(&event)?);
            }
        }
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn replay_command(args: &ReplayArgs) -> Result<()> {
    let trace_store = SqliteTraceStore::open(&args.trace_db)?;
    let run_id = parse_run_id(&args.run_id)?;
    if args.rerun_provider {
        let source_run = trace_store
            .get_run(run_id)?
            .ok_or_else(|| anyhow!("run_id {run_id} not found"))?;
        let snapshot = trace_store
            .get_workflow_snapshot(&source_run.workflow_hash)?
            .ok_or_else(|| anyhow!("workflow snapshot {} not found", source_run.workflow_hash))?;
        let normalized_workflow: NormalizedWorkflow =
            serde_json::from_value(snapshot.normalized_json.clone())
                .map_err(|err| anyhow!("invalid normalized workflow snapshot JSON: {err}"))?;
        let workflow = NormalizedWorkflowEnvelope {
            source_format: snapshot.source_format,
            source_yaml_hash: snapshot.source_yaml_hash,
            normalized_hash: snapshot.workflow_hash,
            normalized_workflow,
            normalized_json: snapshot.normalized_json,
        };

        let context_rows = trace_store.get_step_context_packages(run_id)?;
        let mut by_step: BTreeMap<String, Vec<ContextPackageEnvelope>> = BTreeMap::new();
        for row in context_rows {
            by_step.entry(row.step_key).or_default().push(row.envelope);
        }

        let context_source = StaticContextPackageSource::with_step_packages(by_step);
        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let replay_config = RunConfig {
            run_id: None,
            as_of: Some(source_run.as_of),
            replay_of_run_id: Some(run_id),
            external_correlation_id: source_run
                .external_correlation_id
                .map(|id| format!("{id}:rerun")),
            non_interactive: false,
            cli_args_json: json!({
                "trace_db": args.trace_db,
                "source_run_id": run_id.to_string(),
                "rerun_provider": true,
            }),
            engine_version: "multi-agent-center.v0".to_string(),
            apply_proposed_writes: false,
        };

        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(&workflow, replay_config)?;

        println!(
            "source_run_id={} replay_run_id={} status={} steps_total={} steps_succeeded={} steps_failed_or_rejected={}",
            run_id,
            summary.run_id,
            format_run_status(&summary.status),
            summary.steps_total,
            summary.steps_succeeded,
            summary.steps_failed_or_rejected
        );
    } else {
        let context_source = StaticContextPackageSource::default();
        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let report = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .replay_audit(run_id)?;

        println!(
            "run_id={} events={} chain_valid={}",
            report.run_id, report.events, report.chain_valid
        );
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn export_command(args: &ExportArgs) -> Result<()> {
    let trace_store = SqliteTraceStore::open(&args.trace_db)?;
    let run_id = parse_run_id(&args.run_id)?;
    let events = trace_store.list_events_for_run(run_id)?;
    let event_count = events.len();

    let output = File::create(&args.out)?;
    let mut writer = BufWriter::new(output);
    for event in &events {
        writeln!(writer, "{}", serde_json::to_string(&event)?)?;
    }
    writer.flush()?;

    println!("exported {} events to {}", event_count, args.out.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct CliHumanGateDecider;

impl HumanGateDecider for CliHumanGateDecider {
    fn decide(&self, request: &HumanGateRequest) -> Result<HumanGateResponse> {
        if request.non_interactive {
            return Ok(HumanGateResponse {
                approved: false,
                notes: Some("non-interactive auto-reject".to_string()),
                decided_by: "system.non_interactive".to_string(),
                reason_codes: vec!["rejected.non_interactive".to_string()],
            });
        }

        eprintln!(
            "Human gate '{}' for step '{}' (run={} step={}).",
            request.gate_name, request.step_key, request.run_id, request.step_id
        );
        eprint!("Approve? [y/N]: ");
        std::io::stdout().flush()?;

        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        let normalized = answer.trim().to_ascii_lowercase();
        let approved = matches!(normalized.as_str(), "y" | "yes");

        eprint!("Notes (optional): ");
        std::io::stdout().flush()?;
        let mut notes = String::new();
        std::io::stdin().read_line(&mut notes)?;
        let notes = if notes.trim().is_empty() {
            None
        } else {
            Some(notes.trim().to_string())
        };

        let decided_by = std::env::var("USER").unwrap_or_else(|_| "human.cli".to_string());
        let reason_codes = if approved {
            vec!["approved.human_cli".to_string()]
        } else {
            vec!["rejected.human_cli".to_string()]
        };

        Ok(HumanGateResponse {
            approved,
            notes,
            decided_by,
            reason_codes,
        })
    }
}

fn parse_run_id(input: &str) -> Result<RunId> {
    let value = Ulid::from_str(input).map_err(|err| anyhow!("invalid run_id ULID: {err}"))?;
    Ok(RunId(value))
}

fn parse_rfc3339(input: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(input, &time::format_description::well_known::Rfc3339)
        .map_err(|err| anyhow!("invalid RFC3339 timestamp: {err}"))
}

fn parse_retrieval_mode(input: &str) -> Result<RetrievalMode> {
    RetrievalMode::parse(input)
        .ok_or_else(|| anyhow!("invalid trust_mode '{input}'; use 'safe' or 'exploration'"))
}

fn format_run_status(status: &multi_agent_center_domain::RunStatus) -> &'static str {
    match status {
        multi_agent_center_domain::RunStatus::Pending => "pending",
        multi_agent_center_domain::RunStatus::Running => "running",
        multi_agent_center_domain::RunStatus::Succeeded => "succeeded",
        multi_agent_center_domain::RunStatus::Failed => "failed",
        multi_agent_center_domain::RunStatus::Rejected => "rejected",
    }
}
