#!/usr/bin/env bash
set -euo pipefail

# DX Terminal installer
# Usage: curl -fsSL https://raw.githubusercontent.com/pdaxt/dx-terminal/main/install.sh | bash

VERSION="${DX_VERSION:-0.3.0}"
INSTALL_DIR="${DX_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      arm64)   TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      aarch64) TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

ARCHIVE="dx-terminal-v${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/pdaxt/dx-terminal/releases/download/v${VERSION}/${ARCHIVE}"

echo "Installing DX Terminal v${VERSION} for ${TARGET}..."

# Download and extract
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "Downloading ${URL}..."
curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"

echo "Extracting..."
tar xzf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "${TMPDIR}/dx" "${INSTALL_DIR}/dx"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "${TMPDIR}/dx" "${INSTALL_DIR}/dx"
fi

chmod +x "${INSTALL_DIR}/dx"

echo ""
echo "DX Terminal v${VERSION} installed to ${INSTALL_DIR}/dx"
echo "Run 'dx' to start."
