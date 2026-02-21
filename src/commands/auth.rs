use anyhow::{bail, Result};
use colored::*;
use dialoguer::Input;

use crate::api::ApiClient;
use crate::config::Config;

/// `collected auth login` — Interactive login flow
pub async fn login() -> Result<()> {
    let mut config = Config::load()?;

    println!();
    println!("  {} collected.cards CLI Login", "🃏".to_string());
    println!("  {}", "━".repeat(40).dimmed());
    println!();
    println!("  1. Öffne den folgenden Link im Browser:");
    println!();
    println!("     {}", "https://collected.cards/settings/cli".cyan().underline());
    println!();
    println!("  2. Melde dich an (falls nötig)");
    println!("  3. Klicke auf {} und kopiere den Token", "\"Token generieren\"".bold());
    println!();

    // Wait for user to paste the token
    let token: String = Input::new()
        .with_prompt("  Token hier einfügen")
        .interact_text()?;

    let token = token.trim().to_string();
    if token.is_empty() {
        bail!("Kein Token eingegeben.");
    }

    // Validate token format: {uuid}.{hex}.{hex}
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        bail!("Ungültiges Token-Format. Erwartet: UUID.timestamp.signatur");
    }

    // Test the token against the API
    println!();
    println!("  {} Token wird überprüft...", "⏳".to_string());

    config.set_token(token)?;
    let api = ApiClient::new(&config)?;

    let result = api
        .query(
            "{ me { id username } }",
            None,
        )
        .await;

    match result {
        Ok(data) => {
            let username = data["me"]["username"]
                .as_str()
                .unwrap_or("unbekannt");
            println!(
                "  {} Angemeldet als {}",
                "✅".to_string(),
                username.green().bold()
            );
            println!();
            println!(
                "  Token gespeichert in: {}",
                Config::config_path()?.display().to_string().dimmed()
            );
            println!();
        }
        Err(e) => {
            // Remove invalid token
            config.clear_token()?;
            bail!("Token ungültig oder abgelaufen: {}", e);
        }
    }

    Ok(())
}

/// `collected auth logout`
pub async fn logout() -> Result<()> {
    let mut config = Config::load()?;
    config.clear_token()?;
    println!("  {} Abgemeldet. Token entfernt.", "👋".to_string());
    Ok(())
}

/// `collected auth status`
pub async fn status() -> Result<()> {
    let config = Config::load()?;

    match config.get_token() {
        Some(_) => {
            let api = ApiClient::new(&config)?;
            match api.query("{ me { id username } }", None).await {
                Ok(data) => {
                    let username = data["me"]["username"].as_str().unwrap_or("?");
                    let id = data["me"]["id"].as_str().unwrap_or("?");
                    println!("  {} Angemeldet als {} ({})", "✅".to_string(), username.green().bold(), id.dimmed());
                }
                Err(_) => {
                    println!("  {} Token gespeichert, aber ungültig/abgelaufen", "⚠️".to_string());
                    println!("  Nutze {} zum erneuten Anmelden", "collected auth login".cyan());
                }
            }
        }
        None => {
            println!("  {} Nicht angemeldet", "❌".to_string());
            println!("  Nutze {} zum Anmelden", "collected auth login".cyan());
        }
    }

    println!();
    println!("  Config: {}", Config::config_path()?.display().to_string().dimmed());
    println!("  API:    {}", config.api.endpoint.dimmed());
    Ok(())
}
