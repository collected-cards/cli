use anyhow::Result;
use colored::*;

use crate::api::ApiClient;
use crate::i18n::t;

/// `collected stats` — Platform and personal statistics
pub async fn platform_stats(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query(
            "{ 
                platformStats { 
                    totalCards totalEntries totalValue collectionCount
                } 
                myCollections { 
                    id name entryCount
                } 
            }",
            None,
        )
        .await?;

    let platform = &data["platformStats"];
    let my_collections = data["myCollections"].as_array().cloned().unwrap_or_default();

    println!();
    println!("  {} {}", "🃏", t("stats.title").bold().white());
    println!("  {}", "━".repeat(40).dimmed());
    
    // Platform stats
    let total_cards = platform["totalCards"].as_i64().unwrap_or(0);
    let total_entries = platform["totalEntries"].as_i64().unwrap_or(0);
    let total_value = platform["totalValue"].as_f64().unwrap_or(0.0);
    let collection_count = platform["collectionCount"].as_i64().unwrap_or(0);
    
    println!("  {} {}", "📊", t("stats.platform").bold());
    println!("     {}+ {} · {} {} · €{:.0}",
        format_large_number(total_cards).cyan(),
        t("common.cards"),
        total_entries.to_string().cyan(),
        t("stats.entries"),
        total_value,
    );
    println!("     {} {}",
        collection_count.to_string().cyan(),
        t("collection.collections"),
    );
    
    // My stats
    let my_total: i64 = my_collections.iter()
        .map(|c| c["entryCount"].as_i64().unwrap_or(0))
        .sum();
    
    println!();
    println!("  {} {}", "👤", t("stats.my_stats").bold());
    println!("     {} {} · {} {}",
        my_total.to_string().green(),
        t("common.cards"),
        my_collections.len().to_string().green(),
        t("collection.collections"),
    );
    
    println!();
    Ok(())
}

fn format_large_number(num: i64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{}k", num / 1_000)
    } else {
        num.to_string()
    }
}
