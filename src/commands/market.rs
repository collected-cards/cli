use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde::{Deserialize};
use serde_json::json;

use crate::api::ApiClient;
use crate::display;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketplaceItem {
    item_name: Option<String>,
    item_set: Option<String>,
    item_image: Option<String>,
    tcg_slug: Option<String>,
    offer_count: Option<i64>,
    min_price: Option<f64>,
    max_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListingInfo {
    id: String,
    price: f64,
    condition: Option<String>,
    language: Option<String>,
    is_foil: Option<bool>,
    status: Option<String>,
    item_name: Option<String>,
    item_set: Option<String>,
}

/// `collected market search <query>` — Search marketplace
pub async fn search(api: &ApiClient, query: &str, tcg: Option<&str>, limit: Option<i32>) -> Result<()> {
    let limit = limit.unwrap_or(20);

    let data = api
        .query(
            "query($tcg: String, $q: String, $limit: Int) {
                searchMarketplaceItems(tcgSlug: $tcg, query: $q, limit: $limit) {
                    itemName itemSet tcgSlug offerCount minPrice maxPrice
                }
            }",
            Some(json!({
                "q": query,
                "tcg": tcg,
                "limit": limit,
            })),
        )
        .await?;

    let items: Vec<MarketplaceItem> =
        serde_json::from_value(data["searchMarketplaceItems"].clone()).unwrap_or_default();

    if items.is_empty() {
        println!("  Keine Angebote gefunden für: {}", query.yellow());
        return Ok(());
    }

    println!(
        "  {} Marktplatz-Ergebnis{} für {}",
        items.len().to_string().green().bold(),
        if items.len() == 1 { "" } else { "se" },
        query.yellow().bold()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Set", "TCG", "Angebote", "Ab", "Bis"]);

    for item in &items {
        table.add_row(vec![
            item.item_name.clone().unwrap_or_default(),
            item.item_set.clone().unwrap_or_default(),
            item.tcg_slug.clone().unwrap_or_default().to_uppercase(),
            item.offer_count.unwrap_or(0).to_string(),
            display::format_price(item.min_price),
            display::format_price(item.max_price),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// `collected market listings` — Show own listings
pub async fn my_listings(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query(
            "{ myListings { id price condition language isFoil status itemName itemSet } }",
            None,
        )
        .await?;

    let listings: Vec<ListingInfo> =
        serde_json::from_value(data["myListings"].clone()).unwrap_or_default();

    if listings.is_empty() {
        println!("  Keine aktiven Listings.");
        println!("  Erstelle eines mit: {}", "collected market sell <card-id> --price <EUR>".cyan());
        return Ok(());
    }

    println!(
        "  {} Listing{}",
        listings.len().to_string().green().bold(),
        if listings.len() == 1 { "" } else { "s" }
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Karte", "Set", "Preis", "Zustand", "Foil", "Status"]);

    for l in &listings {
        table.add_row(vec![
            l.id[..8].to_string(),
            l.item_name.clone().unwrap_or_default(),
            l.item_set.clone().unwrap_or_default(),
            format!("€{:.2}", l.price),
            l.condition.clone().unwrap_or_default(),
            if l.is_foil.unwrap_or(false) { "✨" } else { "" }.to_string(),
            l.status.clone().unwrap_or_default(),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// `collected market sell <card-id> --price <EUR>`
pub async fn create_listing(
    api: &ApiClient,
    card_id: &str,
    price: f64,
    condition: Option<&str>,
    language: Option<&str>,
    foil: bool,
    description: Option<&str>,
) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query(
            "mutation($cardId: ID, $price: Float!, $condition: String, $language: String, $isFoil: Boolean, $description: String) {
                createListing(cardId: $cardId, price: $price, condition: $condition, language: $language, isFoil: $isFoil, description: $description) {
                    id price status
                }
            }",
            Some(json!({
                "cardId": card_id,
                "price": price,
                "condition": condition,
                "language": language,
                "isFoil": foil,
                "description": description,
            })),
        )
        .await?;

    let listing = &data["createListing"];
    let id = listing["id"].as_str().unwrap_or("?");

    println!(
        "  {} Listing erstellt: {} (€{:.2})",
        "✅".to_string(),
        id[..8].to_string().green(),
        price
    );

    Ok(())
}
