use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use clap::{Args, Parser, Subcommand, ValueEnum};
use hmac::{Hmac, Mac};
use memory_kernel_core::{
    build_context_package, build_recall_context_package, default_recall_record_types, Authority,
    ConstraintEffect, ConstraintPayload, ConstraintScope, LinkType, MemoryId, MemoryPayload,
    MemoryRecord, MemoryVersionId, QueryRequest, RecordType, TruthStatus,
};
use memory_kernel_outcome_cli::OutcomeCommand as OutcomeCliCommand;
use memory_kernel_store_sqlite::{ExportManifest, SqliteStore};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use ulid::Ulid;

const CLI_CONTRACT_VERSION: &str = "cli.v1";
const MANIFEST_FILE: &str = "manifest.json";
const MANIFEST_SIG_FILE: &str = "manifest.sig";
const MANIFEST_SECURITY_FILE: &str = "manifest.security.json";
const ENCRYPTION_MAGIC: &[u8] = b"MKENC1";
const ENCRYPTION_ALGORITHM: &str = "xchacha20poly1305";
const SIGNATURE_ALGORITHM: &str = "hmac-sha256";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Parser)]
#[command(name = "mk")]
#[command(about = "Memory Kernel CLI")]
struct Cli {
    #[arg(long, default_value = "./memory_kernel.sqlite3")]
    db: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Db {
        #[command(subcommand)]
        command: Box<DbCommand>,
    },
    Memory {
        #[command(subcommand)]
        command: Box<MemoryCommand>,
    },
    Query {
        #[command(subcommand)]
        command: Box<QueryCommand>,
    },
    Context {
        #[command(subcommand)]
        command: Box<ContextCommand>,
    },
    Outcome {
        #[command(subcommand)]
        command: Box<OutcomeCliCommand>,
    },
}

#[derive(Debug, Subcommand)]
enum DbCommand {
    SchemaVersion,
    Migrate(DbMigrateArgs),
    Export(DbExportArgs),
    Import(DbImportArgs),
    Backup(DbBackupArgs),
    Restore(DbRestoreArgs),
    IntegrityCheck,
}

#[derive(Debug, Args)]
struct DbMigrateArgs {
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct DbExportArgs {
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    signing_key_file: Option<PathBuf>,
    #[arg(long)]
    encrypt_key_file: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct DbImportArgs {
    #[arg(long = "in")]
    input: PathBuf,
    #[arg(long, default_value_t = true)]
    skip_existing: bool,
    #[arg(long)]
    verify_key_file: Option<PathBuf>,
    #[arg(long)]
    decrypt_key_file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    allow_unsigned: bool,
}

#[derive(Debug, Args)]
struct DbBackupArgs {
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct DbRestoreArgs {
    #[arg(long = "in")]
    input: PathBuf,
}

#[derive(Debug, Subcommand)]
enum MemoryCommand {
    Add {
        #[command(subcommand)]
        command: Box<AddCommand>,
    },
    Link(LinkArgs),
    List,
}

#[derive(Debug, Subcommand)]
enum AddCommand {
    Constraint(AddConstraintArgs),
    Decision(AddSummaryArgs),
    Preference(AddSummaryArgs),
    Event(AddSummaryArgs),
    Outcome(AddSummaryArgs),
}

#[derive(Debug, Args)]
struct AddConstraintArgs {
    #[arg(long)]
    actor: String,
    #[arg(long)]
    action: String,
    #[arg(long)]
    resource: String,
    #[arg(long)]
    effect: EffectArg,
    #[arg(long)]
    note: Option<String>,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Debug, Args)]
struct AddSummaryArgs {
    #[arg(long)]
    summary: String,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Debug, Args)]
struct WriteArgs {
    #[arg(long)]
    memory_id: Option<String>,
    #[arg(long, default_value_t = 1)]
    version: u32,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
    #[arg(long)]
    source_uri: String,
    #[arg(long)]
    source_hash: Option<String>,
    #[arg(long = "evidence")]
    evidence: Vec<String>,
    #[arg(long)]
    confidence: Option<f32>,
    #[arg(long)]
    truth_status: TruthStatusArg,
    #[arg(long)]
    authority: AuthorityArg,
    #[arg(long)]
    created_at: Option<String>,
    #[arg(long)]
    effective_at: Option<String>,
    #[arg(long = "supersedes")]
    supersedes: Vec<String>,
    #[arg(long = "contradicts")]
    contradicts: Vec<String>,
}

#[derive(Debug, Args)]
struct LinkArgs {
    #[arg(long)]
    from: String,
    #[arg(long)]
    to: String,
    #[arg(long)]
    relation: RelationArg,
    #[arg(long)]
    writer: String,
    #[arg(long)]
    justification: String,
}

#[derive(Debug, Subcommand)]
enum QueryCommand {
    Ask(QueryAskArgs),
    Recall(QueryRecallArgs),
}

#[derive(Debug, Args)]
struct QueryAskArgs {
    #[arg(long)]
    text: String,
    #[arg(long)]
    actor: String,
    #[arg(long)]
    action: String,
    #[arg(long)]
    resource: String,
    #[arg(long)]
    as_of: Option<String>,
}

#[derive(Debug, Args)]
struct QueryRecallArgs {
    #[arg(long)]
    text: String,
    #[arg(long = "record-type", value_enum)]
    record_types: Vec<RecordTypeArg>,
    #[arg(long)]
    as_of: Option<String>,
}

#[derive(Debug, Subcommand)]
enum ContextCommand {
    Show(ContextShowArgs),
}

#[derive(Debug, Args)]
struct ContextShowArgs {
    #[arg(long)]
    context_package_id: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TruthStatusArg {
    Asserted,
    Observed,
    Inferred,
    Speculative,
    Retracted,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AuthorityArg {
    Authoritative,
    Derived,
    Note,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum EffectArg {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RelationArg {
    Supersedes,
    Contradicts,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RecordTypeArg {
    Constraint,
    Decision,
    Preference,
    Event,
    Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SnapshotSecurityMetadata {
    encrypted_files: Vec<String>,
    encryption_algorithm: Option<String>,
    signature_file: Option<String>,
    signature_algorithm: Option<String>,
}

fn with_contract_version(value: Value) -> Value {
    match value {
        Value::Object(mut object) => {
            object.insert(
                "contract_version".to_string(),
                Value::String(CLI_CONTRACT_VERSION.to_string()),
            );
            Value::Object(object)
        }
        other => serde_json::json!({
            "contract_version": CLI_CONTRACT_VERSION,
            "payload": other
        }),
    }
}

fn emit_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&with_contract_version(value))?);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Db { command } => {
            let mut store = SqliteStore::open(&cli.db)?;
            run_db(*command, &mut store)
        }
        Command::Memory { command } => {
            let mut store = SqliteStore::open(&cli.db)?;
            run_memory(*command, &mut store)
        }
        Command::Query { command } => {
            let mut store = SqliteStore::open(&cli.db)?;
            run_query(*command, &mut store)
        }
        Command::Context { command } => {
            let mut store = SqliteStore::open(&cli.db)?;
            run_context(*command, &mut store)
        }
        Command::Outcome { command } => {
            memory_kernel_outcome_cli::run_outcome_with_db(&cli.db, *command)
        }
    }
}

fn run_db(command: DbCommand, store: &mut SqliteStore) -> Result<()> {
    match command {
        DbCommand::SchemaVersion => run_db_schema_version(store),
        DbCommand::Migrate(args) => run_db_migrate(&args, store),
        DbCommand::Export(args) => run_db_export(&args, store),
        DbCommand::Import(args) => run_db_import(&args, store),
        DbCommand::Backup(args) => run_db_backup(&args, store),
        DbCommand::Restore(args) => run_db_restore(&args, store),
        DbCommand::IntegrityCheck => run_db_integrity_check(store),
    }
}

fn run_db_schema_version(store: &SqliteStore) -> Result<()> {
    let status = store.schema_status()?;
    emit_json(serde_json::json!({
        "current_version": status.current_version,
        "target_version": status.target_version,
        "pending_versions": status.pending_versions,
        "up_to_date": status.pending_versions.is_empty(),
        "inferred_from_legacy": status.inferred_from_legacy
    }))
}

fn run_db_migrate(args: &DbMigrateArgs, store: &mut SqliteStore) -> Result<()> {
    let before = store.schema_status()?;
    if args.dry_run {
        emit_json(serde_json::json!({
            "dry_run": true,
            "current_version": before.current_version,
            "target_version": before.target_version,
            "would_apply_versions": before.pending_versions,
            "inferred_from_legacy": before.inferred_from_legacy
        }))?;
        return Ok(());
    }

    store.migrate()?;
    let after = store.schema_status()?;
    emit_json(serde_json::json!({
        "dry_run": false,
        "before_version": before.current_version,
        "applied_versions": before.pending_versions,
        "after_version": after.current_version,
        "target_version": after.target_version,
        "up_to_date": after.pending_versions.is_empty()
    }))
}

fn run_db_export(args: &DbExportArgs, store: &mut SqliteStore) -> Result<()> {
    store.migrate()?;
    let mut manifest = store.export_snapshot(&args.out)?;
    let mut security = SnapshotSecurityMetadata::default();

    if let Some(key_path) = args.encrypt_key_file.as_ref() {
        let encryption_key = read_hex_key_file(key_path)?;
        encrypt_snapshot_files(&args.out, &mut manifest, &encryption_key)?;
        security.encrypted_files = manifest.files.iter().map(|file| file.path.clone()).collect();
        security.encryption_algorithm = Some(ENCRYPTION_ALGORITHM.to_string());
    }

    write_manifest(&args.out, &manifest)?;

    if let Some(key_path) = args.signing_key_file.as_ref() {
        let signing_key = read_hex_key_file(key_path)?;
        let manifest_path = args.out.join(MANIFEST_FILE);
        let manifest_bytes = fs::read(&manifest_path)
            .with_context(|| format!("failed to read manifest file {}", manifest_path.display()))?;
        write_manifest_signature(&args.out, &manifest_bytes, &signing_key)?;
        security.signature_file = Some(MANIFEST_SIG_FILE.to_string());
        security.signature_algorithm = Some(SIGNATURE_ALGORITHM.to_string());
    } else {
        remove_if_exists(&args.out.join(MANIFEST_SIG_FILE))?;
    }

    if security.encryption_algorithm.is_some() || security.signature_algorithm.is_some() {
        write_security_metadata(&args.out, &security)?;
    } else {
        remove_if_exists(&args.out.join(MANIFEST_SECURITY_FILE))?;
    }

    emit_json(serde_json::json!({
        "out_dir": args.out,
        "manifest": manifest
    }))
}

fn run_db_import(args: &DbImportArgs, store: &mut SqliteStore) -> Result<()> {
    let verify_key =
        args.verify_key_file.as_ref().map(|path| read_hex_key_file(path)).transpose()?;
    let decrypt_key =
        args.decrypt_key_file.as_ref().map(|path| read_hex_key_file(path)).transpose()?;

    let prepared = prepare_import_input(
        &args.input,
        verify_key.as_ref(),
        decrypt_key.as_ref(),
        args.allow_unsigned,
    )?;
    let summary = store.import_snapshot(&prepared, args.skip_existing)?;
    if prepared != args.input {
        fs::remove_dir_all(&prepared).with_context(|| {
            format!("failed to cleanup temporary import directory {}", prepared.display())
        })?;
    }
    emit_json(serde_json::json!({
        "in_dir": args.input,
        "skip_existing": args.skip_existing,
        "summary": summary
    }))
}

fn run_db_backup(args: &DbBackupArgs, store: &mut SqliteStore) -> Result<()> {
    store.migrate()?;
    store.backup_database(&args.out)?;
    emit_json(serde_json::json!({
        "backup_path": args.out,
        "status": "ok"
    }))
}

fn run_db_restore(args: &DbRestoreArgs, store: &mut SqliteStore) -> Result<()> {
    store.restore_database(&args.input)?;
    let status = store.schema_status()?;
    emit_json(serde_json::json!({
        "restored_from": args.input,
        "current_version": status.current_version,
        "target_version": status.target_version,
        "pending_versions": status.pending_versions
    }))
}

fn run_db_integrity_check(store: &SqliteStore) -> Result<()> {
    let report = store.integrity_check()?;
    emit_json(serde_json::to_value(&report).context("failed to serialize integrity report")?)
}

fn run_memory(command: MemoryCommand, store: &mut SqliteStore) -> Result<()> {
    store.migrate()?;
    match command {
        MemoryCommand::Add { command } => {
            let record = match *command {
                AddCommand::Constraint(args) => build_record(
                    MemoryPayload::Constraint(ConstraintPayload {
                        scope: ConstraintScope {
                            actor: args.actor,
                            action: args.action,
                            resource: args.resource,
                        },
                        effect: match args.effect {
                            EffectArg::Allow => ConstraintEffect::Allow,
                            EffectArg::Deny => ConstraintEffect::Deny,
                        },
                        note: args.note,
                    }),
                    args.write,
                )?,
                AddCommand::Decision(args) => build_record(
                    MemoryPayload::Decision(memory_kernel_core::DecisionPayload {
                        summary: args.summary,
                    }),
                    args.write,
                )?,
                AddCommand::Preference(args) => build_record(
                    MemoryPayload::Preference(memory_kernel_core::PreferencePayload {
                        summary: args.summary,
                    }),
                    args.write,
                )?,
                AddCommand::Event(args) => build_record(
                    MemoryPayload::Event(memory_kernel_core::EventPayload {
                        summary: args.summary,
                    }),
                    args.write,
                )?,
                AddCommand::Outcome(args) => build_record(
                    MemoryPayload::Outcome(memory_kernel_core::OutcomePayload {
                        summary: args.summary,
                    }),
                    args.write,
                )?,
            };

            store.write_record(&record)?;
            emit_json(serde_json::to_value(&record).context("failed to serialize memory record")?)
        }
        MemoryCommand::Link(args) => {
            let from = parse_memory_version_id(&args.from)?;
            let to = parse_memory_version_id(&args.to)?;
            let relation = match args.relation {
                RelationArg::Supersedes => LinkType::Supersedes,
                RelationArg::Contradicts => LinkType::Contradicts,
            };

            store.add_link(from, to, relation, &args.writer, &args.justification)?;
            emit_json(serde_json::json!({
                "from_memory_version_id": from.to_string(),
                "to_memory_version_id": to.to_string(),
                "relation": relation.as_str(),
                "writer": args.writer,
                "justification": args.justification,
            }))
        }
        MemoryCommand::List => {
            let records = store.list_records()?;
            emit_json(serde_json::json!({ "records": records }))
        }
    }
}

fn run_query(command: QueryCommand, store: &mut SqliteStore) -> Result<()> {
    store.migrate()?;
    match command {
        QueryCommand::Ask(args) => {
            let as_of = parse_optional_rfc3339(args.as_of.as_deref())?;
            let records = store.list_records()?;
            let snapshot_id = compute_snapshot_id(
                &records,
                as_of,
                &args.text,
                &[
                    "query_mode=policy".to_string(),
                    format!("actor={}", args.actor),
                    format!("action={}", args.action),
                    format!("resource={}", args.resource),
                ],
            );

            let package = build_context_package(
                &records,
                QueryRequest {
                    text: args.text,
                    actor: args.actor,
                    action: args.action,
                    resource: args.resource,
                    as_of,
                },
                &snapshot_id,
            )?;

            store.save_context_package(&package)?;
            emit_json(
                serde_json::to_value(&package).context("failed to serialize context package")?,
            )
        }
        QueryCommand::Recall(args) => {
            let as_of = parse_optional_rfc3339(args.as_of.as_deref())?;
            let records = store.list_records()?;
            let selected_record_types = if args.record_types.is_empty() {
                default_recall_record_types()
            } else {
                args.record_types.iter().copied().map(RecordTypeArg::into_record_type).collect()
            };

            let mut type_names = selected_record_types
                .iter()
                .map(|record_type| record_type.as_str())
                .collect::<Vec<_>>();
            type_names.sort_unstable();

            let snapshot_id = compute_snapshot_id(
                &records,
                as_of,
                &args.text,
                &[
                    "query_mode=recall".to_string(),
                    format!("record_types={}", type_names.join(",")),
                ],
            );

            let package = build_recall_context_package(
                &records,
                QueryRequest {
                    text: args.text,
                    actor: "*".to_string(),
                    action: "*".to_string(),
                    resource: "*".to_string(),
                    as_of,
                },
                &snapshot_id,
                &selected_record_types,
            )?;

            store.save_context_package(&package)?;
            emit_json(
                serde_json::to_value(&package).context("failed to serialize context package")?,
            )
        }
    }
}

fn run_context(command: ContextCommand, store: &mut SqliteStore) -> Result<()> {
    store.migrate()?;
    match command {
        ContextCommand::Show(args) => {
            let Some(package) = store.get_context_package(&args.context_package_id)? else {
                return Err(anyhow!("context package not found: {}", args.context_package_id));
            };

            emit_json(
                serde_json::to_value(&package).context("failed to serialize context package")?,
            )
        }
    }
}

fn build_record(payload: MemoryPayload, write: WriteArgs) -> Result<MemoryRecord> {
    let created_at = parse_optional_rfc3339(write.created_at.as_deref())?;
    let effective_at = match write.effective_at {
        Some(value) => parse_rfc3339(&value)?,
        None => created_at,
    };

    let supersedes = write
        .supersedes
        .iter()
        .map(|raw| parse_memory_version_id(raw))
        .collect::<Result<Vec<_>>>()?;
    let contradicts = write
        .contradicts
        .iter()
        .map(|raw| parse_memory_version_id(raw))
        .collect::<Result<Vec<_>>>()?;
    let memory_id = match write.memory_id.as_deref() {
        Some(raw) => parse_memory_id(raw)?,
        None => MemoryId::new(),
    };

    Ok(MemoryRecord {
        memory_version_id: MemoryVersionId::new(),
        memory_id,
        version: write.version,
        created_at,
        effective_at,
        truth_status: match write.truth_status {
            TruthStatusArg::Asserted => TruthStatus::Asserted,
            TruthStatusArg::Observed => TruthStatus::Observed,
            TruthStatusArg::Inferred => TruthStatus::Inferred,
            TruthStatusArg::Speculative => TruthStatus::Speculative,
            TruthStatusArg::Retracted => TruthStatus::Retracted,
        },
        authority: match write.authority {
            AuthorityArg::Authoritative => Authority::Authoritative,
            AuthorityArg::Derived => Authority::Derived,
            AuthorityArg::Note => Authority::Note,
        },
        confidence: write.confidence,
        writer: write.writer,
        justification: write.justification,
        provenance: memory_kernel_core::Provenance {
            source_uri: write.source_uri,
            source_hash: write.source_hash,
            evidence: write.evidence,
        },
        supersedes,
        contradicts,
        payload,
    })
}

fn compute_snapshot_id(
    records: &[MemoryRecord],
    as_of: OffsetDateTime,
    text: &str,
    scope_parts: &[String],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(as_of.unix_timestamp().to_string().as_bytes());
    for part in scope_parts {
        hasher.update(part.as_bytes());
    }

    let mut sorted_ids = records
        .iter()
        .map(|record| format!("{}:{}", record.memory_id, record.memory_version_id))
        .collect::<Vec<_>>();
    sorted_ids.sort();

    for memory_id in sorted_ids {
        hasher.update(memory_id.as_bytes());
    }

    let digest = hasher.finalize();
    let digest_hex = format!("{digest:x}");
    format!("txn_{}", &digest_hex[..16])
}

fn read_hex_key_file(path: &Path) -> Result<[u8; 32]> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("failed to read key file {}", path.display()))?;
    let trimmed = body.trim();
    let bytes = hex::decode(trimmed)
        .with_context(|| format!("key file must contain hex bytes: {}", path.display()))?;
    if bytes.len() != 32 {
        return Err(anyhow!(
            "key file {} must decode to exactly 32 bytes (got {})",
            path.display(),
            bytes.len()
        ));
    }

    let mut key = [0_u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn encrypt_payload_bytes(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0_u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce_bytes), plaintext)
        .map_err(|err| anyhow!("failed to encrypt payload bytes: {err}"))?;

    let mut out = Vec::with_capacity(ENCRYPTION_MAGIC.len() + nonce_bytes.len() + ciphertext.len());
    out.extend_from_slice(ENCRYPTION_MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_payload_bytes(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() <= ENCRYPTION_MAGIC.len() + 24 {
        return Err(anyhow!("encrypted payload is too short"));
    }
    if !encrypted.starts_with(ENCRYPTION_MAGIC) {
        return Err(anyhow!("encrypted payload is missing expected header"));
    }

    let nonce_start = ENCRYPTION_MAGIC.len();
    let nonce_end = nonce_start + 24;
    let nonce = XNonce::from_slice(&encrypted[nonce_start..nonce_end]);
    let ciphertext = &encrypted[nonce_end..];
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| anyhow!("failed to decrypt payload bytes: {err}"))
}

fn write_manifest(out_dir: &Path, manifest: &ExportManifest) -> Result<()> {
    let manifest_path = out_dir.join(MANIFEST_FILE);
    let body = serde_json::to_vec_pretty(manifest)
        .context("failed to serialize updated export manifest")?;
    fs::write(&manifest_path, body)
        .with_context(|| format!("failed to write manifest file {}", manifest_path.display()))
}

fn write_manifest_signature(out_dir: &Path, manifest_bytes: &[u8], key: &[u8; 32]) -> Result<()> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|err| anyhow!("failed to initialize signature key: {err}"))?;
    mac.update(manifest_bytes);
    let signature_hex = hex::encode(mac.finalize().into_bytes());
    let signature_path = out_dir.join(MANIFEST_SIG_FILE);
    fs::write(&signature_path, signature_hex)
        .with_context(|| format!("failed to write manifest signature {}", signature_path.display()))
}

fn verify_manifest_signature(in_dir: &Path, manifest_bytes: &[u8], key: &[u8; 32]) -> Result<()> {
    let signature_path = in_dir.join(MANIFEST_SIG_FILE);
    let signature_body = fs::read_to_string(&signature_path).with_context(|| {
        format!("failed to read manifest signature file {}", signature_path.display())
    })?;
    let signature = hex::decode(signature_body.trim()).with_context(|| {
        format!("manifest signature file is not valid hex: {}", signature_path.display())
    })?;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|err| anyhow!("failed to initialize signature verification key: {err}"))?;
    mac.update(manifest_bytes);
    mac.verify_slice(&signature).map_err(|_| {
        anyhow!("manifest signature verification failed for {}", signature_path.display())
    })
}

fn write_security_metadata(out_dir: &Path, metadata: &SnapshotSecurityMetadata) -> Result<()> {
    let path = out_dir.join(MANIFEST_SECURITY_FILE);
    let body =
        serde_json::to_vec_pretty(metadata).context("failed to serialize security metadata")?;
    fs::write(&path, body)
        .with_context(|| format!("failed to write security metadata {}", path.display()))
}

fn read_security_metadata(in_dir: &Path) -> Result<Option<SnapshotSecurityMetadata>> {
    let path = in_dir.join(MANIFEST_SECURITY_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read security metadata {}", path.display()))?;
    let metadata: SnapshotSecurityMetadata = serde_json::from_str(&body)
        .with_context(|| format!("failed to parse security metadata {}", path.display()))?;
    Ok(Some(metadata))
}

fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove file {}", path.display()))?;
    }
    Ok(())
}

fn encrypt_snapshot_files(
    out_dir: &Path,
    manifest: &mut ExportManifest,
    key: &[u8; 32],
) -> Result<()> {
    for file in &mut manifest.files {
        let path = out_dir.join(&file.path);
        let plaintext = fs::read(&path)
            .with_context(|| format!("failed to read export file {}", path.display()))?;
        let encrypted = encrypt_payload_bytes(key, &plaintext)?;
        fs::write(&path, &encrypted)
            .with_context(|| format!("failed to write encrypted export file {}", path.display()))?;
        file.sha256 = sha256_hex(&encrypted);
    }
    Ok(())
}

fn count_ndjson_records_bytes(bytes: &[u8]) -> usize {
    let body = String::from_utf8_lossy(bytes);
    body.lines().filter(|line| !line.trim().is_empty()).count()
}

fn prepare_import_input(
    input_dir: &Path,
    verify_key: Option<&[u8; 32]>,
    decrypt_key: Option<&[u8; 32]>,
    allow_unsigned: bool,
) -> Result<PathBuf> {
    let manifest_path = input_dir.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;

    let signature_path = input_dir.join(MANIFEST_SIG_FILE);
    if signature_path.exists() {
        let key = verify_key.ok_or_else(|| {
            anyhow!(
                "snapshot is signed; provide --verify-key-file to verify {}",
                signature_path.display()
            )
        })?;
        verify_manifest_signature(input_dir, &manifest_bytes, key)?;
    } else if !allow_unsigned {
        return Err(anyhow!(
            "snapshot is unsigned; rerun with --allow-unsigned for explicit override"
        ));
    }

    let Some(security) = read_security_metadata(input_dir)? else {
        return Ok(input_dir.to_path_buf());
    };
    if security.encrypted_files.is_empty() {
        return Ok(input_dir.to_path_buf());
    }

    let key = decrypt_key.ok_or_else(|| {
        anyhow!(
            "snapshot files are encrypted; provide --decrypt-key-file to import {}",
            input_dir.display()
        )
    })?;
    if security.encryption_algorithm.as_deref() != Some(ENCRYPTION_ALGORITHM) {
        return Err(anyhow!(
            "unsupported encryption algorithm in security metadata for {}",
            input_dir.display()
        ));
    }

    let mut manifest: ExportManifest = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("failed to parse manifest {}", manifest_path.display()))?;
    let tmp_dir =
        std::env::temp_dir().join(format!("memorykernel-import-decrypted-{}", Ulid::new()));
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create temporary import dir {}", tmp_dir.display()))?;

    for file in &mut manifest.files {
        let encrypted_path = input_dir.join(&file.path);
        let encrypted_bytes = fs::read(&encrypted_path).with_context(|| {
            format!("failed to read encrypted snapshot file {}", encrypted_path.display())
        })?;
        let decrypted_bytes = decrypt_payload_bytes(key, &encrypted_bytes)?;
        let output_path = tmp_dir.join(&file.path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create parent directory {}", parent.display())
            })?;
        }
        fs::write(&output_path, &decrypted_bytes).with_context(|| {
            format!("failed to write decrypted snapshot file {}", output_path.display())
        })?;
        file.sha256 = sha256_hex(&decrypted_bytes);
        file.records = count_ndjson_records_bytes(&decrypted_bytes);
    }

    write_manifest(&tmp_dir, &manifest)?;
    Ok(tmp_dir)
}

fn parse_optional_rfc3339(value: Option<&str>) -> Result<OffsetDateTime> {
    match value {
        Some(raw) => parse_rfc3339(raw),
        None => Ok(OffsetDateTime::now_utc()),
    }
}

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime> {
    let parsed = OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .with_context(|| format!("invalid RFC3339 UTC timestamp: {value}"))?;

    if parsed.offset() != time::UtcOffset::UTC {
        return Err(anyhow!("timestamp MUST use UTC offset Z (received: {value})"));
    }

    Ok(parsed)
}

fn parse_memory_id(value: &str) -> Result<MemoryId> {
    let parsed = Ulid::from_string(value).with_context(|| format!("invalid ULID: {value}"))?;
    Ok(MemoryId(parsed))
}

fn parse_memory_version_id(value: &str) -> Result<MemoryVersionId> {
    let parsed = Ulid::from_string(value).with_context(|| format!("invalid ULID: {value}"))?;
    Ok(MemoryVersionId(parsed))
}

impl RecordTypeArg {
    fn into_record_type(self) -> RecordType {
        match self {
            Self::Constraint => RecordType::Constraint,
            Self::Decision => RecordType::Decision,
            Self::Preference => RecordType::Preference,
            Self::Event => RecordType::Event,
            Self::Outcome => RecordType::Outcome,
        }
    }
}
