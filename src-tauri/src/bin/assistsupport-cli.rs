//! AssistSupport CLI - Local automation tool (Phase 18.3)
//!
//! Provides command-line access to:
//! - Ingestion triggers
//! - Backup/export operations
//! - Job status queries
//!
//! Usage:
//!   assistsupport-cli backup --output <path>
//!   assistsupport-cli jobs list [--status <status>]
//!   assistsupport-cli jobs get <job_id>
//!   assistsupport-cli kb stats
//!   assistsupport-cli kb index [--force]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

// Re-use types from the library
use assistsupport_lib::db::Database;
use assistsupport_lib::jobs::JobStatus;
use assistsupport_lib::kb::indexer::KbIndexer;
use assistsupport_lib::kb::search::HybridSearch;

/// CLI command structure
#[derive(Debug)]
enum Command {
    Backup { output: Option<PathBuf> },
    Jobs(JobsCommand),
    Kb(KbCommand),
    Help,
    Version,
}

#[derive(Debug)]
enum JobsCommand {
    List { status: Option<String> },
    Get { job_id: String },
    Cleanup { days: i64 },
}

#[derive(Debug)]
enum KbCommand {
    Stats,
    Index {
        #[allow(dead_code)]
        force: bool,
    },
    Search {
        query: String,
        limit: usize,
    },
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    match parse_args(&args) {
        Ok(cmd) => match run_command(cmd) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Ok(Command::Help);
    }

    match args[1].as_str() {
        "help" | "--help" | "-h" => Ok(Command::Help),
        "version" | "--version" | "-V" => Ok(Command::Version),

        "backup" => {
            let output = args
                .get(2)
                .filter(|a| *a == "--output" || *a == "-o")
                .and_then(|_| args.get(3))
                .map(PathBuf::from);
            Ok(Command::Backup { output })
        }

        "jobs" => {
            if args.len() < 3 {
                return Err("Missing jobs subcommand. Use: list, get, cleanup".to_string());
            }
            match args[2].as_str() {
                "list" => {
                    let status = args
                        .get(3)
                        .filter(|a| *a == "--status" || *a == "-s")
                        .and_then(|_| args.get(4))
                        .cloned();
                    Ok(Command::Jobs(JobsCommand::List { status }))
                }
                "get" => {
                    let job_id = args.get(3).ok_or("Missing job ID")?.clone();
                    Ok(Command::Jobs(JobsCommand::Get { job_id }))
                }
                "cleanup" => {
                    let days = args.get(3).and_then(|d| d.parse().ok()).unwrap_or(30);
                    Ok(Command::Jobs(JobsCommand::Cleanup { days }))
                }
                _ => Err(format!("Unknown jobs subcommand: {}", args[2])),
            }
        }

        "kb" => {
            if args.len() < 3 {
                return Err("Missing kb subcommand. Use: stats, index".to_string());
            }
            match args[2].as_str() {
                "stats" => Ok(Command::Kb(KbCommand::Stats)),
                "index" => {
                    let force = args
                        .get(3)
                        .map(|a| a == "--force" || a == "-f")
                        .unwrap_or(false);
                    Ok(Command::Kb(KbCommand::Index { force }))
                }
                "search" => {
                    let query = args.get(3).ok_or("Missing search query")?.clone();
                    let limit = args
                        .get(4)
                        .and_then(|a| if a == "--limit" || a == "-n" { args.get(5) } else { None })
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(5);
                    Ok(Command::Kb(KbCommand::Search { query, limit }))
                }
                _ => Err(format!("Unknown kb subcommand: {}", args[2])),
            }
        }

        _ => Err(format!("Unknown command: {}", args[1])),
    }
}

fn run_command(cmd: Command) -> Result<(), String> {
    match cmd {
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::Version => {
            println!("assistsupport-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Backup { output } => run_backup(output),
        Command::Jobs(jobs_cmd) => run_jobs_command(jobs_cmd),
        Command::Kb(kb_cmd) => run_kb_command(kb_cmd),
    }
}

fn print_help() {
    println!(
        r#"AssistSupport CLI - Local automation tool

USAGE:
    assistsupport-cli <COMMAND> [OPTIONS]

COMMANDS:
    backup              Create a database backup
        --output, -o    Output path for backup file

    jobs list           List jobs
        --status, -s    Filter by status (queued, running, succeeded, failed, cancelled)

    jobs get <ID>       Get job details

    jobs cleanup [DAYS] Cleanup old jobs (default: 30 days)

    kb stats            Show knowledge base statistics

    kb index            Trigger KB indexing
        --force, -f     Force re-index all documents

    kb search <QUERY>   Search the knowledge base
        --limit, -n     Number of results (default: 5)

    help                Show this help message
    version             Show version information

EXAMPLES:
    assistsupport-cli backup --output ~/backups/assist.db.bak
    assistsupport-cli jobs list --status running
    assistsupport-cli jobs get abc123
    assistsupport-cli kb stats
"#
    );
}

fn get_db_path() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("AssistSupport/assistsupport.db"))
        .expect("Could not determine data directory")
}

fn open_database() -> Result<Database, String> {
    let db_path = get_db_path();
    if !db_path.exists() {
        return Err(format!(
            "Database not found at {:?}. Run the app first to initialize.",
            db_path
        ));
    }

    // For CLI, we need to get the master key
    // This requires the app to have been initialized first
    use assistsupport_lib::security::FileKeyStore;

    let master_key = FileKeyStore::get_master_key()
        .map_err(|e| format!("Failed to get master key: {}. Initialize the app first.", e))?;

    Database::open(&db_path, &master_key).map_err(|e| format!("Failed to open database: {}", e))
}

fn run_backup(output: Option<PathBuf>) -> Result<(), String> {
    let db = open_database()?;

    let backup_path = match output {
        Some(p) => {
            // Use file copy approach for SQLCipher
            db.conn()
                .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|e| format!("Checkpoint failed: {}", e))?;
            std::fs::copy(get_db_path(), &p).map_err(|e| format!("Backup copy failed: {}", e))?;
            p
        }
        None => db.backup().map_err(|e| format!("Backup failed: {}", e))?,
    };

    println!("Backup created: {:?}", backup_path);
    Ok(())
}

fn run_jobs_command(cmd: JobsCommand) -> Result<(), String> {
    let db = open_database()?;

    match cmd {
        JobsCommand::List { status } => {
            let status_filter = status.as_ref().and_then(|s| match s.as_str() {
                "queued" => Some(JobStatus::Queued),
                "running" => Some(JobStatus::Running),
                "succeeded" => Some(JobStatus::Succeeded),
                "failed" => Some(JobStatus::Failed),
                "cancelled" => Some(JobStatus::Cancelled),
                _ => None,
            });
            let jobs = db
                .list_jobs(status_filter, 100)
                .map_err(|e| format!("Failed to list jobs: {}", e))?;

            if jobs.is_empty() {
                println!("No jobs found.");
            } else {
                println!(
                    "{:<36} {:<12} {:<20} {:<8}",
                    "ID", "TYPE", "STATUS", "PROGRESS"
                );
                println!("{}", "-".repeat(80));
                for job in jobs {
                    println!(
                        "{:<36} {:<12} {:<20} {:>6.1}%",
                        job.id,
                        job.job_type.to_string(),
                        job.status.to_string(),
                        job.progress
                    );
                }
            }
            Ok(())
        }
        JobsCommand::Get { job_id } => {
            let job = db
                .get_job(&job_id)
                .map_err(|e| format!("Failed to get job: {}", e))?
                .ok_or_else(|| format!("Job not found: {}", job_id))?;

            println!("Job: {}", job.id);
            println!("Type: {}", job.job_type);
            println!("Status: {}", job.status);
            println!("Progress: {:.1}%", job.progress);
            println!("Created: {}", job.created_at);
            println!("Updated: {}", job.updated_at);
            if let Some(error) = job.error {
                println!("Error: {}", error);
            }

            // Get logs (last 50)
            let logs = db
                .get_job_logs(&job_id, 50)
                .map_err(|e| format!("Failed to get logs: {}", e))?;
            if !logs.is_empty() {
                println!("\nLogs:");
                for log in logs {
                    println!("  [{}] {}: {}", log.timestamp, log.level, log.message);
                }
            }

            Ok(())
        }
        JobsCommand::Cleanup { days } => {
            let deleted = db
                .cleanup_old_jobs(days)
                .map_err(|e| format!("Failed to cleanup: {}", e))?;
            println!("Deleted {} old jobs (older than {} days)", deleted, days);
            Ok(())
        }
    }
}

fn run_kb_command(cmd: KbCommand) -> Result<(), String> {
    let db = open_database()?;

    match cmd {
        KbCommand::Stats => {
            // Get document and chunk counts
            let doc_count: i64 = db
                .conn()
                .query_row("SELECT COUNT(*) FROM kb_documents", [], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?;

            let chunk_count: i64 = db
                .conn()
                .query_row("SELECT COUNT(*) FROM kb_chunks", [], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?;

            let source_count: i64 = db
                .conn()
                .query_row("SELECT COUNT(*) FROM ingest_sources", [], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?;

            let namespace_count: i64 = db
                .conn()
                .query_row("SELECT COUNT(*) FROM namespaces", [], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?;

            println!("Knowledge Base Statistics");
            println!("{}", "-".repeat(30));
            println!("Documents:  {}", doc_count);
            println!("Chunks:     {}", chunk_count);
            println!("Sources:    {}", source_count);
            println!("Namespaces: {}", namespace_count);

            // Get vector consent status
            let vector_enabled = db.get_vector_consent().map(|c| c.enabled).unwrap_or(false);
            println!(
                "Vectors:    {}",
                if vector_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );

            Ok(())
        }
        KbCommand::Index { force: _ } => {
            let folder_path: String = db
                .conn()
                .query_row(
                    "SELECT value FROM settings WHERE key = 'kb_folder'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|_| "KB folder not configured. Set it in the app first (Settings > Knowledge Base).")?;

            let path = std::path::Path::new(&folder_path);
            if !path.exists() {
                return Err(format!("KB folder does not exist: {}", folder_path));
            }

            println!("Indexing KB folder: {}", folder_path);

            let indexer = KbIndexer::new();
            let result = indexer
                .index_folder(&db, path, |progress| {
                    use assistsupport_lib::kb::indexer::IndexProgress;
                    match progress {
                        IndexProgress::Started { total_files } => {
                            println!("Found {} files to index", total_files);
                        }
                        IndexProgress::Processing {
                            current,
                            total,
                            file_name,
                        } => {
                            println!("[{}/{}] {}", current, total, file_name);
                        }
                        IndexProgress::Completed {
                            indexed,
                            skipped,
                            errors,
                        } => {
                            println!("\nIndexing complete:");
                            println!("  Indexed: {}", indexed);
                            println!("  Skipped: {}", skipped);
                            println!("  Errors:  {}", errors);
                        }
                        IndexProgress::Error {
                            file_name,
                            message,
                        } => {
                            eprintln!("  Error in {}: {}", file_name, message);
                        }
                    }
                })
                .map_err(|e| format!("Indexing failed: {}", e))?;

            println!(
                "\nDone. {} files indexed, {} skipped, {} errors.",
                result.indexed, result.skipped, result.errors
            );
            Ok(())
        }
        KbCommand::Search { query, limit } => {
            println!("Query: \"{}\"", query);
            println!("{}", "-".repeat(60));

            let results = HybridSearch::search(&db, &query, limit)
                .map_err(|e| format!("Search failed: {}", e))?;

            if results.is_empty() {
                println!("No results found.");
            } else {
                for (i, r) in results.iter().enumerate() {
                    let filename = std::path::Path::new(&r.file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&r.file_path);
                    let heading = r.heading_path.as_deref().unwrap_or("");
                    println!(
                        "  {}. [score: {:.4}] {}",
                        i + 1,
                        r.score,
                        filename
                    );
                    if !heading.is_empty() {
                        println!("     heading: {}", heading);
                    }
                    // Show first 120 chars of content
                    let preview: String = r.content.chars().take(120).collect();
                    let preview = preview.replace('\n', " ");
                    println!("     {}...", preview.trim());
                    println!();
                }
            }
            Ok(())
        }
    }
}
