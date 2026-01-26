//! Jira Cloud integration module
//! Phase 18: Added comment posting, timeout/retry configuration

use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use zeroize::Zeroize;

use crate::security::SecureString;

#[derive(Debug, Error)]
pub enum JiraError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Not configured")]
    NotConfigured,
    #[error("Authentication failed - check your API token")]
    AuthFailed,
    #[error("Rate limited - try again later")]
    RateLimited,
    #[error("Request timeout")]
    Timeout,
}

/// Jira request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraRequestConfig {
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Number of retries for transient errors (default: 2)
    pub max_retries: u32,
    /// Retry delay in milliseconds (default: 1000)
    pub retry_delay_ms: u64,
}

impl Default for JiraRequestConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 2,
            retry_delay_ms: 1000,
        }
    }
}

/// Comment visibility options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentVisibility {
    /// Visible to everyone
    Public,
    /// Internal note (Service Desk)
    Internal,
    /// Restricted to a specific role
    Role(String),
    /// Restricted to a specific group
    Group(String),
}

/// Jira configuration (stored in DB, token in Keychain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub base_url: String,
    pub email: String,
}

/// Jira ticket/issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub reporter: String,
    pub created: String,
    pub updated: String,
    pub issue_type: String,
}

/// Jira API client with secure token handling
/// Auth credentials are zeroed when the client is dropped
pub struct JiraClient {
    client: Client,
    base_url: String,
    auth_header: SecureString,
    config: JiraRequestConfig,
}

impl JiraClient {
    /// Create a new Jira client
    /// The api_token is immediately encoded and the original cleared
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Self {
        Self::with_config(base_url, email, api_token, JiraRequestConfig::default())
    }

    /// Create a new Jira client with custom request configuration
    pub fn with_config(base_url: &str, email: &str, api_token: &str, config: JiraRequestConfig) -> Self {
        let mut auth = format!("{}:{}", email, api_token);
        let auth_header = SecureString::new(format!("Basic {}", general_purpose::STANDARD.encode(&auth)));
        auth.zeroize(); // Clear the intermediate auth string

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header,
            config,
        }
    }

    /// Test the connection by fetching current user
    pub async fn test_connection(&self) -> Result<bool, JiraError> {
        let url = format!("{}/rest/api/3/myself", self.base_url);
        let resp = self.client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header.as_str())
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;

        Ok(resp.status().is_success())
    }

    /// Get a ticket by key (e.g., "HELP-123")
    pub async fn get_ticket(&self, ticket_key: &str) -> Result<JiraTicket, JiraError> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary,description,status,priority,assignee,reporter,created,updated,issuetype",
            self.base_url, ticket_key
        );

        let resp = self.client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header.as_str())
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            // Log size only to avoid leaking sensitive data in responses
            if let Ok(body) = resp.text().await {
                tracing::debug!(
                    "Jira API error response received (status: {}, bytes: {})",
                    status,
                    body.len()
                );
            }
            return Err(JiraError::Api(format!("HTTP {} error fetching ticket", status)));
        }

        let json: serde_json::Value = resp.json().await?;

        // Parse the Jira API response
        let fields = &json["fields"];

        // Description can be in Atlassian Document Format (ADF) or plain text
        let description = Self::parse_description(fields);

        Ok(JiraTicket {
            key: json["key"].as_str().unwrap_or("").to_string(),
            summary: fields["summary"].as_str().unwrap_or("").to_string(),
            description,
            status: fields["status"]["name"].as_str().unwrap_or("").to_string(),
            priority: fields["priority"]["name"].as_str().map(|s| s.to_string()),
            assignee: fields["assignee"]["displayName"].as_str().map(|s| s.to_string()),
            reporter: fields["reporter"]["displayName"].as_str().unwrap_or("").to_string(),
            created: fields["created"].as_str().unwrap_or("").to_string(),
            updated: fields["updated"].as_str().unwrap_or("").to_string(),
            issue_type: fields["issuetype"]["name"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Parse description from Jira's Atlassian Document Format (ADF)
    fn parse_description(fields: &serde_json::Value) -> Option<String> {
        let desc = &fields["description"];

        if desc.is_null() {
            return None;
        }

        // If it's a string (older API), return directly
        if let Some(s) = desc.as_str() {
            return Some(s.to_string());
        }

        // If it's ADF format, extract text from content blocks
        if let Some(content) = desc["content"].as_array() {
            let mut text_parts: Vec<String> = Vec::new();

            for block in content {
                Self::extract_text_from_block(block, &mut text_parts);
            }

            if !text_parts.is_empty() {
                return Some(text_parts.join("\n"));
            }
        }

        None
    }

    /// Recursively extract text from ADF blocks
    fn extract_text_from_block(block: &serde_json::Value, parts: &mut Vec<String>) {
        // Handle text nodes
        if let Some(text) = block["text"].as_str() {
            parts.push(text.to_string());
            return;
        }

        // Handle blocks with content
        if let Some(content) = block["content"].as_array() {
            for child in content {
                Self::extract_text_from_block(child, parts);
            }
        }
    }

    /// Add a comment to a Jira ticket
    pub async fn add_comment(
        &self,
        ticket_key: &str,
        body: &str,
        visibility: Option<CommentVisibility>,
    ) -> Result<String, JiraError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/comment",
            self.base_url, ticket_key
        );

        // Build comment body in ADF format
        let mut comment_body = serde_json::json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": body
                            }
                        ]
                    }
                ]
            }
        });

        // Add visibility restriction if specified
        if let Some(vis) = visibility {
            match vis {
                CommentVisibility::Public => {
                    // No restriction needed
                }
                CommentVisibility::Internal => {
                    // Service Desk internal note
                    comment_body["properties"] = serde_json::json!([
                        {
                            "key": "sd.public.comment",
                            "value": {"internal": true}
                        }
                    ]);
                }
                CommentVisibility::Role(role) => {
                    comment_body["visibility"] = serde_json::json!({
                        "type": "role",
                        "value": role
                    });
                }
                CommentVisibility::Group(group) => {
                    comment_body["visibility"] = serde_json::json!({
                        "type": "group",
                        "value": group
                    });
                }
            }
        }

        // Execute with retry logic
        let response = self.execute_with_retry(|| async {
            self.client
                .post(&url)
                .header(header::AUTHORIZATION, self.auth_header.as_str())
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json")
                .json(&comment_body)
                .send()
                .await
        }).await?;

        let json: serde_json::Value = response.json().await?;
        let comment_id = json["id"].as_str().unwrap_or("").to_string();

        Ok(comment_id)
    }

    /// Add a comment with KB source citations
    pub async fn add_comment_with_citations(
        &self,
        ticket_key: &str,
        response_text: &str,
        citations: &[KbCitation],
        visibility: Option<CommentVisibility>,
    ) -> Result<String, JiraError> {
        // Build formatted comment with citations
        let mut formatted_body = response_text.to_string();

        if !citations.is_empty() {
            formatted_body.push_str("\n\n---\nSources:\n");
            for (i, citation) in citations.iter().enumerate() {
                let source_line = match &citation.url {
                    Some(url) => format!("[{}] {} - {}\n", i + 1, citation.title, url),
                    None => format!("[{}] {}\n", i + 1, citation.title),
                };
                formatted_body.push_str(&source_line);
            }
        }

        self.add_comment(ticket_key, &formatted_body, visibility).await
    }

    /// Execute a request with retry logic for transient errors
    /// Does NOT retry on auth errors (401/403)
    async fn execute_with_retry<F, Fut>(&self, request_fn: F) -> Result<reqwest::Response, JiraError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match request_fn().await {
                Ok(response) => {
                    let status = response.status();

                    // Auth errors: fail immediately, no retry
                    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
                        return Err(JiraError::AuthFailed);
                    }

                    // Rate limited: fail immediately with specific error
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        return Err(JiraError::RateLimited);
                    }

                    // Success
                    if status.is_success() {
                        return Ok(response);
                    }

                    // Server errors (5xx): retry
                    if status.is_server_error() {
                        last_error = Some(JiraError::Api(format!("Server error: {}", status)));
                        if attempt < self.config.max_retries {
                            tokio::time::sleep(Duration::from_millis(
                                self.config.retry_delay_ms * (attempt as u64 + 1)
                            )).await;
                            continue;
                        }
                    }

                    // Other client errors: fail immediately
                    let body = response.text().await.unwrap_or_default();
                    return Err(JiraError::Api(format!("HTTP {}: {}", status, body)));
                }
                Err(e) => {
                    // Timeout: wrap in specific error
                    if e.is_timeout() {
                        last_error = Some(JiraError::Timeout);
                    } else if e.is_connect() || e.is_request() {
                        // Connection errors: retry
                        last_error = Some(JiraError::Request(e));
                    } else {
                        // Other errors: fail immediately
                        return Err(JiraError::Request(e));
                    }

                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(
                            self.config.retry_delay_ms * (attempt as u64 + 1)
                        )).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(JiraError::Api("Unknown error".to_string())))
    }
}

/// KB source citation for Jira comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbCitation {
    pub title: String,
    pub url: Option<String>,
    pub chunk_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_description_null() {
        let fields = serde_json::json!({
            "description": null
        });
        assert_eq!(JiraClient::parse_description(&fields), None);
    }

    #[test]
    fn test_parse_description_string() {
        let fields = serde_json::json!({
            "description": "Simple description"
        });
        assert_eq!(JiraClient::parse_description(&fields), Some("Simple description".to_string()));
    }

    #[test]
    fn test_parse_description_adf() {
        let fields = serde_json::json!({
            "description": {
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            { "type": "text", "text": "First paragraph" }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "content": [
                            { "type": "text", "text": "Second paragraph" }
                        ]
                    }
                ]
            }
        });
        assert_eq!(JiraClient::parse_description(&fields), Some("First paragraph\nSecond paragraph".to_string()));
    }
}
