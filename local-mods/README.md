# COSMIC desktop customizations

Tested on Pop!_OS 24.04, COSMIC Files 1.11.0, source revision `3cc4b80`, Rust 1.95.

## Dependencies

```sh
sudo apt install cargo rustc libclang-dev libglib2.0-dev libxkbcommon-dev pkg-config build-essential python3-gi gir1.2-gtk-3.0 gir1.2-gtklayershell-0.1
```

Rust 1.93 or newer is required. Pop!_OS provides a newer compiler than stock Ubuntu 24.04.

## Transparent desktop labels

```sh
./local-mods/desktop-labels.sh build
./local-mods/desktop-labels.sh install
pkill -f '^(/usr/bin/)?cosmic-files-applet( |$)'
```

COSMIC Session restarts the desktop applet. The file manager and your documents remain open. The only rendering change is in `src/tab.rs`, in the desktop branch of `button_appearance`: no background for idle labels. Hover and selection still use the upstream styles. Text has no extra shadow, so contrast depends on your wallpaper.

The installer uses a local `dpkg-divert` to preserve `/usr/bin/cosmic-files-applet.distrib`. Package updates refresh that original rather than overwriting the custom executable. Rebuild/rebase this fork to receive updates in the customized executable; restore the distribution version if you do not want to maintain a fork.

Restore:

```sh
./local-mods/desktop-labels.sh restore
pkill -f '^(/usr/bin/)?cosmic-files-applet( |$)'
```

## Bottom-right Show Desktop button

```sh
./local-mods/install-corner.sh
```

A GTK 3 layer-shell surface sits on the bottom-right corner of each monitor. It does not reserve screen space or take keyboard focus. Left-click toggles minimization through a small Rust service using COSMIC toplevel/workspace protocols. No compositor patches, input-device access, pointer polling or global key injection are used.

The restore snapshot stays in memory; restarting the controller loses that snapshot, but windows can still be restored from the dock. Only windows captured by the action are restored. Fullscreen apps can be overlapped by the 10 × 36 corner surface. Multi-monitor placement follows monitor hotplug events. The action covers active workspaces across monitors rather than only the clicked monitor.

Diagnostics:

```sh
~/.local/bin/cosmic-show-desktop-control --status
systemctl --user status cosmic-show-desktop-control.service cosmic-show-desktop-corner.service
```

Stop it and disable login activation:

```sh
./local-mods/remove-corner.sh
```

## Validation

- Release build of cosmic-files-applet and its dynamic library dependencies checked.
- Release build and unit test of the show-desktop controller.
- Live minimize/restore cycle checked separately on the installed COSMIC session.
- Shell syntax, Python syntax and Git whitespace checks.

## Attribution and licenses

COSMIC Files is copyright its upstream authors and licensed GPL-3.0-only; see the root LICENSE.
The window controller is adapted from [Samkit Jain's cosmic-ext-showdesktop](https://github.com/SamkitJain660/cosmic-ext-showdesktop), MIT licensed. Its MIT notice is retained in `show-desktop/LICENSE`.
The corner Python script and helper scripts are provided under MIT; see `LICENSE` in this directory.
