#!/usr/bin/env bash
#
# Builds "COSMIC Files.app" from the release binary so the app can be launched
# from Finder. See docs/macos-porting-notes.md section 5 for why the bundle is
# shaped this way.
#
# Usage: scripts/macos-bundle.sh
#
# CARGO_TARGET_DIR is honoured; it defaults to <repo>/target. The bundle is
# written to $CARGO_TARGET_DIR/macos/COSMIC Files.app.
#
# Build-machine prerequisites, copied into the bundle so the app needs neither
# on the machine it runs on:
#   COSMIC_ICON_THEME_DIR  the Cosmic icon theme, default ~/.local/share/icons/Cosmic
#                          (github.com/pop-os/cosmic-icons, `just install`)
#   SHARED_MIME_INFO_DIR   the shared MIME database, default /opt/homebrew/share/mime
#                          (brew install shared-mime-info)
#
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(dirname -- "$script_dir")
target_dir=${CARGO_TARGET_DIR:-$repo_root/target}

bin_name=cosmic-files
app_name="COSMIC Files"
binary="$target_dir/release/$bin_name"
app="$target_dir/macos/$app_name.app"

icon_theme_dir=${COSMIC_ICON_THEME_DIR:-$HOME/.local/share/icons/Cosmic}
mime_dir=${SHARED_MIME_INFO_DIR:-/opt/homebrew/share/mime}

# The release build. Linux-only default features are off; `quicklook` is in the
# default set but has to be re-listed because --no-default-features drops it.
build_features=bzip2,lzma-rust2,wgpu,quicklook

if [ ! -x "$binary" ]; then
    echo "==> building $bin_name (release)"
    (
        cd -- "$repo_root"
        CARGO_TARGET_DIR="$target_dir" cargo build --release \
            --no-default-features --features "$build_features"
    )
fi

echo "==> laying out $app"
rm -rf -- "$app"
mkdir -p -- "$app/Contents/MacOS" "$app/Contents/Resources"
cp -- "$binary" "$app/Contents/MacOS/$bin_name"

# The freedesktop data the app resolves icons through. Launched from Finder it
# has no XDG_DATA_DIRS and no Homebrew (porting notes 5.4), so both have to
# travel inside the bundle; src/launch_macos.rs points XDG_DATA_DIRS here.
copy_share() {
    local src=$1 dest=$2 what=$3 hint=$4
    if [ ! -d "$src" ]; then
        echo "error: no $what at $src" >&2
        echo "       install it ($hint) or set the variable to where it is" >&2
        exit 1
    fi
    mkdir -p -- "$(dirname -- "$dest")"
    # -R and not -L: the icon theme may use symlinks for aliased names, and
    # following them would multiply the copy.
    cp -R -- "$src" "$dest"
}

echo "==> copying the Cosmic icon theme from $icon_theme_dir"
copy_share "$icon_theme_dir" "$app/Contents/Resources/share/icons/Cosmic" \
    "Cosmic icon theme" "COSMIC_ICON_THEME_DIR, from github.com/pop-os/cosmic-icons"

echo "==> copying the shared MIME database from $mime_dir"
copy_share "$mime_dir" "$app/Contents/Resources/share/mime" \
    "shared MIME database" "SHARED_MIME_INFO_DIR, from brew install shared-mime-info"
# packages/ is update-mime-database's input, never read at runtime, and Homebrew
# ships it as a symlink into the Cellar. Left in place it dangles once copied,
# and codesign --verify then fails the whole bundle with ENOENT.
rm -rf -- "$app/Contents/Resources/share/mime/packages"

echo "==> writing Info.plist"
version=$(sed -n '/^\[package\]/,/^\[/ s/^version = "\(.*\)"/\1/p' \
    "$repo_root/Cargo.toml" | head -n 1)
if [ -z "$version" ]; then
    echo "error: no version found in $repo_root/Cargo.toml" >&2
    exit 1
fi
sed "s|@SHORT_VERSION@|$version|g" "$repo_root/res/macos/Info.plist.in" \
    > "$app/Contents/Info.plist"
plutil -lint -- "$app/Contents/Info.plist" > /dev/null

# The icon is generated rather than committed: the sources are the hicolor SVGs
# the Linux packaging already ships, and sips rasterises an SVG at any size, so
# even the 1024px retina representation comes out of the vector art rather than
# an upscale.
echo "==> generating cosmic-files.icns"
icons_src="$repo_root/res/icons/hicolor"
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
# iconutil insists on the .iconset extension.
iconset="$tmp/$bin_name.iconset"
mkdir -- "$iconset"

render() {
    local px=$1 out=$2
    # The SVG drawn for this size where there is one, so the small
    # representations keep their hinting; the 256px art otherwise.
    local src="$icons_src/${px}x${px}/apps/com.system76.CosmicFiles.svg"
    if [ ! -f "$src" ]; then
        src="$icons_src/256x256/apps/com.system76.CosmicFiles.svg"
    fi
    sips -s format png --resampleHeightWidth "$px" "$px" "$src" \
        --out "$iconset/$out" > /dev/null
}

render 16 icon_16x16.png
render 32 icon_16x16@2x.png
render 32 icon_32x32.png
render 64 icon_32x32@2x.png
render 128 icon_128x128.png
render 256 icon_128x128@2x.png
render 256 icon_256x256.png
render 512 icon_256x256@2x.png
render 512 icon_512x512.png
render 1024 icon_512x512@2x.png

iconutil --convert icns --output "$app/Contents/Resources/$bin_name.icns" "$iconset"

# A universal binary makes macOS re-prompt for privacy permissions on every
# launch and forget the grants in between; see porting notes 4.4.
arch_info=$(lipo -info "$app/Contents/MacOS/$bin_name")
case $arch_info in
*"Non-fat file"*"architecture: arm64") ;;
*)
    echo "error: the bundled binary must be thin arm64, got: $arch_info" >&2
    exit 1
    ;;
esac

# Ad-hoc, and without --options runtime, which crashes ad-hoc builds. Signing
# is inside-out: the executable first, the bundle last, so the bundle seal
# covers an already-valid executable. --deep is for verifying, never signing.
echo "==> signing (ad-hoc)"
xattr -crs "$app"
codesign --force -s - "$app/Contents/MacOS/$bin_name"
codesign --force -s - "$app"
codesign --verify --deep --strict "$app"

echo "==> done: $app"
