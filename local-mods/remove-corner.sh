#!/usr/bin/env bash
set -euo pipefail
systemctl --user stop cosmic-show-desktop-corner.service cosmic-show-desktop-control.service
# Preserve files for a reversible removal; disable login activation.
if test -f "$HOME/.config/autostart/cosmic-show-desktop.desktop"; then
    mv "$HOME/.config/autostart/cosmic-show-desktop.desktop" "$HOME/.config/autostart/cosmic-show-desktop.desktop.disabled"
fi
