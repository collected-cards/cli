# 🃏 collected — TCG Collection CLI

Command-line tool for [collected.cards](https://collected.cards) — manage your Trading Card Game collection from the terminal.

Supports **Magic: The Gathering**, **Pokémon**, **Yu-Gi-Oh!**, **Lorcana**, **One Piece**, **Flesh and Blood**, **Star Wars: Unlimited**, **Digimon**, **Dragon Ball Super**, **Dragon Ball Fusion World**, **Battle Spirits** and **Force of Will**.

**🌍 Multi-language** — auto-detects your system language (DE, EN, FR, ES, JA).

## ⚡ Quick Install

```bash
curl -fsSL https://collected.cards/install.sh | bash
```

Or with wget:
```bash
wget -qO- https://collected.cards/install.sh | bash
```

## 📦 Manual Install

Download the latest binary for your platform from [Releases](https://github.com/collected-cards/cli/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `collected-linux-amd64` |
| Linux ARM64 | `collected-linux-arm64` |
| macOS x86_64 (Intel) | `collected-macos-amd64` |
| macOS ARM64 (Apple Silicon) | `collected-macos-arm64` |

```bash
chmod +x collected-linux-amd64
sudo mv collected-linux-amd64 /usr/local/bin/collected
```

## 🔨 Build from Source

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
git clone https://github.com/collected-cards/cli.git
cd cli
cargo build --release
# Binary at target/release/collected
```

## 🔐 Authentication

1. Log in to [collected.cards](https://collected.cards) in your browser
2. Go to [Settings → CLI](https://collected.cards/settings/cli) and generate a token
3. Run:

```bash
collected auth login
# Paste your token when prompted
```

Token is stored securely in `~/.config/collected/config.toml` (file permissions 600).

## 📖 Commands

```
collected <COMMAND>

  auth         Authentication (login/logout/status)
  search       Search cards
  card         Show card detail with terminal image
  collections  List collections
  collection   Show collection entries
  stats        Collection statistics
  add          Add a card to a collection
  remove       Remove an entry
  export       Export collection (CSV/Arena/Moxfield/Text)
  import       Import cards from CSV file
  deck         Deck management (list/show/export)
  wantlist     Wantlist (list/add/remove)
  price        Price history with ASCII chart
  settings     Account settings (email/location)
  account      Account management (delete)
  market       Marketplace (search/listings/sell)
  trade        Trading (status/offers/wants/matches/list)
```

### Search & Card Detail

```bash
# Search across all TCGs
collected search "Black Lotus"

# Filter by TCG
collected search "Charizard" --tcg pokemon

# Card detail with terminal image
collected card "Black Lotus" --tcg mtg --art
```

### Collections

```bash
# List your collections
collected collections

# Show cards in a collection
collected collection "MTG Main" --limit 20

# Collection statistics
collected stats
```

### Add & Remove Cards

```bash
# Interactive: search → pick card → pick collection
collected add "Lightning Bolt" --tcg mtg

# With options
collected add "Pikachu" --tcg pokemon --quantity 2 --condition NM --foil

# Remove an entry
collected remove <entry-id>
```

### Import & Export

```bash
# Export collection as CSV
collected export "MTG Main" --format csv --output cards.csv

# Export as Arena format
collected export "MTG Main" --format arena

# Import from CSV (auto-detects columns, processes in batches)
collected import cards.csv --collection "MTG Main"
```

### Decks

```bash
# List your decks
collected deck list

# Show deck contents
collected deck show "Commander Deck"

# Export deck
collected deck export "Commander Deck" --format arena
```

### Wantlist

```bash
# Show your wantlist
collected wantlist

# Add card to wantlist
collected wantlist add "Mew" --tcg pokemon

# Remove from wantlist
collected wantlist remove <id>
```

### Price History

```bash
# Show price chart (default: 30 days)
collected price "Black Lotus" --tcg mtg

# Different periods
collected price "Charizard" --tcg pokemon --period 90d
```

### Trading

```bash
# Trade profile & access status
collected trade status

# Cards you offer / want
collected trade offers
collected trade wants

# Find matches
collected trade matches --limit 10

# Your full tradelist
collected trade list
```

### Marketplace

```bash
# Search listings
collected market search "Mew"

# Your active listings
collected market listings

# Create listing
collected market sell <card-id> --price 9.99 --condition NM
```

### Settings & Account

```bash
# View current settings
collected settings

# Update email / location
collected settings --email me@example.com
collected settings --location "München, Bayern"

# Delete account (triple confirmation)
collected account delete
```

## 🌍 Language

The CLI auto-detects your system language from `LANG`, `LC_ALL`, or `LC_MESSAGES`:

| Language | Example |
|----------|---------|
| 🇬🇧 English | `LANG=en_US.UTF-8` |
| 🇩🇪 Deutsch | `LANG=de_DE.UTF-8` |
| 🇫🇷 Français | `LANG=fr_FR.UTF-8` |
| 🇪🇸 Español | `LANG=es_ES.UTF-8` |
| 🇯🇵 日本語 | `LANG=ja_JP.UTF-8` |

Override temporarily: `LANG=ja_JP.UTF-8 collected search "Pikachu"`

## ⚙️ Configuration

Config is stored at `~/.config/collected/config.toml` (permissions `600`):

```toml
[api]
endpoint = "https://api.collected.cards/graphql"

[auth]
token = "your-token-here"
```

## 🛡️ Security

- Token file protected with `chmod 600` (owner-only read/write)
- Token validated against API before saving to disk
- TLS certificate validation enforced, HTTPS-only
- API error messages sanitized (no internal details leaked)
- Account deletion requires triple confirmation
- CSV import uses batch processing (handles large files)

## 📜 License

[Custom License](LICENSE) — free to use and modify. **No commercial use, no weapons/military.**

## 🔗 Links

- **Website:** [collected.cards](https://collected.cards)
- **Issues:** [GitHub Issues](https://github.com/collected-cards/cli/issues)
