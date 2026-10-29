#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary=/usr/bin/cosmic-files-applet
case "${1:-}" in
 build)
  cd "$repo"
  CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release --locked -p cosmic-files-applet -j 4
  ;;
 install)
  if (( EUID != 0 )); then exec pkexec "$repo/local-mods/desktop-labels.sh" install; fi
  test -x "$repo/target/release/cosmic-files-applet"
  if test -n "$(dpkg-divert --list "$binary")"; then
    echo "Existing diversion: restore or review it first." >&2
    exit 1
  fi
  dpkg-divert --local --rename --add "$binary"
  if ! install -m755 "$repo/target/release/cosmic-files-applet" "$binary"; then
    dpkg-divert --local --rename --remove "$binary"
    exit 1
  fi
  ;;
 restore)
  if (( EUID != 0 )); then exec pkexec "$repo/local-mods/desktop-labels.sh" restore; fi
  test -f "$binary.distrib"
  # Keep the custom binary in the source checkout for a reversible restore.
  if test -f "$binary"; then mv "$binary" "$repo/cosmic-files-applet.retired"; fi
  dpkg-divert --local --rename --remove "$binary"
  ;;
 *) echo 'Usage: desktop-labels.sh build|install|restore' >&2; exit 2;;
esac
