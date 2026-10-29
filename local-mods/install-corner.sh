#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
export CARGO_TARGET_DIR="$repo/target"
cargo build --release --locked --manifest-path "$repo/local-mods/show-desktop/Cargo.toml" -j 4
# Stop only our own services before replacing a running executable.
if test -f "$HOME/.config/systemd/user/cosmic-show-desktop-control.service"; then
    systemctl --user stop cosmic-show-desktop-corner.service cosmic-show-desktop-control.service
fi
install -Dm755 "$repo/target/release/cosmic-show-desktop-control" "$HOME/.local/bin/cosmic-show-desktop-control"
install -Dm755 "$repo/local-mods/show-desktop/corner.py" "$HOME/.local/bin/cosmic-show-desktop-corner"
mkdir -p "$HOME/.config/systemd/user" "$HOME/.config/autostart"
cat > "$HOME/.config/systemd/user/cosmic-show-desktop-control.service" <<EOF
[Unit]
Description=COSMIC show desktop window controller
PartOf=graphical-session.target
[Service]
ExecStart=%h/.local/bin/cosmic-show-desktop-control
Restart=on-failure
RestartSec=3
EOF
cat > "$HOME/.config/systemd/user/cosmic-show-desktop-corner.service" <<EOF
[Unit]
Description=Bottom right show desktop button
Requires=cosmic-show-desktop-control.service
After=cosmic-show-desktop-control.service
PartOf=graphical-session.target
[Service]
Environment=GDK_BACKEND=wayland
ExecStart=%h/.local/bin/cosmic-show-desktop-corner
Restart=on-failure
RestartSec=3
EOF
cat > "$HOME/.config/autostart/cosmic-show-desktop.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Mostrar escritorio
Exec=systemctl --user start cosmic-show-desktop-corner.service
X-GNOME-Autostart-enabled=true
EOF
systemctl --user daemon-reload
systemctl --user restart cosmic-show-desktop-corner.service
