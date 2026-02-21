#!/bin/bash
set -euo pipefail

REPO="collected-cards/cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY="collected"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${CYAN}→${NC} $*"; }
ok()    { echo -e "${GREEN}✓${NC} $*"; }
fail()  { echo -e "${RED}✗${NC} $*" >&2; exit 1; }

echo -e "${BOLD}🃏 collected.cards CLI installer${NC}"
echo ""

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *)      fail "Unsupported OS: $OS (Linux and macOS only)" ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_SUFFIX="amd64" ;;
  aarch64|arm64) ARCH_SUFFIX="arm64" ;;
  *)             fail "Unsupported architecture: $ARCH (x86_64 and ARM64 only)" ;;
esac

ARTIFACT="${BINARY}-${PLATFORM}-${ARCH_SUFFIX}"
info "Detected: ${PLATFORM}/${ARCH_SUFFIX}"

# Get latest release URL
info "Fetching latest release..."
DOWNLOAD_URL=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep "browser_download_url.*${ARTIFACT}" \
  | head -1 \
  | cut -d '"' -f 4) || fail "Could not find release for ${ARTIFACT}"

[ -z "$DOWNLOAD_URL" ] && fail "No binary found for ${ARTIFACT} in latest release"

info "Downloading ${ARTIFACT}..."
TMP="$(mktemp)"
curl -fsSL "$DOWNLOAD_URL" -o "$TMP" || fail "Download failed"
chmod +x "$TMP"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "${INSTALL_DIR}/${BINARY}"
else
  info "Need sudo to install to ${INSTALL_DIR}"
  sudo mv "$TMP" "${INSTALL_DIR}/${BINARY}"
fi

ok "Installed ${BOLD}${BINARY}${NC} to ${INSTALL_DIR}/${BINARY}"
echo ""
echo -e "  Get started:"
echo -e "    ${CYAN}collected auth login${NC}    # Authenticate"
echo -e "    ${CYAN}collected search Pikachu${NC} # Search cards"
echo ""
echo -e "  ${BOLD}https://collected.cards/settings/cli${NC} to generate your token"
echo ""
