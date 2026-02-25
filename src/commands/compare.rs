use anyhow::Result;
use colored::*;
use serde_json::json;

use crate::api::{ApiClient, CardResult};
use crate::display;
use crate::i18n::t;

/// `collected compare <card1> <card2>` — Compare two cards side by side
pub async fn compare_cards(api: &ApiClient, card1_query: &str, card2_query: &str, tcg: Option<&str>) -> Result<()> {
    // Search for both cards
    let card1_data = api
        .query(
            "query($tcg: String!, $q: String!) {
                searchCards(tcg: $tcg, query: $q, limit: 1) {
                    id name setCode setName collectorNumber rarity currentPrice bracket gameChanger
                }
            }",
            Some(json!({ "tcg": tcg.unwrap_or("mtg"), "q": card1_query })),
        )
        .await?;

    let card2_data = api
        .query(
            "query($tcg: String!, $q: String!) {
                searchCards(tcg: $tcg, query: $q, limit: 1) {
                    id name setCode setName collectorNumber rarity currentPrice bracket gameChanger
                }
            }",
            Some(json!({ "tcg": tcg.unwrap_or("mtg"), "q": card2_query })),
        )
        .await?;

    let cards1: Vec<CardResult> = serde_json::from_value(card1_data["searchCards"].clone()).unwrap_or_default();
    let cards2: Vec<CardResult> = serde_json::from_value(card2_data["searchCards"].clone()).unwrap_or_default();

    let card1 = match cards1.first() {
        Some(c) => c,
        None => {
            println!("  {} {}", t("search.no_card_found"), card1_query.yellow());
            return Ok(());
        }
    };

    let card2 = match cards2.first() {
        Some(c) => c,
        None => {
            println!("  {} {}", t("search.no_card_found"), card2_query.yellow());
            return Ok(());
        }
    };

    // Display comparison
    println!();
    println!("  {} — {} {} {}", 
        t("compare.title").bold().white(),
        card1.name.cyan(),
        t("compare.vs").dimmed(),
        card2.name.cyan()
    );
    
    let width = 25;
    let _total_width = 2 + width + 1 + width + 1; // borders + widths + separator
    
    println!("  ┌{}┬{}┐", "─".repeat(width), "─".repeat(width));
    
    // Names
    println!("  │ {:<width$} │ {:<width$} │", 
        truncate(&card1.name, width - 1),
        truncate(&card2.name, width - 1),
        width = width - 1
    );
    
    // Set and rarity
    let set1 = format!("{} · {}", 
        card1.set_code.as_deref().unwrap_or("???").to_uppercase(),
        card1.rarity.as_deref().unwrap_or("???")
    );
    let set2 = format!("{} · {}", 
        card2.set_code.as_deref().unwrap_or("???").to_uppercase(),
        card2.rarity.as_deref().unwrap_or("???")
    );
    
    println!("  │ {:<width$} │ {:<width$} │", 
        truncate(&set1, width - 1),
        truncate(&set2, width - 1),
        width = width - 1
    );
    
    // Price
    let price1 = display::format_price(card1.current_price);
    let price2 = display::format_price(card2.current_price);
    
    println!("  │ {:<width$} │ {:<width$} │", 
        truncate(&price1, width - 1),
        truncate(&price2, width - 1),
        width = width - 1
    );
    
    // Bracket
    let bracket1 = format_bracket_info(card1);
    let bracket2 = format_bracket_info(card2);
    
    println!("  │ {:<width$} │ {:<width$} │", 
        truncate(&bracket1, width - 1),
        truncate(&bracket2, width - 1),
        width = width - 1
    );
    
    println!("  └{}┴{}┘", "─".repeat(width), "─".repeat(width));
    println!();
    
    Ok(())
}

fn format_bracket_info(card: &CardResult) -> String {
    if card.game_changer.unwrap_or(false) {
        "⚠ Game Changer".to_string()
    } else if let Some(bracket) = card.bracket {
        if bracket > 0 {
            let bracket_name = match bracket {
                1 => t("bracket.exhibition"),
                2 => t("bracket.core"),
                3 => t("bracket.upgraded"),
                4 => t("bracket.optimized"),
                5 => t("bracket.cedh"),
                6 => t("bracket.gc"),
                _ => "Unknown",
            };
            format!("}}{}{{ {}", bracket, bracket_name)
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}