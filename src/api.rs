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
            .build()?;

        Ok(Self {
            client,
            endpoint: config.api.endpoint.clone(),
            token: config.auth.token.clone(),
        })
    }

    pub fn require_auth(&self) -> Result<()> {
        if self.token.is_none() {
            bail!("Nicht angemeldet. Nutze 'collected auth login' zum Einloggen.");
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

        let resp = builder.send().await.context("API nicht erreichbar")?;
        let status = resp.status();
        if !status.is_success() {
            bail!("API-Fehler: HTTP {}", status);
        }

        let body: GqlResponse = resp.json().await.context("Ungültige API-Antwort")?;

        if let Some(errors) = body.errors {
            let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            bail!("API-Fehler: {}", msgs.join(", "));
        }

        body.data.context("Keine Daten in API-Antwort")
    }
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
