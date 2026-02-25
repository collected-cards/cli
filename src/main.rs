mod api;
mod cache;
mod commands;
mod config;
mod display;
mod i18n;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
#[derive(Parser)]
#[command(
    name = "collected",
    about = "🃏 collected.cards CLI — Your TCG collection in the terminal",
    version,
    author = "collected.cards"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication & token management
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Search cards
    Search {
        /// Search query
        query: String,
        /// Filter by TCG (mtg, pokemon, yugioh, ...)
        #[arg(long)]
        tcg: Option<String>,
        /// Max results
        #[arg(long, short)]
        limit: Option<i32>,
        /// Skip cache and fetch fresh data
        #[arg(long)]
        no_cache: bool,
    },

    /// Show card detail
    Card {
        /// Card name or ID
        query: String,
        /// Filter by TCG
        #[arg(long)]
        tcg: Option<String>,
        /// Show card art in terminal
        #[arg(long)]
        art: bool,
    },

    /// List collections
    Collections,

    /// Show collection
    Collection {
        /// Collection name or ID
        name: String,
        /// Sort by (name, price, date)
        #[arg(long)]
        sort: Option<String>,
        /// Max cards
        #[arg(long, short)]
        limit: Option<i32>,
    },

    /// Collection statistics (deprecated - use stats command)
    CollectionStats {
        /// Filter by TCG
        #[arg(long)]
        tcg: Option<String>,
    },

    /// Add a card to a collection
    Add {
        /// Card name to search for
        query: String,
        /// TCG (mtg, pokemon, yugioh, ...)
        #[arg(long)]
        tcg: Option<String>,
        /// Quantity
        #[arg(long, short)]
        quantity: Option<i32>,
        /// Condition (NM, LP, MP, HP, DMG)
        #[arg(long)]
        condition: Option<String>,
        /// Foil card
        #[arg(long)]
        foil: bool,
        /// Language (de, en, fr, ...)
        #[arg(long)]
        lang: Option<String>,
        /// Target collection name
        #[arg(long)]
        collection: Option<String>,
    },

    /// Remove an entry from a collection
    Remove {
        /// Entry ID
        entry_id: String,
    },

    /// Export collection(s)
    Export {
        /// Collection name (omit for all)
        collection: Option<String>,
        /// Format: csv, arena, moxfield, text
        #[arg(long, short, default_value = "csv")]
        format: String,
        /// Output file (omit for stdout)
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Import cards from file
    Import {
        /// CSV file path
        file: String,
        /// Target collection name
        #[arg(long)]
        collection: Option<String>,
    },

    /// Deck management
    Deck {
        #[command(subcommand)]
        action: DeckAction,
    },

    /// Wantlist management
    Wantlist {
        #[command(subcommand)]
        action: Option<WantlistAction>,
    },

    /// Price history
    Price {
        /// Card name
        query: String,
        /// TCG
        #[arg(long)]
        tcg: Option<String>,
        /// Period: 7d, 30d, 90d, 1y
        #[arg(long, default_value = "30d")]
        period: String,
    },

    /// Account settings
    Settings {
        /// Update email
        #[arg(long)]
        email: Option<String>,
        /// Update location
        #[arg(long)]
        location: Option<String>,
    },

    /// Account management
    Account {
        #[command(subcommand)]
        action: AccountAction,
    },

    /// Marketplace
    Market {
        #[command(subcommand)]
        action: MarketAction,
    },

    /// Trading — trade with other collectors
    Trade {
        #[command(subcommand)]
        action: TradeAction,
    },

    /// Platform and collection statistics
    Stats,

    /// Show a random card
    Random {
        /// Filter by TCG (mtg, pokemon, yugioh, ...)
        #[arg(long)]
        tcg: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell type
        shell: Shell,
    },

    /// Update CLI to latest version
    Update,

    /// Bulk add cards from file
    BulkAdd {
        /// Text file path (one card per line)
        file: String,
        /// Target collection name
        #[arg(long)]
        collection: Option<String>,
        /// TCG for all cards
        #[arg(long)]
        tcg: Option<String>,
    },

    /// Compare two cards side by side
    Compare {
        /// First card name
        card1: String,
        /// Second card name 
        card2: String,
        /// TCG (default: mtg)
        #[arg(long, default_value = "mtg")]
        tcg: String,
    },

    /// Show portfolio value with trend
    Portfolio,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Log in via browser and set up token
    Login,
    /// Log out (remove token)
    Logout,
    /// Show login status
    Status,
}

#[derive(Subcommand)]
enum DeckAction {
    /// List your decks
    List,
    /// Show deck contents
    Show {
        /// Deck name or ID
        name: String,
    },
    /// Export deck
    Export {
        /// Deck name or ID
        name: String,
        /// Format: arena, moxfield, text
        #[arg(long, short, default_value = "text")]
        format: String,
    },
    /// Show bracket analysis for a deck
    Bracket {
        /// Deck name or ID
        name: String,
    },
}

#[derive(Subcommand)]
enum WantlistAction {
    /// Add card to wantlist
    Add {
        /// Card name
        card_name: String,
        /// TCG
        #[arg(long)]
        tcg: Option<String>,
    },
    /// Remove from wantlist
    Remove {
        /// Wantlist entry ID
        id: String,
    },
}

#[derive(Subcommand)]
enum AccountAction {
    /// Delete your account permanently
    Delete,
}

#[derive(Subcommand)]
enum TradeAction {
    /// Trade status & profile
    Status,
    /// Offers (cards you want to trade)
    Offers,
    /// Wants (cards you're looking for)
    Wants,
    /// Matches — matching trade partners
    Matches {
        #[arg(long, short)]
        limit: Option<i32>,
    },
    /// Tradelist — all tradeable cards
    List {
        #[arg(long, short)]
        limit: Option<i32>,
    },
}

#[derive(Subcommand)]
enum MarketAction {
    /// Search marketplace
    Search {
        query: String,
        #[arg(long)]
        tcg: Option<String>,
        #[arg(long, short)]
        limit: Option<i32>,
    },
    /// Show own listings
    Listings,
    /// Create a listing
    Sell {
        /// Card ID
        card_id: String,
        /// Price in EUR
        #[arg(long)]
        price: f64,
        /// Condition (NM, LP, MP, HP, DMG)
        #[arg(long)]
        condition: Option<String>,
        /// Language
        #[arg(long)]
        lang: Option<String>,
        /// Foil
        #[arg(long)]
        foil: bool,
        /// Description
        #[arg(long)]
        desc: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    i18n::init();
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
        Commands::Search { query, tcg, limit, no_cache } => {
            let api = api::ApiClient::new(&config)?;
            commands::search::search(&api, &query, tcg.as_deref(), limit, no_cache).await?;
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

        // ─── Collection Stats (old) ──────────────────────
        Commands::CollectionStats { tcg } => {
            let api = api::ApiClient::new(&config)?;
            commands::collection::stats(&api, tcg.as_deref()).await?;
        }

        // ─── Add ─────────────────────────────────────────
        Commands::Add { query, tcg, quantity, condition, foil, lang, collection } => {
            let api = api::ApiClient::new(&config)?;
            commands::add::add_card(
                &api, &query, tcg.as_deref(), quantity,
                condition.as_deref(), foil, lang.as_deref(),
                collection.as_deref(),
            ).await?;
        }

        // ─── Remove ──────────────────────────────────────
        Commands::Remove { entry_id } => {
            let api = api::ApiClient::new(&config)?;
            commands::remove::remove_entry(&api, &entry_id).await?;
        }

        // ─── Export ──────────────────────────────────────
        Commands::Export { collection, format, output } => {
            let api = api::ApiClient::new(&config)?;
            commands::export::export(&api, collection.as_deref(), &format, output.as_deref()).await?;
        }

        // ─── Import ──────────────────────────────────────
        Commands::Import { file, collection } => {
            let api = api::ApiClient::new(&config)?;
            commands::import::import_file(&api, &file, collection.as_deref()).await?;
        }

        // ─── Deck ────────────────────────────────────────
        Commands::Deck { action } => {
            let api = api::ApiClient::new(&config)?;
            match action {
                DeckAction::List => commands::deck::list_decks(&api).await?,
                DeckAction::Show { name } => commands::deck::show_deck(&api, &name).await?,
                DeckAction::Export { name, format } => commands::deck::export_deck(&api, &name, &format).await?,
                DeckAction::Bracket { name } => commands::deck_bracket::deck_bracket_analysis(&api, &name).await?,
            }
        }

        // ─── Wantlist ────────────────────────────────────
        Commands::Wantlist { action } => {
            let api = api::ApiClient::new(&config)?;
            match action {
                None => commands::wantlist::show_wantlist(&api).await?,
                Some(WantlistAction::Add { card_name, tcg }) => {
                    commands::wantlist::add_to_wantlist(&api, &card_name, tcg.as_deref()).await?;
                }
                Some(WantlistAction::Remove { id }) => {
                    commands::wantlist::remove_from_wantlist(&api, &id).await?;
                }
            }
        }

        // ─── Price ───────────────────────────────────────
        Commands::Price { query, tcg, period } => {
            let api = api::ApiClient::new(&config)?;
            commands::price::price_history(&api, &query, tcg.as_deref(), &period).await?;
        }

        // ─── Settings ────────────────────────────────────
        Commands::Settings { email, location } => {
            let api = api::ApiClient::new(&config)?;
            commands::settings::settings(&api, email.as_deref(), location.as_deref()).await?;
        }

        // ─── Account ─────────────────────────────────────
        Commands::Account { action } => {
            let api = api::ApiClient::new(&config)?;
            match action {
                AccountAction::Delete => commands::account::delete_account(&api).await?,
            }
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
                    card_id, price, condition, lang, foil, desc,
                } => {
                    commands::market::create_listing(
                        &api, &card_id, price, condition.as_deref(),
                        lang.as_deref(), foil, desc.as_deref(),
                    ).await?;
                }
            }
        }

        // ─── New Commands ────────────────────────────────
        Commands::Stats => {
            let api = api::ApiClient::new(&config)?;
            commands::stats::platform_stats(&api).await?;
        }

        Commands::Random { tcg } => {
            let api = api::ApiClient::new(&config)?;
            commands::random::random_card(&api, tcg.as_deref()).await?;
        }

        Commands::Completions { shell } => {
            commands::completions::generate_completions(shell)?;
        }

        Commands::Update => {
            commands::update::self_update().await?;
        }

        Commands::BulkAdd { file, collection, tcg } => {
            let api = api::ApiClient::new(&config)?;
            commands::bulk_add::bulk_add_cards(&api, &file, collection.as_deref(), tcg.as_deref()).await?;
        }

        Commands::Compare { card1, card2, tcg } => {
            let api = api::ApiClient::new(&config)?;
            commands::compare::compare_cards(&api, &card1, &card2, Some(&tcg)).await?;
        }

        Commands::Portfolio => {
            let api = api::ApiClient::new(&config)?;
            commands::portfolio::portfolio(&api).await?;
        }
    }

    Ok(())
}
