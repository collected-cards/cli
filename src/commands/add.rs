use anyhow::Result;
use colored::*;
use dialoguer::Select;
use serde_json::json;

use crate::api::{ApiClient, CardResult, CollectionInfo};
use crate::i18n::t;

pub async fn add_card(
    api: &ApiClient,
    query: &str,
    tcg: Option<&str>,
    quantity: Option<i32>,
    condition: Option<&str>,
    foil: bool,
    lang: Option<&str>,
    collection_name: Option<&str>,
) -> Result<()> {
    api.require_auth()?;

    let tcg = tcg.unwrap_or("mtg");
    let quantity = quantity.unwrap_or(1);

    // Search for cards
    println!("  {} {}", "🔍".to_string(), t("add.searching"));

    let data = api
        .query(
            "query($tcg: String!, $q: String!, $limit: Int) {
                searchCards(tcg: $tcg, query: $q, limit: $limit) {
                    id name setCode setName collectorNumber rarity currentPrice
                }
            }",
            Some(json!({ "tcg": tcg, "q": query, "limit": 5 })),
        )
        .await?;

    let cards: Vec<CardResult> =
        serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    if cards.is_empty() {
        println!("  {} {}", t("search.no_results"), query.yellow());
        return Ok(());
    }

    // Select card
    let card_names: Vec<String> = cards
        .iter()
        .map(|c| {
            let set = c.set_code.as_deref().unwrap_or("").to_uppercase();
            let nr = c.collector_number.as_deref().unwrap_or("");
            format!("{} [{}{}]", c.name, set, if nr.is_empty() { String::new() } else { format!(" #{}", nr) })
        })
        .collect();

    let card_idx = if cards.len() == 1 {
        0
    } else {
        Select::new()
            .with_prompt(t("add.select_card"))
            .items(&card_names)
            .default(0)
            .interact()?
    };

    let card = &cards[card_idx];

    // Get collections
    let data = api
        .query("{ myCollections { id name tcgSlug entryCount } }", None)
        .await?;

    let collections: Vec<CollectionInfo> =
        serde_json::from_value(data["myCollections"].clone()).unwrap_or_default();

    // Filter by TCG if possible
    let filtered: Vec<&CollectionInfo> = collections
        .iter()
        .filter(|c| c.tcg_slug.as_deref().map(|s| s == tcg).unwrap_or(true))
        .collect();

    let available = if filtered.is_empty() { &collections } else {
        // A bit awkward but works for selecting
        &collections
    };

    if available.is_empty() {
        println!("  {}", t("add.no_collections_for_tcg"));
        return Ok(());
    }

    // Select collection
    let collection = if let Some(name) = collection_name {
        collections
            .iter()
            .find(|c| c.name.to_lowercase().contains(&name.to_lowercase()))
            .ok_or_else(|| anyhow::anyhow!("{}: {}", t("collection.not_found"), name))?
    } else if collections.len() == 1 {
        &collections[0]
    } else {
        let col_names: Vec<String> = collections.iter().map(|c| {
            format!("{} ({})", c.name, c.tcg_slug.as_deref().unwrap_or("").to_uppercase())
        }).collect();

        let col_idx = Select::new()
            .with_prompt(t("add.select_collection"))
            .items(&col_names)
            .default(0)
            .interact()?;

        &collections[col_idx]
    };

    // Add card
    api.query(
        "mutation($collectionId: ID!, $cardId: ID!, $quantity: Int, $condition: String, $language: String, $foil: Boolean) {
            addCardToCollection(collectionId: $collectionId, cardId: $cardId, quantity: $quantity, condition: $condition, language: $language, foil: $foil) {
                id
            }
        }",
        Some(json!({
            "collectionId": collection.id,
            "cardId": card.id,
            "quantity": quantity,
            "condition": condition,
            "language": lang,
            "foil": foil,
        })),
    ).await?;

    println!(
        "  {} {} {}x {} → {}",
        "✅".to_string(),
        t("add.added_to"),
        quantity,
        card.name.green().bold(),
        collection.name.cyan()
    );

    Ok(())
}
