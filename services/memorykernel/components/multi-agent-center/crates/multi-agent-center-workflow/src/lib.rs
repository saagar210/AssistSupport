#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use multi_agent_center_domain::{
    ensure_non_empty, hash_bytes, hash_json, AgentDefinition, NormalizedWorkflow,
    NormalizedWorkflowEnvelope, WorkflowStepDefinition,
};

const NORMALIZATION_VERSION: u32 = 1;

/// Load workflow YAML from disk and normalize it into canonical internal form.
///
/// # Errors
/// Returns an error when the file cannot be read, parsed, validated, or normalized.
pub fn load_workflow_from_path(path: &Path) -> Result<NormalizedWorkflowEnvelope> {
    let content = fs::read_to_string(path)?;
    normalize_workflow_yaml(&content)
}

/// Normalize workflow YAML into deterministic canonical JSON + hash.
///
/// # Errors
/// Returns an error when YAML parsing, validation, or serialization fails.
pub fn normalize_workflow_yaml(yaml: &str) -> Result<NormalizedWorkflowEnvelope> {
    let source_yaml_hash = hash_bytes(yaml.as_bytes());
    let mut workflow: NormalizedWorkflow = serde_yaml::from_str(yaml)
        .map_err(|err| anyhow!("invalid workflow YAML structure: {err}"))?;

    validate_workflow(&workflow)?;
    normalize_workflow(&mut workflow);
    validate_workflow(&workflow)?;

    let normalized_json = serde_json::to_value(&workflow)?;
    let normalized_hash = hash_json(&normalized_json)?;

    Ok(NormalizedWorkflowEnvelope {
        source_format: "yaml".to_string(),
        source_yaml_hash,
        normalized_hash,
        normalized_workflow: workflow,
        normalized_json,
    })
}

fn validate_workflow(workflow: &NormalizedWorkflow) -> Result<()> {
    ensure_non_empty("workflow_name", &workflow.workflow_name)?;
    ensure_non_empty("workflow_version", &workflow.workflow_version)?;

    let mut agent_names = BTreeSet::new();
    for agent in &workflow.agents {
        validate_agent(agent)?;
        if !agent_names.insert(agent.agent_name.clone()) {
            return Err(anyhow!("duplicate agent_name: {}", agent.agent_name));
        }
    }

    let agent_name_set: BTreeSet<&str> = workflow
        .agents
        .iter()
        .map(|agent| agent.agent_name.as_str())
        .collect();

    let mut step_keys = BTreeSet::new();
    for step in &workflow.steps {
        validate_step(step)?;
        if !step_keys.insert(step.step_key.clone()) {
            return Err(anyhow!("duplicate step_key: {}", step.step_key));
        }
        if !agent_name_set.contains(step.agent_name.as_str()) {
            return Err(anyhow!(
                "step {} references unknown agent {}",
                step.step_key,
                step.agent_name
            ));
        }
    }

    let step_key_set: BTreeSet<&str> = workflow
        .steps
        .iter()
        .map(|step| step.step_key.as_str())
        .collect();
    for step in &workflow.steps {
        for dep in &step.depends_on {
            if !step_key_set.contains(dep.as_str()) {
                return Err(anyhow!(
                    "step {} depends_on unknown step {}",
                    step.step_key,
                    dep
                ));
            }
        }
    }

    let gate_name_set: BTreeSet<&str> = workflow
        .gates
        .iter()
        .map(|gate| gate.gate_name.as_str())
        .collect();
    for step in &workflow.steps {
        for gate_name in &step.gate_points {
            if !gate_name_set.contains(gate_name.as_str()) {
                return Err(anyhow!(
                    "step {} references unknown gate {}",
                    step.step_key,
                    gate_name
                ));
            }
        }
    }

    detect_cycle(workflow)?;

    Ok(())
}

fn validate_agent(agent: &AgentDefinition) -> Result<()> {
    ensure_non_empty("agent_name", &agent.agent_name)?;
    ensure_non_empty("role", &agent.role)?;
    ensure_non_empty("provider_name", &agent.provider.provider_name)?;
    ensure_non_empty("model_id", &agent.provider.model_id)?;
    Ok(())
}

fn validate_step(step: &WorkflowStepDefinition) -> Result<()> {
    ensure_non_empty("step_key", &step.step_key)?;
    ensure_non_empty("agent_name", &step.agent_name)?;
    Ok(())
}

fn normalize_workflow(workflow: &mut NormalizedWorkflow) {
    workflow.normalization_version = NORMALIZATION_VERSION;

    workflow
        .agents
        .sort_by(|lhs, rhs| lhs.agent_name.cmp(&rhs.agent_name));
    for agent in &mut workflow.agents {
        agent
            .permissions
            .allowed_record_types
            .sort_by(|lhs, rhs| lhs.as_str().cmp(rhs.as_str()));
        agent.permissions.allowed_tools.sort();
        agent.default_instructions.sort();
        let ordered_metadata: BTreeMap<String, String> =
            agent.metadata.clone().into_iter().collect();
        agent.metadata = ordered_metadata;
    }

    workflow
        .gates
        .sort_by(|lhs, rhs| lhs.gate_name.cmp(&rhs.gate_name));
    for gate in &mut workflow.gates {
        gate.gate_name = gate.gate_name.trim().to_string();
    }

    for step in &mut workflow.steps {
        step.depends_on.sort();
        step.depends_on.dedup();
        step.gate_points.sort();
        step.gate_points.dedup();
    }
}

fn detect_cycle(workflow: &NormalizedWorkflow) -> Result<()> {
    let mut remaining: BTreeMap<&str, BTreeSet<&str>> = workflow
        .steps
        .iter()
        .map(|step| {
            (
                step.step_key.as_str(),
                step.depends_on.iter().map(String::as_str).collect(),
            )
        })
        .collect();

    loop {
        let ready: Vec<&str> = remaining
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(key, _)| *key)
            .collect();
        if ready.is_empty() {
            break;
        }

        for key in ready {
            remaining.remove(key);
            for deps in remaining.values_mut() {
                deps.remove(key);
            }
        }
    }

    if remaining.is_empty() {
        return Ok(());
    }

    let mut keys: Vec<&str> = remaining.keys().copied().collect();
    keys.sort_unstable();
    Err(anyhow!(
        "workflow dependency cycle detected among steps: {}",
        keys.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::normalize_workflow_yaml;

    #[test]
    fn normalize_hash_is_stable() {
        let yaml = r"
workflow_name: test
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: b
    role: r
    provider:
      provider_name: mock
      model_id: m
    permissions:
      allowed_record_types: [decision, constraint]
      allowed_tools: [z, a]
      max_context_items: 5
      can_propose_memory_writes: false
      fail_on_permission_prune: false
    default_instructions: [b, a]
steps:
  - step_key: s1
    agent_name: b
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
";
        let first = normalize_workflow_yaml(yaml);
        let second = normalize_workflow_yaml(yaml);
        assert!(first.is_ok());
        assert!(second.is_ok());
        match (first, second) {
            (Ok(first), Ok(second)) => assert_eq!(first.normalized_hash, second.normalized_hash),
            _ => unreachable!(),
        }
    }
}
