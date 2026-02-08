use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ulid::Ulid;

#[derive(Debug, Clone, thiserror::Error, Eq, PartialEq)]
pub enum KernelError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("query error: {0}")]
    Query(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MemoryId(pub Ulid);

impl MemoryId {
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MemoryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MemoryVersionId(pub Ulid);

impl MemoryVersionId {
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for MemoryVersionId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MemoryVersionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Constraint,
    Decision,
    Preference,
    Event,
    Outcome,
}

impl RecordType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Constraint => "constraint",
            Self::Decision => "decision",
            Self::Preference => "preference",
            Self::Event => "event",
            Self::Outcome => "outcome",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "constraint" => Some(Self::Constraint),
            "decision" => Some(Self::Decision),
            "preference" => Some(Self::Preference),
            "event" => Some(Self::Event),
            "outcome" => Some(Self::Outcome),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Supersedes,
    Contradicts,
}

impl LinkType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contradicts => "contradicts",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TruthStatus {
    Asserted,
    Observed,
    Inferred,
    Speculative,
    Retracted,
}

impl TruthStatus {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Observed => 5,
            Self::Asserted => 4,
            Self::Inferred => 3,
            Self::Speculative => 2,
            Self::Retracted => 1,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Observed => "observed",
            Self::Inferred => "inferred",
            Self::Speculative => "speculative",
            Self::Retracted => "retracted",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "asserted" => Some(Self::Asserted),
            "observed" => Some(Self::Observed),
            "inferred" => Some(Self::Inferred),
            "speculative" => Some(Self::Speculative),
            "retracted" => Some(Self::Retracted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    Authoritative,
    Derived,
    Note,
}

impl Authority {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Authoritative => 3,
            Self::Derived => 2,
            Self::Note => 1,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authoritative => "authoritative",
            Self::Derived => "derived",
            Self::Note => "note",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "authoritative" => Some(Self::Authoritative),
            "derived" => Some(Self::Derived),
            "note" => Some(Self::Note),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintEffect {
    Allow,
    Deny,
}

impl ConstraintEffect {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Provenance {
    pub source_uri: String,
    pub source_hash: Option<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ConstraintScope {
    pub actor: String,
    pub action: String,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ConstraintPayload {
    pub scope: ConstraintScope,
    pub effect: ConstraintEffect,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DecisionPayload {
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PreferencePayload {
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EventPayload {
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OutcomePayload {
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "record_type", content = "payload", rename_all = "snake_case")]
pub enum MemoryPayload {
    Constraint(ConstraintPayload),
    Decision(DecisionPayload),
    Preference(PreferencePayload),
    Event(EventPayload),
    Outcome(OutcomePayload),
}

impl MemoryPayload {
    #[must_use]
    pub fn record_type(&self) -> RecordType {
        match self {
            Self::Constraint(_) => RecordType::Constraint,
            Self::Decision(_) => RecordType::Decision,
            Self::Preference(_) => RecordType::Preference,
            Self::Event(_) => RecordType::Event,
            Self::Outcome(_) => RecordType::Outcome,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    pub memory_version_id: MemoryVersionId,
    pub memory_id: MemoryId,
    pub version: u32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub effective_at: OffsetDateTime,
    pub truth_status: TruthStatus,
    pub authority: Authority,
    pub confidence: Option<f32>,
    pub writer: String,
    pub justification: String,
    pub provenance: Provenance,
    #[serde(default)]
    pub supersedes: Vec<MemoryVersionId>,
    #[serde(default)]
    pub contradicts: Vec<MemoryVersionId>,
    pub payload: MemoryPayload,
}

impl MemoryRecord {
    /// Validate one append-only memory version against Foundation invariants.
    ///
    /// # Errors
    /// Returns [`KernelError::Validation`] when required identity, accountability,
    /// provenance, confidence, or payload constraints are violated.
    pub fn validate(&self) -> Result<(), KernelError> {
        if self.version == 0 {
            return Err(KernelError::Validation(
                "version MUST be >= 1 for append-only lineage".to_string(),
            ));
        }

        if self.writer.trim().is_empty() {
            return Err(KernelError::Validation(
                "writer MUST be provided for every write".to_string(),
            ));
        }

        if self.justification.trim().is_empty() {
            return Err(KernelError::Validation(
                "justification MUST be provided for every write".to_string(),
            ));
        }

        if self.provenance.source_uri.trim().is_empty() {
            return Err(KernelError::Validation("source_uri MUST be provided".to_string()));
        }

        if let Some(source_hash) = &self.provenance.source_hash {
            if !source_hash.starts_with("sha256:") || source_hash.len() <= 7 {
                return Err(KernelError::Validation(
                    "source_hash MUST be formatted as sha256:<hex>".to_string(),
                ));
            }
        }

        if let Some(confidence) = self.confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(KernelError::Validation(
                    "confidence MUST be in [0.0, 1.0]".to_string(),
                ));
            }
        }

        if matches!(self.truth_status, TruthStatus::Inferred | TruthStatus::Speculative)
            && self.confidence.is_none()
        {
            return Err(KernelError::Validation(
                "confidence MUST be provided for inferred/speculative records".to_string(),
            ));
        }

        if self.payload.record_type() == RecordType::Constraint {
            let MemoryPayload::Constraint(constraint) = &self.payload else {
                return Err(KernelError::Validation("constraint payload mismatch".to_string()));
            };
            for field in
                [&constraint.scope.actor, &constraint.scope.action, &constraint.scope.resource]
            {
                if field.trim().is_empty() {
                    return Err(KernelError::Validation(
                        "constraint scope fields MUST be non-empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct QueryRequest {
    pub text: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    #[serde(with = "time::serde::rfc3339")]
    pub as_of: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerResult {
    Allow,
    Deny,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleScores {
    pub scope_match: f32,
    pub authority_rank: u8,
    pub truth_status_rank: u8,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Why {
    pub included: bool,
    pub reasons: Vec<String>,
    pub rule_scores: Option<RuleScores>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextItem {
    pub rank: usize,
    pub memory_version_id: MemoryVersionId,
    pub memory_id: MemoryId,
    pub record_type: RecordType,
    pub version: u32,
    pub truth_status: TruthStatus,
    pub confidence: Option<f32>,
    pub authority: Authority,
    pub why: Why,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeterminismMetadata {
    pub ruleset_version: String,
    pub snapshot_id: String,
    pub tie_breakers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Answer {
    pub result: AnswerResult,
    pub why: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextPackage {
    pub context_package_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
    pub query: QueryRequest,
    pub determinism: DeterminismMetadata,
    pub answer: Answer,
    pub selected_items: Vec<ContextItem>,
    pub excluded_items: Vec<ContextItem>,
    pub ordering_trace: Vec<String>,
}

#[derive(Debug, Clone)]
struct PolicyCandidate<'a> {
    record: &'a MemoryRecord,
    scope_score: u8,
    confidence: f32,
}

impl PolicyCandidate<'_> {
    fn cmp(lhs: &Self, rhs: &Self) -> Ordering {
        rhs.scope_score
            .cmp(&lhs.scope_score)
            .then_with(|| rhs.record.authority.rank().cmp(&lhs.record.authority.rank()))
            .then_with(|| rhs.record.truth_status.rank().cmp(&lhs.record.truth_status.rank()))
            .then_with(|| rhs.confidence.partial_cmp(&lhs.confidence).unwrap_or(Ordering::Equal))
            .then_with(|| rhs.record.effective_at.cmp(&lhs.record.effective_at))
            .then_with(|| rhs.record.created_at.cmp(&lhs.record.created_at))
            .then_with(|| lhs.record.memory_id.cmp(&rhs.record.memory_id))
            .then_with(|| lhs.record.memory_version_id.cmp(&rhs.record.memory_version_id))
    }
}

#[derive(Debug, Clone)]
struct RecallCandidate<'a> {
    record: &'a MemoryRecord,
    matched_terms: usize,
    total_terms: usize,
    lexical_score: f32,
    confidence: f32,
}

impl RecallCandidate<'_> {
    fn cmp(lhs: &Self, rhs: &Self) -> Ordering {
        rhs.matched_terms
            .cmp(&lhs.matched_terms)
            .then_with(|| rhs.record.authority.rank().cmp(&lhs.record.authority.rank()))
            .then_with(|| rhs.record.truth_status.rank().cmp(&lhs.record.truth_status.rank()))
            .then_with(|| rhs.confidence.partial_cmp(&lhs.confidence).unwrap_or(Ordering::Equal))
            .then_with(|| rhs.record.effective_at.cmp(&lhs.record.effective_at))
            .then_with(|| rhs.record.created_at.cmp(&lhs.record.created_at))
            .then_with(|| lhs.record.memory_id.cmp(&rhs.record.memory_id))
            .then_with(|| lhs.record.memory_version_id.cmp(&rhs.record.memory_version_id))
    }
}

#[must_use]
pub fn default_tie_breakers() -> Vec<String> {
    vec![
        "scope_specificity desc".to_string(),
        "authority_rank desc".to_string(),
        "truth_status_rank desc".to_string(),
        "confidence desc".to_string(),
        "effective_at desc".to_string(),
        "created_at desc".to_string(),
        "memory_id asc".to_string(),
        "memory_version_id asc".to_string(),
    ]
}

#[must_use]
pub fn default_recall_tie_breakers() -> Vec<String> {
    vec![
        "lexical_match_count desc".to_string(),
        "authority_rank desc".to_string(),
        "truth_status_rank desc".to_string(),
        "confidence desc".to_string(),
        "effective_at desc".to_string(),
        "created_at desc".to_string(),
        "memory_id asc".to_string(),
        "memory_version_id asc".to_string(),
    ]
}

#[must_use]
pub fn default_recall_record_types() -> Vec<RecordType> {
    vec![RecordType::Decision, RecordType::Preference, RecordType::Event, RecordType::Outcome]
}

fn scope_specificity(scope: &ConstraintScope, query: &QueryRequest) -> Option<u8> {
    let fields = [
        (&scope.actor, &query.actor),
        (&scope.action, &query.action),
        (&scope.resource, &query.resource),
    ];

    let mut specificity_score = 0_u8;
    for (field, query_value) in fields {
        if field == query_value {
            specificity_score += 1;
            continue;
        }

        if field == "*" {
            continue;
        }

        return None;
    }

    Some(specificity_score)
}

fn collect_superseded_ids(records: &[MemoryRecord]) -> std::collections::BTreeSet<MemoryVersionId> {
    let mut superseded_ids = std::collections::BTreeSet::new();
    for record in records {
        for superseded in &record.supersedes {
            superseded_ids.insert(*superseded);
        }
    }
    superseded_ids
}

fn excluded_item(record: &MemoryRecord, reason: &str) -> ContextItem {
    ContextItem {
        rank: 0,
        memory_version_id: record.memory_version_id,
        memory_id: record.memory_id,
        record_type: record.payload.record_type(),
        version: record.version,
        truth_status: record.truth_status,
        confidence: record.confidence,
        authority: record.authority,
        why: Why { included: false, reasons: vec![reason.to_string()], rule_scores: None },
    }
}

fn collect_policy_candidates_and_exclusions<'a>(
    records: &'a [MemoryRecord],
    query: &QueryRequest,
    superseded_ids: &std::collections::BTreeSet<MemoryVersionId>,
) -> (Vec<PolicyCandidate<'a>>, Vec<ContextItem>) {
    let mut candidates: Vec<PolicyCandidate<'a>> = Vec::new();
    let mut excluded: Vec<ContextItem> = Vec::new();

    for record in records {
        let MemoryPayload::Constraint(constraint) = &record.payload else {
            continue;
        };

        let Some(scope_score) = scope_specificity(&constraint.scope, query) else {
            continue;
        };

        if record.truth_status == TruthStatus::Retracted {
            excluded.push(excluded_item(record, "truth_status is retracted"));
            continue;
        }

        if superseded_ids.contains(&record.memory_version_id) {
            excluded.push(excluded_item(record, "record is superseded by a newer linked record"));
            continue;
        }

        candidates.push(PolicyCandidate {
            record,
            scope_score,
            confidence: record.confidence.unwrap_or(0.5),
        });
    }

    (candidates, excluded)
}

fn selected_policy_item(index: usize, candidate: &PolicyCandidate<'_>) -> ContextItem {
    ContextItem {
        rank: index + 1,
        memory_version_id: candidate.record.memory_version_id,
        memory_id: candidate.record.memory_id,
        record_type: candidate.record.payload.record_type(),
        version: candidate.record.version,
        truth_status: candidate.record.truth_status,
        confidence: candidate.record.confidence,
        authority: candidate.record.authority,
        why: Why {
            included: true,
            reasons: vec![
                format!(
                    "scope specificity score={} for actor/action/resource",
                    candidate.scope_score
                ),
                "passed active filters (not retracted, not superseded)".to_string(),
            ],
            rule_scores: Some(RuleScores {
                scope_match: f32::from(candidate.scope_score) / 3.0,
                authority_rank: candidate.record.authority.rank(),
                truth_status_rank: candidate.record.truth_status.rank(),
                confidence: candidate.confidence,
            }),
        },
    }
}

fn tokenize_query_terms(value: &str) -> Vec<String> {
    use std::collections::BTreeSet;

    let mut terms = BTreeSet::new();
    for raw in value.split_whitespace() {
        let normalized = raw
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
            .collect::<String>()
            .to_ascii_lowercase();
        if normalized.len() >= 2 {
            terms.insert(normalized);
        }
    }
    terms.into_iter().collect()
}

fn record_terms(record: &MemoryRecord) -> std::collections::BTreeSet<String> {
    use std::collections::BTreeSet;

    let mut terms = BTreeSet::new();
    match &record.payload {
        MemoryPayload::Constraint(payload) => {
            for input in [
                payload.scope.actor.as_str(),
                payload.scope.action.as_str(),
                payload.scope.resource.as_str(),
                payload.note.as_deref().unwrap_or(""),
            ] {
                for term in tokenize_query_terms(input) {
                    terms.insert(term);
                }
            }
        }
        MemoryPayload::Decision(payload) => {
            for term in tokenize_query_terms(&payload.summary) {
                terms.insert(term);
            }
        }
        MemoryPayload::Preference(payload) => {
            for term in tokenize_query_terms(&payload.summary) {
                terms.insert(term);
            }
        }
        MemoryPayload::Event(payload) => {
            for term in tokenize_query_terms(&payload.summary) {
                terms.insert(term);
            }
        }
        MemoryPayload::Outcome(payload) => {
            for term in tokenize_query_terms(&payload.summary) {
                terms.insert(term);
            }
        }
    }
    terms
}

fn collect_recall_candidates_and_exclusions<'a>(
    records: &'a [MemoryRecord],
    allowed_types: &std::collections::BTreeSet<RecordType>,
    query_terms: &[String],
    superseded_ids: &std::collections::BTreeSet<MemoryVersionId>,
) -> (Vec<RecallCandidate<'a>>, Vec<ContextItem>) {
    let mut candidates: Vec<RecallCandidate<'a>> = Vec::new();
    let mut excluded: Vec<ContextItem> = Vec::new();

    for record in records {
        let record_type = record.payload.record_type();
        if !allowed_types.contains(&record_type) {
            continue;
        }

        if record.truth_status == TruthStatus::Retracted {
            excluded.push(excluded_item(record, "truth_status is retracted"));
            continue;
        }

        if superseded_ids.contains(&record.memory_version_id) {
            excluded.push(excluded_item(record, "record is superseded by a newer linked record"));
            continue;
        }

        let terms = record_terms(record);
        let matched_terms = query_terms.iter().filter(|term| terms.contains(*term)).count();
        if matched_terms == 0 {
            excluded.push(excluded_item(record, "no lexical overlap with query text"));
            continue;
        }
        let matched_terms_f32 = f32::from(u16::try_from(matched_terms).unwrap_or(u16::MAX));
        let total_terms_f32 = f32::from(u16::try_from(query_terms.len()).unwrap_or(u16::MAX));

        candidates.push(RecallCandidate {
            record,
            matched_terms,
            total_terms: query_terms.len(),
            lexical_score: matched_terms_f32 / total_terms_f32,
            confidence: record.confidence.unwrap_or(0.5),
        });
    }

    (candidates, excluded)
}

fn selected_recall_item(index: usize, candidate: &RecallCandidate<'_>) -> ContextItem {
    ContextItem {
        rank: index + 1,
        memory_version_id: candidate.record.memory_version_id,
        memory_id: candidate.record.memory_id,
        record_type: candidate.record.payload.record_type(),
        version: candidate.record.version,
        truth_status: candidate.record.truth_status,
        confidence: candidate.record.confidence,
        authority: candidate.record.authority,
        why: Why {
            included: true,
            reasons: vec![
                format!(
                    "lexical relevance matched {}/{} normalized terms",
                    candidate.matched_terms, candidate.total_terms
                ),
                format!(
                    "record_type={} included in recall scope",
                    candidate.record.payload.record_type().as_str()
                ),
                "passed active filters (not retracted, not superseded)".to_string(),
            ],
            rule_scores: Some(RuleScores {
                scope_match: candidate.lexical_score,
                authority_rank: candidate.record.authority.rank(),
                truth_status_rank: candidate.record.truth_status.rank(),
                confidence: candidate.confidence,
            }),
        },
    }
}

fn assign_exclusion_ranks(excluded: &mut [ContextItem]) {
    excluded.sort_by(|lhs, rhs| {
        lhs.memory_id
            .cmp(&rhs.memory_id)
            .then_with(|| lhs.memory_version_id.cmp(&rhs.memory_version_id))
    });
    for (index, item) in excluded.iter_mut().enumerate() {
        item.rank = index + 1;
    }
}

fn make_context_package_id(query: &QueryRequest, snapshot_id: &str) -> Result<String, KernelError> {
    let as_of = query
        .as_of
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| KernelError::Query(format!("invalid as_of format: {err}")))?;
    Ok(format!("cpkg_{as_of}_{snapshot_id}"))
}

/// Build a deterministic Context Package for a normalized policy query.
///
/// # Errors
/// Returns [`KernelError::Query`] when deterministic snapshot metadata is invalid,
/// or [`KernelError::Validation`] when any source record violates domain invariants.
pub fn build_context_package(
    records: &[MemoryRecord],
    query: QueryRequest,
    snapshot_id: &str,
) -> Result<ContextPackage, KernelError> {
    if snapshot_id.trim().is_empty() {
        return Err(KernelError::Query(
            "snapshot_id MUST be provided for deterministic replay".to_string(),
        ));
    }

    for record in records {
        record.validate()?;
    }

    let superseded_ids = collect_superseded_ids(records);
    let (mut candidates, mut excluded) =
        collect_policy_candidates_and_exclusions(records, &query, &superseded_ids);
    candidates.sort_by(PolicyCandidate::cmp);
    let selected: Vec<ContextItem> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| selected_policy_item(index, candidate))
        .collect();
    assign_exclusion_ranks(&mut excluded);
    let answer = derive_answer(&selected, records);
    let context_package_id = make_context_package_id(&query, snapshot_id)?;

    Ok(ContextPackage {
        context_package_id,
        generated_at: query.as_of,
        query,
        determinism: DeterminismMetadata {
            ruleset_version: "ordering.v1".to_string(),
            snapshot_id: snapshot_id.to_string(),
            tie_breakers: default_tie_breakers(),
        },
        answer,
        selected_items: selected,
        excluded_items: excluded,
        ordering_trace: vec![
            "filter: record_type=constraint".to_string(),
            "filter: scope_match(actor, action, resource)".to_string(),
            "exclude: retracted and superseded".to_string(),
            "sort: precedence tuple with deterministic tie-breakers".to_string(),
        ],
    })
}

/// Build a deterministic Context Package for memory recall across selected record types.
///
/// # Errors
/// Returns [`KernelError::Query`] when deterministic snapshot metadata or query text is invalid,
/// or [`KernelError::Validation`] when any source record violates domain invariants.
pub fn build_recall_context_package(
    records: &[MemoryRecord],
    query: QueryRequest,
    snapshot_id: &str,
    record_types: &[RecordType],
) -> Result<ContextPackage, KernelError> {
    use std::collections::BTreeSet;

    if snapshot_id.trim().is_empty() {
        return Err(KernelError::Query(
            "snapshot_id MUST be provided for deterministic replay".to_string(),
        ));
    }

    if query.text.trim().is_empty() {
        return Err(KernelError::Query("recall query text MUST be non-empty".to_string()));
    }

    for record in records {
        record.validate()?;
    }

    let allowed_types = if record_types.is_empty() {
        default_recall_record_types().into_iter().collect::<BTreeSet<_>>()
    } else {
        record_types.iter().copied().collect::<BTreeSet<_>>()
    };

    let query_terms = tokenize_query_terms(&query.text);
    if query_terms.is_empty() {
        return Err(KernelError::Query(
            "recall query text MUST include at least one alphanumeric term".to_string(),
        ));
    }

    let superseded_ids = collect_superseded_ids(records);
    let (mut candidates, mut excluded) = collect_recall_candidates_and_exclusions(
        records,
        &allowed_types,
        &query_terms,
        &superseded_ids,
    );
    candidates.sort_by(RecallCandidate::cmp);
    let selected: Vec<ContextItem> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| selected_recall_item(index, candidate))
        .collect();
    assign_exclusion_ranks(&mut excluded);
    let context_package_id = make_context_package_id(&query, snapshot_id)?;

    let mut selected_types = allowed_types.into_iter().map(RecordType::as_str).collect::<Vec<_>>();
    selected_types.sort_unstable();
    let selected_types = selected_types.join(", ");

    Ok(ContextPackage {
        context_package_id,
        generated_at: query.as_of,
        query,
        determinism: DeterminismMetadata {
            ruleset_version: "recall-ordering.v1".to_string(),
            snapshot_id: snapshot_id.to_string(),
            tie_breakers: default_recall_tie_breakers(),
        },
        answer: Answer {
            result: AnswerResult::Inconclusive,
            why: format!(
                "Recall query selected {} memories across record types [{}]",
                selected.len(),
                selected_types
            ),
        },
        selected_items: selected,
        excluded_items: excluded,
        ordering_trace: vec![
            format!("filter: record_type in [{selected_types}]"),
            "filter: lexical overlap with normalized query terms".to_string(),
            "exclude: retracted and superseded".to_string(),
            "sort: recall precedence tuple with deterministic tie-breakers".to_string(),
        ],
    })
}

fn derive_answer(selected: &[ContextItem], records: &[MemoryRecord]) -> Answer {
    const DEFAULT_CONFIDENCE: f32 = 0.5;
    let mut top_ranked_ids: Vec<MemoryVersionId> = Vec::new();
    let Some(top) = selected.first() else {
        return Answer {
            result: AnswerResult::Inconclusive,
            why: "No active matching constraints were found".to_string(),
        };
    };

    for item in selected {
        let same_confidence = item.confidence.unwrap_or(DEFAULT_CONFIDENCE).to_bits()
            == top.confidence.unwrap_or(DEFAULT_CONFIDENCE).to_bits();
        if item.rank == 1 {
            top_ranked_ids.push(item.memory_version_id);
            continue;
        }

        if item.authority == top.authority
            && item.truth_status == top.truth_status
            && same_confidence
        {
            top_ranked_ids.push(item.memory_version_id);
        } else {
            break;
        }
    }

    let mut has_allow = false;
    let mut has_deny = false;

    for memory_version_id in top_ranked_ids {
        if let Some(effect) = constraint_effect_by_version_id(records, memory_version_id) {
            match effect {
                ConstraintEffect::Allow => has_allow = true,
                ConstraintEffect::Deny => has_deny = true,
            }
        }
    }

    match (has_allow, has_deny) {
        (true, true) => Answer {
            result: AnswerResult::Inconclusive,
            why: "Top-precedence constraints conflict (allow and deny)".to_string(),
        },
        (true, false) => Answer {
            result: AnswerResult::Allow,
            why: "Highest-precedence active constraint allows the action".to_string(),
        },
        (false, true) => Answer {
            result: AnswerResult::Deny,
            why: "Highest-precedence active constraint denies the action".to_string(),
        },
        (false, false) => Answer {
            result: AnswerResult::Inconclusive,
            why: "No effective constraint decision could be derived".to_string(),
        },
    }
}

fn constraint_effect_by_version_id(
    records: &[MemoryRecord],
    memory_version_id: MemoryVersionId,
) -> Option<ConstraintEffect> {
    records.iter().find_map(|record| {
        if record.memory_version_id != memory_version_id {
            return None;
        }

        match &record.payload {
            MemoryPayload::Constraint(constraint) => Some(constraint.effect),
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use time::Duration;

    fn fixture_time() -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(1_700_000_000)
    }

    fn fixture_id(input: &str) -> MemoryId {
        match Ulid::from_string(input) {
            Ok(id) => MemoryId(id),
            Err(err) => panic!("invalid fixture ULID {input}: {err}"),
        }
    }

    fn fixture_version_id(input: &str) -> MemoryVersionId {
        match Ulid::from_string(input) {
            Ok(id) => MemoryVersionId(id),
            Err(err) => panic!("invalid fixture ULID {input}: {err}"),
        }
    }

    fn seeded_permutation(records: &[MemoryRecord], seed: u64) -> Vec<MemoryRecord> {
        fn splitmix64(mut value: u64) -> u64 {
            value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
            value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            value ^ (value >> 31)
        }

        let mut keyed = records
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, record)| {
                let index_u64 = u64::try_from(index).unwrap_or(u64::MAX);
                let key = splitmix64(seed ^ index_u64);
                (key, record)
            })
            .collect::<Vec<_>>();
        keyed.sort_by_key(|(key, _)| *key);
        keyed.into_iter().map(|(_, record)| record).collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn mk_constraint(
        memory_id: MemoryId,
        authority: Authority,
        truth_status: TruthStatus,
        confidence: Option<f32>,
        effect: ConstraintEffect,
        supersedes: Vec<MemoryVersionId>,
        scope_actor: &str,
        scope_action: &str,
        scope_resource: &str,
    ) -> MemoryRecord {
        MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id,
            version: 1,
            created_at: fixture_time(),
            effective_at: fixture_time(),
            truth_status,
            authority,
            confidence,
            writer: "tester".to_string(),
            justification: "fixture".to_string(),
            provenance: Provenance {
                source_uri: "file:///policy.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes,
            contradicts: vec![],
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: scope_actor.to_string(),
                    action: scope_action.to_string(),
                    resource: scope_resource.to_string(),
                },
                effect,
                note: None,
            }),
        }
    }

    fn mk_summary(
        memory_id: MemoryId,
        record_type: RecordType,
        authority: Authority,
        truth_status: TruthStatus,
        confidence: Option<f32>,
        summary: &str,
        supersedes: Vec<MemoryVersionId>,
    ) -> MemoryRecord {
        let payload = match record_type {
            RecordType::Decision => {
                MemoryPayload::Decision(DecisionPayload { summary: summary.to_string() })
            }
            RecordType::Preference => {
                MemoryPayload::Preference(PreferencePayload { summary: summary.to_string() })
            }
            RecordType::Event => {
                MemoryPayload::Event(EventPayload { summary: summary.to_string() })
            }
            RecordType::Outcome => {
                MemoryPayload::Outcome(OutcomePayload { summary: summary.to_string() })
            }
            RecordType::Constraint => {
                panic!("mk_summary does not support constraint payloads")
            }
        };

        MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id,
            version: 1,
            created_at: fixture_time(),
            effective_at: fixture_time(),
            truth_status,
            authority,
            confidence,
            writer: "tester".to_string(),
            justification: "fixture".to_string(),
            provenance: Provenance {
                source_uri: "file:///memory.md".to_string(),
                source_hash: Some("sha256:abc123".to_string()),
                evidence: vec![],
            },
            supersedes,
            contradicts: vec![],
            payload,
        }
    }

    fn assert_validation_error_contains(record: &MemoryRecord, expected_substring: &str) {
        let err = match record.validate() {
            Ok(()) => panic!("expected validation error containing: {expected_substring}"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(expected_substring),
            "validation error `{err}` did not contain `{expected_substring}`"
        );
    }

    // Test IDs: TWR-001
    #[test]
    fn validate_rejects_missing_writer() {
        let mut record = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E4"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        record.writer = " ".to_string();

        assert_validation_error_contains(&record, "writer MUST be provided");
    }

    // Test IDs: TWR-002
    #[test]
    fn validate_rejects_missing_justification() {
        let mut record = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E5"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        record.justification = "  ".to_string();

        assert_validation_error_contains(&record, "justification MUST be provided");
    }

    // Test IDs: TWR-003
    #[test]
    fn validate_rejects_missing_source_uri() {
        let mut record = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E6"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        record.provenance.source_uri = String::new();

        assert_validation_error_contains(&record, "source_uri MUST be provided");
    }

    // Test IDs: TWR-004
    #[test]
    fn validate_rejects_invalid_source_hash_format() {
        let mut record = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E7"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        record.provenance.source_hash = Some("md5:deadbeef".to_string());

        assert_validation_error_contains(&record, "source_hash MUST be formatted as sha256:<hex>");
    }

    // Test IDs: TWR-005
    #[test]
    fn validate_rejects_inferred_without_confidence() {
        let record = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E8"),
            Authority::Derived,
            TruthStatus::Inferred,
            None,
            ConstraintEffect::Allow,
            vec![],
            "user",
            "use",
            "usb_drive",
        );

        assert_validation_error_contains(
            &record,
            "confidence MUST be provided for inferred/speculative records",
        );
    }

    // Test IDs: TRES-001
    #[test]
    fn retracted_constraints_are_excluded_with_reason() {
        let retracted = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E9"),
            Authority::Authoritative,
            TruthStatus::Retracted,
            Some(0.2),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );

        let package = match build_context_package(
            &[retracted],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_retracted",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert!(package.selected_items.is_empty());
        assert_eq!(package.excluded_items.len(), 1);
        assert!(package.excluded_items[0]
            .why
            .reasons
            .iter()
            .any(|reason| reason.contains("truth_status is retracted")));
    }

    // Test IDs: TRES-003
    #[test]
    fn conflicting_top_precedence_constraints_return_inconclusive() {
        let allow = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E0"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.8),
            ConstraintEffect::Allow,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        let deny = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E1"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.8),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );

        let package = match build_context_package(
            &[allow, deny],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_conflict",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert_eq!(package.answer.result, AnswerResult::Inconclusive);
    }

    // Test IDs: TID-004
    #[test]
    fn context_items_include_memory_version_ids_for_selected_and_excluded() {
        let selected = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E2"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        let excluded = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2E3"),
            Authority::Authoritative,
            TruthStatus::Retracted,
            Some(0.1),
            ConstraintEffect::Allow,
            vec![],
            "user",
            "use",
            "usb_drive",
        );

        let selected_id = selected.memory_version_id;
        let excluded_id = excluded.memory_version_id;

        let package = match build_context_package(
            &[selected, excluded],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_ids",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert_eq!(package.selected_items.len(), 1);
        assert_eq!(package.excluded_items.len(), 1);
        assert_eq!(package.selected_items[0].memory_version_id, selected_id);
        assert_eq!(package.excluded_items[0].memory_version_id, excluded_id);
    }

    // Test IDs: TRES-004
    #[test]
    fn deterministic_sort_prefers_authority_then_truth_then_confidence() {
        let a = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DF"),
            Authority::Derived,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Allow,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        let b = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DG"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.1),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );

        let package = match build_context_package(
            &[a, b],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_1",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert_eq!(package.selected_items[0].authority, Authority::Authoritative);
        assert_eq!(package.answer.result, AnswerResult::Deny);
    }

    // Test IDs: TRES-002
    #[test]
    fn superseded_records_are_excluded() {
        let old_memory_id = fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DH");
        let old = mk_constraint(
            old_memory_id,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        let new = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DJ"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.95),
            ConstraintEffect::Deny,
            vec![old.memory_version_id],
            "user",
            "use",
            "usb_drive",
        );

        let package = match build_context_package(
            &[old, new],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_2",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert_eq!(package.selected_items.len(), 1);
        assert_eq!(package.excluded_items.len(), 1);
        assert_eq!(package.excluded_items[0].memory_id, old_memory_id);
    }

    // Test IDs: TDET-003
    #[test]
    fn deterministic_tie_break_uses_memory_version_id() {
        let shared_memory_id = fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DK");

        let mut a = mk_constraint(
            shared_memory_id,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        a.memory_version_id = fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DM");

        let mut b = mk_constraint(
            shared_memory_id,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        b.memory_version_id = fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DN");

        let package = match build_context_package(
            &[b, a],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_3",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        assert_eq!(
            package.selected_items[0].memory_version_id,
            fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DM")
        );
    }

    // Test IDs: TDET-001, TDET-002
    #[test]
    fn context_package_json_is_stable_for_permuted_input() {
        let old_memory_id = fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DP");
        let mut old = mk_constraint(
            old_memory_id,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.8),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        old.memory_version_id = fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DQ");

        let mut new = mk_constraint(
            old_memory_id,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            ConstraintEffect::Deny,
            vec![old.memory_version_id],
            "user",
            "use",
            "usb_drive",
        );
        new.memory_version_id = fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DR");

        let mut retracted = mk_constraint(
            fixture_id("01HZY9D4Q3SG7PV9A6EXJ8N2DS"),
            Authority::Derived,
            TruthStatus::Retracted,
            Some(0.4),
            ConstraintEffect::Allow,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        retracted.memory_version_id = fixture_version_id("01HZY9D4Q3SG7PV9A6EXJ8N2DT");

        let package_a = match build_context_package(
            &[retracted.clone(), new.clone(), old.clone()],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_4",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };
        let package_b = match build_context_package(
            &[old, retracted, new],
            QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            },
            "txn_4",
        ) {
            Ok(package) => package,
            Err(err) => panic!("context package should build: {err}"),
        };

        let json_a = match serde_json::to_string(&package_a) {
            Ok(value) => value,
            Err(err) => panic!("json serialization should succeed: {err}"),
        };
        let json_b = match serde_json::to_string(&package_b) {
            Ok(value) => value,
            Err(err) => panic!("json serialization should succeed: {err}"),
        };

        assert_eq!(json_a, json_b);
    }

    // Test IDs: TRES-005
    #[test]
    fn recall_query_returns_mixed_records_with_explainable_exclusions() {
        let decision = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY11"),
            RecordType::Decision,
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.9),
            "Decision: USB media usage requires manager approval",
            vec![],
        );
        let retracted_preference = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY12"),
            RecordType::Preference,
            Authority::Derived,
            TruthStatus::Retracted,
            Some(0.4),
            "Preference: avoid USB devices",
            vec![],
        );
        let event = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY13"),
            RecordType::Event,
            Authority::Derived,
            TruthStatus::Observed,
            Some(0.7),
            "Weekly team lunch and budget review",
            vec![],
        );
        let old_outcome = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY14"),
            RecordType::Outcome,
            Authority::Authoritative,
            TruthStatus::Observed,
            Some(0.8),
            "Outcome: USB incident created compliance finding",
            vec![],
        );
        let new_outcome = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY15"),
            RecordType::Outcome,
            Authority::Authoritative,
            TruthStatus::Observed,
            Some(0.95),
            "Outcome: USB policy tightened after audit finding",
            vec![old_outcome.memory_version_id],
        );

        let package = match build_recall_context_package(
            &[decision, retracted_preference, event, old_outcome, new_outcome],
            QueryRequest {
                text: "usb policy outcome".to_string(),
                actor: "*".to_string(),
                action: "*".to_string(),
                resource: "*".to_string(),
                as_of: fixture_time(),
            },
            "txn_recall_explainability",
            &[RecordType::Decision, RecordType::Preference, RecordType::Event, RecordType::Outcome],
        ) {
            Ok(package) => package,
            Err(err) => panic!("recall context package should build: {err}"),
        };

        assert_eq!(package.determinism.ruleset_version, "recall-ordering.v1");
        assert_eq!(package.selected_items.len(), 2);
        assert_eq!(package.excluded_items.len(), 3);
        assert!(package
            .selected_items
            .iter()
            .all(|item| item.why.included && item.why.rule_scores.is_some()));
        assert!(package.excluded_items.iter().any(|item| item
            .why
            .reasons
            .iter()
            .any(|reason| reason.contains("retracted"))));
        assert!(package.excluded_items.iter().any(|item| item
            .why
            .reasons
            .iter()
            .any(|reason| reason.contains("superseded"))));
        assert!(package.excluded_items.iter().any(|item| item
            .why
            .reasons
            .iter()
            .any(|reason| reason.contains("no lexical overlap"))));
    }

    // Test IDs: TDET-004
    #[test]
    fn recall_context_package_json_is_stable_for_permuted_mixed_input() {
        let mut decision = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY21"),
            RecordType::Decision,
            Authority::Authoritative,
            TruthStatus::Observed,
            Some(0.8),
            "Decision: USB use is disabled in production",
            vec![],
        );
        decision.memory_version_id = fixture_version_id("01K1D3A7E9J5MNNN8F5JVCJY31");

        let mut preference = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY22"),
            RecordType::Preference,
            Authority::Derived,
            TruthStatus::Asserted,
            Some(0.7),
            "Preference: use encrypted storage over USB",
            vec![],
        );
        preference.memory_version_id = fixture_version_id("01K1D3A7E9J5MNNN8F5JVCJY32");

        let mut event = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY23"),
            RecordType::Event,
            Authority::Note,
            TruthStatus::Observed,
            Some(0.6),
            "Event: USB policy briefing was completed",
            vec![],
        );
        event.memory_version_id = fixture_version_id("01K1D3A7E9J5MNNN8F5JVCJY33");

        let mut outcome = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY24"),
            RecordType::Outcome,
            Authority::Authoritative,
            TruthStatus::Observed,
            Some(0.9),
            "Outcome: policy compliance improved after USB lockdown",
            vec![],
        );
        outcome.memory_version_id = fixture_version_id("01K1D3A7E9J5MNNN8F5JVCJY34");

        let query = QueryRequest {
            text: "usb policy compliance".to_string(),
            actor: "*".to_string(),
            action: "*".to_string(),
            resource: "*".to_string(),
            as_of: fixture_time(),
        };

        let package_a = match build_recall_context_package(
            &[decision.clone(), preference.clone(), event.clone(), outcome.clone()],
            query.clone(),
            "txn_recall_stability",
            &[RecordType::Decision, RecordType::Preference, RecordType::Event, RecordType::Outcome],
        ) {
            Ok(package) => package,
            Err(err) => panic!("recall context package should build: {err}"),
        };
        let package_b = match build_recall_context_package(
            &[outcome, event, preference, decision],
            query,
            "txn_recall_stability",
            &[RecordType::Decision, RecordType::Preference, RecordType::Event, RecordType::Outcome],
        ) {
            Ok(package) => package,
            Err(err) => panic!("recall context package should build: {err}"),
        };

        let json_a = match serde_json::to_string(&package_a) {
            Ok(value) => value,
            Err(err) => panic!("json serialization should succeed: {err}"),
        };
        let json_b = match serde_json::to_string(&package_b) {
            Ok(value) => value,
            Err(err) => panic!("json serialization should succeed: {err}"),
        };

        assert_eq!(json_a, json_b);
    }

    // Test IDs: TRES-006
    #[test]
    fn recall_defaults_to_non_constraint_record_types() {
        let constraint = mk_constraint(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY41"),
            Authority::Authoritative,
            TruthStatus::Asserted,
            Some(0.95),
            ConstraintEffect::Deny,
            vec![],
            "user",
            "use",
            "usb_drive",
        );
        let decision = mk_summary(
            fixture_id("01K1D3A7E9J5MNNN8F5JVCJY42"),
            RecordType::Decision,
            Authority::Authoritative,
            TruthStatus::Observed,
            Some(0.8),
            "Decision: USB usage was blocked by endpoint control",
            vec![],
        );

        let package = match build_recall_context_package(
            &[constraint, decision],
            QueryRequest {
                text: "usb usage blocked".to_string(),
                actor: "*".to_string(),
                action: "*".to_string(),
                resource: "*".to_string(),
                as_of: fixture_time(),
            },
            "txn_recall_default_types",
            &[],
        ) {
            Ok(package) => package,
            Err(err) => panic!("recall context package should build: {err}"),
        };

        assert_eq!(package.selected_items.len(), 1);
        assert_eq!(package.selected_items[0].record_type, RecordType::Decision);
        assert_eq!(package.answer.result, AnswerResult::Inconclusive);
    }

    // Test IDs: TPERF-001
    #[test]
    fn policy_context_package_meets_baseline_budget() {
        let records = (0..500)
            .map(|_| {
                mk_constraint(
                    MemoryId::new(),
                    Authority::Authoritative,
                    TruthStatus::Asserted,
                    Some(0.8),
                    ConstraintEffect::Deny,
                    vec![],
                    "user",
                    "use",
                    "usb_drive",
                )
            })
            .collect::<Vec<_>>();
        let query = QueryRequest {
            text: "Am I allowed to use a USB drive?".to_string(),
            actor: "user".to_string(),
            action: "use".to_string(),
            resource: "usb_drive".to_string(),
            as_of: fixture_time(),
        };

        let start = std::time::Instant::now();
        for _ in 0..25 {
            let result = build_context_package(&records, query.clone(), "txn_perf_policy");
            if let Err(err) = result {
                panic!("policy performance fixture should build: {err}");
            }
        }
        assert!(
            start.elapsed() <= std::time::Duration::from_secs(4),
            "policy context package exceeded baseline budget"
        );
    }

    // Test IDs: TPERF-002
    #[test]
    fn recall_context_package_meets_baseline_budget() {
        let records = (0..500)
            .map(|index| {
                let record_type = match index % 4 {
                    0 => RecordType::Decision,
                    1 => RecordType::Preference,
                    2 => RecordType::Event,
                    _ => RecordType::Outcome,
                };
                mk_summary(
                    MemoryId::new(),
                    record_type,
                    Authority::Authoritative,
                    TruthStatus::Observed,
                    Some(0.85),
                    "USB security and compliance benchmark fixture",
                    vec![],
                )
            })
            .collect::<Vec<_>>();
        let query = QueryRequest {
            text: "usb security compliance".to_string(),
            actor: "*".to_string(),
            action: "*".to_string(),
            resource: "*".to_string(),
            as_of: fixture_time(),
        };

        let start = std::time::Instant::now();
        for _ in 0..25 {
            let result = build_recall_context_package(
                &records,
                query.clone(),
                "txn_perf_recall",
                &default_recall_record_types(),
            );
            if let Err(err) = result {
                panic!("recall performance fixture should build: {err}");
            }
        }
        assert!(
            start.elapsed() <= std::time::Duration::from_secs(4),
            "recall context package exceeded baseline budget"
        );
    }

    // Test IDs: TDET-005
    proptest! {
        #[test]
        fn property_policy_context_is_deterministic_under_seeded_permutations(seed_a in any::<u64>(), seed_b in any::<u64>()) {
            let old = mk_constraint(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY51"),
                Authority::Authoritative,
                TruthStatus::Asserted,
                Some(0.8),
                ConstraintEffect::Deny,
                vec![],
                "user",
                "use",
                "usb_drive",
            );
            let new = mk_constraint(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY52"),
                Authority::Authoritative,
                TruthStatus::Asserted,
                Some(0.9),
                ConstraintEffect::Deny,
                vec![old.memory_version_id],
                "user",
                "use",
                "usb_drive",
            );
            let retracted = mk_constraint(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY53"),
                Authority::Derived,
                TruthStatus::Retracted,
                Some(0.3),
                ConstraintEffect::Allow,
                vec![],
                "user",
                "use",
                "usb_drive",
            );
            let base = vec![old, new, retracted];
            let records_a = seeded_permutation(&base, seed_a);
            let records_b = seeded_permutation(&base, seed_b);
            let query = QueryRequest {
                text: "Am I allowed to use a USB drive?".to_string(),
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
                as_of: fixture_time(),
            };

            let package_a = build_context_package(&records_a, query.clone(), "txn_prop_policy");
            let package_b = build_context_package(&records_b, query, "txn_prop_policy");
            prop_assert!(package_a.is_ok());
            prop_assert!(package_b.is_ok());

            let json_a = serde_json::to_string(&package_a.unwrap_or_else(|_| unreachable!()));
            let json_b = serde_json::to_string(&package_b.unwrap_or_else(|_| unreachable!()));
            prop_assert!(json_a.is_ok());
            prop_assert!(json_b.is_ok());
            prop_assert_eq!(
                json_a.unwrap_or_else(|_| unreachable!()),
                json_b.unwrap_or_else(|_| unreachable!())
            );
        }
    }

    // Test IDs: TDET-006
    proptest! {
        #[test]
        fn property_recall_context_is_deterministic_under_seeded_permutations(seed_a in any::<u64>(), seed_b in any::<u64>()) {
            let decision = mk_summary(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY61"),
                RecordType::Decision,
                Authority::Authoritative,
                TruthStatus::Observed,
                Some(0.8),
                "Decision: USB controls are required",
                vec![],
            );
            let preference = mk_summary(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY62"),
                RecordType::Preference,
                Authority::Derived,
                TruthStatus::Asserted,
                Some(0.6),
                "Preference: avoid unknown USB devices",
                vec![],
            );
            let event = mk_summary(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY63"),
                RecordType::Event,
                Authority::Note,
                TruthStatus::Observed,
                Some(0.7),
                "Event: USB training completed",
                vec![],
            );
            let outcome = mk_summary(
                fixture_id("01K1D3A7E9J5MNNN8F5JVCJY64"),
                RecordType::Outcome,
                Authority::Authoritative,
                TruthStatus::Observed,
                Some(0.9),
                "Outcome: USB compliance improved",
                vec![],
            );

            let base = vec![decision, preference, event, outcome];
            let records_a = seeded_permutation(&base, seed_a);
            let records_b = seeded_permutation(&base, seed_b);
            let query = QueryRequest {
                text: "usb compliance controls".to_string(),
                actor: "*".to_string(),
                action: "*".to_string(),
                resource: "*".to_string(),
                as_of: fixture_time(),
            };

            let package_a = build_recall_context_package(
                &records_a,
                query.clone(),
                "txn_prop_recall",
                &default_recall_record_types(),
            );
            let package_b = build_recall_context_package(
                &records_b,
                query,
                "txn_prop_recall",
                &default_recall_record_types(),
            );
            prop_assert!(package_a.is_ok());
            prop_assert!(package_b.is_ok());

            let json_a = serde_json::to_string(&package_a.unwrap_or_else(|_| unreachable!()));
            let json_b = serde_json::to_string(&package_b.unwrap_or_else(|_| unreachable!()));
            prop_assert!(json_a.is_ok());
            prop_assert!(json_b.is_ok());
            prop_assert_eq!(
                json_a.unwrap_or_else(|_| unreachable!()),
                json_b.unwrap_or_else(|_| unreachable!())
            );
        }
    }
}
