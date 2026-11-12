#!/usr/bin/env bash
# Zenthree Linux Uninstaller
set -e

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"

echo "==> Uninstalling Zenthree from user directory..."

# 1. Remove binary
if [ -f "${BIN_DIR}/zenthree" ]; then
    rm -f "${BIN_DIR}/zenthree"
    echo "    Removed ${BIN_DIR}/zenthree"
fi

# 2. Remove desktop launcher
if [ -f "${APP_DIR}/zenthree.desktop" ]; then
    rm -f "${APP_DIR}/zenthree.desktop"
    echo "    Removed ${APP_DIR}/zenthree.desktop"
fi

# 3. Remove icons
for size in 16 24 32 48 64 128 256 512; do
    icon_file="${HOME}/.local/share/icons/hicolor/${size}x${size}/apps/zenthree.png"
    if [ -f "${icon_file}" ]; then
        rm -f "${icon_file}"
    fi
done
svg_icon="${HOME}/.local/share/icons/hicolor/scalable/apps/zenthree.svg"
if [ -f "${svg_icon}" ]; then
    rm -f "${svg_icon}"
fi

# 4. Refresh desktop environment cache
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APP_DIR}" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "==> Zenthree has been completely uninstalled."

