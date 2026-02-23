use anyhow::{bail, Result};
use colored::*;
use dialoguer::Select;
use serde_json::json;
use std::io::{BufRead, BufReader};

use crate::api::{ApiClient, CollectionInfo};
use crate::i18n::t;

const BATCH_SIZE: usize = 100;

/// Parse a CSV line respecting quoted fields (handles commas inside quotes)
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                // Check for escaped quote ""
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

pub async fn import_file(
    api: &ApiClient,
    file_path: &str,
    collection_name: Option<&str>,
) -> Result<()> {
    api.require_auth()?;

    println!("  {} {}", "📂".to_string(), t("import.reading_file"));

    let file = std::fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read and parse header
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => bail!("{}: empty file", t("import.format_error")),
    };

    let header_lower = header_line.to_lowercase();
    let has_header = header_lower.contains("name") || header_lower.contains("quantity") || header_lower.contains("count");

    // Detect column indices from header
    let header_fields: Vec<String> = if has_header {
        parse_csv_line(&header_line).iter().map(|s| s.to_lowercase()).collect()
    } else {
        Vec::new()
    };

    let col_name = header_fields.iter().position(|h| h.contains("name") && !h.contains("set"));
    let col_set = header_fields.iter().position(|h| h.contains("set") || h.contains("edition"));
    let col_number = header_fields.iter().position(|h| h.contains("number") || h.contains("collector"));
    let col_qty = header_fields.iter().position(|h| h.contains("qty") || h.contains("quantity") || h.contains("count"));
    let col_condition = header_fields.iter().position(|h| h.contains("condition"));
    let col_foil = header_fields.iter().position(|h| h.contains("foil"));
    let col_lang = header_fields.iter().position(|h| h.contains("lang"));
    let col_extid = header_fields.iter().position(|h| h.contains("externalid") || h.contains("external_id"));

    // Select collection first
    let data = api
        .query("{ myCollections { id name tcgSlug entryCount } }", None)
        .await?;
    let collections: Vec<CollectionInfo> =
        serde_json::from_value(data["myCollections"].clone()).unwrap_or_default();

    if collections.is_empty() {
        bail!("{}", t("collection.no_collections"));
    }

    let collection = if let Some(name) = collection_name {
        collections
            .iter()
            .find(|c| c.name.to_lowercase().contains(&name.to_lowercase()))
            .ok_or_else(|| anyhow::anyhow!("{}: {}", t("collection.not_found"), name))?
    } else if collections.len() == 1 {
        &collections[0]
    } else {
        let col_names: Vec<String> = collections.iter().map(|c| {
            format!("{} ({}, {} cards)", c.name, c.tcg_slug.as_deref().unwrap_or("?").to_uppercase(), c.entry_count.unwrap_or(0))
        }).collect();
        let idx = Select::new()
            .with_prompt(t("add.select_collection"))
            .items(&col_names)
            .default(0)
            .interact()?;
        &collections[idx]
    };

    println!("  {} → {}", t("import.target"), collection.name.cyan());

    // Stream-parse lines and send in batches
    let mut batch = Vec::new();
    let mut total_imported = 0usize;
    let mut total_skipped = 0usize;
    let mut line_num = 1usize;

    // If no header, the first line is data — re-parse it
    let first_data_line = if !has_header { Some(header_line.clone()) } else { None };

    let data_lines = first_data_line.into_iter()
        .chain(lines.filter_map(|l| l.ok()));

    for line in data_lines {
        line_num += 1;
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let fields = parse_csv_line(&line);
        if fields.len() < 2 {
            total_skipped += 1;
            continue;
        }

        // Extract fields by detected column index, or fall back to positional
        let name = if let Some(i) = col_name { fields.get(i).cloned() } else { fields.get(0).cloned() };
        let set_code = if let Some(i) = col_set { fields.get(i).cloned() } else { fields.get(1).cloned() };
        let number = if let Some(i) = col_number { fields.get(i).cloned() } else { fields.get(2).cloned() };
        let quantity = if let Some(i) = col_qty {
            fields.get(i).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1)
        } else {
            fields.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1)
        };
        let condition = if let Some(i) = col_condition { fields.get(i).cloned() } else { fields.get(4).cloned() };
        let foil = if let Some(i) = col_foil {
            fields.get(i).map(|f| {
                let fl = f.to_lowercase();
                fl == "yes" || fl == "true" || fl.contains("foil")
            }).unwrap_or(false)
        } else {
            fields.get(5).map(|f| {
                let fl = f.to_lowercase();
                fl == "yes" || fl == "true" || fl.contains("foil")
            }).unwrap_or(false)
        };
        let language = if let Some(i) = col_lang { fields.get(i).cloned() } else { fields.get(6).cloned() };
        let external_id = if let Some(i) = col_extid { fields.get(i).cloned() } else { None };

        let name = match name {
            Some(ref n) if !n.is_empty() => n.clone(),
            _ => {
                total_skipped += 1;
                continue;
            }
        };

        let mut card = json!({
            "name": name,
            "setCode": set_code.unwrap_or_default(),
            "quantity": quantity,
            "condition": condition.unwrap_or_else(|| "NM".to_string()),
            "foil": foil,
            "language": language.unwrap_or_else(|| "en".to_string()),
        });

        if let Some(ref nr) = number {
            if !nr.is_empty() {
                card["collectorNumber"] = json!(nr);
            }
        }
        if let Some(ref eid) = external_id {
            if !eid.is_empty() {
                card["externalId"] = json!(eid);
            }
        }

        batch.push(card);

        // Send batch when full
        if batch.len() >= BATCH_SIZE {
            let count = batch.len();
            match send_batch(api, &collection.id, &batch).await {
                Ok(_) => {
                    total_imported += count;
                    eprint!("\r  {} {} {}", "📥".to_string(), total_imported.to_string().green(), t("import.cards_processed"));
                }
                Err(e) => {
                    eprintln!("\n  {} Batch error at line {}: {}", "⚠️".to_string(), line_num, t("common.api_error"));
                    total_skipped += count;
                }
            }
            batch.clear();
        }
    }

    // Send remaining batch
    if !batch.is_empty() {
        let count = batch.len();
        match send_batch(api, &collection.id, &batch).await {
            Ok(_) => total_imported += count,
            Err(_) => {
                eprintln!("  {} {}", "⚠️".to_string(), t("common.api_error"));
                total_skipped += count;
            }
        }
    }

    println!();
    println!();
    println!(
        "  {} {} {} {}",
        "✅".to_string(),
        total_imported.to_string().green().bold(),
        t("import.imported_count"),
        if total_skipped > 0 {
            format!("({} {})", total_skipped, t("import.skipped"))
        } else {
            String::new()
        }
    );

    Ok(())
}

async fn send_batch(api: &ApiClient, collection_id: &str, cards: &[serde_json::Value]) -> Result<()> {
    api.query(
        "mutation($collectionId: ID!, $cards: [ImportCardInput!]!) { importCards(collectionId: $collectionId, cards: $cards) { imported skipped errors } }",
        Some(json!({ "collectionId": collection_id, "cards": cards })),
    ).await?;
    Ok(())
}
