# Bundled themes

These four theme files come from
[`cosmic-utils/cosmic-ext-themes`](https://github.com/cosmic-utils/cosmic-ext-themes),
copied unmodified on 2026-09-21. That project is GPL-3.0, the same licence as this one;
its `LICENSE` sits beside them here.

Each file is a serialized `cosmic_theme::ThemeBuilder` in RON — the format cosmic-settings
imports and exports, so these interoperate with the rest of the COSMIC ecosystem and with
anything a user installs themselves.

`scripts/macos-bundle.sh` copies this directory to
`Contents/Resources/share/themes/cosmic` inside the app bundle. The bundle's `share` is
first on `XDG_DATA_DIRS`, and `src/theme_catalog.rs` scans `themes/cosmic` under every XDG
data directory, so these are found by the same pass that finds the user's own themes — and
a user's theme of the same name takes precedence over the copy shipped here.

To add a theme, drop a `.ron` file in. The display name is the file stem, and whether it
is dark is read from its palette, so no naming convention is required. `cargo test
theme_catalog` checks that everything here still parses.

Note these files use `is_frosted`, a field that has since been renamed to `frosted` in
`cosmic-theme`. They still load because the replacement fields carry `#[serde(default)]`;
`vendored_themes_parse` is the regression test for that.
