use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde_json::json;

use crate::api::{ApiClient, CollectionEntry, CollectionInfo};
use crate::display;

/// `collected collections` — List user's collections
pub async fn list_collections(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query("{ myCollections { id name tcgSlug entryCount } }", None)
        .await?;

    let collections: Vec<CollectionInfo> =
        serde_json::from_value(data["myCollections"].clone()).unwrap_or_default();

    if collections.is_empty() {
        println!("  Keine Sammlungen vorhanden.");
        println!("  Erstelle eine auf {}", "https://collected.cards/collection".cyan());
        return Ok(());
    }

    println!(
        "  {} Sammlung{}",
        collections.len().to_string().green().bold(),
        if collections.len() == 1 { "" } else { "en" }
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Name", "TCG", "Karten"]);

    for c in &collections {
        table.add_row(vec![
            c.id[..8].to_string(),
            c.name.clone(),
            c.tcg_slug.clone().unwrap_or_default().to_uppercase(),
            c.entry_count.unwrap_or(0).to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// `collected collection <name|id>` — Show collection entries
pub async fn show_collection(
    api: &ApiClient,
    name_or_id: &str,
    sort: Option<&str>,
    limit: Option<i32>,
) -> Result<()> {
    api.require_auth()?;

    let limit = limit.unwrap_or(50);

    // First get collections to find the right one
    let data = api
        .query("{ myCollections { id name tcgSlug entryCount } }", None)
        .await?;

    let collections: Vec<CollectionInfo> =
        serde_json::from_value(data["myCollections"].clone()).unwrap_or_default();

    let collection = collections
        .iter()
        .find(|c| {
            c.id.starts_with(name_or_id)
                || c.name.to_lowercase().contains(&name_or_id.to_lowercase())
        });

    let collection = match collection {
        Some(c) => c,
        None => {
            println!("  Sammlung '{}' nicht gefunden.", name_or_id.yellow());
            println!("  Verfügbare Sammlungen:");
            for c in &collections {
                println!("    • {} ({})", c.name, c.id[..8].to_string().dimmed());
            }
            return Ok(());
        }
    };

    // Fetch entries
    let data = api
        .query(
            "query($id: ID!, $limit: Int) { 
                collectionEntries(collectionId: $id, limit: $limit) { 
                    id quantity foil condition language
                    card { name setCode setName collectorNumber rarity currentPrice foilPrice }
                } 
            }",
            Some(json!({
                "id": collection.id,
                "limit": limit,
            })),
        )
        .await?;

    let entries: Vec<CollectionEntry> =
        serde_json::from_value(data["collectionEntries"].clone()).unwrap_or_default();

    println!(
        "  {} — {} Karten",
        collection.name.bold().white(),
        entries.len().to_string().green()
    );

    let total_value: f64 = entries
        .iter()
        .map(|e| {
            let price = e.card.as_ref().and_then(|c| {
                if e.foil { c.foil_price.or(c.current_price) } else { c.current_price }
            });
            price.unwrap_or(0.0) * e.quantity as f64
        })
        .sum();

    if total_value > 0.0 {
        println!("  Wert: {}", format!("€{:.2}", total_value).green());
    }
    println!();

    if entries.is_empty() {
        println!("  Keine Karten in dieser Sammlung.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Qty", "Name", "Set", "Nr.", "Zustand", "Foil", "Preis"]);

    for e in &entries {
        let c = e.card.as_ref();
        let price = c.and_then(|c| {
            if e.foil { c.foil_price.or(c.current_price) } else { c.current_price }
        });
        table.add_row(vec![
            format!("{}x", e.quantity),
            c.and_then(|c| c.name.clone()).unwrap_or_default(),
            c.and_then(|c| c.set_code.clone()).unwrap_or_default().to_uppercase(),
            c.and_then(|c| c.collector_number.clone()).unwrap_or_default(),
            e.condition.clone().unwrap_or_default(),
            if e.foil { "✨" } else { "" }.to_string(),
            display::format_price(price),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// `collected stats` — Collection statistics
pub async fn stats(api: &ApiClient, tcg: Option<&str>) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query(
            "{ myCollectionStats { totalCards totalValue valueChange valueChangePercent collections { name cardCount totalValue } } }",
            None,
        )
        .await?;

    let stats = &data["myCollectionStats"];
    let collections = stats["collections"].as_array();

    println!();
    println!("  {} Deine Sammlung", "📊".to_string());
    println!("  {}", "━".repeat(30).dimmed());
    println!(
        "  Sammlungen:  {}",
        collections.map(|c| c.len()).unwrap_or(0).to_string().green()
    );
    println!(
        "  Karten:      {}",
        stats["totalCards"].as_i64().unwrap_or(0).to_string().green()
    );
    println!(
        "  Wert:        {}",
        format!(
            "€{:.2}",
            stats["totalValue"].as_f64().unwrap_or(0.0)
        )
        .green()
        .bold()
    );
    let change = stats["valueChange"].as_f64().unwrap_or(0.0);
    if change.abs() > 0.01 {
        let pct = stats["valueChangePercent"].as_f64().unwrap_or(0.0);
        let arrow = if change > 0.0 { "📈" } else { "📉" };
        println!(
            "  24h:         {} {}{:.2} ({:.1}%)",
            arrow,
            if change > 0.0 { "+" } else { "" },
            change,
            pct
        );
    }
    println!();

    Ok(())
}
