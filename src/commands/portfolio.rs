use anyhow::Result;
use colored::*;

use crate::api::ApiClient;
use crate::i18n::t;

/// `collected portfolio` — Show portfolio overview
pub async fn portfolio(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query(
            "{ 
                myCollections { 
                    id name entryCount 
                }
                platformStats {
                    totalValue
                }
            }",
            None,
        )
        .await?;

    let collections = data["myCollections"].as_array().cloned().unwrap_or_default();

    println!();
    println!("  {} {}", "📈", t("portfolio.title").bold().white());
    println!("  {}", "━".repeat(32).dimmed());

    if collections.is_empty() {
        println!("  {}", t("collection.no_collections"));
        return Ok(());
    }

    let mut total_cards = 0i64;
    for coll in &collections {
        let name = coll["name"].as_str().unwrap_or("?");
        let entries = coll["entryCount"].as_i64().unwrap_or(0);
        total_cards += entries;
        println!("  {} {} ({})", "📦", name.cyan(), format!("{} cards", entries).dimmed());
    }

    println!("  {}", "─".repeat(32).dimmed());
    println!("  {} {} · {} {}", 
        t("common.total"),
        format!("{}", total_cards).green().bold(),
        collections.len(),
        t("collection.collections"),
    );

    println!();
    Ok(())
}
