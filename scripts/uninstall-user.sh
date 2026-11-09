#!/bin/sh
set -eu
base=${XDG_DATA_HOME:-"$HOME/.local/share"}
rm -f "$base/applications/org.telerust.Client.desktop" "$base/icons/hicolor/scalable/apps/org.telerust.Client.svg"
rm -rf "$base/telerust"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$base/applications" || true
printf '%s\n' 'Removed the Telerust application files. User configuration and Telegram session data were preserved.'
