#!/bin/sh
set -eu
base=${XDG_DATA_HOME:-"$HOME/.local/share"}
rm -f "$base/applications/org.telerust.Client.desktop" "$base/icons/hicolor/scalable/apps/org.telerust.Client.svg"
rm -rf "$base/telerust/bin" "$base/telerust/lib"
rm -f "$base/telerust/telerust"
rmdir "$base/telerust" 2>/dev/null || true
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$base/applications" || true
printf '%s\n' 'Removed Telerust executable, runtime library, desktop entry, and icon. Configuration and Telegram session databases were preserved.'
