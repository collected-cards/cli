use anyhow::Result;
use colored::*;
use serde_json::json;

use crate::api::ApiClient;
use crate::i18n::t;

pub async fn settings(api: &ApiClient, email: Option<&str>, location: Option<&str>) -> Result<()> {
    api.require_auth()?;

    if let Some(email) = email {
        api.query(
            "mutation($email: String) { updateEmail(email: $email) }",
            Some(json!({ "email": email })),
        ).await?;
        println!("  {} {} {}", "✅".to_string(), t("settings.email_updated"), email.green());
        return Ok(());
    }

    if let Some(location) = location {
        api.query(
            "mutation($locationName: String) { upsertTradeProfile(locationName: $locationName) }",
            Some(json!({ "locationName": location })),
        ).await?;
        println!("  {} {} {}", "✅".to_string(), t("settings.location_updated"), location.green());
        return Ok(());
    }

    // Show current settings
    let data = api
        .query("{ me { id username email } myTradeProfile { locationName } }", None)
        .await?;

    println!();
    println!("  {} {}", "⚙️".to_string(), t("settings.current"));
    println!("  {}", "━".repeat(30).dimmed());

    let username = data["me"]["username"].as_str().unwrap_or("?");
    let email = data["me"]["email"].as_str().unwrap_or("—");
    let location = data["myTradeProfile"]["locationName"].as_str().unwrap_or("—");

    println!("  👤 {}", username.bold());
    println!("  {}: {}", t("settings.email"), email);
    println!("  {}: {}", t("settings.location"), location);
    println!();

    Ok(())
}
