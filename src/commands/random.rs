use anyhow::Result;
use colored::*;
use serde_json::json;
use rand::Rng;

use crate::api::{self, ApiClient, CardResult};
use crate::display;
use crate::i18n::t;

/// `collected random [tcg]` — Show a random card
pub async fn random_card(api: &ApiClient, tcg: Option<&str>) -> Result<()> {
    let tcg = tcg.unwrap_or("mtg");

    let random_terms = [
        "the", "of", "a", "fire", "dragon", "knight", "angel", "demon",
        "sword", "shield", "dark", "light", "shadow", "storm", "blood",
        "stone", "spell", "ward", "bolt", "rage", "soul", "wind", "star",
    ];
    
    let mut rng = rand::thread_rng();
    let random_term = random_terms[rng.gen_range(0..random_terms.len())];

    let data = api
        .query(
            "query($tcg: String!, $q: String!, $limit: Int) {
                searchCards(tcg: $tcg, query: $q, limit: $limit) {
                    id name setCode setName collectorNumber rarity imageUrl currentPrice bracket gameChanger
                }
            }",
            Some(json!({ "tcg": tcg, "q": random_term, "limit": 20 })),
        )
        .await?;

    let cards: Vec<CardResult> = serde_json::from_value(data["searchCards"].clone()).unwrap_or_default();

    let card = match cards.len() {
        0 => {
            println!("  {}", t("random.no_cards"));
            return Ok(());
        }
        n => &cards[rng.gen_range(0..n)],
    };

    println!();
    println!("  {} {}", "🎲", t("random.title").bold().white());

    // Fetch rich detail from tcgcache for MTG cards
    let enriched = if tcg == "mtg" {
        api::fetch_tcgcache_detail(card).await
    } else {
        None
    };

    match enriched {
        Some(rich) => display::print_virtual_card(&rich),
        None => display::print_virtual_card(card),
    }

    println!();
    Ok(())
}
