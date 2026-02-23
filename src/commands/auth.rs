use anyhow::{bail, Result};
use colored::*;
use dialoguer::Input;

use crate::api::ApiClient;
use crate::config::Config;
use crate::i18n::t;

/// `collected auth login` — Interactive login flow
pub async fn login() -> Result<()> {
    let mut config = Config::load()?;

    println!();
    println!("  {} collected.cards CLI Login", "🃏".to_string());
    println!("  {}", "━".repeat(40).dimmed());
    println!();
    println!("  {}", t("auth.login_step1"));
    println!();
    println!("     {}", "https://collected.cards/settings/cli".cyan().underline());
    println!();
    println!("  {}", t("auth.login_step2"));
    println!("  {} {} {}", t("auth.login_step3"), t("auth.generate_token").bold(), t("auth.login_step3b"));
    println!();

    let token: String = Input::new()
        .with_prompt(format!("  {}", t("auth.login_prompt")))
        .interact_text()?;

    let token = token.trim().to_string();
    if token.is_empty() {
        bail!("{}", t("auth.no_token"));
    }

    // Basic format sanity check
    if token.len() < 10 || token.contains(' ') || token.contains('\n') {
        bail!("{}", t("auth.invalid_format"));
    }

    println!();
    println!("  {} {}", "⏳".to_string(), t("auth.token_checking"));

    // Validate token BEFORE saving — test with a temporary ApiClient
    let mut temp_config = Config::default();
    temp_config.api.endpoint = config.api.endpoint.clone();
    temp_config.auth.token = Some(token.clone());
    let api = ApiClient::new(&temp_config)?;

    match api.query("{ me { id username } }", None).await {
        Ok(data) => {
            let username = data["me"]["username"].as_str().unwrap_or("?");

            // Token validated — NOW save it
            config.set_token(token)?;

            println!(
                "  {} {} {}",
                "✅".to_string(),
                t("auth.logged_in_as"),
                username.green().bold()
            );
            println!();
            println!(
                "  {}: {}",
                t("auth.token_saved_at"),
                Config::config_path()?.display().to_string().dimmed()
            );
            println!();
        }
        Err(_) => {
            // Don't save invalid token, don't leak API error details
            bail!("{}", t("auth.token_invalid"));
        }
    }

    Ok(())
}

/// `collected auth logout`
pub async fn logout() -> Result<()> {
    let mut config = Config::load()?;
    config.clear_token()?;
    println!("  {} {}", "👋".to_string(), t("auth.logged_out"));
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
                    println!("  {} {} {} ({})", "✅".to_string(), t("auth.logged_in_as"), username.green().bold(), id.dimmed());
                }
                Err(_) => {
                    println!("  {} {}", "⚠️".to_string(), t("auth.token_stored_invalid"));
                    println!("  {} {} {}", t("auth.use_login"), "collected auth login".cyan(), t("auth.to_relogin"));
                }
            }
        }
        None => {
            println!("  {} {}", "❌".to_string(), t("auth.not_logged_in"));
            println!("  {} {} {}", t("auth.use_login"), "collected auth login".cyan(), t("auth.to_login"));
        }
    }

    println!();
    println!("  Config: {}", Config::config_path()?.display().to_string().dimmed());
    println!("  API:    {}", config.api.endpoint.dimmed());
    Ok(())
}
