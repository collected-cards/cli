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

/// Try to get a higher-resolution image URL
fn upgrade_image_url(url: &str) -> String {
    // Scryfall: replace /small/ or /normal/ with /large/ or /png/
    let upgraded = url
        .replace("/small/", "/large/")
        .replace("/normal/", "/large/");
    // tcgcache: replace /small/ with /high/ or just use large
    let upgraded = upgraded
        .replace("/img/mtg/small/", "/img/mtg/large/")
        .replace("/img/pokemon/en/", "/img/pokemon/en/")  // already good
        .replace("/low.webp", "/high.webp");
    upgraded
}

/// Display card image in terminal using viuer
pub async fn show_card_image(image_url: &str, config_mode: &str) -> Result<()> {
    let mode = detect_image_mode(config_mode);
    if matches!(mode, ImageMode::None) {
        return Ok(());
    }

    // Try high-res first, fall back to original
    let hires_url = upgrade_image_url(image_url);
    let client = reqwest::Client::new();

    let bytes = match client.get(&hires_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.bytes().await?,
        _ => {
            // Fallback to original URL
            let resp = client.get(image_url).send().await?;
            resp.bytes().await?
        }
    };
    let img = image::load_from_memory(&bytes)?;

    // Use full terminal width minus small margin
    let (term_w, term_h) = terminal_size::terminal_size()
        .map(|(w, h)| (w.0 as u32, h.0 as u32))
        .unwrap_or_else(|| {
            // Fallback: try COLUMNS/LINES env vars
            let w = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(80);
            let h = std::env::var("LINES").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
            (w, h)
        });

    // Card aspect ratio ~0.715 (width/height). Use up to 90% of terminal height.
    let max_height = (term_h * 9 / 10).max(20);
    // Width from height: each char is ~2 pixels tall (halfblock), ~1 pixel wide
    // So for a card that's 1.4x taller than wide, width = height * 0.715 / 2 * 2 ≈ height * 0.715
    let width_from_height = (max_height as f32 * 0.72) as u32;
    let img_width = width_from_height.max(20).min(term_w - 4);

    let conf = viuer::Config {
        width: Some(img_width),
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

#[allow(dead_code)]
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

/// Beautiful virtual card with all info — replaces pixel image
pub fn print_virtual_card(card: &CardResult) {
    let name = &card.name;
    let mana = card.mana_cost.as_deref().unwrap_or("");
    let type_line = card.type_line.as_deref().unwrap_or("");
    let oracle = card.oracle_text.as_deref().unwrap_or("");
    let set = card.set_code.as_deref().unwrap_or("???").to_uppercase();
    let set_name = card.set_name.as_deref().unwrap_or("");
    let rarity = card.rarity.as_deref().unwrap_or("");
    let cn = card.collector_number.as_deref().unwrap_or("");
    let pt = match (&card.power, &card.toughness) {
        (Some(p), Some(t)) => format!(" ⚔ {}/{}", p, t),
        _ => String::new(),
    };

    // Terminal width detection
    let term_w = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or_else(|| {
            std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(72)
        });
    let w = (term_w - 4).min(60).max(36);
    let inner = w - 4;

    // Rarity color
    let rarity_color = match rarity {
        "mythic" => "red",
        "rare" => "yellow",
        "uncommon" => "cyan",
        _ => "white",
    };

    // Mana cost prettified: {W}{U}{B} → ⬜🔵⚫ 
    let mana_pretty = prettify_mana(mana);
    
    // Price + bracket line
    let price_str = match card.current_price {
        Some(p) if p > 0.0 => format!("€{:.2}", p),
        _ => "—".to_string(),
    };
    let bracket_str = if card.game_changer.unwrap_or(false) {
        " ⚠ Game Changer".to_string()
    } else {
        match card.bracket {
            Some(1) => " }1{ Exhibition".to_string(),
            Some(2) => " }2{ Core".to_string(),
            Some(3) => " }3{ Upgraded".to_string(),
            Some(4) => " }4{ Optimized".to_string(),
            Some(5) => " }5{ cEDH".to_string(),
            Some(6) => " }GC{ Game Changer".to_string(),
            _ => String::new(),
        }
    };

    // ── Top border ──
    println!("  ╭{}╮", "─".repeat(w - 2));
    
    // ── Name + Mana ──
    let name_space = inner - mana_pretty.chars().count();
    let name_display = truncate(name, name_space);
    println!("  │ {}{}{} │", 
        name_display.bold().white(),
        " ".repeat(inner - name_display.chars().count() - mana_pretty.chars().count()),
        mana_pretty.yellow(),
    );

    // ── Separator ──
    println!("  ├{}┤", "─".repeat(w - 2));

    // ── Type line ──
    let type_with_pt = if pt.is_empty() {
        type_line.to_string()
    } else {
        format!("{}{}", type_line, pt)
    };
    println!("  │ {}{} │",
        truncate(&type_with_pt, inner).cyan(),
        " ".repeat(inner.saturating_sub(type_with_pt.chars().count())),
    );

    // ── Oracle text ──
    if !oracle.is_empty() {
        println!("  ├{}┤", "╌".repeat(w - 2));
        let wrapped = wrap_text(oracle, inner);
        for line in &wrapped {
            println!("  │ {}{} │", line, " ".repeat(inner.saturating_sub(line.chars().count())));
        }
    }

    // ── Separator ──
    println!("  ├{}┤", "─".repeat(w - 2));

    // ── Set + Rarity ──
    let set_line = format!("{} · #{} · {}", set, cn, rarity);
    let set_display = truncate(&set_line, inner);
    let colored_set = match rarity_color {
        "red" => set_display.red(),
        "yellow" => set_display.yellow(),
        "cyan" => set_display.cyan(),
        _ => set_display.white(),
    };
    println!("  │ {}{} │",
        colored_set,
        " ".repeat(inner.saturating_sub(set_line.chars().count())),
    );

    // ── Price + Bracket ──
    let price_line = format!("{}{}", price_str, bracket_str);
    let price_display = truncate(&price_line, inner);
    println!("  │ {}{} │",
        price_display.green().bold(),
        " ".repeat(inner.saturating_sub(price_line.chars().count())),
    );

    // ── Set name (small) ──
    if !set_name.is_empty() {
        let sn = truncate(set_name, inner);
        println!("  │ {}{} │",
            sn.dimmed(),
            " ".repeat(inner.saturating_sub(set_name.chars().count())),
        );
    }

    // ── Bottom border ──
    println!("  ╰{}╯", "─".repeat(w - 2));
}

/// Convert {W}{U}{B}{R}{G}{C} mana symbols to colored text
fn prettify_mana(mana: &str) -> String {
    if mana.is_empty() { return String::new(); }
    mana.replace("{W}", "◈W")
        .replace("{U}", "◈U")
        .replace("{B}", "◈B")
        .replace("{R}", "◈R")
        .replace("{G}", "◈G")
        .replace("{C}", "◈C")
        .replace("{X}", "X")
        .replace("{T}", "⟳")
        .replace("{", "")
        .replace("}", "")
}

/// ASCII art card frame (fallback when no image protocol available)
#[allow(dead_code)]

#[allow(dead_code)]
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
