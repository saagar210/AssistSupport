#![forbid(unsafe_code)]

use anyhow::Result;
use multi_agent_center_domain::{
    hash_json, now_utc, ProposedMemoryWrite, ProviderCallRecord, StepOutputEnvelope, StepRequest,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;
use ulid::Ulid;

pub trait ProviderAdapter {
    fn provider_name(&self) -> &'static str;

    #[allow(clippy::missing_errors_doc)]
    fn invoke(&self, request: &StepRequest) -> Result<ProviderInvocation>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderInvocation {
    pub provider_call: ProviderCallRecord,
    pub output: StepOutputEnvelope,
    pub proposed_memory_writes: Vec<ProposedMemoryWrite>,
}

#[derive(Debug, Clone)]
pub struct MockProvider {
    adapter_version: String,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            adapter_version: "mock.v1".to_string(),
        }
    }
}

impl MockProvider {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn deterministic_token(&self, request: &StepRequest) -> String {
        let mut hasher = Sha256::new();
        hasher.update(request.input_hash.as_bytes());
        hasher.update(request.agent.provider.model_id.as_bytes());
        hasher.update(self.adapter_version.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl ProviderAdapter for MockProvider {
    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn invoke(&self, request: &StepRequest) -> Result<ProviderInvocation> {
        let started_at = now_utc();
        let token = self.deterministic_token(request);
        let request_json = build_request_json(request, self.provider_name(), &self.adapter_version);
        let request_hash = hash_json(&request_json)?;

        let msg = format!(
            "mock:{}:{}",
            request.step_key,
            token.chars().take(16).collect::<String>()
        );

        let output_payload = json!({
            "deterministic_token": token,
            "step_key": request.step_key,
            "agent_name": request.agent.agent_name,
            "context_packages": request.injected_context_packages.len(),
        });

        let response_json = json!({
            "message": msg,
            "payload": output_payload,
        });
        let response_hash = hash_json(&response_json)?;

        let ended_at = now_utc();
        let base_len = request
            .step_key
            .len()
            .saturating_add(request.agent.agent_name.len());
        let base_len_u64 = u64::try_from(base_len).unwrap_or(u64::MAX);
        let latency_ms = Some(5 + (base_len_u64 % 17));

        let provider_call = ProviderCallRecord {
            provider_call_id: Ulid::new(),
            provider_name: self.provider_name().to_string(),
            adapter_version: self.adapter_version.clone(),
            model_id: request.agent.provider.model_id.clone(),
            request_json,
            request_hash,
            response_json: response_json.clone(),
            response_hash,
            latency_ms,
            input_tokens: None,
            output_tokens: None,
            started_at,
            ended_at,
            status: "succeeded".to_string(),
            error_text: None,
        };

        let output = StepOutputEnvelope {
            message: msg,
            payload: response_json,
        };

        Ok(ProviderInvocation {
            provider_call,
            output,
            proposed_memory_writes: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpJsonProvider {
    adapter_version: String,
}

impl Default for HttpJsonProvider {
    fn default() -> Self {
        Self {
            adapter_version: "http_json.v1".to_string(),
        }
    }
}

impl HttpJsonProvider {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProviderAdapter for HttpJsonProvider {
    fn provider_name(&self) -> &'static str {
        "http_json"
    }

    fn invoke(&self, request: &StepRequest) -> Result<ProviderInvocation> {
        let config = HttpProviderConfig::from_provider_params(&request.agent.provider.params)?;
        let started_at = now_utc();
        let request_json = build_request_json(request, self.provider_name(), &self.adapter_version);
        let request_hash = hash_json(&request_json)?;

        let outbound_json = json!({
            "model_id": request.agent.provider.model_id,
            "request": request_json,
        });

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build();

        let mut req = agent
            .request("POST", &config.url)
            .set("content-type", "application/json");
        for (header, value) in &config.headers {
            req = req.set(header, value);
        }
        if let Some(token) = &config.auth_bearer_token {
            req = req.set("authorization", &format!("Bearer {token}"));
        }

        let (status, error_text, status_code, body_json) = match req.send_json(&outbound_json) {
            Ok(response) => {
                let code = response.status();
                let body: Value = response.into_json()?;
                ("succeeded".to_string(), None, code, body)
            }
            Err(ureq::Error::Status(code, response)) => {
                let body = match response.into_json::<Value>() {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                (
                    "failed".to_string(),
                    Some(format!("http status {code}")),
                    code,
                    body,
                )
            }
            Err(ureq::Error::Transport(err)) => {
                return Err(anyhow::anyhow!("http transport failure: {err}"));
            }
        };

        let response_json = json!({
            "status_code": status_code,
            "body": body_json,
        });
        let response_hash = hash_json(&response_json)?;
        let ended_at = now_utc();
        let latency_ms = {
            let millis = (ended_at - started_at).whole_milliseconds();
            if millis <= 0 {
                Some(0)
            } else {
                u64::try_from(millis).ok()
            }
        };

        let provider_call = ProviderCallRecord {
            provider_call_id: Ulid::new(),
            provider_name: self.provider_name().to_string(),
            adapter_version: self.adapter_version.clone(),
            model_id: request.agent.provider.model_id.clone(),
            request_json: outbound_json,
            request_hash,
            response_json: response_json.clone(),
            response_hash,
            latency_ms,
            input_tokens: None,
            output_tokens: None,
            started_at,
            ended_at,
            status: status.clone(),
            error_text,
        };

        let output_message = if status == "succeeded" {
            format!("http_json:{}:ok", request.step_key)
        } else {
            format!("http_json:{}:failed", request.step_key)
        };

        Ok(ProviderInvocation {
            provider_call,
            output: StepOutputEnvelope {
                message: output_message,
                payload: response_json,
            },
            proposed_memory_writes: Vec::new(),
        })
    }
}

fn build_request_json(request: &StepRequest, provider_name: &str, adapter_version: &str) -> Value {
    let selected_count: usize = request
        .injected_context_packages
        .iter()
        .map(|pkg| pkg.context_package.selected_items.len())
        .sum();
    json!({
        "provider_name": provider_name,
        "adapter_version": adapter_version,
        "run_id": request.run_id.to_string(),
        "step_id": request.step_id.to_string(),
        "step_key": request.step_key,
        "agent_name": request.agent.agent_name,
        "model_id": request.agent.provider.model_id,
        "task_payload": request.task_payload,
        "context_selected_count": selected_count,
        "trust_gate_count": request.trust_gate_attachments.len(),
        "input_hash": request.input_hash,
    })
}

#[derive(Debug, Clone)]
struct HttpProviderConfig {
    url: String,
    timeout_ms: u64,
    headers: BTreeMap<String, String>,
    auth_bearer_token: Option<String>,
}

impl HttpProviderConfig {
    fn from_provider_params(params: &Value) -> Result<Self> {
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("http_json provider requires params.url"))?
            .to_string();

        let method = params
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("POST")
            .to_ascii_uppercase();
        if method != "POST" {
            return Err(anyhow::anyhow!(
                "http_json provider only supports POST, got '{method}'"
            ));
        }

        let timeout_ms = params
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(30_000);

        let mut headers = BTreeMap::new();
        if let Some(raw_headers) = params.get("headers") {
            let obj = raw_headers
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("params.headers must be an object"))?;
            for (key, value) in obj {
                let str_value = value.as_str().ok_or_else(|| {
                    anyhow::anyhow!("params.headers values must be strings, key='{key}'")
                })?;
                headers.insert(key.clone(), str_value.to_string());
            }
        }

        let auth_bearer_token = if let Some(env_name) =
            params.get("auth_bearer_env").and_then(Value::as_str)
        {
            Some(std::env::var(env_name).map_err(|_| {
                anyhow::anyhow!("missing env var '{env_name}' required by params.auth_bearer_env")
            })?)
        } else {
            None
        };

        Ok(Self {
            url,
            timeout_ms,
            headers,
            auth_bearer_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpJsonProvider, MockProvider, ProviderAdapter};
    use memory_kernel_core::RecordType;
    use multi_agent_center_domain::{
        AgentDefinition, AgentPermissions, EffectivePermissions, ProviderBinding, RunId,
        StepConstraints, StepId, StepRequest,
    };
    use serde_json::json;

    fn fixture_request(provider_name: &str, params: serde_json::Value) -> StepRequest {
        let agent = AgentDefinition {
            agent_name: "agent".to_string(),
            role: "role".to_string(),
            provider: ProviderBinding {
                provider_name: provider_name.to_string(),
                model_id: "model-x".to_string(),
                params,
            },
            permissions: AgentPermissions {
                allowed_record_types: vec![RecordType::Constraint],
                allowed_tools: Vec::new(),
                max_context_items: Some(10),
                can_propose_memory_writes: false,
                fail_on_permission_prune: false,
            },
            default_instructions: vec!["do work".to_string()],
            metadata: std::collections::BTreeMap::default(),
        };
        StepRequest {
            run_id: RunId::new(),
            step_id: StepId::new(),
            step_key: "step_a".to_string(),
            as_of: time::OffsetDateTime::now_utc(),
            agent,
            task_payload: json!({"text":"hello"}),
            injected_context_packages: Vec::new(),
            trust_gate_attachments: Vec::new(),
            effective_permissions: EffectivePermissions {
                allowed_record_types: vec![RecordType::Constraint],
                allowed_tools: Vec::new(),
                max_context_items: Some(10),
                can_propose_memory_writes: false,
                fail_on_permission_prune: false,
            },
            constraints: StepConstraints::default(),
            input_hash: "fixture-input-hash".to_string(),
        }
    }

    #[test]
    fn mock_provider_output_is_stable_for_same_input() {
        let request = fixture_request("mock", json!({}));
        let provider = MockProvider::new();

        let first = provider.invoke(&request);
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());

        let second = provider.invoke(&request);
        assert!(second.is_ok());
        let second = second.unwrap_or_else(|_| unreachable!());

        assert_eq!(first.output, second.output);
        assert_eq!(
            first.provider_call.request_hash,
            second.provider_call.request_hash
        );
        assert_eq!(
            first.provider_call.response_hash,
            second.provider_call.response_hash
        );
    }

    #[test]
    fn http_provider_requires_url() {
        let request = fixture_request("http_json", json!({}));
        let provider = HttpJsonProvider::new();
        let result = provider.invoke(&request);
        assert!(result.is_err());
    }
}
