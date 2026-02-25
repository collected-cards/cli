use anyhow::Result;
use colored::*;
use serde::Deserialize;
use serde_json::json;
use crate::api::{ApiClient, CollectionInfo};
use crate::i18n::t;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct FlatEntry {
    card_name: Option<String>,
    set_code: Option<String>,
    collector_number: Option<String>,
    quantity: i32,
    condition: Option<String>,
    language: Option<String>,
    foil: bool,
    current_price: Option<f64>,
    tcg_slug: Option<String>,
    external_id: Option<String>,
}

pub async fn export(
    api: &ApiClient,
    collection_name: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> Result<()> {
    api.require_auth()?;

    println!("  {} {}", "📦".to_string(), t("export.exporting"));

    let entries: Vec<FlatEntry> = if let Some(name) = collection_name {
        // Find collection
        let data = api
            .query("{ myCollections { id name tcgSlug entryCount } }", None)
            .await?;
        let collections: Vec<CollectionInfo> =
            serde_json::from_value(data["myCollections"].clone()).unwrap_or_default();
        let col = collections
            .iter()
            .find(|c| c.name.to_lowercase().contains(&name.to_lowercase()))
            .ok_or_else(|| anyhow::anyhow!("{}: {}", t("collection.not_found"), name))?;

        let data = api
            .query(
                "query($id: ID!) { collectionEntries(collectionId: $id) { id quantity foil condition language card { name setCode collectorNumber currentPrice } } }",
                Some(json!({ "id": col.id })),
            )
            .await?;

        // Convert to flat entries
        let raw: Vec<serde_json::Value> = serde_json::from_value(data["collectionEntries"].clone()).unwrap_or_default();
        raw.iter().map(|e| {
            let card = &e["card"];
            FlatEntry {
                card_name: card["name"].as_str().map(String::from),
                set_code: card["setCode"].as_str().map(String::from),
                collector_number: card["collectorNumber"].as_str().map(String::from),
                quantity: e["quantity"].as_i64().unwrap_or(1) as i32,
                condition: e["condition"].as_str().map(String::from),
                language: e["language"].as_str().map(String::from),
                foil: e["foil"].as_bool().unwrap_or(false),
                current_price: card["currentPrice"].as_f64(),
                tcg_slug: None,
                external_id: None,
            }
        }).collect()
    } else {
        let data = api
            .query("{ allUserCollectionEntries { cardName setCode collectorNumber quantity condition language foil currentPrice tcgSlug externalId } }", None)
            .await?;
        serde_json::from_value(data["allUserCollectionEntries"].clone()).unwrap_or_default()
    };

    let content = match format {
        "csv" => format_csv(&entries),
        "arena" => format_arena(&entries),
        "moxfield" => format_moxfield(&entries),
        "text" => format_text(&entries),
        _ => format_csv(&entries),
    };

    if let Some(path) = output {
        std::fs::write(path, &content)?;
        println!("  {} {} {} ({})", "✅".to_string(), t("export.exported_to"), path.green(), format);
    } else {
        print!("{}", content);
        println!();
        println!("  {} ({})", t("export.to_stdout"), format);
    }

    Ok(())
}

fn format_csv(entries: &[FlatEntry]) -> String {
    let mut out = String::from("Name,Set,Number,Quantity,Condition,Language,Foil,Price\n");
    for e in entries {
        out.push_str(&format!(
            "\"{}\",{},{},{},{},{},{},{}\n",
            e.card_name.as_deref().unwrap_or(""),
            e.set_code.as_deref().unwrap_or(""),
            e.collector_number.as_deref().unwrap_or(""),
            e.quantity,
            e.condition.as_deref().unwrap_or("NM"),
            e.language.as_deref().unwrap_or("en"),
            if e.foil { "Yes" } else { "No" },
            e.current_price.unwrap_or(0.0),
        ));
    }
    out
}

fn format_arena(entries: &[FlatEntry]) -> String {
    let mut out = String::new();
    for e in entries {
        let name = e.card_name.as_deref().unwrap_or("");
        let set = e.set_code.as_deref().unwrap_or("").to_uppercase();
        let nr = e.collector_number.as_deref().unwrap_or("");
        out.push_str(&format!("{} {} ({}) {}\n", e.quantity, name, set, nr));
    }
    out
}

fn format_moxfield(entries: &[FlatEntry]) -> String {
    let mut out = String::from("Count,Name,Edition,Collector Number,Condition,Language,Foil\n");
    for e in entries {
        out.push_str(&format!(
            "{},\"{}\",{},{},{},{},{}\n",
            e.quantity,
            e.card_name.as_deref().unwrap_or(""),
            e.set_code.as_deref().unwrap_or(""),
            e.collector_number.as_deref().unwrap_or(""),
            e.condition.as_deref().unwrap_or("NM"),
            e.language.as_deref().unwrap_or("en"),
            if e.foil { "foil" } else { "" },
        ));
    }
    out
}

fn format_text(entries: &[FlatEntry]) -> String {
    let mut out = String::new();
    for e in entries {
        let name = e.card_name.as_deref().unwrap_or("");
        let set = e.set_code.as_deref().unwrap_or("").to_uppercase();
        let foil = if e.foil { " *F*" } else { "" };
        out.push_str(&format!("{}x {} [{}]{}\n", e.quantity, name, set, foil));
    }
    out
}
