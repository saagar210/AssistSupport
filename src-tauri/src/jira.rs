//! Jira Cloud integration module

use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
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
}

impl JiraClient {
    /// Create a new Jira client
    /// The api_token is immediately encoded and the original cleared
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Self {
        let mut auth = format!("{}:{}", email, api_token);
        let auth_header = SecureString::new(format!("Basic {}", general_purpose::STANDARD.encode(&auth)));
        auth.zeroize(); // Clear the intermediate auth string

        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header,
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
            let body = resp.text().await.unwrap_or_default();
            return Err(JiraError::Api(format!("HTTP {}: {}", status, body)));
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
