use anyhow::Result;
use dialoguer::Confirm;
use serde_json::json;

use crate::api::ApiClient;
use crate::i18n::t;

pub async fn remove_entry(api: &ApiClient, entry_id: &str) -> Result<()> {
    api.require_auth()?;

    let confirmed = Confirm::new()
        .with_prompt(t("common.confirm_remove"))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("  {}", t("common.aborted"));
        return Ok(());
    }

    api.query(
        "mutation($entryId: ID!) { removeEntryFromCollection(entryId: $entryId) }",
        Some(json!({ "entryId": entry_id })),
    ).await?;

    println!("  {} {}", "✅".to_string(), t("common.removed"));

    Ok(())
}
