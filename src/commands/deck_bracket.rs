use anyhow::Result;
use colored::*;
use serde_json::json;
use serde::Deserialize;

use crate::api::ApiClient;
use crate::i18n::t;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeckInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BracketData {
    pub bracket: i32,
    pub game_changer: bool,
}

/// `collected deck bracket <name>` — Show bracket analysis for a deck
pub async fn deck_bracket_analysis(api: &ApiClient, deck_name: &str) -> Result<()> {
    api.require_auth()?;

    // First, find the deck by name
    let data = api
        .query(
            "{ myDecks { id name } }",
            None,
        )
        .await?;

    let decks: Vec<DeckInfo> = serde_json::from_value(data["myDecks"].clone()).unwrap_or_default();
    
    let deck = decks
        .iter()
        .find(|d| d.name.to_lowercase().contains(&deck_name.to_lowercase()) || d.id.starts_with(deck_name));

    let deck = match deck {
        Some(d) => d,
        None => {
            println!("  {} '{}'.", t("deck_bracket.not_found"), deck_name.yellow());
            println!("  {}:", t("deck.decks"));
            for d in &decks {
                println!("    • {}", d.name);
            }
            return Ok(());
        }
    };

    // Get bracket data for the deck
    let data = api
        .query(
            "query($deckId: ID!) { 
                deckBrackets(deckId: $deckId) { 
                    bracket gameChanger 
                } 
            }",
            Some(json!({ "deckId": deck.id })),
        )
        .await?;

    let brackets: Vec<BracketData> = serde_json::from_value(data["deckBrackets"].clone()).unwrap_or_default();
    
    if brackets.is_empty() {
        println!("  {}", t("deck_bracket.no_brackets"));
        return Ok(());
    }

    // Calculate bracket distribution
    let mut bracket_counts = std::collections::HashMap::new();
    let mut gc_count = 0;

    for bracket_data in &brackets {
        if bracket_data.game_changer {
            gc_count += 1;
        } else {
            *bracket_counts.entry(bracket_data.bracket).or_insert(0) += 1;
        }
    }

    // Determine overall deck bracket (mode or highest frequency)
    let deck_bracket = bracket_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&bracket, _)| bracket)
        .unwrap_or(2);

    let deck_bracket_name = match deck_bracket {
        1 => t("bracket.exhibition"),
        2 => t("bracket.core"),
        3 => t("bracket.upgraded"),
        4 => t("bracket.optimized"),
        5 => t("bracket.cedh"),
        6 => t("bracket.gc"),
        _ => "Unknown",
    };

    println!();
    println!("  {} {} — {}{}{} {}", 
        "🃏".to_string(),
        deck.name.bold().white(),
        "}".dimmed(),
        deck_bracket.to_string().bold(),
        "{".dimmed(),
        deck_bracket_name.cyan()
    );
    println!("  {}", "━".repeat(29).dimmed());

    // Show Game Changer cards first
    if gc_count > 0 {
        let bar_length = scale_bar_length(gc_count, total_cards(&bracket_counts, gc_count));
        let bar = "█".repeat(bar_length);
        println!("  {}{}{} {}  {} cards",
            "}".dimmed(),
            "GC".red().bold(),
            "{".dimmed(),
            bar.red(),
            gc_count.to_string().bold()
        );
    }

    // Show brackets in reverse order (6 to 1)
    for bracket in (1..=6).rev() {
        if let Some(&count) = bracket_counts.get(&bracket) {
            let bar_length = scale_bar_length(count, total_cards(&bracket_counts, gc_count));
            let bar = "█".repeat(bar_length);
            
            let colored_bar = match bracket {
                1 => bar.white(),
                2 => bar.green(),
                3 => bar.blue(),
                4 => bar.purple(),
                5 => bar.yellow(),
                6 => bar.red(),
                _ => bar.normal(),
            };
            
            println!("  {}{}{} {}  {} cards",
                "}".dimmed(),
                bracket.to_string().bold(),
                "{".dimmed(),
                colored_bar,
                count.to_string().bold()
            );
        }
    }

    println!();
    Ok(())
}

fn total_cards(bracket_counts: &std::collections::HashMap<i32, i32>, gc_count: i32) -> i32 {
    bracket_counts.values().sum::<i32>() + gc_count
}

fn scale_bar_length(count: i32, total: i32) -> usize {
    if total == 0 {
        return 0;
    }
    let max_width = 20;
    ((count as f32 / total as f32) * max_width as f32) as usize
}