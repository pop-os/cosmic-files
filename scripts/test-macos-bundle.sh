#!/usr/bin/env bash
#
# Tests for scripts/macos-bundle.sh.
#
# Runs the bundler once, then asserts the properties a macOS application bundle
# must have for Finder to launch it: the Contents layout, the Info.plist keys,
# a valid signature, and a single-architecture executable.
#
# Usage: scripts/test-macos-bundle.sh
#
set -uo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(dirname -- "$script_dir")
target_dir=${CARGO_TARGET_DIR:-$repo_root/target}
app="$target_dir/macos/COSMIC Files.app"

passed=0
failed=0

ok() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

no() {
    printf 'FAIL %s\n       %s\n' "$1" "${2-}"
    failed=$((failed + 1))
}

assert_dir() {
    if [ -d "$1" ]; then
        ok "$2"
    else
        no "$2" "not a directory: $1"
    fi
}

assert_executable() {
    if [ -x "$1" ] && [ -f "$1" ]; then
        ok "$2"
    else
        no "$2" "not an executable file: $1"
    fi
}

echo "# running the bundler"
if ! "$script_dir/macos-bundle.sh"; then
    echo "FAIL the bundler exited non-zero; no further assertions are meaningful" >&2
    exit 1
fi

echo "# bundle structure"
assert_dir "$app" "the bundle is at target/macos/COSMIC Files.app"
assert_dir "$app/Contents/MacOS" "Contents/MacOS exists"
assert_dir "$app/Contents/Resources" "Contents/Resources exists"
assert_executable "$app/Contents/MacOS/cosmic-files" "Contents/MacOS/cosmic-files is executable"

echo "# Info.plist"
plist="$app/Contents/Info.plist"

assert_plist_value() {
    local key=$1 expected=$2 name=$3 actual
    actual=$(plutil -extract "$key" raw -o - -- "$plist" 2>&1)
    if [ "$actual" = "$expected" ]; then
        ok "$name"
    else
        no "$name" "$key is '$actual', expected '$expected'"
    fi
}

assert_plist_absent() {
    local key=$1 name=$2
    if plutil -extract "$key" raw -o - -- "$plist" >/dev/null 2>&1; then
        no "$name" "$key is present and should not be"
    else
        ok "$name"
    fi
}

if plutil -lint -- "$plist" >/dev/null 2>&1; then
    ok "Info.plist is a well-formed property list"
else
    no "Info.plist is a well-formed property list" "plutil -lint failed for $plist"
fi

# The bundle identifier is the key TCC grants attach to; see porting notes 4.1.
assert_plist_value CFBundleIdentifier com.system76.CosmicFiles \
    "CFBundleIdentifier is the fixed com.system76.CosmicFiles"
assert_plist_value CFBundleExecutable cosmic-files \
    "CFBundleExecutable names the file in Contents/MacOS"
assert_plist_value CFBundlePackageType APPL "CFBundlePackageType is APPL"
assert_plist_value CFBundleName "COSMIC Files" "CFBundleName is set"
assert_plist_value CFBundleDisplayName "COSMIC Files" "CFBundleDisplayName is set"
assert_plist_value CFBundleIconFile cosmic-files "CFBundleIconFile names the icns"
# Without this the bundle renders 1x magnified; see porting notes 5.2.
assert_plist_value NSHighResolutionCapable true "NSHighResolutionCapable is true"
# NO opts the app in to the system dark appearance.
assert_plist_value NSRequiresAquaSystemAppearance false \
    "NSRequiresAquaSystemAppearance is false"
assert_plist_value LSMinimumSystemVersion 11.0 "LSMinimumSystemVersion is set"

cargo_version=$(sed -n '/^\[package\]/,/^\[/ s/^version = "\(.*\)"/\1/p' \
    "$repo_root/Cargo.toml" | head -n 1)
assert_plist_value CFBundleShortVersionString "$cargo_version" \
    "CFBundleShortVersionString matches the Cargo.toml version"

# Cargo-bundle inheritance with nothing behind it; see porting notes 5.2.
assert_plist_absent LSRequiresCarbon "LSRequiresCarbon is omitted"
assert_plist_absent CSResourcesFileMapped "CSResourcesFileMapped is omitted"

echo "# bundled freedesktop data"
# Without these the app has no UI icons and no per-file-type icons on a machine
# that never installed cosmic-icons or shared-mime-info; src/launch_macos.rs
# puts Contents/Resources/share in front of XDG_DATA_DIRS to find them.
assert_dir "$app/Contents/Resources/share/icons/Cosmic" \
    "the Cosmic icon theme is in Contents/Resources/share/icons"

assert_file() {
    if [ -f "$1" ]; then
        ok "$2"
    else
        no "$2" "missing: $1"
    fi
}

# The icon lookup only treats a directory as a theme when it has an index.theme.
assert_file "$app/Contents/Resources/share/icons/Cosmic/index.theme" \
    "the bundled icon theme carries its index.theme"

assert_dir "$app/Contents/Resources/share/mime" \
    "the shared MIME database is in Contents/Resources/share/mime"

# The exact files xdg-mime-rs reads out of <datadir>/mime. globs2 is how a name
# becomes a type and generic-icons is how a type becomes an icon name, so a copy
# missing either leaves every file with the fallback icon.
for mime_file in aliases globs2 icons generic-icons subclasses magic; do
    assert_file "$app/Contents/Resources/share/mime/$mime_file" \
        "the bundled MIME database carries $mime_file"
done

# Homebrew ships mime/packages as a symlink into the Cellar. It is
# update-mime-database's input, never read at runtime, and a dangling symlink
# inside the bundle makes codesign --verify fail with "No such file or
# directory" - so the bundler must not carry it.
if [ -e "$app/Contents/Resources/share/mime/packages" ]; then
    no "the MIME source packages are not shipped in the bundle" \
        "Resources/share/mime/packages was copied in"
else
    ok "the MIME source packages are not shipped in the bundle"
fi

dangling=$(find "$app" -type l ! -exec test -e {} \; -print)
if [ -z "$dangling" ]; then
    ok "no dangling symlinks are left inside the bundle"
else
    no "no dangling symlinks are left inside the bundle" \
        "$(printf '%s' "$dangling" | head -n 3)"
fi

echo "# icon"
icns="$app/Contents/Resources/cosmic-files.icns"
if [ -f "$icns" ]; then
    ok "Contents/Resources/cosmic-files.icns exists"
else
    no "Contents/Resources/cosmic-files.icns exists" "missing: $icns"
fi

# A Dock and Finder icon that stays crisp needs the retina variants, so the
# largest representation has to be 1024px, not the 256px the SVG nominally is.
largest=$(sips -g pixelWidth "$icns" 2>/dev/null | sed -n 's/.*pixelWidth: //p')
if [ "${largest:-0}" -ge 1024 ]; then
    ok "the icns carries a 1024px representation"
else
    no "the icns carries a 1024px representation" "largest representation is ${largest:-none}"
fi

if [ -d "$app/Contents/Resources/cosmic-files.iconset" ]; then
    no "the intermediate iconset is not shipped in the bundle" \
        "Resources/cosmic-files.iconset was left behind"
else
    ok "the intermediate iconset is not shipped in the bundle"
fi

echo "# signature"
if codesign --verify --deep --strict "$app" 2>/dev/null; then
    ok "the bundle passes codesign --verify --deep --strict"
else
    no "the bundle passes codesign --verify --deep --strict" \
        "$(codesign --verify --deep --strict "$app" 2>&1 | head -n 3)"
fi

signing_info=$(codesign --display --verbose=2 "$app" 2>&1)
if printf '%s' "$signing_info" | grep -q '^Signature=adhoc$'; then
    ok "the bundle is ad-hoc signed"
else
    no "the bundle is ad-hoc signed" \
        "$(printf '%s' "$signing_info" | grep -i '^Signature' || echo 'no Signature line')"
fi

# The bundle's signature must seal the Info.plist; that only happens when the
# bundle is signed after its contents, which is also what binds the identifier.
if printf '%s' "$signing_info" | grep -q '^Identifier=com.system76.CosmicFiles$'; then
    ok "the signature carries the bundle identifier"
else
    no "the signature carries the bundle identifier" \
        "$(printf '%s' "$signing_info" | grep -i '^Identifier' || echo 'no Identifier line')"
fi

# The hardened runtime crashes ad-hoc builds; see porting notes 5.1.
if printf '%s' "$signing_info" | grep -q 'flags=.*runtime'; then
    no "the hardened runtime is not enabled" \
        "$(printf '%s' "$signing_info" | grep -i 'flags=')"
else
    ok "the hardened runtime is not enabled"
fi

# Quarantine and resource-fork xattrs are what break a signature; the kernel's
# own com.apple.provenance cannot be removed and does not matter.
stray_xattrs=$(xattr -r "$app" | grep -E 'com\.apple\.(quarantine|ResourceFork|FinderInfo)' || true)
if [ -z "$stray_xattrs" ]; then
    ok "no quarantine or resource-fork attributes are left on the bundle"
else
    no "no quarantine or resource-fork attributes are left on the bundle" \
        "$(printf '%s' "$stray_xattrs" | head -n 3)"
fi

echo "# architecture"
# Universal builds re-prompt for TCC permissions on every launch; see porting
# notes 4.4. The bundled binary must stay thin.
arch_info=$(lipo -info "$app/Contents/MacOS/cosmic-files" 2>&1)
if printf '%s' "$arch_info" | grep -q '^Non-fat file:.*architecture: arm64$'; then
    ok "the bundled binary is thin arm64"
else
    no "the bundled binary is thin arm64" "$arch_info"
fi

# ---------------------------------------------------------------------------
# Privacy and Finder integration (issues #9 and #10).
#
# These assertions were added separately from the ones above; keep them in this
# block so the two sets stay easy to tell apart.
# ---------------------------------------------------------------------------

assert_plist_nonempty() {
    local key=$1 name=$2 actual
    actual=$(plutil -extract "$key" raw -o - -- "$plist" 2>&1)
    if [ -n "$actual" ] && ! printf '%s' "$actual" | grep -q 'error:'; then
        ok "$name"
    else
        no "$name" "$key is missing or empty ('$actual')"
    fi
}

echo "# privacy usage descriptions"
# Without these a TCC-gated read is silently denied: no prompt, and no entry in
# System Settings for the user to grant afterwards. See porting notes 4.2/5.2.
assert_plist_nonempty NSDesktopFolderUsageDescription \
    "NSDesktopFolderUsageDescription gives a reason"
assert_plist_nonempty NSDocumentsFolderUsageDescription \
    "NSDocumentsFolderUsageDescription gives a reason"
assert_plist_nonempty NSDownloadsFolderUsageDescription \
    "NSDownloadsFolderUsageDescription gives a reason"
assert_plist_nonempty NSRemovableVolumesUsageDescription \
    "NSRemovableVolumesUsageDescription gives a reason"
assert_plist_nonempty NSNetworkVolumesUsageDescription \
    "NSNetworkVolumesUsageDescription gives a reason"

echo
printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
