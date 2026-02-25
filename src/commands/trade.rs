use anyhow::Result;
use colored::*;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY, ContentArrangement};
use serde::Deserialize;

use crate::api::ApiClient;
use crate::display;
use crate::i18n::t;

// ─── Types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeProfile {
    display_name: Option<String>,
    username: String,
    location_name: Option<String>,
    radius_km: i32,
    bio: Option<String>,
    offer_count: i32,
    want_count: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeStats {
    active_traders: i32,
    total_trade_cards: i32,
    my_wants: i32,
    my_offers: i32,
    matched_wants: i32,
    wanted_offers: i32,
    match_score: i32,
    trade_chance: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeOffer {
    id: String,
    card_name: String,
    card_set: Option<String>,
    card_price: Option<f64>,
    quantity: i32,
    condition: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeWant {
    id: String,
    card_name: String,
    card_price: Option<f64>,
    tcg_slug: Option<String>,
    quantity: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeMatch {
    username: String,
    display_name: Option<String>,
    location_name: Option<String>,
    distance_km: Option<f64>,
    they_have: Vec<TradeMatchCard>,
    you_have: Vec<TradeMatchCard>,
    match_score: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeMatchCard {
    card_name: String,
    card_price: Option<f64>,
    set_name: Option<String>,
    tcg_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct TradeAccess {
    radius_km: i32,
    active_until: String,
    days_remaining: i32,
    is_active: bool,
}

// ─── Status / Profile ────────────────────────────────

pub async fn status(api: &ApiClient) -> Result<()> {
    let data = api
        .query(
            "{ myTradeProfile { username displayName locationName radiusKm bio offerCount wantCount } myTradeAccess { radiusKm activeUntil daysRemaining isActive } tradeStats { activeTraders totalTradeCards myWants myOffers matchedWants wantedOffers matchScore tradeChance } }",
            None,
        )
        .await?;

    println!();

    if let Some(access) = data.get("myTradeAccess").and_then(|v| {
        serde_json::from_value::<TradeAccess>(v.clone()).ok()
    }) {
        if access.is_active {
            println!(
                "  {} {} — {} {} ({}{})",
                "✅".to_string(),
                t("trade.access_active"),
                access.days_remaining.to_string().green(),
                t("trade.active_days"),
                access.radius_km,
                t("trade.km_radius")
            );
        } else {
            println!("  {} {}", "❌".to_string(), t("trade.access_expired"));
        }
    } else {
        println!("  {} {}", "❌".to_string(), t("trade.no_access"));
        println!("  {} https://collected.cards/trades", t("trade.activate_hint"));
        println!();
        return Ok(());
    }

    if let Some(profile) = data.get("myTradeProfile").and_then(|v| {
        serde_json::from_value::<TradeProfile>(v.clone()).ok()
    }) {
        println!();
        let name = profile.display_name.as_deref().unwrap_or(&profile.username);
        println!("  {} {}", "👤".to_string(), name.bold());
        if let Some(loc) = &profile.location_name {
            println!("  📍 {}", loc);
        }
        if let Some(bio) = &profile.bio {
            if !bio.is_empty() {
                println!("  💬 {}", bio.dimmed());
            }
        }
        println!(
            "  📦 {} {}  |  🔍 {} {}",
            profile.offer_count.to_string().green(),
            t("trade.offers_count"),
            profile.want_count.to_string().cyan(),
            t("trade.wants_count")
        );
    }

    if let Some(stats) = data.get("tradeStats").and_then(|v| {
        serde_json::from_value::<TradeStats>(v.clone()).ok()
    }) {
        println!();
        println!("  {}", "━".repeat(40).dimmed());
        println!(
            "  🤝 {} {}  |  🃏 {} {}",
            stats.active_traders, t("trade.active_traders"),
            stats.total_trade_cards, t("trade.cards_in_trade")
        );
        if stats.my_offers > 0 || stats.my_wants > 0 {
            println!(
                "  📊 {} {}  |  {} {}",
                stats.matched_wants.to_string().green(),
                t("trade.matches_found"),
                stats.wanted_offers.to_string().cyan(),
                t("trade.your_cards_wanted")
            );

            let chance = stats.trade_chance.min(100).max(0);
            let filled = chance / 5;
            let empty = 20 - filled;
            let bar = format!(
                "{}{}",
                "█".repeat(filled as usize),
                "░".repeat(empty as usize)
            );
            let color_bar = if chance >= 70 {
                bar.green()
            } else if chance >= 40 {
                bar.yellow()
            } else {
                bar.red()
            };
            println!("  🎯 {}: {} {}%", t("trade.trade_chance"), color_bar, chance);
        }
    }

    println!();
    Ok(())
}

// ─── Offers ──────────────────────────────────────────

pub async fn offers(api: &ApiClient) -> Result<()> {
    let data = api
        .query(
            "{ myTradeOffers { id cardName cardSet cardPrice quantity condition } }",
            None,
        )
        .await?;

    let offers: Vec<TradeOffer> =
        serde_json::from_value(data["myTradeOffers"].clone()).unwrap_or_default();

    if offers.is_empty() {
        println!();
        println!("  {}", t("trade.no_offers"));
        println!("  {} https://collected.cards/collection", t("trade.mark_hint"));
        println!();
        return Ok(());
    }

    println!();
    println!("  📦 {} {}", offers.len().to_string().green(), t("trade.offers_count"));
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![t("header.qty"), t("header.name"), t("header.set"), t("header.condition"), t("header.price")]);

    for o in &offers {
        table.add_row(vec![
            format!("{}x", o.quantity),
            o.card_name.clone(),
            o.card_set.clone().unwrap_or_default(),
            o.condition.clone().unwrap_or_default(),
            display::format_price(o.card_price),
        ]);
    }

    println!("{table}");
    println!();
    Ok(())
}

// ─── Wants ───────────────────────────────────────────

pub async fn wants(api: &ApiClient) -> Result<()> {
    let data = api
        .query(
            "{ myTradeWants { id cardName cardPrice tcgSlug quantity } }",
            None,
        )
        .await?;

    let wants: Vec<TradeWant> =
        serde_json::from_value(data["myTradeWants"].clone()).unwrap_or_default();

    if wants.is_empty() {
        println!();
        println!("  {}", t("trade.no_wants"));
        println!("  {} https://collected.cards/trades", t("trade.add_wants_hint"));
        println!();
        return Ok(());
    }

    println!();
    println!("  🔍 {} {}", wants.len().to_string().cyan(), t("trade.wants_count"));
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![t("header.qty"), t("header.name"), t("header.tcg"), t("header.price")]);

    for w in &wants {
        table.add_row(vec![
            format!("{}x", w.quantity),
            w.card_name.clone(),
            w.tcg_slug.clone().unwrap_or_default().to_uppercase(),
            display::format_price(w.card_price),
        ]);
    }

    println!("{table}");
    println!();
    Ok(())
}

// ─── Matches ─────────────────────────────────────────

pub async fn matches(api: &ApiClient, limit: Option<i32>) -> Result<()> {
    let data = api
        .query(
            "{ wantlistMatches { username displayName locationName distanceKm theyHave { cardName cardPrice setName tcgSlug } youHave { cardName cardPrice setName tcgSlug } matchScore } }",
            None,
        )
        .await?;

    let mut matches: Vec<TradeMatch> =
        serde_json::from_value(data["wantlistMatches"].clone()).unwrap_or_default();

    if matches.is_empty() {
        println!();
        println!("  {}", t("trade.no_matches"));
        println!("  {}", t("trade.more_hint"));
        println!();
        return Ok(());
    }

    matches.sort_by(|a, b| b.match_score.cmp(&a.match_score));

    let limit = limit.unwrap_or(10) as usize;
    let total = matches.len();
    let matches = &matches[..limit.min(total)];

    println!();
    println!(
        "  🤝 {} {} ({} {})",
        total.to_string().green(),
        t("trade.matches"),
        t("trade.showing"),
        matches.len()
    );
    println!();

    for (i, m) in matches.iter().enumerate() {
        let name = m.display_name.as_deref().unwrap_or(&m.username);
        let loc = m.location_name.as_deref().unwrap_or("?");
        let dist = m.distance_km.map(|d| format!("{:.0}km", d)).unwrap_or_default();

        println!(
            "  {}. {} {} ({} {})",
            (i + 1).to_string().bold(),
            name.bold(),
            format!("(Score: {})", m.match_score).dimmed(),
            loc.dimmed(),
            dist.dimmed()
        );

        if !m.they_have.is_empty() {
            let cards: Vec<String> = m.they_have.iter().map(|c| {
                let price = c.card_price.map(|p| format!(" €{:.2}", p)).unwrap_or_default();
                format!("{}{}", c.card_name, price.dimmed())
            }).collect();
            println!("     🟢 {}: {}", t("trade.has"), cards.join(", "));
        }

        if !m.you_have.is_empty() {
            let cards: Vec<String> = m.you_have.iter().map(|c| {
                let price = c.card_price.map(|p| format!(" €{:.2}", p)).unwrap_or_default();
                format!("{}{}", c.card_name, price.dimmed())
            }).collect();
            println!("     🔵 {}: {}", t("trade.seeks"), cards.join(", "));
        }

        if i < matches.len() - 1 {
            println!("     {}", "─".repeat(40).dimmed());
        }
    }

    if total > limit {
        println!();
        println!("  … {} {} {}", total - limit, t("trade.and_more"), total);
    }

    println!();
    Ok(())
}

// ─── Tradelist ───────────────────────────────────────

pub async fn tradelist(api: &ApiClient, limit: Option<i32>) -> Result<()> {
    let data = api
        .query(
            "{ myTradelist { id quantity foil condition language card { name setCode collectorNumber currentPrice foilPrice } } }",
            None,
        )
        .await?;

    let entries: Vec<crate::api::CollectionEntry> =
        serde_json::from_value(data["myTradelist"].clone()).unwrap_or_default();

    if entries.is_empty() {
        println!();
        println!("  {}", t("trade.no_tradelist"));
        println!("  {}", t("trade.mark_button"));
        println!();
        return Ok(());
    }

    let limit = limit.unwrap_or(50) as usize;
    let total = entries.len();

    let total_value: f64 = entries
        .iter()
        .map(|e| {
            let price = e.card.as_ref().and_then(|c| {
                if e.foil { c.foil_price.or(c.current_price) } else { c.current_price }
            });
            price.unwrap_or(0.0) * e.quantity as f64
        })
        .sum();

    println!();
    println!(
        "  🤝 {} — {} {} ({})",
        t("trade.tradelist"),
        total.to_string().green(),
        t("trade.tradelist_cards"),
        format!("€{:.2}", total_value).green().bold()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![t("header.qty"), t("header.name"), t("header.set"), t("header.number"), t("header.condition"), t("header.foil"), t("header.price")]);

    for e in entries.iter().take(limit) {
        let c = e.card.as_ref();
        let price = c.and_then(|c| {
            if e.foil { c.foil_price.or(c.current_price) } else { c.current_price }
        });
        table.add_row(vec![
            format!("{}x", e.quantity),
            c.and_then(|c| c.name.clone()).unwrap_or_default(),
            c.and_then(|c| c.set_code.clone()).unwrap_or_default().to_uppercase(),
            c.and_then(|c| c.collector_number.clone()).unwrap_or_default(),
            e.condition.clone().unwrap_or_default(),
            if e.foil { "✨" } else { "" }.to_string(),
            display::format_price(price),
        ]);
    }

    println!("{table}");

    if total > limit {
        println!();
        println!("  … {} {} {}", total - limit, t("common.more_results"), total);
    }

    println!();
    Ok(())
}
