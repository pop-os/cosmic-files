# macOS (Apple Silicon) porting notes

Findings from reading four shipped Rust macOS apps, selected for overlap with this
project's stack: **alacritty** (winit — our exact windowing layer), **zed** (Metal +
objc2 + gestures), **wezterm** (hand-rolled AppKit interop), **tauri** (bundling,
signing, TCC).

Each claim is tagged **[V]** verified against primary source (read the code, ran the
command) or **[I]** inferred. Claims marked *"checked here"* were re-verified against
this repo or this machine rather than taken from a report.

---

## 1. Verified against this repo

| Finding | Status |
|---|---|
| Our binary links **only** system libraries (14 entries, none outside `/usr/lib`, `/System/Library`) | **[V] good** — immune to the hardened-runtime library-validation crashes below |
| Our tree carries legacy `objc 0.2.7` + `block 0.1.6` | **[V] liability** — see §6.1 |
| `ScrollDelta::Pixels` arrives in **physical** pixels | **[V] bug** — see §2.1 |

---

## 2. Scrolling and gestures

### 2.1 `ScrollDelta::Pixels` is physical, not logical — scroll speed varies per monitor

winit applies the scale factor before emitting the event, and iced passes it through
untouched:

```rust
// winit-appkit/src/view.rs:680
let delta = LogicalPosition::new(x, y).to_physical(self.scale_factor());
MouseScrollDelta::PixelDelta(delta)

// iced/winit/src/conversion.rs:331 — no division
MouseScrollDelta::PixelDelta(position) => mouse::ScrollDelta::Pixels {
    x: position.x as f32, y: position.y as f32,
}
```

**[V] checked here.** Any pixel threshold in app code is therefore scale-dependent: the
same finger travel produces 2× the number on a Retina panel as on a 1× external display.
The symptom is *"scrolling/zoom feels twice as fast on the laptop screen"*.

**Applies to us now:** `PIXELS_PER_ZOOM_STEP` in `src/tab.rs` is specified in physical
pixels, so Ctrl+scroll zoom is twice as sensitive on the built-in display. Normalise by
`scale_factor`, or define the constant in physical pixels deliberately and document it.
Alacritty shipped this bug class (CHANGELOG 0.5.0, *"Touchpad scrolling scrolled less
than it should … on scaled outputs"*).

### 2.2 Momentum is indistinguishable from real input — nobody has solved this

winit folds `NSEvent.momentumPhase` into `TouchPhase` and discards it:

```rust
// winit-appkit/src/view.rs:691
let phase = match event.momentumPhase() {
    NSEventPhase::MayBegin | NSEventPhase::Began => TouchPhase::Started,
    NSEventPhase::Ended | NSEventPhase::Cancelled => TouchPhase::Ended,
    _ => match event.phase() { ..., _ => TouchPhase::Moved },
};
```

`momentumPhase == Changed` falls through to `Moved`, byte-identical to a finger on the
glass. iced then drops `phase` entirely (`WindowEvent::MouseWheel { delta, .. }`), so the
information is gone twice over.

**[V] checked here.** Corroborated independently by all three GUI apps studied:

- **zed** never reads `momentumPhase`; open bug #61637 "Jumpy scrolling".
- **wezterm** never reads it either — no occurrence of `phase`/`momentumPhase` anywhere
  in `window/src/os/macos/`.
- **alacritty** resets its scroll accumulator on `TouchPhase::Started`, which momentum
  **re-emits mid-flick** — so one physical flick is
  `Started → Moved… → Ended → Started(momentum) → Moved… → Ended`.

**Consequence [V]:** anything that must not fire on inertia — zoom steps, pagination
triggers, rubber-band stop, scroll-to-select — cannot be gated on winit's phase. An idle
timeout does not work either: momentum events arrive ~16ms apart.

**What the ecosystem actually ships** is wezterm's heuristic, and it is the best
available without interop: a staleness window (250ms) that rounds the first sub-unit
delta away from zero, a fractional remainder carried between events, and a reset on
direction change. Recovering true momentum state requires patching winit or reading
`NSEvent.momentumPhase` through AppKit directly.

**Do not reset accumulators on `TouchPhase::Started` [V]** — momentum re-emits it.

### 2.3 Trackpad deltas are diagonal; lock the axis yourself

```rust
// alacritty input/mod.rs:743 — ~25 degrees
if lpos.x.abs() / lpos.x.hypot(lpos.y) > 0.9 { lpos.y = 0. } else { lpos.x = 0. }
```

**[V]** macOS does not axis-lock for you. A two-axis view drifts sideways on every
flick without this. The 0.9 cosine is a tested value.

### 2.4 Apply sensitivity to the float delta, before quantising

**[V]** alacritty `f83d55f0` *"Fix ignoring of slow touchpad scrolling"*, `fe8577cc`.
Multiplying after integer conversion discards every slow gesture — the *"trackpad feels
dead at low speed"* complaint.

### 2.5 Pinch is `PinchGesture`, not Ctrl+scroll

**[V]** winit emits `PinchGesture`/`RotationGesture`/`DoubleTapGesture` from
`magnifyWithEvent:`/`rotateWithEvent:`/`smartMagnifyWithEvent:`. Ctrl+scroll zoom is the
X11/Wayland convention; on macOS the native gesture is a two-finger pinch, and Ctrl+scroll
additionally collides with the system Accessibility zoom. `magnification()` is a unitless
per-event delta — accumulate and compare against ~0.05–0.1 per step.

**Done here:** iced never converts winit's `PinchGesture`, so `src/gesture_macos.rs` installs an
`NSEvent` local monitor for `NSEventMaskMagnify` and feeds the pure reducer in `src/gesture.rs`,
which banks spread at 0.1 per zoom step and resets on `Began`, `Ended` and direction change.

Note **[V]** `PanGesture` does **not** exist on macOS (iOS and Wayland only), and winit
implements no `swipeWithEvent:` at all. Three- and four-finger swipes are reserved by the
window server and never reach the app.

---

## 3. Scale factor and Retina

### 3.1 Never rescale state by the scale-factor *ratio*

Alacritty multiplies font size by `new/old` on `ScaleFactorChanged`. This is **lossy**:
drag 1×→2×→1× and you do not return to the original size (their issue #8309, *"Scaling
text with Ctrl-+/- is irreversible on a multi-DPI setup"*). **[V]**

Store zoom as an absolute value independent of scale and recompute. Dragging between the
built-in Retina panel and a 1× external is the single most common macOS multi-monitor
action — test it explicitly.

### 3.2 Assume the first scale factor is wrong

- **[V]** zed: `backingScaleFactor` *"sometimes would return 0"* (#6412), and `screen`
  can be nil — both fall back to 2.0.
- **[V]** wezterm ships a *"stupid hack"*: toggling the window style mask at show time
  because `CAMetalLayer` *"seems to get stuck with a scale factor of 2"*.
- **[V]** zed #13672: content rendered into the top-left quarter of the window, correct
  after any resize. Hit-testing matched the *correct* layout while pixels were wrong.

Smoke check: the first presented frame's drawable size should equal
`logical_size * scale_factor`. *"Fixes itself on resize"* is the signature of this bug.

### 3.3 Rasterise per-DPI; key caches on scale

**[V]** zed #58438 — an SVG baked once at a hardcoded 2× goes soft at any other DPI.
Icon and thumbnail caches must be keyed on `(size, scale_factor)` and invalidated on
`ScaleFactorChanged`. Directly relevant: our thumbnail cache is scale-blind today.

### 3.4 Ignore `Resized(0, 0)`

**[V]** alacritty guards it explicitly. `wgpu::configure_surface` panics on a zero
dimension; minimise and Space transitions trigger it.

---

## 4. TCC and permissions — the part that matters most for a file manager

### 4.1 Grants key on the signing identity, not the path

**[V]** tauri #11085 (maintainer): *"Permissions on macOS are bound to the signing
identity/certificate … ad-hoc/self-signing is not enough for the permissions to
persist."*

So bundling alone does not fix our current problem. A `.app` gets its own
`CFBundleIdentifier` and becomes its own responsible process instead of the terminal —
but an **ad-hoc signature's cdhash changes on every rebuild**, so each `cargo build`
loses the grant. Durable grants need a Developer ID certificate.

**[V]** zed: unbundled, `bundleIdentifier` is nil and URL-scheme registration, bundle
lookups and `restart()` all refuse. `cargo run` and the `.app` are two different apps to
TCC.

### 4.2 There is no entitlement for Desktop/Documents/Downloads/Trash

**[V]** `search/code` for `com.apple.security` across all of tauri → **0 hits**; tauri
ships no default entitlements. `com.apple.security.files.user-selected.*` are **App
Sandbox** keys and are meaningless in a non-sandboxed app.

`~/.Trash` and screen recording sit behind Full Disk Access / Screen Recording, which
have **no prompting API**. Detect the failure and deep-link:
`x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`.

What *is* needed is the Info.plist usage strings — see §5.2.

### 4.3 Denied folder access is an `EPERM` that lies, not a clean error

**[V]** zed #5138: with Desktop access denied, `read_dir` **succeeds** while `open` and
`metadata` fail with *"Operation not permitted (os error 1)"*. A listing can therefore
show files that cannot be touched — render a per-entry permission state.

**[V]** zed #49003 *"Zed is attempting to read my entire disk"*: declining the prompt
sent CPU *"berserk"* and made the machine unusable. A retry-on-error thumbnail queue is
exactly how that looks.

**Rules for us:** treat `EPERM` as a displayable state, never a retry loop; do not
recurse into `~/Library`, `~/.Trash`, `~/.cache` for sizes or thumbnails; never fan a
thumbnailer across a TCC-protected tree before the user navigates there.

### 4.4 Do not ship a universal binary

**[V]** tauri #8041 (open): universal builds re-prompt for permissions *"every time"* and
*"forget"* grants, while single-architecture builds prompt once. Reproduced in Electron
too, so it is a macOS/TCC-vs-fat-binary problem, not a toolkit bug. Ship thin
`aarch64-apple-darwin`.

---

## 5. Bundling

### 5.1 Minimum viable bundle

```
CosmicFiles.app/Contents/Info.plist
CosmicFiles.app/Contents/MacOS/cosmic-files
CosmicFiles.app/Contents/Resources/cosmic-files.icns
```

**[V]** No `PkgInfo` (tauri emits none). No `Frameworks` while the dylib graph stays
clean. Local dev signing — ad-hoc, and **without** `--options runtime`, which crashes
ad-hoc builds:

```sh
xattr -crs CosmicFiles.app
codesign --force -s - CosmicFiles.app/Contents/MacOS/cosmic-files
codesign --force -s - CosmicFiles.app          # inside-out: binary first, bundle last
```

**[V]** alacritty must `codesign --remove-signature` before re-signing, because `lipo`
invalidates the linker's signature and an *invalid* signature is worse than none —
macOS kills the process outright.

### 5.2 Info.plist keys that are load-bearing

| Key | Why |
|---|---|
| `CFBundleIdentifier` | the TCC grant key — pick once, 3+ components. Changing it resets every grant **[V]** |
| `CFBundleExecutable` | must match `MacOS/<name>` |
| `NSHighResolutionCapable` | **without it the bundle runs 1× magnified** — the "everything is blurry" report. Bundle-only: `cargo run` is high-res by default, so this bug appears *only after* you start bundling **[V]** |
| `NSRequiresAquaSystemAppearance` = `NO` | opt in to dark mode |
| `NS{Desktop,Documents,Downloads,RemovableVolumes}FolderUsageDescription` | without these a TCC-gated read is **silently denied with no prompt and no System Settings entry** **[V]** |
| `CFBundleDocumentTypes` | how a file manager claims "Open With". Hand-write `LSItemContentTypes = [public.folder, public.directory]` — tauri cannot express folder handling at all **[V]** |
| `LSHandlerRank` | tauri only added it recently (#13159); set `Alternate` for types you merely open |

**Omit** `LSRequiresCarbon` and `CSResourcesFileMapped` — **[V]** present in tauri's
generator with a 2016 cargo-bundle copyright header and *zero* commits justifying them.

### 5.3 Signing and notarization gotchas

- **[V]** `--deep` is for `--verify`, never for signing. Sign inside-out.
- **[V]** Passing `Info.plist` as `--entitlements` produces *"The application can't be
  opened"* **after** signing and notarization both succeed (tauri #9738). Worst failure
  mode in the set: every check passes, app won't launch.
- **[V]** Archive with `ditto -c -k --keepParent --sequesterRsrc`, never `zip` —
  *"removes almost 99% of false alarm in notarization"*.
- **[V]** On failure always run `xcrun notarytool log <id>`; rejections are otherwise
  opaque.
- **[V]** Hardened runtime + any non-system dylib = `DYLD` library-not-loaded or
  *"mapped file has no Team ID"*. **Our dylib graph is clean — keep it that way**, and
  consider a CI guard on `otool -L`.

### 5.4 Unbundled vs bundled behaviour differences [V]

All three bite a file manager harder than a terminal:

- **cwd** — a Finder-launched bundle starts at `/`. Alacritty forces `$HOME` at startup.
- **locale** — a bundle inherits no `LANG`/`LC_*`. i18n silently falls back to English
  **in the bundled build only**, while `cargo run` works fine. Alacritty derives one from
  `NSLocale` using `languageCode`+`countryCode` (explicitly *not* `localeIdentifier`,
  which returns extra metadata that is not a valid locale).
- **PATH** — a bundle gets the bare launchd `PATH`; `/opt/homebrew/bin` is absent. Any
  external tool must be resolved by absolute path.

### 5.5 Bundling in this repo

`scripts/macos-bundle.sh` builds `target/macos/COSMIC Files.app` — the space is
deliberate, it is the name the Dock and the menu bar show. The script is the single
command the bundle is produced by; it takes the release binary from
`$CARGO_TARGET_DIR/release/cosmic-files` and builds it with the macOS feature set if it
is not there.

The `Info.plist` comes from the template `res/macos/Info.plist.in`, which carries
everything from 5.2 that is settled today and marks where the privacy usage strings and
`CFBundleDocumentTypes` go. Only `@SHORT_VERSION@` is substituted, from the Cargo.toml
version. `LSMinimumSystemVersion` is 11.0, matching the `LC_BUILD_VERSION` `minos` of the
aarch64 binary.

`CFBundleIdentifier` is `com.system76.CosmicFiles` — the same string the config directory
already uses, and **the key every TCC privacy grant attaches to**. Changing it resets
every permission the user has granted, so it is fixed in the template and asserted by the
tests. Under an ad-hoc signature the grants still do not survive a rebuild (4.1); the
identifier is what makes them survivable at all once there is a Developer ID.

The icon is generated, not committed. `sips` rasterises the hicolor SVGs in
`res/icons/hicolor` straight into an iconset — it reads SVG, so the 1024px retina
representation comes out of the vector art rather than an upscale — and `iconutil`
converts that to `Contents/Resources/cosmic-files.icns`. Each size uses the SVG drawn for
it where one exists, so the small representations keep their hinting.

Signing is ad-hoc and inside-out, with no `--options runtime`: `xattr -crs`, then
`codesign --force -s -` on `Contents/MacOS/cosmic-files`, then the same on the bundle,
verified with `codesign --verify --deep --strict`. The script refuses to sign anything
that is not thin arm64 (4.4).

`scripts/test-macos-bundle.sh` runs the bundler and asserts all of the above: the
`Contents` layout, every plist key by `plutil -extract` (including that `LSRequiresCarbon`
and `CSResourcesFileMapped` stay absent), a 1024px icns, an ad-hoc signature without the
hardened runtime that carries the bundle identifier, and `lipo -info` reporting a non-fat
arm64 binary.

**Launched from Finder the app has no `XDG_DATA_DIRS`** (5.4), so the shared MIME database
under `/opt/homebrew/share` is not found and per-file-type icons fall back to generic
ones. `XDG_DATA_HOME` is unset too, but its spec default is `~/.local/share`, so the
Cosmic icon theme installed there is still found.

---

## 6. FFI, threading, and lifecycle

### 6.1 The legacy `objc`/`block` crates are on a compile deadline

**[V] checked here.** `cargo tree -i objc@0.2.7`:

```
objc v0.2.7
└── clipboard_macos → window_clipboard (pop-os fork) → iced → libcosmic → cosmic-files
```

`block 0.1.6` arrives similarly via `cocoa 0.25` ← pop-os's softbuffer fork ←
`iced_tiny_skia`. Zed's PR #59572: *"Rust 1.96 emits a warning when compiling the `block`
crate, that it will no longer compile past some future version of Rust."* Both blockers
are **pop-os forks**, not libcosmic proper. `objc2`/`block2` are already in our tree, so
the migration target exists.

### 6.2 Completion-handler → async bridging

**[V]** zed's pattern, worth copying verbatim: `block2::RcBlock::new(closure)` with the
once-only `oneshot::Sender` in a `Cell<Option<_>>`, and every invocation doing
`let Some(tx) = tx.take() else { return };`. **Never assume a block fires exactly once.**

### 6.3 AppKit calls back into you synchronously — re-entrancy is the top crash class

- **[V]** zed: `NSNotificationCenter` *"delivers this notification synchronously and it
  may fire while the App is already borrowed"* → defer to the next run-loop iteration.
- **[V]** wezterm: `setFrame:` re-enters `windowDidResize:`; they cannot take a second
  mutable borrow and say so in a comment.
- **[V]** zed #51992: AppKit messages to a **closed** `NSWindow` raise an ObjC exception
  and **abort the process**. Any deferred task holding a window pointer must re-check
  liveness inside the main-thread turn.

iced/libcosmic hold app state borrowed during `update`/`view`, so any ObjC observer we
add must post a message rather than touch state inline.

### 6.4 `MainThreadMarker` is the cheap proof

**[V]** `MainThreadMarker::new().expect("not on main thread")` — `Copy` and `!Send`, so
it cannot leak to a worker. Alacritty asserts with it before every AppKit call.

### 6.5 Objective-C `BOOL` differs by architecture

**[V]** wezterm `2582d8b`: `BOOL` is `i8` on x86_64 but `bool` on aarch64. Raw
comparisons against `YES` are an **Apple-Silicon-specific** footgun; go through helpers.

---

## 7. Two free wins

### 7.1 `NSAutoFillHeuristicController` — progressive slowdown on macOS 26

Found **independently by zed and alacritty**, which is the strongest corroboration in this
set. AppKit's autofill heuristics attach to text inputs and degrade the app over time.
Alacritty's fix is six lines, called before the event loop **[V]**:

```rust
NSUserDefaults::standardUserDefaults().registerDefaults(
    &NSDictionary::from_slices(
        &[ns_string!("NSAutoFillHeuristicControllerEnabled")],
        &[ns_string!("NO")],
    ),
);
```

We have rename, search and path-bar text fields. This applies.

### 7.2 Pin the NSWindow colour space to sRGB

**[V]** alacritty `bcd6d0d9`, CHANGELOG *"Always use sRGB color space on macOS"*. MacBook
Pro panels are Display P3; libcosmic palettes are authored in sRGB, so greys and accents
are oversaturated out of the box. One `setColorSpace` call through the
`raw_window_handle` → `NSView` → `NSWindow` escape hatch, on the main thread.

---

## 8. Windowing behaviour worth copying

- **[V]** A new macOS window is **not focused**, and `Focused` may never arrive. Alacritty
  calls `window.focus_window()` unconditionally on macOS for new windows.
- **[V]** Create the window **hidden**, present one frame, then `set_visible(true)` —
  otherwise macOS shows a frame of undefined content.
- **[V]** Handle `Occluded(true)` by stopping presentation. Fires on Space switches and
  full coverage.
- **[V]** *"Bump winit"* is the correct first diagnostic for any macOS-only bug.
  Alacritty's macOS crash history — Sonoma resize, macOS 15 modifier crash, macOS 26
  slowdown, 15.2 IME crash — is almost entirely fixed by version bumps, not app code.
  Corollary: a winit **major** bump is a keyboard-regression event; smoke-test a CJK IME
  after every one.
- **[V]** macOS needs a **separate binding table** on `SUPER`, not a global Ctrl→Cmd
  substitution — a blanket swap breaks readline-style Ctrl+A/E in text fields and the
  bindings with no macOS analogue. Alacritty `cfg`s out its common table entirely.
- **[V]** Gate all key bindings on IME preedit state, or single-letter shortcuts fire
  while composing in Pinyin/Kotoeri.
- **[V]** macOS app-level operations (`hide_application`) live on `ActiveEventLoop`, not
  `Window` — plan the plumbing up front.
- **[V]** There is no primary selection on macOS; model it as `Option` so its absence is
  a compile-time fact.

---

## 9. Research technique notes

Collected across all four investigations, since they agreed strongly.

**`gh api search/commits` is unreliable — do not size a topic with it.** All four agents
hit this independently. It indexes commit *messages*, not diffs, and its coverage is
partial: it returned `total_count: 0` for `LSRequiresCarbon`, `NSHighResolutionCapable`,
`magnify`, `notarize` and `Retina`, all of which are demonstrably present in the
respective codebases.

**Clone sparse and blobless, then grep locally.** The decisive move:

```sh
git clone --filter=blob:none --no-checkout https://github.com/OWNER/REPO
git sparse-checkout init --cone && git sparse-checkout set <dirs> && git checkout
```

Every subsequent query is free. `search/*` is rate-limited to 30/min (core API is
5000/hr) and agents hit 403s mid-run.

**Highest-yield single query: grep the CHANGELOG, then pickaxe back to the commit.**

```sh
grep -in "macos" CHANGELOG.md | grep -iE "scroll|dpi|scale|crash|resize|leak|hang"
git log --oneline -S'<exact changelog line>' -- CHANGELOG.md
```

Symptom vocabulary lives in changelogs; commit messages do not carry it.

**Grep comments, not code.** In comment-dense codebases nearly every finding came from:

```sh
grep -nE '// .*(FIXME|HACK|for some reason|otherwise|must be|main thread|avoid|never)'
```

Those comments cite their own issue numbers, which then resolve cheaply.

**Mine reverts.** `search/commits q='Revert macOS'` and `git log --grep=<topic> -i` to
find landed→reverted→re-landed triples. A fix that shipped twice encodes a gotcha the
naive fix gets wrong; these were the highest-confidence findings in every report.

**Search the symptom string, not the concept.** `"library validation"` returns docs
noise; `"mapped file has no Team ID"` returns the actual crash. Apple's own error text is
the highest-signal query available.

**Negative results are evidence.** `search/code` for `com.apple.security` → 0 hits proved
tauri ships no default entitlements. Grepping a target for what is *absent* (wezterm has
no `magnifyWithEvent:`, no `momentumPhase`) was itself a finding.

**`in:title` is mandatory on `search/issues`**, plus `-f sort=comments -f order=desc`.
Without it GitHub returns the repo's most-commented issues regardless of relevance. High
comment count is a good prior for signal but not a guarantee — one 39-comment thread was
pure flamewar whose only fact was "they said no".

**Verify the premise, not just the question.** One code search
(`repo:rust-windowing/winit magnifyWithEvent`) collapsed an entire "we must hand-roll a
view" brief into "winit already does this".

**zsh:** quote every glob. `--include=*.rs` fails with `no matches found` *before the
command runs*, producing a confident and wrong "no matches".
