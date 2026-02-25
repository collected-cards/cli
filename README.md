# 🃏 collected — TCG Collection CLI

Command-line tool for [collected.cards](https://collected.cards) — manage your Trading Card Game collection from the terminal.

Supports **12 TCGs**: Magic: The Gathering, Pokémon, Yu-Gi-Oh!, Lorcana, One Piece, Flesh and Blood, Star Wars: Unlimited, Digimon, Dragon Ball Super, Dragon Ball Fusion World, Battle Spirits, and Force of Will.

**🌍 22 Languages** — auto-detects your system language.

## ⚡ Quick Install

```bash
curl -fsSL https://collected.cards/install.sh | bash
```

Or with wget:
```bash
wget -qO- https://collected.cards/install.sh | bash
```

## 📦 Manual Install

Download the latest binary from [Releases](https://github.com/collected-cards/cli/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `collected-linux-amd64` |
| Linux ARM64 | `collected-linux-arm64` |
| macOS x86_64 (Intel) | `collected-macos-amd64` |
| macOS ARM64 (Apple Silicon) | `collected-macos-arm64` |
| Windows x86_64 | `collected-windows-amd64.exe` |
| Windows ARM64 | `collected-windows-arm64.exe` |

**Linux / macOS:**
```bash
chmod +x collected-linux-amd64
sudo mv collected-linux-amd64 /usr/local/bin/collected
```

**Windows:**
Download `collected-windows-amd64.exe`, rename to `collected.exe`, and add to your PATH.

## 🔨 Build from Source

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
git clone https://github.com/collected-cards/cli.git
cd cli
cargo build --release
```

## 🔐 Authentication

1. Log in to [collected.cards](https://collected.cards)
2. Go to **Settings → CLI** and generate a token
3. Run `collected auth login` and paste your token

## 📖 Commands

### Search & Discovery

```bash
# Search cards (supports all 12 TCGs)
collected search "Lightning Bolt" --tcg mtg
collected search "Charizard" --tcg pokemon
collected search "Dark Magician" --tcg yugioh

# Card detail with virtual card display
collected card "Rhystic Study"

# Card detail with terminal image
collected card "Rhystic Study" --art

# Random card
collected random --tcg mtg

# Compare two cards side by side
collected compare "Sol Ring" "Mana Crypt"

# Price history with ASCII chart
collected price "Black Lotus" --period 90d
```

Example output:
```
╭──────────────────────────────────────────────╮
│ Sheoldred, the Apocalypse              2◈B◈B │
├──────────────────────────────────────────────┤
│ Legendary Creature — Phyrexian Praetor ⚔ 4/5 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Deathtouch                                    │
│ Whenever you draw a card, you gain 2 life.    │
│ Whenever an opponent draws a card, they lose  │
│ 2 life.                                       │
├──────────────────────────────────────────────┤
│ DMU · #436 · mythic                           │
│ €127.06                                       │
│ Dominaria United                              │
╰──────────────────────────────────────────────╯
```

### Collections & Portfolio

```bash
# List your collections
collected collections

# Show cards in a collection
collected collection "MTG Main" --limit 20

# Platform & personal statistics
collected stats

# Portfolio overview
collected portfolio
```

### Add & Remove Cards

```bash
# Interactive: search → pick card → pick collection
collected add "Lightning Bolt" --tcg mtg

# With options
collected add "Pikachu" --tcg pokemon --quantity 2 --condition NM --foil

# Bulk add from file
collected bulk-add cards.txt --collection "Main" --tcg mtg

# Remove an entry
collected remove <entry-id>
```

### Import & Export

```bash
# Export as CSV, Arena, Moxfield, or text
collected export "MTG Main" --format csv --output cards.csv
collected export "MTG Main" --format arena

# Import from CSV
collected import cards.csv --collection "MTG Main"
```

### Decks

```bash
collected deck list
collected deck show "Commander Deck"
collected deck export "Commander Deck" --format arena

# Commander bracket analysis (MTG)
collected deck bracket "Commander Deck"
```

### Wantlist

```bash
collected wantlist
collected wantlist add "Mew" --tcg pokemon
collected wantlist remove <id>
```

### Trading

```bash
collected trade status
collected trade offers
collected trade wants
collected trade matches
collected trade list
```

### Marketplace

```bash
collected market search "Mew"
collected market listings
collected market sell <card-id> --price 9.99
```

### Settings

```bash
collected settings
collected settings --email me@example.com
collected settings --location "Berlin"
collected account delete
```

### Shell Completions

```bash
# Auto-installs to the correct location
collected completions bash
collected completions zsh
collected completions fish
```

### Self-Update

```bash
collected update
```

## 🌍 Languages

Auto-detects from `LANG` / `LC_ALL`. Override: `LANG=ja_JP.UTF-8 collected search "Pikachu"`

🇬🇧 🇩🇪 🇫🇷 🇪🇸 🇯🇵 🇮🇹 🇧🇷 🇰🇷 🇨🇳 🇷🇺 🇳🇱 🇵🇱 🇹🇷 🇷🇴 🇧🇦 🇦🇱 🇭🇷 🇷🇸 🇭🇺 🇨🇿 🇸🇪 🇬🇷

## 🛡️ Security

- Config file protected with `chmod 600`
- Token validated before saving
- TLS enforced, certificate validation enabled
- Error messages sanitized
- Account deletion with triple confirmation

## 📜 License

[Custom License](LICENSE) — free to use and modify. No commercial use, no weapons/military.

## 🔗 Links

- **Website:** [collected.cards](https://collected.cards)
- **Issues:** [GitHub Issues](https://github.com/collected-cards/cli/issues)
