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
tdlib_source=${TELERUST_TDLIB_SOURCE:-}
if [ -n "$tdlib_source" ]; then
  if [ ! -f "$tdlib_source" ] || [ ! -r "$tdlib_source" ]; then
    echo "TELERUST_TDLIB_SOURCE must point to a readable libtdjson.so." >&2
    exit 1
  fi
  cargo run --quiet --manifest-path "$repo_dir/Cargo.toml" -p telerust-tdlib-bridge --example probe -- "$tdlib_source"
  mkdir -p "$base/telerust/lib"
  chmod 700 "$base/telerust/lib"
  install -m 644 "$tdlib_source" "$base/telerust/lib/libtdjson.so"
elif [ -f "$base/telerust/lib/libtdjson.so" ]; then
  cargo run --quiet --manifest-path "$repo_dir/Cargo.toml" -p telerust-tdlib-bridge --example probe -- "$base/telerust/lib/libtdjson.so"
fi
# Desktop Exec must be an absolute path and shell expansion is not supported.
sed "s|^Exec=telerust$|Exec=$base/telerust/telerust|" "$base/applications/org.telerust.Client.desktop" > "$base/applications/org.telerust.Client.desktop.tmp"
mv "$base/applications/org.telerust.Client.desktop.tmp" "$base/applications/org.telerust.Client.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$base/applications" || true
if [ -f "$base/telerust/lib/libtdjson.so" ]; then
  printf '%s\n' "Installed Telerust and its verified user-local TDLib runtime."
else
  printf '%s\n' "Installed Telerust. Install TDLib or rerun with TELERUST_TDLIB_SOURCE=/path/to/libtdjson.so."
fi
printf '%s\n' "Uninstall with: $repo_dir/scripts/uninstall-user.sh"
