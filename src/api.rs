use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

pub struct ApiClient {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct GqlRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

#[derive(Deserialize)]
struct GqlResponse {
    data: Option<Value>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            // Enforce HTTPS certificate validation (default, but explicit)
            .danger_accept_invalid_certs(false)
            // Only allow HTTPS for the API endpoint
            .https_only(config.api.endpoint.starts_with("https://"))
            .build()?;

        Ok(Self {
            client,
            endpoint: config.api.endpoint.clone(),
            token: config.auth.token.clone(),
        })
    }

    pub fn require_auth(&self) -> Result<()> {
        if self.token.is_none() {
            bail!("{} — collected auth login", crate::i18n::t("auth.not_logged_in"));
        }
        Ok(())
    }

    pub async fn query(&self, query: &str, variables: Option<Value>) -> Result<Value> {
        let req = GqlRequest {
            query: query.to_string(),
            variables,
        };

        let mut builder = self.client.post(&self.endpoint).json(&req);
        if let Some(ref token) = self.token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        let resp = builder.send().await
            .map_err(|_| anyhow::anyhow!("{}", crate::i18n::t("common.api_unreachable")))?;
        let status = resp.status();
        if !status.is_success() {
            bail!("{} (HTTP {})", crate::i18n::t("common.api_error"), status.as_u16());
        }

        let body: GqlResponse = resp.json().await
            .map_err(|_| anyhow::anyhow!("{}", crate::i18n::t("common.api_invalid_response")))?;

        if let Some(errors) = body.errors {
            // Sanitize error messages: only show user-facing parts
            let msgs: Vec<_> = errors.iter().map(|e| {
                sanitize_error(&e.message)
            }).collect();
            bail!("{}", msgs.join(", "));
        }

        body.data.ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("common.api_no_data")))
    }
}

/// Return a clean, user-facing error message.
fn sanitize_error(msg: &str) -> String {
    let msg = msg.trim();
    let dominated_by_internals = ["SELECT ", "INSERT ", "DELETE ", "UPDATE ",
        "/opt/", "/app/", "panicked at", "sqlx::", "tokio::", "thread '"]
        .iter().any(|p| msg.contains(p));
    if dominated_by_internals {
        return crate::i18n::t("common.api_error").to_string();
    }
    if msg.len() > 200 {
        return format!("{}…", &msg[..200]);
    }
    msg.to_string()
}

// ─── Response Types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub is_admin: Option<bool>,
    pub is_dealer: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardResult {
    pub id: String,
    pub name: String,
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub collector_number: Option<String>,
    pub rarity: Option<String>,
    pub image_url: Option<String>,
    pub current_price: Option<f64>,
    pub foil_price: Option<f64>,
    pub type_line: Option<String>,
    pub mana_cost: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInfo {
    pub id: String,
    pub name: String,
    pub tcg_slug: Option<String>,
    pub entry_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionEntry {
    pub id: String,
    pub quantity: i32,
    pub foil: bool,
    pub condition: Option<String>,
    pub language: Option<String>,
    pub card: Option<CardInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInfo {
    pub name: Option<String>,
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub collector_number: Option<String>,
    pub rarity: Option<String>,
    pub current_price: Option<f64>,
    pub foil_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStats {
    pub total_cards: i64,
    pub total_users: i64,
    pub total_value: f64,
    pub total_listings: i64,
}
