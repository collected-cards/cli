use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde::Deserialize;
use serde_json::json;

use crate::api::ApiClient;
use crate::i18n::t;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeckInfo {
    id: String,
    name: String,
    tcg_slug: Option<String>,
    card_count: Option<i32>,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeckDetail {
    id: String,
    name: String,
    tcg_slug: Option<String>,
    format: Option<String>,
    cards: Option<Vec<DeckCard>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeckCard {
    card_name: Option<String>,
    set_code: Option<String>,
    collector_number: Option<String>,
    quantity: i32,
    section: Option<String>,
}

pub async fn list_decks(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query("{ myDecks { id name tcgSlug cardCount format } }", None)
        .await?;

    let decks: Vec<DeckInfo> =
        serde_json::from_value(data["myDecks"].clone()).unwrap_or_default();

    if decks.is_empty() {
        println!("  {}", t("deck.no_decks"));
        return Ok(());
    }

    println!(
        "  {} {}",
        decks.len().to_string().green().bold(),
        if decks.len() == 1 { t("deck.deck") } else { t("deck.decks") }
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![t("header.id"), t("header.name"), t("header.tcg"), t("header.format"), t("header.cards")]);

    for d in &decks {
        table.add_row(vec![
            d.id[..8].to_string(),
            d.name.clone(),
            d.tcg_slug.clone().unwrap_or_default().to_uppercase(),
            d.format.clone().unwrap_or_default(),
            d.card_count.unwrap_or(0).to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub async fn show_deck(api: &ApiClient, name: &str) -> Result<()> {
    api.require_auth()?;

    // Find deck by name
    let data = api
        .query("{ myDecks { id name tcgSlug cardCount format } }", None)
        .await?;

    let decks: Vec<DeckInfo> =
        serde_json::from_value(data["myDecks"].clone()).unwrap_or_default();

    let deck_info = decks
        .iter()
        .find(|d| d.name.to_lowercase().contains(&name.to_lowercase()) || d.id.starts_with(name))
        .ok_or_else(|| anyhow::anyhow!("{}: {}", t("collection.not_found"), name))?;

    let data = api
        .query(
            "query($id: ID!) { deck(id: $id) { id name tcgSlug format cards { cardName setCode collectorNumber quantity section } } }",
            Some(json!({ "id": deck_info.id })),
        )
        .await?;

    let deck: DeckDetail = serde_json::from_value(data["deck"].clone())?;

    println!();
    println!("  {} {}", "🃏".to_string(), deck.name.bold().white());
    if let Some(ref fmt) = deck.format {
        println!("  {} {}", t("header.format"), fmt.dimmed());
    }
    println!();

    if let Some(ref cards) = deck.cards {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![t("header.qty"), t("header.name"), t("header.set"), t("header.number")]);

        for c in cards {
            table.add_row(vec![
                format!("{}x", c.quantity),
                c.card_name.clone().unwrap_or_default(),
                c.set_code.clone().unwrap_or_default().to_uppercase(),
                c.collector_number.clone().unwrap_or_default(),
            ]);
        }

        println!("{table}");
    }

    Ok(())
}

pub async fn export_deck(api: &ApiClient, name: &str, format: &str) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query("{ myDecks { id name tcgSlug cardCount format } }", None)
        .await?;

    let decks: Vec<DeckInfo> =
        serde_json::from_value(data["myDecks"].clone()).unwrap_or_default();

    let deck_info = decks
        .iter()
        .find(|d| d.name.to_lowercase().contains(&name.to_lowercase()) || d.id.starts_with(name))
        .ok_or_else(|| anyhow::anyhow!("{}: {}", t("collection.not_found"), name))?;

    let data = api
        .query(
            "query($id: ID!) { deck(id: $id) { id name tcgSlug format cards { cardName setCode collectorNumber quantity section } } }",
            Some(json!({ "id": deck_info.id })),
        )
        .await?;

    let deck: DeckDetail = serde_json::from_value(data["deck"].clone())?;
    let cards = deck.cards.unwrap_or_default();

    let output = match format {
        "arena" => {
            let mut out = String::new();
            for c in &cards {
                out.push_str(&format!(
                    "{} {} ({}) {}\n",
                    c.quantity,
                    c.card_name.as_deref().unwrap_or(""),
                    c.set_code.as_deref().unwrap_or("").to_uppercase(),
                    c.collector_number.as_deref().unwrap_or("")
                ));
            }
            out
        }
        "moxfield" => {
            let mut out = String::from("Count,Name,Edition,Collector Number\n");
            for c in &cards {
                out.push_str(&format!(
                    "{},\"{}\",{},{}\n",
                    c.quantity,
                    c.card_name.as_deref().unwrap_or(""),
                    c.set_code.as_deref().unwrap_or(""),
                    c.collector_number.as_deref().unwrap_or("")
                ));
            }
            out
        }
        _ => {
            let mut out = String::new();
            for c in &cards {
                let foil_mark = "";
                out.push_str(&format!(
                    "{}x {} [{}]{}\n",
                    c.quantity,
                    c.card_name.as_deref().unwrap_or(""),
                    c.set_code.as_deref().unwrap_or("").to_uppercase(),
                    foil_mark
                ));
            }
            out
        }
    };

    print!("{}", output);
    println!();
    println!("  {} {} ({})", "✅".to_string(), t("deck.export_done"), format);

    Ok(())
}
