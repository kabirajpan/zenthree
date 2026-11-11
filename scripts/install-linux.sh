#!/usr/bin/env bash
# Zenthree Linux Installer
set -e

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"

echo "==> Installing Zenthree..."

mkdir -p "${BIN_DIR}"
mkdir -p "${APP_DIR}"
mkdir -p "${ICON_DIR}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Install binary
if [ -f "${SCRIPT_DIR}/zenthree" ]; then
    cp "${SCRIPT_DIR}/zenthree" "${BIN_DIR}/zenthree"
    chmod +x "${BIN_DIR}/zenthree"
elif [ -f "${SCRIPT_DIR}/target/release/zenthree" ]; then
    cp "${SCRIPT_DIR}/target/release/zenthree" "${BIN_DIR}/zenthree"
    chmod +x "${BIN_DIR}/zenthree"
fi

# Install desktop entry and icon
if [ -f "${SCRIPT_DIR}/assets/linux/zenthree.desktop" ]; then
    cp "${SCRIPT_DIR}/assets/linux/zenthree.desktop" "${APP_DIR}/zenthree.desktop"
elif [ -f "${SCRIPT_DIR}/zenthree.desktop" ]; then
    cp "${SCRIPT_DIR}/zenthree.desktop" "${APP_DIR}/zenthree.desktop"
fi

if [ -f "${SCRIPT_DIR}/assets/logo/zenthree.png" ]; then
    cp "${SCRIPT_DIR}/assets/logo/zenthree.png" "${ICON_DIR}/zenthree.png"
elif [ -f "${SCRIPT_DIR}/zenthree.png" ]; then
    cp "${SCRIPT_DIR}/zenthree.png" "${ICON_DIR}/zenthree.png"
fi

# Update desktop database if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APP_DIR}" 2>/dev/null || true
fi

echo "==> Zenthree installed successfully!"
echo "    Executable: ${BIN_DIR}/zenthree"
echo "    Make sure ${BIN_DIR} is in your PATH."
