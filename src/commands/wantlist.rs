use anyhow::Result;
use colored::*;
use comfy_table::{presets::UTF8_BORDERS_ONLY, ContentArrangement, Table};
use dialoguer::Select;
use serde::Deserialize;
use serde_json::json;

use crate::api::{ApiClient, CardResult};
use crate::display;
use crate::i18n::t;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WantlistItem {
    id: String,
    card_name: String,
    card_price: Option<f64>,
    tcg_slug: Option<String>,
    quantity: i32,
}

pub async fn show_wantlist(api: &ApiClient) -> Result<()> {
    api.require_auth()?;

    let data = api
        .query("{ myWantlist { id cardName cardPrice tcgSlug quantity } }", None)
        .await?;

    let items: Vec<WantlistItem> =
        serde_json::from_value(data["myWantlist"].clone()).unwrap_or_default();

    if items.is_empty() {
        println!("  {}", t("wantlist.empty"));
        return Ok(());
    }

    println!();
    println!("  📋 {} — {} items", t("wantlist.title"), items.len().to_string().green());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["#", t("header.qty"), t("header.name"), t("header.tcg"), t("header.price")]);

    for (i, item) in items.iter().enumerate() {
        table.add_row(vec![
            (i + 1).to_string(),
            format!("{}x", item.quantity),
            item.card_name.clone(),
            item.tcg_slug.clone().unwrap_or_default().to_uppercase(),
            display::format_price(item.card_price),
        ]);
    }

    println!("{table}");
    println!();
    Ok(())
}

pub async fn add_to_wantlist(api: &ApiClient, card_name: &str, tcg: Option<&str>) -> Result<()> {
    api.require_auth()?;

    let tcg = tcg.unwrap_or("mtg");

    // Search to validate
    let data = api
        .query(
            "query($tcg: String!, $q: String!, $limit: Int) {
                searchCards(tcg: $tcg, query: $q, limit: $limit) { id name setCode setName }
            }",
            Some(json!({ "tcg": tcg, "q": card_name, "limit": 5 })),
        )
        .await?;

    let cards: Vec<CardResult> =
        serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    let name = if cards.is_empty() {
        // Use the raw name
        card_name.to_string()
    } else if cards.len() == 1 {
        cards[0].name.clone()
    } else {
        let names: Vec<String> = cards.iter().map(|c| {
            let set = c.set_code.as_deref().unwrap_or("").to_uppercase();
            format!("{} [{}]", c.name, set)
        }).collect();
        let idx = Select::new()
            .with_prompt(t("add.select_card"))
            .items(&names)
            .default(0)
            .interact()?;
        cards[idx].name.clone()
    };

    api.query(
        "mutation($cardName: String!, $tcgSlug: String!) { addToWantlist(cardName: $cardName, tcgSlug: $tcgSlug) }",
        Some(json!({ "cardName": name, "tcgSlug": tcg })),
    ).await?;

    println!("  {} {} {}", "✅".to_string(), t("wantlist.added"), name.green().bold());

    Ok(())
}

pub async fn remove_from_wantlist(api: &ApiClient, id: &str) -> Result<()> {
    api.require_auth()?;

    api.query(
        "mutation($id: ID!) { removeFromWantlist(id: $id) }",
        Some(json!({ "id": id })),
    ).await?;

    println!("  {} {}", "✅".to_string(), t("wantlist.removed"));

    Ok(())
}
