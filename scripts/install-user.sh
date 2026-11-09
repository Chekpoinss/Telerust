#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cargo build --release -p telerust
base=${XDG_DATA_HOME:-"$HOME/.local/share"}
mkdir -p "$base/applications" "$base/icons/hicolor/scalable/apps" "$base/telerust/bin"
install -m 755 "$repo_dir/target/release/telerust" "$base/telerust/bin/telerust"
install -m 755 "$repo_dir/scripts/telerust-launcher" "$base/telerust/telerust"
install -m 644 "$repo_dir/packaging/org.telerust.Client.desktop" "$base/applications/org.telerust.Client.desktop"
install -m 644 "$repo_dir/packaging/icons/hicolor/scalable/apps/org.telerust.Client.svg" "$base/icons/hicolor/scalable/apps/org.telerust.Client.svg"
# Desktop Exec must be an absolute path and shell expansion is not supported.
sed "s|^Exec=telerust$|Exec=$base/telerust/telerust|" "$base/applications/org.telerust.Client.desktop" > "$base/applications/org.telerust.Client.desktop.tmp"
mv "$base/applications/org.telerust.Client.desktop.tmp" "$base/applications/org.telerust.Client.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$base/applications" || true
printf '%s\n' "Installed Telerust for this user. TDLib is loaded automatically if libtdjson.so is in the system library path." "Uninstall with: $repo_dir/scripts/uninstall-user.sh"
