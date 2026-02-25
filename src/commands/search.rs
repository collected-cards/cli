use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde_json::json;

use crate::api::{self, ApiClient, CardResult};
use crate::display;
use crate::i18n::t;

/// `collected search <query>` — Search cards via Meilisearch
pub async fn search(api: &ApiClient, query: &str, tcg: Option<&str>, limit: Option<i32>, no_cache: bool) -> Result<()> {
    let limit = limit.unwrap_or(20);

    // Cache key based on query parameters
    let cache_key = format!("search_{}_{:?}_{}", query, tcg, limit);
    
    let data = if !no_cache {
        // Try cache first
        if let Ok(cache) = crate::cache::Cache::new() {
            if let Some(cached_data) = cache.get(&cache_key) {
                cached_data
            } else {
                let data = api
                    .query(
                        "query($tcg: String, $q: String!, $limit: Int) {
                            searchCards(tcg: $tcg, query: $q, limit: $limit) {
                                id name setCode setName collectorNumber rarity imageUrl currentPrice bracket gameChanger
                            }
                        }",
                        Some(json!({ "q": query, "tcg": tcg, "limit": limit })),
                    )
                    .await?;
                
                // Cache the result
                let _ = cache.set(&cache_key, &data);
                data
            }
        } else {
            // Fall back to direct API call if cache fails
            api
                .query(
                    "query($tcg: String, $q: String!, $limit: Int) {
                        searchCards(tcg: $tcg, query: $q, limit: $limit) {
                            id name setCode setName collectorNumber rarity imageUrl currentPrice bracket gameChanger
                        }
                    }",
                    Some(json!({ "q": query, "tcg": tcg, "limit": limit })),
                )
                .await?
        }
    } else {
        // Direct API call (no cache)
        api
            .query(
                "query($tcg: String, $q: String!, $limit: Int) {
                    searchCards(tcg: $tcg, query: $q, limit: $limit) {
                        id name setCode setName collectorNumber rarity imageUrl currentPrice bracket gameChanger
                    }
                }",
                Some(json!({ "q": query, "tcg": tcg, "limit": limit })),
            )
            .await?
    };

    let cards: Vec<CardResult> =
        serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    if cards.is_empty() {
        println!("  {} {}", t("search.no_results"), query.yellow());
        return Ok(());
    }

    println!(
        "  {} {} {} {}",
        cards.len().to_string().green().bold(),
        if cards.len() == 1 { t("search.results_count") } else { t("search.results_count_plural") },
        t("search.for"),
        query.yellow().bold()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["#", t("header.name"), t("header.set"), t("header.number"), t("header.rarity"), t("header.price")]);

    for (i, card) in cards.iter().enumerate() {
        let price_with_bracket = format_price_with_bracket(card.current_price, &card);
        table.add_row(vec![
            (i + 1).to_string(),
            card.name.clone(),
            card.set_code.clone().unwrap_or_default().to_uppercase(),
            card.collector_number.clone().unwrap_or_default(),
            card.rarity.clone().unwrap_or_default(),
            price_with_bracket,
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "  {} {}: {}",
        "💡".to_string(),
        t("search.detail_hint"),
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
            println!("  {} {}", t("search.no_card_found"), query.yellow());
            return Ok(());
        }
    };

    // Enrich with tcgcache data (oracle text, type line, mana cost)
    let enriched = if tcg == "mtg" {
        api::fetch_tcgcache_detail(card).await
    } else {
        None
    };
    let display_card = enriched.as_ref().unwrap_or(card);

    if show_art {
        if let Some(ref url) = card.image_url {
            let full_url = if url.starts_with("http") {
                url.clone()
            } else {
                format!("https://collected.cards{}", url)
            };
            let _ = display::show_card_image(&full_url, image_mode).await;
        }
        display::print_card_detail(display_card);
    } else {
        // Default: show virtual card (readable text card)
        display::print_virtual_card(display_card);
    }

    Ok(())
}

/// Format price with bracket information
fn format_price_with_bracket(price: Option<f64>, card: &CardResult) -> String {
    let price_str = display::format_price(price);
    
    if card.game_changer.unwrap_or(false) {
        format!("{} {}", price_str, "⚠ Game Changer".red())
    } else if let Some(bracket) = card.bracket {
        if bracket > 0 {
            let bracket_str = format!("}}{}{{", bracket);
            let colored_bracket = match bracket {
                1 => bracket_str.white(),
                2 => bracket_str.green(), 
                3 => bracket_str.blue(),
                4 => bracket_str.purple(),
                5 => bracket_str.yellow(),
                6 => bracket_str.red(),
                _ => bracket_str.normal(),
            };
            format!("{} {}", price_str, colored_bracket)
        } else {
            price_str
        }
    } else {
        price_str
    }
}
