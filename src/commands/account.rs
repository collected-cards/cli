use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input};

use crate::api::ApiClient;
use crate::config::Config;
use crate::i18n::t;

pub async fn delete_account(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    // Get username
    let data = api.query("{ me { username } }", None).await?;
    let username = data["me"]["username"].as_str().unwrap_or("?");

    println!();
    println!("  {} ⚠️  {}", "🗑️".to_string(), "DANGER".red().bold());
    println!();
    println!("  {}", t("account.delete_warning"));
    println!();
    println!("  {} {}", "•".red(), t("account.delete_items_collections"));
    println!("  {} {}", "•".red(), t("account.delete_items_listings"));
    println!("  {} {}", "•".red(), t("account.delete_items_trades"));
    println!("  {} {}", "•".red(), t("account.delete_items_messages"));
    println!("  {} {}", "•".red(), t("account.delete_items_decks"));
    println!();

    // First confirmation: Are you sure?
    let first = Confirm::new()
        .with_prompt(format!("  {}", t("account.delete_confirm")))
        .default(false)
        .interact()?;

    if !first {
        println!("  {}", t("common.aborted"));
        return Ok(());
    }

    // Second confirmation: Type username
    println!();
    let input: String = Input::new()
        .with_prompt(format!("  {} ({})", t("account.type_username"), username.bold()))
        .interact_text()?;

    if input.trim() != username {
        println!("  {} {}", "❌".to_string(), t("account.mismatch"));
        return Ok(());
    }

    // Final confirmation
    let final_confirm = Confirm::new()
        .with_prompt(format!("  {} {}", "⚠️".to_string(), t("account.final_confirm")))
        .default(false)
        .interact()?;

    if !final_confirm {
        println!("  {}", t("common.aborted"));
        return Ok(());
    }

    match api.query("mutation { deleteUser }", None).await {
        Ok(_) => {
            // Clear local token
            let mut config = Config::load()?;
            config.clear_token()?;
            println!("  {} {}", "✅".to_string(), t("account.deleted"));
        }
        Err(_) => {
            println!("  {} {}", "❌".to_string(), t("account.delete_failed"));
        }
    }

    Ok(())
}
