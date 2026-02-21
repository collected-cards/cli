use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde_json::json;

use crate::api::{ApiClient, CardResult};
use crate::display;

/// `collected search <query>` — Search cards via Meilisearch
pub async fn search(api: &ApiClient, query: &str, tcg: Option<&str>, limit: Option<i32>) -> Result<()> {
    let limit = limit.unwrap_or(20);
    
    let data = api
        .query(
            "query($tcg: String, $q: String!, $limit: Int) { 
                searchCards(tcg: $tcg, query: $q, limit: $limit) { 
                    id name setCode setName collectorNumber rarity imageUrl currentPrice
                } 
            }",
            Some(json!({
                "q": query,
                "tcg": tcg,
                "limit": limit
            })),
        )
        .await?;

    let cards: Vec<CardResult> =
        serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    if cards.is_empty() {
        println!("  Keine Karten gefunden für: {}", query.yellow());
        return Ok(());
    }

    println!(
        "  {} Ergebnis{} für {}",
        cards.len().to_string().green().bold(),
        if cards.len() == 1 { "" } else { "se" },
        query.yellow().bold()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["#", "Name", "Set", "Nr.", "Seltenheit", "Preis"]);

    for (i, card) in cards.iter().enumerate() {
        table.add_row(vec![
            (i + 1).to_string(),
            card.name.clone(),
            card.set_code.clone().unwrap_or_default().to_uppercase(),
            card.collector_number.clone().unwrap_or_default(),
            card.rarity.clone().unwrap_or_default(),
            display::format_price(card.current_price),
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "  {} für Details: {}",
        "💡".to_string(),
        "collected card <name>".cyan()
    );

    Ok(())
}

/// `collected card <name>` — Show card detail
pub async fn card_detail(api: &ApiClient, query: &str, tcg: Option<&str>, show_art: bool, image_mode: &str) -> Result<()> {
    let tcg = tcg.unwrap_or("mtg");
    let data = api
        .query(
            "query($tcg: String!, $q: String!) { 
                searchCards(tcg: $tcg, query: $q, limit: 1) { 
                    id name setCode setName collectorNumber rarity imageUrl 
                    currentPrice typeLine manaCost oracleText power toughness
                } 
            }",
            Some(json!({ "tcg": tcg, "q": query })),
        )
        .await?;

    let cards: Vec<CardResult> =
        serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    let card = match cards.first() {
        Some(c) => c,
        None => {
            println!("  Keine Karte gefunden für: {}", query.yellow());
            return Ok(());
        }
    };

    // Show card image if requested
    if show_art {
        if let Some(ref url) = card.image_url {
            let full_url = if url.starts_with("http") {
                url.clone()
            } else {
                format!("https://collected.cards{}", url)
            };
            match display::show_card_image(&full_url, image_mode).await {
                Ok(_) => {}
                Err(_) => {
                    // Fallback to ASCII
                    display::print_ascii_card(card);
                    return Ok(());
                }
            }
        }
    }

    display::print_card_detail(card);

    // If not showing art, offer ASCII card
    if !show_art {
        println!(
            "  {} Bild anzeigen: {}",
            "🎨".to_string(),
            format!("collected card \"{}\" --art", card.name).cyan()
        );
        println!();
    }

    Ok(())
}
