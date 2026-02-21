# 🃏 collected — TCG Collection CLI

Command-line tool for [collected.cards](https://collected.cards) — manage your Trading Card Game collection from the terminal.

Supports **Magic: The Gathering**, **Pokémon**, **Yu-Gi-Oh!**, **Lorcana**, **One Piece**, **Flesh and Blood**, **Star Wars: Unlimited**, **Digimon**, **Dragon Ball Super**, **Dragon Ball Fusion World**, **Battle Spirits** and **Force of Will**.

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
# Example: Linux x86_64
chmod +x collected-linux-amd64
sudo mv collected-linux-amd64 /usr/local/bin/collected
```

## 🔨 Build from Source

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
git clone https://github.com/collected-cards/cli.git
cd collected-cli
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

## 📖 Usage

```
🃏 collected.cards CLI — Deine TCG-Sammlung im Terminal

Usage: collected <COMMAND>

Commands:
  search      Search cards across all TCGs
  card        Show card details
  collection  Manage collections
  stats       Platform statistics
  market      Marketplace
  trade       Trading
  auth        Authentication
  help        Print help
```

### Search Cards

```bash
# Search across all TCGs
collected search "Black Lotus"

# Filter by TCG
collected search "Charizard" --tcg pokemon

# Limit results
collected search "Dark Magician" --limit 5
```

### View Card Details

```bash
collected card <card-id>
```

Shows price, set info, and card image right in your terminal (Sixel/Kitty/iTerm2 supported, falls back to Unicode blocks).

### Collections

```bash
# List your collections
collected collection list

# Show cards in a collection
collected collection show <collection-id>

# Collection statistics
collected stats
```

### Marketplace

```bash
# Search listings
collected market search "Mew"

# Your active listings
collected market mine
```

### Trading

```bash
# Your trade profile & access
collected trade status

# Cards you offer for trade
collected trade offers

# Cards you're looking for
collected trade wants

# Find trade matches
collected trade matches --limit 10

# Your full tradelist
collected trade list
```

## ⚙️ Configuration

Config is stored at `~/.config/collected/config.toml`:

```toml
[api]
endpoint = "https://api.collected.cards/graphql"

[auth]
token = "your-token-here"
```

## 🛡️ License

[Custom License](LICENSE) — free to use and modify. **No commercial use, no weapons/military.**

## 🔗 Links

- **Website:** [collected.cards](https://collected.cards)
- **Issues:** [GitHub Issues](https://github.com/collected-cards/cli/issues)
