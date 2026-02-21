mod api;
mod commands;
mod config;
mod display;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(
    name = "collected",
    about = "🃏 collected.cards CLI — Deine TCG-Sammlung im Terminal",
    version,
    author = "collected.cards"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Anmeldung & Token-Verwaltung
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Karten suchen
    Search {
        /// Suchbegriff
        query: String,
        /// TCG filtern (mtg, pokemon, yugioh, ...)
        #[arg(long)]
        tcg: Option<String>,
        /// Max. Ergebnisse
        #[arg(long, short)]
        limit: Option<i32>,
    },

    /// Karten-Detail anzeigen
    Card {
        /// Kartenname oder ID
        query: String,
        /// TCG filtern (mtg, pokemon, yugioh, ...)
        #[arg(long)]
        tcg: Option<String>,
        /// Kartenbild im Terminal anzeigen
        #[arg(long)]
        art: bool,
    },

    /// Sammlungen auflisten
    Collections,

    /// Sammlung anzeigen
    Collection {
        /// Sammlungsname oder ID
        name: String,
        /// Sortierung (name, price, date)
        #[arg(long)]
        sort: Option<String>,
        /// Max. Karten
        #[arg(long, short)]
        limit: Option<i32>,
    },

    /// Sammlungsstatistiken
    Stats {
        /// TCG filtern
        #[arg(long)]
        tcg: Option<String>,
    },

    /// Marktplatz
    Market {
        #[command(subcommand)]
        action: MarketAction,
    },

    /// Trading — Tauschen mit anderen Sammlern
    Trade {
        #[command(subcommand)]
        action: TradeAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Im Browser anmelden und Token einrichten
    Login,
    /// Abmelden (Token entfernen)
    Logout,
    /// Anmeldestatus anzeigen
    Status,
}

#[derive(Subcommand)]
enum TradeAction {
    /// Trade-Status & Profil anzeigen
    Status,
    /// Angebote (Karten die du tauschen willst)
    Offers,
    /// Gesuche (Karten die du suchst)
    Wants,
    /// Matches — passende Tauschpartner
    Matches {
        /// Max. Ergebnisse
        #[arg(long, short)]
        limit: Option<i32>,
    },
    /// Tradelist — alle tauschbaren Karten
    List {
        /// Max. Ergebnisse
        #[arg(long, short)]
        limit: Option<i32>,
    },
}

#[derive(Subcommand)]
enum MarketAction {
    /// Marktplatz durchsuchen
    Search {
        query: String,
        #[arg(long)]
        tcg: Option<String>,
        #[arg(long, short)]
        limit: Option<i32>,
    },
    /// Eigene Listings anzeigen
    Listings,
    /// Karte zum Verkauf anbieten
    Sell {
        /// Karten-ID
        card_id: String,
        /// Preis in EUR
        #[arg(long)]
        price: f64,
        /// Zustand (NM, LP, MP, HP, DMG)
        #[arg(long)]
        condition: Option<String>,
        /// Sprache (de, en, fr, ...)
        #[arg(long)]
        lang: Option<String>,
        /// Foil-Karte
        #[arg(long)]
        foil: bool,
        /// Beschreibung
        #[arg(long)]
        desc: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        // ─── Auth ────────────────────────────────────────
        Commands::Auth { action } => match action {
            AuthAction::Login => commands::auth::login().await?,
            AuthAction::Logout => commands::auth::logout().await?,
            AuthAction::Status => commands::auth::status().await?,
        },

        // ─── Search ──────────────────────────────────────
        Commands::Search { query, tcg, limit } => {
            let api = api::ApiClient::new(&config)?;
            commands::search::search(&api, &query, tcg.as_deref(), limit).await?;
        }

        // ─── Card Detail ─────────────────────────────────
        Commands::Card { query, tcg, art } => {
            let api = api::ApiClient::new(&config)?;
            commands::search::card_detail(&api, &query, tcg.as_deref(), art, &config.display.image_mode).await?;
        }

        // ─── Collections ─────────────────────────────────
        Commands::Collections => {
            let api = api::ApiClient::new(&config)?;
            commands::collection::list_collections(&api).await?;
        }

        Commands::Collection { name, sort, limit } => {
            let api = api::ApiClient::new(&config)?;
            commands::collection::show_collection(&api, &name, sort.as_deref(), limit).await?;
        }

        // ─── Stats ───────────────────────────────────────
        Commands::Stats { tcg } => {
            let api = api::ApiClient::new(&config)?;
            commands::collection::stats(&api, tcg.as_deref()).await?;
        }

        // ─── Trade ───────────────────────────────────────
        Commands::Trade { action } => {
            let api = api::ApiClient::new(&config)?;
            match action {
                TradeAction::Status => commands::trade::status(&api).await?,
                TradeAction::Offers => commands::trade::offers(&api).await?,
                TradeAction::Wants => commands::trade::wants(&api).await?,
                TradeAction::Matches { limit } => commands::trade::matches(&api, limit).await?,
                TradeAction::List { limit } => commands::trade::tradelist(&api, limit).await?,
            }
        }

        // ─── Market ──────────────────────────────────────
        Commands::Market { action } => {
            let api = api::ApiClient::new(&config)?;
            match action {
                MarketAction::Search { query, tcg, limit } => {
                    commands::market::search(&api, &query, tcg.as_deref(), limit).await?;
                }
                MarketAction::Listings => {
                    commands::market::my_listings(&api).await?;
                }
                MarketAction::Sell {
                    card_id,
                    price,
                    condition,
                    lang,
                    foil,
                    desc,
                } => {
                    commands::market::create_listing(
                        &api,
                        &card_id,
                        price,
                        condition.as_deref(),
                        lang.as_deref(),
                        foil,
                        desc.as_deref(),
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
