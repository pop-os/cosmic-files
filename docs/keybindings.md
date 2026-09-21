# Custom key bindings

COSMIC Files ships a set of default keyboard shortcuts. You can add to them, rebind them, or
switch them off by editing the configuration file. There is no settings UI for this yet — the file
is the only interface.

Changes are picked up while the application is running: save the file and the new bindings apply
immediately, including in the menus, which show the shortcut for each action.

## Where the file lives

Each configuration field is stored in its own [RON](https://github.com/ron-rs/ron) file, named
after the field. Key bindings live in `keybinds`:

| Platform | Path |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/com.system76.CosmicFiles/v1/keybinds` (usually `~/.config/…`) |
| macOS | `~/Library/Application Support/com.system76.CosmicFiles/v1/keybinds` |
| Windows | `%APPDATA%\com.system76.CosmicFiles\v1\keybinds` |

Create the file if it does not exist.

## Format

The file holds a single map from a key binding to an action:

```ron
{
    "Ctrl+Shift+e": "NewFile",
    "Ctrl+e": "ToggleSort(Size)",
    "F3": "Preview",
    "Ctrl+w": "Disable",
}
```

Entries are overrides layered on top of the defaults. Anything you do not mention keeps working as
before, so the file only ever needs to contain the bindings you actually want to change.

If an entry names a key it cannot parse or an action it does not recognize, that one entry is
skipped with a warning in the log and the rest of the file still applies.

## Key bindings

A binding is zero or more modifiers followed by exactly one key, joined with `+`:

```
Ctrl+Shift+n
Alt+ArrowUp
F5
Ctrl+Space
Ctrl++
```

Modifiers, any capitalization, in any order:

| Modifier | Also accepted |
| --- | --- |
| `Super` | `Logo`, `Meta`, `Cmd`, `Command`, `Win`, `Windows` |
| `Ctrl` | `Control` |
| `Alt` | `Option`, `Opt` |
| `Shift` | |

The key is either:

- **A character**, written literally: `n`, `1`, `,`, `-`, `+`. Case does not matter; `Ctrl+N` and
  `Ctrl+n` are the same binding. The `+` key works as expected: `Ctrl++`.
- **The space bar**, written `Space`.
- **A named key**, written the way the key is named in the [W3C UI Events key
  values](https://www.w3.org/TR/uievents-key/) spec: `Enter`, `Tab`, `Escape`, `Backspace`,
  `Delete`, `Home`, `End`, `PageUp`, `PageDown`, `ArrowUp`, `ArrowDown`, `ArrowLeft`,
  `ArrowRight`, `F1` … `F35`, `Insert`, `MediaPlayPause`, and so on. Capitalization is ignored
  here too.

Surrounding whitespace is ignored, so `"Ctrl + Shift + Tab"` also works.

## Actions

The value is the name of an action, spelled the way it is spelled in the menus' source — for
example `Copy`, `NewFolder`, `TabNew`, `ToggleShowHidden`, `ZoomIn`. The full list:

```
About                     AddToSidebar              Compress
Copy                      CopyPath                  CopyTo
Cut                       CosmicSettingsDesktop     CosmicSettingsDisplays
CosmicSettingsWallpaper   DesktopViewOptions        Delete
EditHistory               EditLocation              Eject
EmptyTrash                ExtractHere               ExtractTo
Gallery                   HistoryNext               HistoryPrevious
ItemDown                  ItemLeft                  ItemPageDown
ItemPageUp                ItemRight                 ItemUp
LocationUp                MoveTo                    NewFile
NewFolder                 Open                      OpenInNewTab
OpenInNewWindow           OpenItemLocation          OpenTerminal
OpenWith                  Paste                     PermanentlyDelete
Preview                   Reload                    RemoveFromRecents
Rename                    RestoreFromTrash          SearchActivate
SelectFirst               SelectLast                SelectAll
Settings                  TabClose                  TabNew
TabNext                   TabPrev                   TabViewGrid
TabViewList               ToggleFoldersFirst        ToggleShowHidden
WindowClose               WindowNew                 ZoomDefault
ZoomIn                    ZoomOut                   Recents
```

Four actions take arguments, written in parentheses:

| Action | Meaning |
| --- | --- |
| `SetSort(<column>, <ascending>)` | Sort by a column in a given direction, e.g. `SetSort(Modified, false)` |
| `ToggleSort(<column>)` | Sort by a column, flipping the direction if it is already sorted by it |
| `RunContextAction(<index>)` | Run the custom context menu action at this index (zero based) |
| `ExecEntryAction(<index>)` | Run a desktop entry action at this index (desktop builds only) |

`<column>` is one of `Name`, `Modified`, `Size`, `TrashedOn`. `<ascending>` is `true` or `false`.

## Removing a default binding

Use the action `Disable` to switch a default off without replacing it:

```ron
{
    "Ctrl+w": "Disable",
}
```

## Notes and limits

- A binding maps to exactly one action. Binding a key that already has a default replaces that
  default; the action's other default keys keep working unless you disable them too.
- Some defaults are only active in certain contexts (for example, tab shortcuts do not exist on the
  desktop). An override for such a binding only takes effect where that context applies.
- Open/save dialogs use the built-in defaults and ignore this file.
- Keys are matched by their logical value, with a fallback to the physical key position for
  non-Latin keyboard layouts, so `Ctrl+c` keeps working on a Cyrillic layout.

## Example

```ron
{
    // Open a terminal in the current folder.
    "Ctrl+Shift+t": "OpenTerminal",
    // Sort by size, largest first.
    "Ctrl+3": "SetSort(Size, false)",
    // Free up Ctrl+q so it cannot close the window by accident.
    "Ctrl+q": "Disable",
    // Second keyboard shortcut for renaming.
    "Ctrl+Shift+r": "Rename",
}
```
