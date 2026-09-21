# Replacing the system "Save As" dialog

Can cosmic-files be made to *be* the file chooser that other applications show — the way a
Linux file manager backs the xdg-desktop-portal FileChooser interface? macOS first, since
that is the active port, then Linux as the baseline.

Each claim is tagged **[V]** verified against a primary source (Apple documentation or
headers, the freedesktop spec or portal source, this machine, this repo) or **[I]**
inferred. Vendor documentation is primary for claims about that vendor's own product.

Not covered: replacing **Finder** as the handler for folder double-clicks. That is a
LaunchServices question and has nothing to do with the panels.

---

## 1. Verified against this repo and this machine

| Finding | Status |
|---|---|
| `src/dialog.rs` is a **widget library**, not a program: no D-Bus, no IPC, no CLI flag | **[V]** — see §2.1 |
| xdg-desktop-portal-cosmic links cosmic-files as a **crate** and wraps that widget in D-Bus | **[V]** — see §2.2 |
| `examples/dialog.rs` is a complete in-process host of the same widget, no `required-features` | **[V]** — see §6.2 |
| The only Wayland-specific code in `dialog.rs` sits behind `#[cfg(feature = "wayland")]` | **[V]** `src/dialog.rs:363` |
| `dialog.rs` compiles on macOS in our build configuration today | **[V]** ran `cargo build --release --no-default-features --features bzip2,lzma-rust2,wgpu,quicklook`, exit 0 |
| `examples/dialog.rs` builds on macOS too — the chooser can already be driven from another Rust app here | **[V]** same command plus `--example dialog`, exit 0 |
| macOS runs Open/Save panels in an Apple-signed process on the sealed system volume | **[V]** — see §3.2 |
| macOS offers **no** API for substituting another app's panel | **[V]** for "none documented"; **[I]** for the absolute — see §3.4 |
| The only third-party code the panel hosts is a File Provider extension | **[V]** — see §5 |

---

## 2. What cosmic-files already does (Linux)

### 2.1 The dialog is a widget, not a process

`src/dialog.rs` exports `Dialog<M>`, `DialogSettings`, `DialogKind`, `DialogFilter`,
`DialogChoice` and `DialogResult`. `Dialog::new` opens its own top-level window through
iced's multi-window support:

```rust
// src/dialog.rs:270
let (window_id, window_command) = window::open(settings);
```

`DialogKind` covers `OpenFile`, `OpenFolder`, `OpenMultipleFiles`, `OpenMultipleFolders`
and `SaveFile { filename }` (`src/dialog.rs:53-59`). The result type is the whole
contract:

```rust
// src/dialog.rs:47-51
pub enum DialogResult {
    Cancel,
    Open(Vec<PathBuf>),
}
```

**[V]** There is no dialog mode in the binary. `src/main.rs` is eleven lines and calls
`cosmic_files::main()`; the argument loop in `src/lib.rs` accepts only `--no-daemon`,
`--trash`, `--recents`, `--network` and paths/URIs (`src/lib.rs:178-214`). Nothing in the
tree opens a socket, a pipe or a bus to serve a dialog. `pub mod dialog;` is unconditional
(`src/lib.rs:23`), so the module is part of the library on every platform.

### 2.2 The portal backend wraps that widget

xdg-desktop-portal-cosmic is the process with the D-Bus name; cosmic-files is a crate
inside it.

**[V]** `Cargo.toml:18` of
[pop-os/xdg-desktop-portal-cosmic](https://github.com/pop-os/xdg-desktop-portal-cosmic)
(read at commit `2f41161`):

```toml
cosmic-files = { git = "https://github.com/pop-os/cosmic-files", default-features = false, features = [
    "gvfs",
    "wayland",
] }
```

**[V]** `src/file_chooser.rs:16` aliases our type directly —
`pub(crate) type Dialog = cosmic_files::dialog::Dialog<Msg>;` — and `:181` declares the
interface:

```rust
#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooser {
    async fn open_file(&self, handle: ObjectPath, app_id: &str, parent_window: &str,
                       title: &str, options: OpenFileOptions) -> PortalResponse<FileChooserResult>
    async fn save_file(...)   // :201
    async fn save_files(...)  // :219
}
```

This matches the spec exactly. **[V]**
[`org.freedesktop.impl.portal.FileChooser`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.impl.portal.FileChooser.html):
`OpenFile(IN handle o, IN app_id s, IN parent_window s, IN title s, IN options a{sv}, OUT
response u, OUT results a{sv})`, and the same shape for `SaveFile` and `SaveFiles`.

**[V]** The response is `(u, a{sv})`: `PORTAL_RESPONSE_SUCCESS = 0`,
`CANCELLED = 1`, `OTHER = 2` (`src/main.rs:33-35`), with the vardict carrying `uris`,
`choices` and `current_filter` (`src/file_chooser.rs:130-136`). The bridge from our type
to that vardict is one function, `file_chooser_update_msg` (`src/file_chooser.rs:277`):
`DialogResult::Cancel` becomes `Cancelled`, and `DialogResult::Open(paths)` is mapped
through `url::Url::from_file_path` into `uris`. **So the interface cosmic-files actually exposes is
`Vec<PathBuf>`; everything D-Bus-shaped lives in the portal.**

### 2.3 How the backend gets chosen

**[V]** A backend is discovered through a `.portal` key file installed in
`{DATADIR}/xdg-desktop-portal/portals`, and selected by a `portals.conf`
([writing-a-new-backend.rst](https://github.com/flatpak/xdg-desktop-portal/blob/main/doc/writing-a-new-backend.rst)).
COSMIC ships `data/cosmic.portal`:

```
[portal]
DBusName=org.freedesktop.impl.portal.desktop.cosmic
Interfaces=org.freedesktop.impl.portal.Access;org.freedesktop.impl.portal.FileChooser;…
UseIn=COSMIC
```

`UseIn` is documented as deprecated in favour of the config file, kept for legacy systems
**[V]** (same doc). The config file is matched per desktop from `XDG_CURRENT_DESKTOP`
**[V]** ([portals.conf.rst.in](https://github.com/flatpak/xdg-desktop-portal/blob/main/doc/portals.conf.rst.in)).

### 2.4 How an application ends up in our dialog without knowing it

**[V]** GTK's own source says it, in `gtk/gtkfilechoosernative.c:176-181`
([GNOME/gtk](https://gitlab.gnome.org/GNOME/gtk/-/blob/main/gtk/gtkfilechoosernative.c)):

> When the `org.freedesktop.portal.FileChooser` portal is available on the session bus, it
> is used to bring up an out-of-process file chooser. Depending on the kind of session the
> application is running in, this may or may not be a GTK file chooser.

The call goes to `org.freedesktop.portal.Desktop` at `/org/freedesktop/portal/desktop`,
method `OpenFile` or `SaveFile` (`gtk/gtkprivate.h:114-118`,
`gtk/gtkfilechoosernativeportal.c:509-515`) **[V]**. The front-end daemon then routes to
whichever backend `portals.conf` names. The application never learns which file manager
drew the window.

### 2.5 The sandbox hand-off

**[V]** A returned URI is not necessarily a path a sandboxed caller can open: the document
portal, a separate `xdg-document-portal` executable, exposes chosen files through a FUSE
filesystem at `/run/user/$UID/doc`, scoped per application
([documents-and-fuse.rst](https://github.com/flatpak/xdg-desktop-portal/blob/main/doc/documents-and-fuse.rst)).
Note the shape, because macOS has the same problem and a different answer (§6.4).

---

## 3. macOS: how Open and Save panels actually work

### 3.1 They have not been in your process since 10.15

**[V]** Apple, [`NSSavePanel`](https://developer.apple.com/documentation/appkit/nssavepanel):

> In macOS 10.15, the system always displays the Save dialog in a separate process,
> regardless of whether the app is sandboxed. When the user saves the document, macOS adds
> the saved file to the app's sandbox (if necessary) so that the app can write to the file.
> Prior to macOS 10.15, the system used a separate process only for sandboxed apps.

**[V]** [`NSOpenPanel`](https://developer.apple.com/documentation/appkit/nsopenpanel) says
the same for Open panels.

**[V]** [Accessing files from the macOS App
Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox):

> The operating system displays open and save panels in a separate process, and extends
> your app's sandbox to include the selected URLs.

That second sentence is the load-bearing one for everything below: **the panel is not just
UI, it is the only thing that can widen a sandbox.**

### 3.2 What that separate process is — checked on this machine

macOS 26.6.2 (25G83), SIP enabled.

**[V]** `/System/Library/Frameworks/AppKit.framework/Versions/C/XPCServices/com.apple.appkit.xpc.openAndSavePanelService.xpc`
exists, `CFBundleName = "Open and Save Panel Service"`, `CFBundlePackageType = XPC!`,
`NSPrincipalClass = NSViewServiceApplication`, `XPCService.ServiceType = Application`.

**[V]** `otool -L` on its binary: it links `ViewBridge.framework` (the remote-view
transport, so the panel's UI is rendered in this process and composited into the host app's
window) and `FinderKit.framework` (the browser is Finder's).

**[V]** `nm -gU` exports `NSOpenAndSavePanelService`, `NSOpenPanelService`,
`NSSavePanelService`, `NSSavePanelServiceSidebarTrackingProxy`, `OpenAndSaveMarshal`; its
strings include the property type `@"NSRemoteSavePanel<NSOpenSaveServicePanelProtocol>"`.
`NSRemoteSavePanel` is the object the host app holds; the real panel lives here.

**[V]** `codesign -dv`: `TeamIdentifier=not set`, `Platform identifier=26` — an Apple
platform binary. Its entitlements include `com.apple.fileprovider.extension-host`,
`com.apple.private.MobileContainerManager.allowed`, `com.apple.authkit.client.private` and
a dozen more `com.apple.private.*` keys.

**[V]** `mount`: `/dev/disk3s1s1 on / (apfs, sealed, local, read-only, journaled)`.

**[V]** `pgrep -fl openAndSavePanelService` returned three live instances, all reparented to
`launchd`. One per host application, as expected.

### 3.3 What a host app may customise

All of this is in-process API on the app's own panel object, marshalled across to the
service. **[V]** `MacOSX.sdk/…/AppKit.framework/Headers/NSSavePanel.h`:

| Knob | Line |
|---|---|
| `accessoryView` — an arbitrary `NSView` the panel hosts | `:84` |
| `identifier` — namespaces the remembered directory in user defaults | `:50` |
| `directoryURL`, `nameFieldStringValue`, `showsHiddenFiles`, `extensionHidden` | `:56`, `:149`, `:159`, `:118` |
| `allowedContentTypes` (`[UTType]`), changeable while running | `:62` |
| `NSOpenPanel`: `canChooseFiles`, `canChooseDirectories`, `allowsMultipleSelection`, `resolvesAliases` | `NSOpenPanel.h:26-43` |

The delegate protocol `NSOpenSavePanelDelegate` (`:213-257`) is seven live methods:
`panel:shouldEnableURL:`, `panel:validateURL:error:`, `panel:didChangeToDirectoryURL:`,
`panel:userEnteredFilename:confirmed:`, `panel:willExpand:`, `panelSelectionDidChange:`,
plus `panel:displayNameForType:` and `panel:didSelectType:` (macOS 15).

The deprecations are informative. `panel:compareFilename:with:caseSensitive:` is marked
*"Filenames in the save panel should not have a custom sort. This method is never called on
10.14"* **[V]** (`:267`). Sorting moved out of reach when the panel moved out of process.

**[I]** Every one of these is scoped to a panel the calling app created. None of them
names, selects or substitutes a panel implementation, and none of them is reachable for a
panel some *other* application created.

### 3.4 There is no substitution mechanism

**[V]** Nothing in `NSSavePanel.h`, `NSOpenPanel.h` or the AppKit reference for either
class exposes a factory, a class registration, a delegate at application scope, or a
defaults key that selects a panel implementation. `+[NSSavePanel savePanel]` is the only
constructor and it vends AppKit's own object.

**[I]** Therefore there is no supported, system-wide replacement. A subclass or a swizzle
would at most affect the caller's own process, and the panel UI itself is in the service
process where the caller's code does not run at all.

---

## 4. Why the injection route is closed

### 4.1 SIP forbids attaching to or injecting into Apple processes

**[V]** Apple, [System Integrity Protection Guide — Runtime
Protections](https://developer.apple.com/library/archive/documentation/Security/Conceptual/System_Integrity_Protection_Guide/RuntimeProtections/RuntimeProtections.html):
processes whose main executable is protected on disk or carries a system entitlement are
flagged protected; `task_for_pid` and `processor_set_tasks` fail with `EPERM`; lldb cannot
attach even as root; *"any dynamic linker (`dyld`) environment variables, such as
`DYLD_LIBRARY_PATH`, are purged when launching protected processes"*; DTrace cannot inspect
system processes.

The panel service is exactly such a process (§3.2): Apple platform binary, on a sealed
read-only volume, with private entitlements.

### 4.2 Library validation forbids loading your code into someone else's app

**[V]** Apple, [Disable Library Validation
Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.disable-library-validation):

> The hardened runtime enables library validation by default. This security-hardening
> feature prevents a program from loading frameworks, plug-ins, or libraries unless they're
> either signed by Apple or signed with the same Team ID as the main executable.

**[V]** [Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime):
notarisation requires the hardened runtime, and shared libraries and in-process plug-ins
inherit the host executable's entitlements rather than bringing their own.

**[I]** The combination is decisive. To get code inside a notarised third-party app you
would need that app's Team ID or that app to ship
`com.apple.security.cs.disable-library-validation` (which Apple notes draws extra Gatekeeper
scrutiny **[V]**, same page). To get code inside the panel service you would need Apple's.
The SIMBL/`mach_inject` generation of hacks depended on neither restriction existing.

### 4.3 What Default Folder X actually does

St. Clair Software's own documentation is primary for its own product, and it describes
augmentation from outside, not replacement.

**[V]** [DFX FAQ](https://www.stclairsoft.com/DefaultFolderX/faq.html), on app
incompatibilities: *"This happens when the framework does not properly support the macOS
Accessibility API, which Default Folder X uses."*

**[V]** [User's Guide](https://www.stclairsoft.com/DefaultFolderX/DefaultFolderXGuide.pdf)
v6.2.8, p16: *"When you bring up an Open or Save dialog in any application, Default Folder
X attaches its toolbar to the dialog … it will either surround the entire Open or Save
dialog or hang off one side."* Its own diagnostic (p48) measures *"how long the system takes
to notify Default Folder X that a window has appeared on screen, and how long Default Folder
X takes to recognize it as a file dialog and display its controls."* That is an
accessibility-notification loop, not a panel.

**[V]** The price is four TCC grants. The FAQ's reset instructions name them:
`Accessibility`, `AppleEvents`, `SystemPolicyAllFiles` (Full Disk Access) and
`ScreenCapture`. Screen recording is needed because *"in some circumstances, it needs to
actually select something from a menu in a file dialog in order to perform an action … it
takes a screenshot of the file dialog and displays that in front of the dialog for a
fraction of a second"*, and to read the panel's appearance because *"there's no macOS API
available to do so"* (their parenthesis).

**[I]** So the state of the art from the vendor that has shipped this since 1992 is:
drive the real panel through the Accessibility API, hang your own window off it, and paper
over the resulting flicker with a screenshot. Nobody replaces the panel. Reimplementing that
would cost cosmic-files two of the most invasive permission grants on the platform
(Accessibility and Screen Recording) and buy a toolbar, not a file manager.

---

## 5. The one sanctioned way third-party code runs inside the panel

**[V]** The panel service holds `com.apple.fileprovider.extension-host` (§3.2). That
entitlement names the supported extension point: a **File Provider extension**.

**[V]** [File Provider](https://developer.apple.com/documentation/fileprovider): *"An
extension other apps use to access files and folders managed by your app and synced with a
remote storage."* Domains partition that content and *"a single file provider can act as if
multiple file providers were installed"*
([`NSFileProviderDomain`](https://developer.apple.com/documentation/fileprovider/nsfileproviderdomain)).

**[V]** [File Provider UI](https://developer.apple.com/documentation/fileproviderui): *"Add
actions to the document browser's context menu."*

**[I]** What this buys and does not buy: a File Provider extension puts a **location** into
the panel (and into Finder) whose contents you enumerate, and can add context-menu actions
inside it. It does not change the panel's chrome, its navigation, its sorting, its keyboard
handling, or what it looks like anywhere outside your own domain. It is the right mechanism
for exposing a remote or virtual filesystem. It is the wrong mechanism for "make the Save
dialog behave like cosmic-files", and cosmic-files manages local files, which is the case
Apple explicitly says needs no extension **[V]** (File Provider, "Share files locally").

---

## 6. Voluntary per-app adoption

The portal's real trick on Linux is not that it can replace a dialog. It is that
applications *opt in* to an indirection, and the desktop fills it. What would the same
opt-in look like on macOS?

### 6.1 There is no macOS portal

**[V]** GTK, the portal's largest client, does not even try: the same file that documents
the portal path documents `MODE_QUARTZ`, and says *"On macOS the `NSSavePanel` and
`NSOpenPanel` classes are used to provide native file chooser dialogs"*
(`gtk/gtkfilechoosernative.c:183-186`, mode enum at `:194-200`).

**[I]** There is no session bus, no `.portal` registry, no daemon that brokers a chooser
between apps. Anything equivalent would be ours to invent and ours alone to populate.

### 6.2 In-process adoption already works

**[V]** `examples/dialog.rs` (230 lines) is a full host: it constructs
`Dialog::new(DialogSettings::new().kind(kind), Message::DialogMessage, Message::DialogResult)`,
forwards `dialog.update`, `dialog.view(window_id)` and `dialog.subscription()`, and reads
`DialogResult` back. It sets `DialogChoice::ComboBox`/`CheckBox` and `DialogFilter`
(`Glob` and `Mime` patterns) the same way the portal does. It declares no
`required-features`, and it builds here: `cargo build --release --no-default-features
--features bzip2,lzma-rust2,wgpu,quicklook --example dialog` exits 0 on this Mac.

**[I]** This is a real answer for exactly one audience: other Rust applications built on
libcosmic/iced. They link the crate and get our chooser with no system involvement at all.
It is not an answer for Safari, Photoshop or any app we do not compile.

### 6.3 Out-of-process adoption would need a mode we do not have

**[I]** The missing piece is small and well understood: a `--dialog` mode that runs the
existing `Dialog<M>` in its own process and reports the `Vec<PathBuf>` back — over stdout
as NUL-separated paths, or over a Unix socket path passed in `argv`. That is the portal's
`file_chooser_update_msg` minus D-Bus, roughly thirty lines plus argument plumbing. The
calling app spawns it, waits, and reads paths. No entitlement, no permission, no system
integration.

**[I]** Cost to an adopting app: it gives up the system panel's sidebar, its iCloud and
File Provider locations, its tag search, its Recents, its keyboard handling and its
familiarity. Very few macOS developers would take that trade for a chooser one of their
users has installed.

### 6.4 The sandbox makes it worse than useless for App Store apps

**[V]** The only thing that extends a sandbox to a chosen file is the system panel:

> The operating system displays open and save panels in a separate process, and extends
> your app's sandbox to include the selected URLs.
>
> The operating system starts security-scoped access on URLs passed from open panels, save
> panels, or items dragged to your app's icon in the Dock, as if you called
> `startAccessingSecurityScopedResource()`.

— [Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)

**[V]** App Sandbox is mandatory for Mac App Store distribution
([App Sandbox](https://developer.apple.com/documentation/security/app-sandbox)).

**[I]** So for any sandboxed caller, a path handed over by cosmic-files is a path the caller
cannot open. There is no third-party route to grant a security scope; the panel process is
the grantor. Linux has the same hazard and a public answer — the document portal's per-app
FUSE view (§2.5). macOS's equivalent is reserved to Apple's panel. **Out-of-process adoption
is therefore limited to non-sandboxed apps**, which on macOS means "not from the App Store
and not most of what a user runs".

---

## 7. Verdict

1. **Replacing the system Save/Open panel on macOS: not possible.** The panel runs in an
   Apple-signed platform binary on a sealed read-only volume, protected by SIP from
   attachment and `DYLD_*` injection; the caller's own process is walled off from it by
   library validation. There is no API, defaults key or extension point for substituting
   it. [V] for each mechanism; [I] for the absolute.
2. **Augmenting it: possible, and expensive.** The Default Folder X route — Accessibility
   notifications plus your own window plus a screenshot to hide the seams — works and is
   shipping today, at the cost of Accessibility, Apple Events, Full Disk Access and Screen
   Recording grants. It yields a toolbar attached to Apple's panel, not our file manager. I
   would not spend the port's budget there.
3. **The sanctioned extension point is the File Provider extension**, and it is a different
   feature: it adds a location to the panel and to Finder, not a chooser. Useful only if
   cosmic-files ever fronts remote storage.
4. **Voluntary adoption is the only honest "yes".** In-process already works for
   libcosmic/iced apps (`examples/dialog.rs`). Out-of-process would need a small `--dialog`
   mode we do not have, and would still be unusable by sandboxed callers, because only the
   system panel can extend a sandbox.
5. **On Linux the answer stays "already done"**, and it is done by the portal backend, not
   by us: `Dialog<M>` is a widget, xdg-desktop-portal-cosmic is the process with the bus
   name, and the contract cosmic-files exposes is `Vec<PathBuf>`.

**The closest useful thing for this project:** add the out-of-process `--dialog` mode
anyway. It is cheap, it reuses code that already compiles on macOS, it gives us a way to
exercise and screenshot the chooser outside the portal on a machine with no D-Bus, and it is
the exact seam any future adopter (or our own "choose a destination" flows) would use.
Frame it as a testing and self-use facility, not as a system integration, because it cannot
be one.

---

## 8. Sources

**Primary, Apple:** `NSSavePanel`, `NSOpenPanel`, `NSOpenSavePanelDelegate` reference;
*Accessing files from the macOS App Sandbox*; *App Sandbox*; *Hardened Runtime*; *Disable
Library Validation Entitlement*; *File Provider*, `NSFileProviderDomain`, *File Provider UI*;
*System Integrity Protection Guide — Runtime Protections*.
`MacOSX.sdk` headers `NSSavePanel.h`, `NSOpenPanel.h` (Xcode 26.3, SDK macosx26).

**Primary, this machine:** the `com.apple.appkit.xpc.openAndSavePanelService.xpc` bundle —
`Info.plist`, `codesign -dv --entitlements -`, `otool -L`, `nm -gU`, `strings`; `csrutil
status`; `mount`; `pgrep`.

**Primary, freedesktop:** `org.freedesktop.impl.portal.FileChooser` spec;
`doc/writing-a-new-backend.rst`, `doc/portals.conf.rst.in`, `doc/documents-and-fuse.rst`,
`doc/for-app-developers.rst` from flatpak/xdg-desktop-portal.

**Primary, source:** pop-os/xdg-desktop-portal-cosmic at `2f41161` (`src/file_chooser.rs`,
`src/main.rs`, `src/subscription.rs`, `data/cosmic.portal`, `Cargo.toml`);
GNOME/gtk `gtk/gtkfilechoosernative.c`, `gtk/gtkfilechoosernativeportal.c`,
`gtk/gtkprivate.h`; this repo.

**Primary, vendor:** St. Clair Software, Default Folder X FAQ and User's Guide v6.2.8.

**No secondary sources were used.** Nothing here rests on a blog post or a forum answer.
