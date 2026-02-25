use anyhow::Result;
use colored::*;
use serde::Deserialize;
use serde_json::json;

use crate::api::{ApiClient, CardResult};
use crate::i18n::t;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PricePoint {
    price: f64,
}

pub async fn price_history(api: &ApiClient, query: &str, tcg: Option<&str>, period: &str) -> Result<()> {
    let tcg = tcg.unwrap_or("mtg");

    // Find card
    let data = api
        .query(
            "query($tcg: String!, $q: String!) {
                searchCards(tcg: $tcg, query: $q, limit: 1) {
                    id name setCode setName currentPrice bracket gameChanger
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

    // Get price history
    let data = api
        .query(
            "query($cardId: ID!, $period: String!) { priceHistory(cardId: $cardId, period: $period) { price } }",
            Some(json!({ "cardId": card.id, "period": period })),
        )
        .await?;

    let history: Vec<PricePoint> =
        serde_json::from_value(data["priceHistory"].clone()).unwrap_or_default();

    println!();
    let bracket_str = if card.game_changer.unwrap_or(false) {
        format!(" {}", "⚠ Game Changer".red())
    } else if let Some(bracket) = card.bracket {
        if bracket > 0 {
            let bracket_badge = format!("}}{}{{", bracket);
            let colored_bracket = match bracket {
                1 => bracket_badge.white(),
                2 => bracket_badge.green(),
                3 => bracket_badge.blue(),
                4 => bracket_badge.purple(),
                5 => bracket_badge.yellow(),
                6 => bracket_badge.red(),
                _ => bracket_badge.normal(),
            };
            format!(" {}", colored_bracket)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    println!("  {} {} — {}{}", "📈".to_string(), t("price.history"), card.name.bold().white(), bracket_str);
    println!("  {}: {}", t("price.period"), period.cyan());

    if let Some(price) = card.current_price {
        println!("  {}: {}", t("price.current"), format!("€{:.2}", price).green().bold());
    }

    println!();

    if history.is_empty() {
        println!("  {}", t("price.no_history"));
        return Ok(());
    }

    // ASCII bar chart
    let max_price = history.iter().map(|p| p.price).fold(0.0f64, f64::max);
    let chart_width = 40;

    for (i, point) in history.iter().enumerate() {
        let bar_len = if max_price > 0.0 {
            ((point.price / max_price) * chart_width as f64) as usize
        } else {
            0
        };

        let bar = "█".repeat(bar_len);
        let colored_bar = if point.price >= max_price * 0.8 {
            bar.green()
        } else if point.price >= max_price * 0.4 {
            bar.yellow()
        } else {
            bar.red()
        };

        let label = format!("{:>3}", i + 1);
        println!(
            "  {} {} €{:.2}",
            label.dimmed(),
            colored_bar,
            point.price
        );
    }

    println!();
    Ok(())
}
