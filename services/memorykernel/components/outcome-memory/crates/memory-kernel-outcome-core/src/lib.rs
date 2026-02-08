use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Display, Formatter};

use memory_kernel_core::MemoryId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{Duration, OffsetDateTime, UtcOffset};
use ulid::Ulid;

#[derive(Debug, Clone, thiserror::Error, Eq, PartialEq)]
pub enum OutcomeError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("projection error: {0}")]
    Projection(String),
    #[error("configuration error: {0}")]
    Configuration(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeEventType {
    Inherited,
    Success,
    Failure,
    Ignored,
    Unknown,
    ManualSetConfidence,
    ManualPromote,
    ManualRetire,
    AuthoritativeContradiction,
}

impl OutcomeEventType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inherited => "inherited",
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Ignored => "ignored",
            Self::Unknown => "unknown",
            Self::ManualSetConfidence => "manual_set_confidence",
            Self::ManualPromote => "manual_promote",
            Self::ManualRetire => "manual_retire",
            Self::AuthoritativeContradiction => "authoritative_contradiction",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "inherited" => Some(Self::Inherited),
            "success" => Some(Self::Success),
            "failure" => Some(Self::Failure),
            "ignored" => Some(Self::Ignored),
            "unknown" => Some(Self::Unknown),
            "manual_set_confidence" => Some(Self::ManualSetConfidence),
            "manual_promote" => Some(Self::ManualPromote),
            "manual_retire" => Some(Self::ManualRetire),
            "authoritative_contradiction" => Some(Self::AuthoritativeContradiction),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Med,
    High,
}

impl Severity {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "med" => Some(Self::Med),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TrustStatus {
    Active,
    Validated,
    Retired,
}

impl TrustStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Validated => "validated",
            Self::Retired => "retired",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "validated" => Some(Self::Validated),
            "retired" => Some(Self::Retired),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    Safe,
    Exploration,
}

impl RetrievalMode {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "safe" => Some(Self::Safe),
            "exploration" => Some(Self::Exploration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MemoryKey {
    pub memory_id: MemoryId,
    pub version: u32,
}

impl Display for MemoryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.memory_id, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeEvent {
    pub event_seq: i64,
    pub event_id: Ulid,
    pub ruleset_version: u32,
    pub memory_id: MemoryId,
    pub version: u32,
    pub event_type: OutcomeEventType,
    pub occurred_at: OffsetDateTime,
    pub recorded_at: OffsetDateTime,
    pub writer: String,
    pub justification: String,
    pub context_id: Option<String>,
    pub edited: bool,
    pub escalated: bool,
    pub severity: Option<Severity>,
    pub manual_confidence: Option<f32>,
    pub override_cap: bool,
    pub payload_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeEventInput {
    pub event_id: Option<Ulid>,
    pub ruleset_version: u32,
    pub memory_id: MemoryId,
    pub version: u32,
    pub event_type: OutcomeEventType,
    pub occurred_at: OffsetDateTime,
    pub writer: String,
    pub justification: String,
    pub context_id: Option<String>,
    pub edited: bool,
    pub escalated: bool,
    pub severity: Option<Severity>,
    pub manual_confidence: Option<f32>,
    pub override_cap: bool,
    pub payload_json: Value,
}

impl OutcomeEventInput {
    /// Validates a write event payload before append.
    ///
    /// # Errors
    /// Returns [`OutcomeError::Validation`] when required fields are missing
    /// or violate schema constraints.
    pub fn validate(&self) -> Result<(), OutcomeError> {
        if self.ruleset_version == 0 {
            return Err(OutcomeError::Validation(
                "ruleset_version MUST be >= 1".to_string(),
            ));
        }

        if self.version == 0 {
            return Err(OutcomeError::Validation("version MUST be >= 1".to_string()));
        }

        if self.writer.trim().is_empty() {
            return Err(OutcomeError::Validation(
                "writer MUST be provided for every write".to_string(),
            ));
        }

        if self.justification.trim().is_empty() {
            return Err(OutcomeError::Validation(
                "justification MUST be provided for every write".to_string(),
            ));
        }

        if self.occurred_at.offset() != UtcOffset::UTC {
            return Err(OutcomeError::Validation(
                "occurred_at MUST be UTC (offset Z)".to_string(),
            ));
        }

        if self.escalated && self.severity.is_none() {
            return Err(OutcomeError::Validation(
                "severity is required when escalated=true".to_string(),
            ));
        }

        if matches!(self.event_type, OutcomeEventType::ManualSetConfidence)
            && self.manual_confidence.is_none()
        {
            return Err(OutcomeError::Validation(
                "manual_set_confidence requires manual_confidence".to_string(),
            ));
        }

        if let Some(confidence) = self.manual_confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(OutcomeError::Validation(
                    "manual_confidence MUST be in [0.0, 1.0]".to_string(),
                ));
            }
        }

        if matches!(self.event_type, OutcomeEventType::Inherited)
            && self.manual_confidence.is_none()
        {
            return Err(OutcomeError::Validation(
                "inherited requires source confidence in manual_confidence".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeRuleset {
    pub ruleset_version: u32,
    pub alpha: f32,
    pub per_event_decay: f32,
    pub success_weight: f32,
    pub edited_success_weight: f32,
    pub failure_weight: f32,
    pub ignored_weight: f32,
    pub severity_low_multiplier: f32,
    pub severity_med_multiplier: f32,
    pub severity_high_multiplier: f32,
    pub inheritance_factor: f32,
    pub inheritance_cap: f32,
    pub base_confidence: f32,
    pub contradiction_degrade: f32,
    pub contradiction_cap: f32,
    pub validated_wins_required: u8,
    pub validated_window_size: usize,
    pub safe_min_confidence: f32,
    pub exploration_min_confidence: f32,
    pub exploration_probe_min_confidence: f32,
    pub exploration_probe_max_confidence: f32,
    pub exploration_probe_budget: f32,
    pub read_decay_lambda_per_day: f32,
}

impl OutcomeRuleset {
    #[must_use]
    pub fn v1() -> Self {
        Self {
            ruleset_version: 1,
            alpha: 0.08,
            per_event_decay: 0.02,
            success_weight: 1.0,
            edited_success_weight: 0.5,
            failure_weight: -1.25,
            ignored_weight: -0.15,
            severity_low_multiplier: 1.0,
            severity_med_multiplier: 1.2,
            severity_high_multiplier: 1.5,
            inheritance_factor: 0.70,
            inheritance_cap: 0.80,
            base_confidence: 0.50,
            contradiction_degrade: 0.30,
            contradiction_cap: 0.40,
            validated_wins_required: 3,
            validated_window_size: 5,
            safe_min_confidence: 0.60,
            exploration_min_confidence: 0.30,
            exploration_probe_min_confidence: 0.15,
            exploration_probe_max_confidence: 0.30,
            exploration_probe_budget: 0.20,
            read_decay_lambda_per_day: 0.01,
        }
    }

    /// Validates ruleset numeric bounds and window invariants.
    ///
    /// # Errors
    /// Returns [`OutcomeError::Configuration`] when one or more
    /// ruleset fields are outside allowed bounds.
    pub fn validate(&self) -> Result<(), OutcomeError> {
        if self.ruleset_version == 0 {
            return Err(OutcomeError::Configuration(
                "ruleset_version MUST be >= 1".to_string(),
            ));
        }

        for (name, value) in [
            ("alpha", self.alpha),
            ("per_event_decay", self.per_event_decay),
            ("base_confidence", self.base_confidence),
            ("inheritance_factor", self.inheritance_factor),
            ("inheritance_cap", self.inheritance_cap),
            ("contradiction_cap", self.contradiction_cap),
            ("safe_min_confidence", self.safe_min_confidence),
            (
                "exploration_min_confidence",
                self.exploration_min_confidence,
            ),
            (
                "exploration_probe_min_confidence",
                self.exploration_probe_min_confidence,
            ),
            (
                "exploration_probe_max_confidence",
                self.exploration_probe_max_confidence,
            ),
            ("exploration_probe_budget", self.exploration_probe_budget),
            ("read_decay_lambda_per_day", self.read_decay_lambda_per_day),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(OutcomeError::Configuration(format!(
                    "{name} MUST be in [0.0, 1.0]"
                )));
            }
        }

        if self.validated_window_size == 0 {
            return Err(OutcomeError::Configuration(
                "validated_window_size MUST be >= 1".to_string(),
            ));
        }

        if usize::from(self.validated_wins_required) > self.validated_window_size {
            return Err(OutcomeError::Configuration(
                "validated_wins_required MUST be <= validated_window_size".to_string(),
            ));
        }

        if self.exploration_probe_min_confidence > self.exploration_probe_max_confidence {
            return Err(OutcomeError::Configuration(
                "exploration probe min cannot exceed max".to_string(),
            ));
        }

        Ok(())
    }

    #[must_use]
    pub fn severity_multiplier(&self, severity: Option<Severity>, escalated: bool) -> f32 {
        if !escalated {
            return 1.0;
        }

        match severity.unwrap_or(Severity::Low) {
            Severity::Low => self.severity_low_multiplier,
            Severity::Med => self.severity_med_multiplier,
            Severity::High => self.severity_high_multiplier,
        }
    }

    /// Decodes and validates a ruleset from JSON.
    ///
    /// # Errors
    /// Returns [`OutcomeError::Configuration`] when JSON decoding fails
    /// or decoded values violate ruleset constraints.
    pub fn from_json(value: &Value) -> Result<Self, OutcomeError> {
        let ruleset: Self = serde_json::from_value(value.clone()).map_err(|err| {
            OutcomeError::Configuration(format!("invalid ruleset JSON payload: {err}"))
        })?;
        ruleset.validate()?;
        Ok(ruleset)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryTrust {
    pub memory_id: MemoryId,
    pub version: u32,
    pub confidence_raw: f32,
    pub confidence_effective: f32,
    pub baseline_confidence: f32,
    pub trust_status: TrustStatus,
    pub contradiction_cap_active: bool,
    pub cap_value: f32,
    pub manual_override_active: bool,
    pub wins_last5: u8,
    pub failures_last5: u8,
    pub last_event_seq: i64,
    pub last_scored_at: Option<OffsetDateTime>,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateDecision {
    pub memory_id: MemoryId,
    pub version: u32,
    pub include: bool,
    pub confidence_effective: f32,
    pub trust_status: TrustStatus,
    pub capped: bool,
    pub reason_codes: Vec<String>,
}

/// Projects append-only outcome events into a trust snapshot.
///
/// # Errors
/// Returns [`OutcomeError::Projection`] for invalid event streams and
/// [`OutcomeError::Configuration`] when referenced rulesets are missing
/// or invalid.
#[allow(clippy::too_many_lines)]
pub fn project_memory_trust(
    events: &[OutcomeEvent],
    rulesets: &BTreeMap<u32, OutcomeRuleset>,
) -> Result<Option<MemoryTrust>, OutcomeError> {
    if events.is_empty() {
        return Ok(None);
    }

    let first = &events[0];
    if first.version == 0 {
        return Err(OutcomeError::Projection("version MUST be >= 1".to_string()));
    }

    let key = MemoryKey {
        memory_id: first.memory_id,
        version: first.version,
    };

    let mut prev_event_seq = 0_i64;
    let mut wins_window: VecDeque<OutcomeWindowEntry> = VecDeque::new();

    let mut baseline = ruleset_for(first.ruleset_version, rulesets)?.base_confidence;
    let mut confidence_raw = baseline;
    let mut confidence_effective = baseline;
    let mut trust_status = TrustStatus::Active;
    let mut contradiction_cap_active = false;
    let mut cap_value = 1.0;
    let mut manual_override_active = false;
    let mut last_scored_at = None;

    for event in events {
        if event.memory_id != key.memory_id || event.version != key.version {
            return Err(OutcomeError::Projection(
                "replay stream MUST contain a single (memory_id, version) key".to_string(),
            ));
        }

        if event.event_seq <= prev_event_seq {
            return Err(OutcomeError::Projection(
                "event_seq MUST be strictly increasing".to_string(),
            ));
        }

        let ruleset = ruleset_for(event.ruleset_version, rulesets)?;
        prev_event_seq = event.event_seq;

        match event.event_type {
            OutcomeEventType::Inherited => {
                let Some(source_confidence) = event.manual_confidence else {
                    return Err(OutcomeError::Projection(
                        "inherited event missing source confidence".to_string(),
                    ));
                };
                baseline = clamp(ruleset.inheritance_factor * source_confidence, 0.0, 1.0)
                    .min(ruleset.inheritance_cap);
                confidence_raw = baseline;
                contradiction_cap_active = false;
                cap_value = 1.0;
                manual_override_active = false;
                wins_window.clear();
                trust_status = TrustStatus::Active;
            }
            OutcomeEventType::Success => {
                let base_weight = if event.edited {
                    ruleset.edited_success_weight
                } else {
                    ruleset.success_weight
                };
                apply_scored_event(&mut confidence_raw, baseline, base_weight, &ruleset, event);
                last_scored_at = Some(event.occurred_at);
                push_window_entry(
                    &mut wins_window,
                    OutcomeWindowEntry::Success,
                    ruleset.validated_window_size,
                );
            }
            OutcomeEventType::Failure => {
                apply_scored_event(
                    &mut confidence_raw,
                    baseline,
                    ruleset.failure_weight,
                    &ruleset,
                    event,
                );
                last_scored_at = Some(event.occurred_at);
                push_window_entry(
                    &mut wins_window,
                    OutcomeWindowEntry::Failure,
                    ruleset.validated_window_size,
                );
            }
            OutcomeEventType::Ignored => {
                apply_scored_event(
                    &mut confidence_raw,
                    baseline,
                    ruleset.ignored_weight,
                    &ruleset,
                    event,
                );
                last_scored_at = Some(event.occurred_at);
                push_window_entry(
                    &mut wins_window,
                    OutcomeWindowEntry::Ignored,
                    ruleset.validated_window_size,
                );
            }
            OutcomeEventType::Unknown => {
                push_window_entry(
                    &mut wins_window,
                    OutcomeWindowEntry::Unknown,
                    ruleset.validated_window_size,
                );
            }
            OutcomeEventType::ManualSetConfidence => {
                let Some(value) = event.manual_confidence else {
                    return Err(OutcomeError::Projection(
                        "manual_set_confidence missing manual_confidence".to_string(),
                    ));
                };
                confidence_raw = clamp(value, 0.0, 1.0);
                manual_override_active = event.override_cap;
            }
            OutcomeEventType::ManualPromote => {
                trust_status = TrustStatus::Active;
                wins_window.clear();
            }
            OutcomeEventType::ManualRetire => {
                trust_status = TrustStatus::Retired;
            }
            OutcomeEventType::AuthoritativeContradiction => {
                contradiction_cap_active = true;
                cap_value = ruleset.contradiction_cap;
                confidence_raw = clamp(confidence_raw - ruleset.contradiction_degrade, 0.0, 1.0);
            }
        }

        confidence_effective = confidence_raw;
        if contradiction_cap_active && !manual_override_active {
            confidence_effective = confidence_effective.min(cap_value);
        }

        if trust_status != TrustStatus::Retired {
            let wins = wins_window
                .iter()
                .filter(|entry| matches!(entry, OutcomeWindowEntry::Success))
                .count();
            let failures = wins_window
                .iter()
                .filter(|entry| matches!(entry, OutcomeWindowEntry::Failure))
                .count();

            if !contradiction_cap_active
                && wins >= usize::from(ruleset.validated_wins_required)
                && failures == 0
            {
                trust_status = TrustStatus::Validated;
            } else {
                trust_status = TrustStatus::Active;
            }
        }
    }

    let wins_last5 = wins_window
        .iter()
        .filter(|entry| matches!(entry, OutcomeWindowEntry::Success))
        .count();
    let failures_last5 = wins_window
        .iter()
        .filter(|entry| matches!(entry, OutcomeWindowEntry::Failure))
        .count();

    let last = events
        .last()
        .ok_or_else(|| OutcomeError::Projection("missing terminal event".to_string()))?;

    Ok(Some(MemoryTrust {
        memory_id: key.memory_id,
        version: key.version,
        confidence_raw,
        confidence_effective,
        baseline_confidence: baseline,
        trust_status,
        contradiction_cap_active,
        cap_value,
        manual_override_active,
        wins_last5: u8::try_from(wins_last5).unwrap_or(u8::MAX),
        failures_last5: u8::try_from(failures_last5).unwrap_or(u8::MAX),
        last_event_seq: last.event_seq,
        last_scored_at,
        updated_at: last.recorded_at,
    }))
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn apply_as_of_decay(
    trust: &MemoryTrust,
    ruleset: &OutcomeRuleset,
    as_of: OffsetDateTime,
) -> MemoryTrust {
    if trust.trust_status == TrustStatus::Retired {
        return trust.clone();
    }

    let Some(last_scored_at) = trust.last_scored_at else {
        return trust.clone();
    };

    if as_of <= last_scored_at {
        return trust.clone();
    }

    let elapsed = as_of - last_scored_at;
    if elapsed <= Duration::ZERO {
        return trust.clone();
    }

    let elapsed_days = elapsed.as_seconds_f64() / Duration::DAY.as_seconds_f64();
    let decay_term = (-f64::from(ruleset.read_decay_lambda_per_day) * elapsed_days).exp() as f32;

    let mut decayed = trust.clone();
    decayed.confidence_raw = clamp(
        decayed.baseline_confidence
            + (decayed.confidence_raw - decayed.baseline_confidence) * decay_term,
        0.0,
        1.0,
    );

    decayed.confidence_effective = decayed.confidence_raw;
    if decayed.contradiction_cap_active && !decayed.manual_override_active {
        decayed.confidence_effective = decayed.confidence_effective.min(decayed.cap_value);
    }

    decayed
}

#[must_use]
pub fn gate_memory(
    trust: &MemoryTrust,
    mode: RetrievalMode,
    context_id: Option<&str>,
    ruleset: &OutcomeRuleset,
) -> GateDecision {
    let mut include = false;
    let mut reason_codes = Vec::new();

    if trust.trust_status == TrustStatus::Retired {
        reason_codes.push("excluded.retired".to_string());
        return GateDecision {
            memory_id: trust.memory_id,
            version: trust.version,
            include,
            confidence_effective: trust.confidence_effective,
            trust_status: trust.trust_status,
            capped: trust.contradiction_cap_active && !trust.manual_override_active,
            reason_codes,
        };
    }

    let capped = trust.contradiction_cap_active && !trust.manual_override_active;

    match mode {
        RetrievalMode::Safe => {
            if trust.trust_status == TrustStatus::Validated
                && !capped
                && trust.confidence_effective >= ruleset.safe_min_confidence
            {
                include = true;
                reason_codes.push("included.safe.validated_threshold".to_string());
            } else {
                reason_codes.push("excluded.safe.threshold_or_status".to_string());
            }
        }
        RetrievalMode::Exploration => {
            if trust.trust_status == TrustStatus::Validated
                && !capped
                && trust.confidence_effective >= ruleset.safe_min_confidence
            {
                include = true;
                reason_codes.push("included.exploration.safe_equivalent".to_string());
            } else if trust.trust_status == TrustStatus::Active
                && trust.confidence_effective >= ruleset.exploration_min_confidence
            {
                include = true;
                reason_codes.push("included.exploration.active_threshold".to_string());
            } else if trust.trust_status == TrustStatus::Active
                && trust.confidence_effective >= ruleset.exploration_probe_min_confidence
                && trust.confidence_effective < ruleset.exploration_probe_max_confidence
            {
                let bucket = deterministic_bucket(&format!(
                    "{}:{}:{}",
                    trust.memory_id,
                    trust.version,
                    context_id.unwrap_or_default()
                ));
                if bucket <= ruleset.exploration_probe_budget {
                    include = true;
                    reason_codes.push("included.exploration.probe_bucket".to_string());
                } else {
                    reason_codes.push("excluded.exploration.probe_bucket".to_string());
                }
            } else {
                reason_codes.push("excluded.exploration.threshold_or_status".to_string());
            }
        }
    }

    GateDecision {
        memory_id: trust.memory_id,
        version: trust.version,
        include,
        confidence_effective: trust.confidence_effective,
        trust_status: trust.trust_status,
        capped,
        reason_codes,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum OutcomeWindowEntry {
    Success,
    Failure,
    Ignored,
    Unknown,
}

fn ruleset_for(
    version: u32,
    rulesets: &BTreeMap<u32, OutcomeRuleset>,
) -> Result<OutcomeRuleset, OutcomeError> {
    let ruleset = rulesets.get(&version).ok_or_else(|| {
        OutcomeError::Configuration(format!(
            "missing ruleset configuration for version {version}"
        ))
    })?;
    ruleset.validate()?;
    Ok(ruleset.clone())
}

fn apply_scored_event(
    confidence_raw: &mut f32,
    baseline: f32,
    base_weight: f32,
    ruleset: &OutcomeRuleset,
    event: &OutcomeEvent,
) {
    let severity_multiplier = ruleset.severity_multiplier(event.severity, event.escalated);
    *confidence_raw = clamp(
        *confidence_raw + (ruleset.alpha * base_weight * severity_multiplier),
        0.0,
        1.0,
    );
    *confidence_raw = clamp(
        baseline + (*confidence_raw - baseline) * (1.0 - ruleset.per_event_decay),
        0.0,
        1.0,
    );
}

fn push_window_entry(
    window: &mut VecDeque<OutcomeWindowEntry>,
    entry: OutcomeWindowEntry,
    window_size: usize,
) {
    window.push_back(entry);
    while window.len() > window_size {
        let _ = window.pop_front();
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn deterministic_bucket(input: &str) -> f32 {
    // Stable FNV-1a hash to avoid platform-randomized hashers.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let fraction = (hash as f64) / (u64::MAX as f64);
    fraction as f32
}

/// Parses an RFC3339 timestamp and requires UTC (`Z`) offset.
///
/// # Errors
/// Returns [`OutcomeError::Validation`] when parsing fails or an input
/// timestamp is not UTC.
pub fn parse_rfc3339_utc(value: &str) -> Result<OffsetDateTime, OutcomeError> {
    let parsed = OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|err| OutcomeError::Validation(format!("invalid RFC3339 timestamp: {err}")))?;

    if parsed.offset() != UtcOffset::UTC {
        return Err(OutcomeError::Validation(
            "timestamp MUST use UTC offset Z".to_string(),
        ));
    }

    Ok(parsed)
}

/// Formats a timestamp as RFC3339 after normalizing to UTC.
///
/// # Errors
/// Returns [`OutcomeError::Validation`] when formatting fails.
pub fn format_rfc3339(value: OffsetDateTime) -> Result<String, OutcomeError> {
    value
        .to_offset(UtcOffset::UTC)
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| {
            OutcomeError::Validation(format!("failed to format RFC3339 timestamp: {err}"))
        })
}

#[must_use]
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_offset(UtcOffset::UTC)
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn days_since(earlier: OffsetDateTime, later: OffsetDateTime) -> f32 {
    if later <= earlier {
        return 0.0;
    }

    let elapsed = later - earlier;
    elapsed.whole_seconds() as f32 / Duration::DAY.whole_seconds() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got error: {err}"),
        }
    }

    fn must_some<T>(value: Option<T>) -> T {
        match value {
            Some(inner) => inner,
            None => panic!("expected Some(..), got None"),
        }
    }

    fn fixture_memory_id() -> MemoryId {
        MemoryId(must_ok(Ulid::from_string("01J0SQQP7M70P6Y3R4T8D8G8M2")))
    }

    fn must_utc(value: &str) -> OffsetDateTime {
        must_ok(parse_rfc3339_utc(value))
    }

    fn fixture_event(seq: i64, event_type: OutcomeEventType) -> OutcomeEvent {
        OutcomeEvent {
            event_seq: seq,
            event_id: Ulid::new(),
            ruleset_version: 1,
            memory_id: fixture_memory_id(),
            version: 1,
            event_type,
            occurred_at: must_utc("2026-02-07T12:00:00Z"),
            recorded_at: must_utc("2026-02-07T12:00:00Z"),
            writer: "tester".to_string(),
            justification: "fixture".to_string(),
            context_id: Some("ctx-1".to_string()),
            edited: false,
            escalated: false,
            severity: None,
            manual_confidence: None,
            override_cap: false,
            payload_json: Value::Object(Map::default()),
        }
    }

    fn ruleset_map() -> BTreeMap<u32, OutcomeRuleset> {
        let mut map = BTreeMap::new();
        map.insert(1, OutcomeRuleset::v1());
        map
    }

    #[test]
    fn edited_success_uses_half_weight() {
        let mut success = fixture_event(1, OutcomeEventType::Success);
        success.edited = true;

        let trust = must_some(must_ok(project_memory_trust(&[success], &ruleset_map())));

        assert!(trust.confidence_raw > 0.5);
        assert!(trust.confidence_raw < 0.6);
    }

    #[test]
    fn contradiction_applies_cap_without_override() {
        let success = fixture_event(1, OutcomeEventType::Success);
        let contradiction = fixture_event(2, OutcomeEventType::AuthoritativeContradiction);

        let trust = must_some(must_ok(project_memory_trust(
            &[success, contradiction],
            &ruleset_map(),
        )));

        assert!(trust.contradiction_cap_active);
        assert!(trust.confidence_effective <= 0.40);
    }

    #[test]
    fn validated_requires_three_wins_and_zero_failures() {
        let events = vec![
            fixture_event(1, OutcomeEventType::Success),
            fixture_event(2, OutcomeEventType::Success),
            fixture_event(3, OutcomeEventType::Success),
        ];

        let trust = must_some(must_ok(project_memory_trust(&events, &ruleset_map())));
        assert_eq!(trust.trust_status, TrustStatus::Validated);
    }

    #[test]
    fn manual_retire_is_sticky_until_promote() {
        let events = vec![
            fixture_event(1, OutcomeEventType::Success),
            fixture_event(2, OutcomeEventType::ManualRetire),
        ];

        let trust = must_some(must_ok(project_memory_trust(&events, &ruleset_map())));
        assert_eq!(trust.trust_status, TrustStatus::Retired);
    }

    #[test]
    fn manual_promote_requires_reearning_validation() {
        let events = vec![
            fixture_event(1, OutcomeEventType::Success),
            fixture_event(2, OutcomeEventType::Success),
            fixture_event(3, OutcomeEventType::Success),
            fixture_event(4, OutcomeEventType::ManualRetire),
            fixture_event(5, OutcomeEventType::ManualPromote),
        ];

        let trust = must_some(must_ok(project_memory_trust(&events, &ruleset_map())));
        assert_eq!(trust.trust_status, TrustStatus::Active);
        assert_eq!(trust.wins_last5, 0);
        assert_eq!(trust.failures_last5, 0);
    }

    #[test]
    fn manual_override_can_bypass_contradiction_cap() {
        let success = fixture_event(1, OutcomeEventType::Success);
        let contradiction = fixture_event(2, OutcomeEventType::AuthoritativeContradiction);
        let mut override_event = fixture_event(3, OutcomeEventType::ManualSetConfidence);
        override_event.manual_confidence = Some(0.90);
        override_event.override_cap = true;

        let trust = must_some(must_ok(project_memory_trust(
            &[success, contradiction, override_event],
            &ruleset_map(),
        )));

        assert!(trust.contradiction_cap_active);
        assert!(trust.manual_override_active);
        assert!(trust.confidence_effective > 0.40);
    }

    #[test]
    fn inheritance_resets_baseline_with_cap() {
        let mut inherited = fixture_event(1, OutcomeEventType::Inherited);
        inherited.manual_confidence = Some(0.95);

        let trust = must_some(must_ok(project_memory_trust(&[inherited], &ruleset_map())));

        assert!((trust.baseline_confidence - 0.665).abs() < 0.0001);
        assert_eq!(trust.trust_status, TrustStatus::Active);
    }

    #[test]
    fn unknown_events_do_not_change_confidence() {
        let unknown = fixture_event(1, OutcomeEventType::Unknown);
        let trust = must_some(must_ok(project_memory_trust(&[unknown], &ruleset_map())));
        assert!((trust.confidence_raw - 0.50).abs() < 0.0001);
    }

    #[test]
    fn safe_gate_excludes_capped_items() {
        let trust = MemoryTrust {
            memory_id: fixture_memory_id(),
            version: 1,
            confidence_raw: 0.9,
            confidence_effective: 0.4,
            baseline_confidence: 0.5,
            trust_status: TrustStatus::Validated,
            contradiction_cap_active: true,
            cap_value: 0.4,
            manual_override_active: false,
            wins_last5: 3,
            failures_last5: 0,
            last_event_seq: 10,
            last_scored_at: Some(must_utc("2026-02-07T12:00:00Z")),
            updated_at: must_utc("2026-02-07T12:00:00Z"),
        };

        let decision = gate_memory(
            &trust,
            RetrievalMode::Safe,
            Some("ctx-1"),
            &OutcomeRuleset::v1(),
        );
        assert!(!decision.include);
    }

    #[test]
    fn exploration_probe_bucket_is_deterministic() {
        let trust = MemoryTrust {
            memory_id: fixture_memory_id(),
            version: 1,
            confidence_raw: 0.2,
            confidence_effective: 0.2,
            baseline_confidence: 0.5,
            trust_status: TrustStatus::Active,
            contradiction_cap_active: false,
            cap_value: 1.0,
            manual_override_active: false,
            wins_last5: 0,
            failures_last5: 1,
            last_event_seq: 10,
            last_scored_at: Some(must_utc("2026-02-07T12:00:00Z")),
            updated_at: must_utc("2026-02-07T12:00:00Z"),
        };

        let ruleset = OutcomeRuleset::v1();
        let first = gate_memory(&trust, RetrievalMode::Exploration, Some("ctx-a"), &ruleset);
        let second = gate_memory(&trust, RetrievalMode::Exploration, Some("ctx-a"), &ruleset);
        assert_eq!(first.include, second.include);
    }

    #[test]
    fn as_of_decay_moves_toward_baseline() {
        let trust = MemoryTrust {
            memory_id: fixture_memory_id(),
            version: 1,
            confidence_raw: 0.9,
            confidence_effective: 0.9,
            baseline_confidence: 0.5,
            trust_status: TrustStatus::Active,
            contradiction_cap_active: false,
            cap_value: 1.0,
            manual_override_active: false,
            wins_last5: 3,
            failures_last5: 0,
            last_event_seq: 2,
            last_scored_at: Some(must_utc("2026-02-01T00:00:00Z")),
            updated_at: must_utc("2026-02-01T00:00:00Z"),
        };

        let decayed = apply_as_of_decay(
            &trust,
            &OutcomeRuleset::v1(),
            must_utc("2026-02-07T00:00:00Z"),
        );

        assert!(decayed.confidence_raw < trust.confidence_raw);
        assert!(decayed.confidence_raw > trust.baseline_confidence);
    }
}
