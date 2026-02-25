use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::fs;

use crate::api::ApiClient;
use crate::i18n::t;

/// `collected bulk-add <file>` — Add cards from a text file
pub async fn bulk_add_cards(
    api: &ApiClient, 
    file_path: &str, 
    collection_name: Option<&str>,
    tcg: Option<&str>
) -> Result<()> {
    api.require_auth()?;
    
    println!("  {}", t("bulk_add.reading"));
    
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?;
    
    let lines: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    if lines.is_empty() {
        println!("  No cards found in file.");
        return Ok(());
    }
    
    // Parse lines into (quantity, card_name)
    let mut cards_to_add = Vec::new();
    
    for line in lines {
        let (quantity, card_name) = parse_card_line(line);
        cards_to_add.push((quantity, card_name));
    }
    
    println!("  Found {} cards to add.", cards_to_add.len());
    
    // Get collections to find target collection ID
    let collection_id = if let Some(collection_name) = collection_name {
        let data = api
            .query("{ myCollections { id name } }", None)
            .await?;
        
        let collections = data["myCollections"].as_array().cloned().unwrap_or_default();
        let collection = collections
            .iter()
            .find(|c| c["name"].as_str().unwrap_or("").to_lowercase().contains(&collection_name.to_lowercase()));
            
        match collection {
            Some(c) => c["id"].as_str().unwrap_or("").to_string(),
            None => {
                println!("  Collection '{}' not found.", collection_name.yellow());
                return Ok(());
            }
        }
    } else {
        // Use default collection or prompt user
        String::new()
    };
    
    // Progress bar
    let pb = ProgressBar::new(cards_to_add.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:40}] {pos}/{len} {msg}")
            .expect("progress bar template")
            .progress_chars("█▉▊▋▌▍▎▏ ")
    );
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (i, (quantity, card_name)) in cards_to_add.iter().enumerate() {
        pb.set_position(i as u64);
        pb.set_message(format!("{} {}...", t("bulk_add.adding"), card_name.yellow()));
        
        // Search for the card first
        let search_result = api
            .query(
                "query($tcg: String, $q: String!) {
                    searchCards(tcg: $tcg, query: $q, limit: 1) {
                        id name setCode
                    }
                }",
                Some(json!({ "q": card_name, "tcg": tcg })),
            )
            .await;
        
        match search_result {
            Ok(data) => {
                let cards = data["searchCards"].as_array().cloned().unwrap_or_default();
                if let Some(card) = cards.first() {
                    // Add the card
                    let add_result = api
                        .query(
                            "mutation($cardId: ID!, $collectionId: ID!, $quantity: Int!) {
                                addCard(cardId: $cardId, collectionId: $collectionId, quantity: $quantity) {
                                    id
                                }
                            }",
                            Some(json!({
                                "cardId": card["id"],
                                "collectionId": collection_id,
                                "quantity": quantity
                            })),
                        )
                        .await;
                    
                    match add_result {
                        Ok(_) => success_count += 1,
                        Err(_) => error_count += 1,
                    }
                } else {
                    error_count += 1;
                }
            },
            Err(_) => error_count += 1,
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    pb.finish_with_message("Done!");
    
    println!();
    println!("  {} {}", t("bulk_add.completed"), "✓".green());
    println!("  {}: {}", t("bulk_add.success"), success_count.to_string().green());
    
    if error_count > 0 {
        println!("  {}: {}", t("bulk_add.errors"), error_count.to_string().red());
    }
    
    Ok(())
}

/// Parse a line like "2x Sol Ring" or "Sol Ring" into (quantity, name)
fn parse_card_line(line: &str) -> (i32, String) {
    let line = line.trim();
    
    // Look for patterns like "2x", "3 ", "4x ", etc.
    if let Some(space_pos) = line.find(' ') {
        let prefix = &line[..space_pos];
        let rest = &line[space_pos..].trim_start();
        
        // Try to parse quantity
        if let Ok(qty) = prefix.trim_end_matches('x').parse::<i32>() {
            return (qty, rest.to_string());
        }
    }
    
    // Default to quantity 1
    (1, line.to_string())
}