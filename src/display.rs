use anyhow::Result;
use colored::*;
use crate::api::CardResult;

/// Detect terminal image capability
pub enum ImageMode {
    Sixel,
    Kitty,
    ITerm,
    Ascii,
    None,
}

pub fn detect_image_mode(preference: &str) -> ImageMode {
    match preference {
        "sixel" => return ImageMode::Sixel,
        "kitty" => return ImageMode::Kitty,
        "iterm" => return ImageMode::ITerm,
        "ascii" => return ImageMode::Ascii,
        "none" => return ImageMode::None,
        _ => {} // "auto"
    }

    // Auto-detect
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program.contains("iTerm") {
            return ImageMode::ITerm;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") {
            return ImageMode::Kitty;
        }
    }
    // Default to viuer which auto-detects sixel/kitty/iterm
    ImageMode::Sixel
}

/// Display card image in terminal using viuer
pub async fn show_card_image(image_url: &str, config_mode: &str) -> Result<()> {
    let mode = detect_image_mode(config_mode);
    if matches!(mode, ImageMode::None) {
        return Ok(());
    }

    // Download image
    let client = reqwest::Client::new();
    let resp = client.get(image_url).send().await?;
    let bytes = resp.bytes().await?;
    let img = image::load_from_memory(&bytes)?;

    let conf = viuer::Config {
        width: Some(40),
        height: Some(20),
        absolute_offset: false,
        ..Default::default()
    };

    viuer::print(&img, &conf)?;
    Ok(())
}

/// Display card as rich text in terminal
pub fn print_card_detail(card: &CardResult) {
    let name = &card.name;
    let mana = card.mana_cost.as_deref().unwrap_or("");

    // Header
    println!();
    println!("  {} {}", name.bold().white(), mana.yellow());
    println!("  {}", "━".repeat(50).dimmed());

    // Type line
    if let Some(ref type_line) = card.type_line {
        let set_info = match (&card.set_name, &card.rarity) {
            (Some(s), Some(r)) => format!("{} · {}", s, r),
            (Some(s), None) => s.clone(),
            _ => String::new(),
        };
        if set_info.is_empty() {
            println!("  {}", type_line.cyan());
        } else {
            println!("  {}  {}", type_line.cyan(), set_info.dimmed());
        }
    }

    // Oracle text
    if let Some(ref text) = card.oracle_text {
        println!();
        for line in text.lines() {
            println!("  {}", line);
        }
    }

    // P/T
    if let (Some(ref p), Some(ref t)) = (&card.power, &card.toughness) {
        println!("  {}", format!("{}/{}", p, t).bold());
    }

    // Collector number
    if let Some(ref cn) = card.collector_number {
        println!();
        println!("  {} {}", "#".dimmed(), cn.dimmed());
    }

    // Price
    println!();
    if let Some(price) = card.current_price {
        if price > 0.0 {
            print!("  {} €{:.2}", "💰".to_string(), price);
            if let Some(foil) = card.foil_price {
                if foil > 0.0 && (foil - price).abs() > 0.01 {
                    print!("  {} €{:.2}", "(Foil:".dimmed(), foil);
                    print!("{}", ")".dimmed());
                }
            }
            println!();
        }
    }
    println!();
}

/// Print a separator line
pub fn separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Format price for display
pub fn format_price(price: Option<f64>) -> String {
    match price {
        Some(p) if p > 0.0 => format!("€{:.2}", p),
        _ => "—".to_string(),
    }
}

/// ASCII art card frame (fallback when no image protocol available)
pub fn print_ascii_card(card: &CardResult) {
    let name = &card.name;
    let mana = card.mana_cost.as_deref().unwrap_or("");
    let type_line = card.type_line.as_deref().unwrap_or("");
    let oracle = card.oracle_text.as_deref().unwrap_or("");
    let price = format_price(card.current_price);
    let set = card.set_name.as_deref().unwrap_or("Unknown");
    let rarity = card.rarity.as_deref().unwrap_or("");
    let pt = match (&card.power, &card.toughness) {
        (Some(p), Some(t)) => format!("{}/{}", p, t),
        _ => String::new(),
    };

    let w = 42;
    let inner = w - 4;

    println!("  ┌{}┐", "─".repeat(w - 2));
    println!("  │ {:<iw$} {:>6} │", truncate(name, inner - 7), mana, iw = inner - 7);
    println!("  ├{}┤", "─".repeat(w - 2));
    println!("  │ {:<iw$} │", type_line, iw = inner);

    // Oracle text wrapped
    if !oracle.is_empty() {
        println!("  ├{}┤", "─".repeat(w - 2));
        for line in wrap_text(oracle, inner) {
            println!("  │ {:<iw$} │", line, iw = inner);
        }
    }

    if !pt.is_empty() {
        println!("  │ {:>iw$} │", pt, iw = inner);
    }

    println!("  ├{}┤", "─".repeat(w - 2));
    println!(
        "  │ {:<sw$} {:>pw$} │",
        format!("{} · {}", set, rarity),
        price,
        sw = inner - price.len() - 1,
        pw = price.len()
    );
    println!("  └{}┘", "─".repeat(w - 2));
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.is_empty() {
                line = word.to_string();
            } else if line.len() + 1 + word.len() <= width {
                line.push(' ');
                line.push_str(word);
            } else {
                lines.push(line);
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}
