// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::menu::key_bind::{KeyBind, Modifier};
use cosmic::{
    iced::core::keyboard::key::Named,
    iced::keyboard::{Key, Modifiers},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::app::Action;
use crate::fl;
use crate::tab;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ModifierName {
    Ctrl,
    Shift,
    Alt,
    Super,
}

impl ModifierName {
    fn to_modifier(self) -> Modifier {
        match self {
            Self::Ctrl => Modifier::Ctrl,
            Self::Shift => Modifier::Shift,
            Self::Alt => Modifier::Alt,
            Self::Super => Modifier::Super,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Binding {
    pub modifiers: Vec<ModifierName>,
    pub key: String,
}

impl Binding {
    fn to_key_bind(&self) -> Option<KeyBind> {
        let key = key_from_string(&self.key)?;
        let mut modifiers = Vec::new();
        for modifier in [
            ModifierName::Ctrl,
            ModifierName::Shift,
            ModifierName::Alt,
            ModifierName::Super,
        ] {
            if self.modifiers.contains(&modifier) {
                modifiers.push(modifier.to_modifier());
            }
        }

        Some(KeyBind { modifiers, key })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum KeyBindAction {
    Disable,
    About,
    AddToSidebar,
    Compress,
    Copy,
    CopyPath,
    CopyTo,
    Cut,
    Delete,
    EditHistory,
    EditLocation,
    // Eject,
    EmptyTrash,
    // TODO requires checking input files
    // ExtractHere,
    // ExtractTo,
    Gallery,
    HistoryNext,
    HistoryPrevious,
    ItemDown,
    ItemLeft,
    ItemPageDown,
    ItemPageUp,
    ItemRight,
    ItemUp,
    LocationUp,
    KeyboardShortcuts,
    MoveTo,
    NewFile,
    NewFolder,
    Open,
    OpenInNewTab,
    OpenInNewWindow,
    OpenItemLocation,
    OpenTerminal,
    OpenWith,
    Paste,
    PermanentlyDelete,
    Preview,
    Reload,
    RemoveFromRecents,
    Rename,
    RestoreFromTrash,
    SearchActivate,
    SelectFirst,
    SelectLast,
    SelectAll,
    Settings,
    TabClose,
    TabNew,
    TabNext,
    TabPrev,
    TabViewGrid,
    TabViewList,
    ToggleFoldersFirst,
    ToggleShowHidden,
    WindowClose,
    WindowNew,
    ZoomDefault,
    ZoomIn,
    ZoomOut,
}

impl KeyBindAction {
    fn to_action(self, mode: &tab::Mode) -> Option<Action> {
        match (self, mode) {
            (Self::Disable, _) => None,
            (Self::About, tab::Mode::App) => Some(Action::About),
            (Self::AddToSidebar, tab::Mode::App) => Some(Action::AddToSidebar),
            (Self::Compress, tab::Mode::App) => Some(Action::Compress),
            (Self::Compress, tab::Mode::Desktop) => Some(Action::Compress),
            (Self::Copy, tab::Mode::App) => Some(Action::Copy),
            (Self::Copy, tab::Mode::Desktop) => Some(Action::Copy),
            (Self::CopyPath, tab::Mode::App) => Some(Action::CopyPath),
            (Self::CopyPath, tab::Mode::Desktop) => Some(Action::CopyPath),
            (Self::CopyTo, tab::Mode::App) => Some(Action::CopyTo),
            (Self::CopyTo, tab::Mode::Desktop) => Some(Action::CopyTo),
            (Self::Cut, tab::Mode::App) => Some(Action::Cut),
            (Self::Cut, tab::Mode::Desktop) => Some(Action::Cut),
            (Self::Delete, tab::Mode::App) => Some(Action::Delete),
            (Self::Delete, tab::Mode::Desktop) => Some(Action::Delete),
            // (Self::Eject, tab::Mode::App) => Some(Action::Eject),
            // (Self::Eject, tab::Mode::Desktop) => Some(Action::Eject),
            (Self::EmptyTrash, tab::Mode::App) => Some(Action::EmptyTrash),
            (Self::EmptyTrash, tab::Mode::Desktop) => Some(Action::EmptyTrash),
            (Self::EditLocation, tab::Mode::App) => Some(Action::EditLocation),
            (Self::EditLocation, tab::Mode::Dialog(_)) => Some(Action::EditLocation),
            (Self::EditHistory, tab::Mode::App) => Some(Action::EditHistory),
            // (Self::ExtractHere, tab::Mode::App) => Some(Action::ExtractHere),
            // (Self::ExtractHere, tab::Mode::Desktop) => Some(Action::ExtractHere),
            // (Self::ExtractTo, tab::Mode::App) => Some(Action::ExtractTo),
            // (Self::ExtractTo, tab::Mode::Desktop) => Some(Action::ExtractTo),
            (Self::Gallery, _) => Some(Action::Gallery),
            (Self::HistoryNext, tab::Mode::App) => Some(Action::HistoryNext),
            (Self::HistoryNext, tab::Mode::Dialog(_)) => Some(Action::HistoryNext),
            (Self::HistoryPrevious, tab::Mode::App) => Some(Action::HistoryPrevious),
            (Self::HistoryPrevious, tab::Mode::Dialog(_)) => Some(Action::HistoryPrevious),
            (Self::ItemDown, _) => Some(Action::ItemDown),
            (Self::ItemLeft, _) => Some(Action::ItemLeft),
            (Self::ItemPageDown, _) => Some(Action::ItemPageDown),
            (Self::ItemPageUp, _) => Some(Action::ItemPageUp),
            (Self::ItemRight, _) => Some(Action::ItemRight),
            (Self::ItemUp, _) => Some(Action::ItemUp),
            (Self::LocationUp, tab::Mode::App) => Some(Action::LocationUp),
            (Self::LocationUp, tab::Mode::Dialog(_)) => Some(Action::LocationUp),
            (Self::KeyboardShortcuts, tab::Mode::App) => Some(Action::KeyboardShortcuts),
            (Self::MoveTo, tab::Mode::App) => Some(Action::MoveTo),
            (Self::MoveTo, tab::Mode::Desktop) => Some(Action::MoveTo),
            (Self::NewFile, _) => Some(Action::NewFile),
            (Self::NewFolder, _) => Some(Action::NewFolder),
            (Self::Open, _) => Some(Action::Open),
            (Self::OpenWith, _) => Some(Action::OpenWith),
            (Self::OpenInNewTab, tab::Mode::App) => Some(Action::OpenInNewTab),
            (Self::OpenInNewWindow, tab::Mode::App) => Some(Action::OpenInNewWindow),
            (Self::OpenInNewWindow, tab::Mode::Desktop) => Some(Action::OpenInNewWindow),
            (Self::OpenItemLocation, tab::Mode::App) => Some(Action::OpenItemLocation),
            (Self::OpenItemLocation, tab::Mode::Desktop) => Some(Action::OpenItemLocation),
            (Self::OpenTerminal, tab::Mode::App) => Some(Action::OpenTerminal),
            (Self::OpenTerminal, tab::Mode::Desktop) => Some(Action::OpenTerminal),
            (Self::Paste, tab::Mode::App) => Some(Action::Paste),
            (Self::Paste, tab::Mode::Desktop) => Some(Action::Paste),
            (Self::PermanentlyDelete, tab::Mode::App) => Some(Action::PermanentlyDelete),
            (Self::PermanentlyDelete, tab::Mode::Desktop) => Some(Action::PermanentlyDelete),
            (Self::Preview, _) => Some(Action::Preview),
            (Self::Reload, _) => Some(Action::Reload),
            (Self::RemoveFromRecents, tab::Mode::App) => Some(Action::RemoveFromRecents),
            (Self::RemoveFromRecents, tab::Mode::Desktop) => Some(Action::RemoveFromRecents),
            (Self::Rename, tab::Mode::App) => Some(Action::Rename),
            (Self::Rename, tab::Mode::Desktop) => Some(Action::Rename),
            (Self::RestoreFromTrash, tab::Mode::App) => Some(Action::RestoreFromTrash),
            (Self::RestoreFromTrash, tab::Mode::Desktop) => Some(Action::RestoreFromTrash),
            (Self::SearchActivate, tab::Mode::App) => Some(Action::SearchActivate),
            (Self::SearchActivate, tab::Mode::Dialog(_)) => Some(Action::SearchActivate),
            (Self::SelectFirst, _) => Some(Action::SelectFirst),
            (Self::SelectLast, _) => Some(Action::SelectLast),
            (Self::SelectAll, _) => Some(Action::SelectAll),
            (Self::Settings, tab::Mode::App) => Some(Action::Settings),
            (Self::TabClose, tab::Mode::App) => Some(Action::TabClose),
            (Self::TabNew, tab::Mode::App) => Some(Action::TabNew),
            (Self::TabNext, tab::Mode::App) => Some(Action::TabNext),
            (Self::TabPrev, tab::Mode::App) => Some(Action::TabPrev),
            (Self::TabViewGrid, _) => Some(Action::TabViewGrid),
            (Self::TabViewList, _) => Some(Action::TabViewList),
            (Self::ToggleShowHidden, _) => Some(Action::ToggleShowHidden),
            (Self::ToggleFoldersFirst, _) => Some(Action::ToggleFoldersFirst),
            (Self::WindowClose, tab::Mode::App) => Some(Action::WindowClose),
            (Self::WindowNew, tab::Mode::App) => Some(Action::WindowNew),
            (Self::ZoomDefault, tab::Mode::App) => Some(Action::ZoomDefault),
            (Self::ZoomDefault, tab::Mode::Dialog(_)) => Some(Action::ZoomDefault),
            (Self::ZoomIn, tab::Mode::App) => Some(Action::ZoomIn),
            (Self::ZoomIn, tab::Mode::Dialog(_)) => Some(Action::ZoomIn),
            (Self::ZoomOut, tab::Mode::App) => Some(Action::ZoomOut),
            (Self::ZoomOut, tab::Mode::Dialog(_)) => Some(Action::ZoomOut),
            (_, _) => None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Shortcuts(pub BTreeMap<Binding, KeyBindAction>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingSource {
    Default,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBinding {
    pub binding: Binding,
    pub source: BindingSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShortcutsConfig {
    defaults: Shortcuts,
    pub custom: Shortcuts,
}

impl ShortcutsConfig {
    pub fn new(custom: Shortcuts) -> Self {
        Self {
            defaults: fallback_shortcuts(&tab::Mode::App),
            custom,
        }
    }

    pub fn key_binds(&self, mode: &tab::Mode) -> HashMap<KeyBind, Action> {
        let mut binds = HashMap::new();
        insert_shortcuts(&self.defaults, &mut binds, false, &mode);
        insert_shortcuts(&self.custom, &mut binds, true, &mode);
        binds
    }

    pub fn bindings_for_action(&self, action: KeyBindAction) -> (Vec<ResolvedBinding>, bool) {
        let mut bindings = Vec::new();

        let mut changed = false;
        for (binding, default_action) in &self.defaults.0 {
            if *default_action != action {
                continue;
            }

            match self.custom.0.get(binding) {
                Some(KeyBindAction::Disable) => {
                    changed = true;
                }
                Some(custom_action) => {
                    if *custom_action == action {
                        bindings.push(ResolvedBinding {
                            binding: binding.clone(),
                            source: BindingSource::Custom,
                        });
                        changed = true;
                    }
                }
                None => bindings.push(ResolvedBinding {
                    binding: binding.clone(),
                    source: BindingSource::Default,
                }),
            }
        }

        for (binding, custom_action) in &self.custom.0 {
            if *custom_action == action
                && !bindings.iter().any(|resolved| resolved.binding == *binding)
            {
                bindings.push(ResolvedBinding {
                    binding: binding.clone(),
                    source: BindingSource::Custom,
                });
                changed = true;
            }
        }

        (bindings, changed)
    }

    pub fn action_for_binding(&self, binding: &Binding) -> Option<KeyBindAction> {
        if let Some(action) = self.custom.0.get(binding) {
            if *action == KeyBindAction::Disable {
                return None;
            }
            return Some(*action);
        }

        self.defaults.0.get(binding).copied()
    }

    pub fn reset_action(&mut self, reset_action: KeyBindAction) {
        self.custom.0.retain(|binding, action| {
            if *action == reset_action {
                // Remove any matching bindings
                return false;
            }
            if let Some(default_action) = self.defaults.0.get(binding)
                && *default_action == reset_action
            {
                // Remove binding that overrode a default
                return false;
            }
            true
        });
    }
}

pub fn action_label(action: KeyBindAction) -> String {
    match action {
        KeyBindAction::Disable => fl!("disable"),
        KeyBindAction::About => fl!("about"),
        KeyBindAction::AddToSidebar => fl!("add-to-sidebar"),
        KeyBindAction::Compress => fl!("compress"),
        KeyBindAction::Copy => fl!("copy"),
        KeyBindAction::CopyPath => fl!("copy-path"),
        KeyBindAction::CopyTo => fl!("copy-to"),
        KeyBindAction::Cut => fl!("cut"),
        KeyBindAction::Delete => fl!("delete"),
        // KeyBindAction::Eject => fl!("eject"),
        KeyBindAction::EditLocation => fl!("edit-location"),
        KeyBindAction::EditHistory => fl!("edit-history"),
        KeyBindAction::EmptyTrash => fl!("empty-trash"),
        // KeyBindAction::ExtractHere => fl!("extract-here"),
        // KeyBindAction::ExtractTo => fl!("extract-to"),
        KeyBindAction::Gallery => fl!("gallery"),
        KeyBindAction::HistoryNext => fl!("history-next"),
        KeyBindAction::HistoryPrevious => fl!("history-previous"),
        KeyBindAction::ItemDown => fl!("item-down"),
        KeyBindAction::ItemLeft => fl!("item-left"),
        KeyBindAction::ItemPageDown => fl!("item-page-down"),
        KeyBindAction::ItemPageUp => fl!("item-page-up"),
        KeyBindAction::ItemRight => fl!("item-right"),
        KeyBindAction::ItemUp => fl!("item-up"),
        KeyBindAction::LocationUp => fl!("location-up"),
        KeyBindAction::KeyboardShortcuts => fl!("keyboard-shortcuts"),
        KeyBindAction::MoveTo => fl!("move-to"),
        KeyBindAction::NewFile => fl!("new-file"),
        KeyBindAction::NewFolder => fl!("new-folder"),
        KeyBindAction::Open => fl!("open"),
        KeyBindAction::OpenInNewTab => fl!("open-in-new-tab"),
        KeyBindAction::OpenInNewWindow => fl!("open-in-new-window"),
        KeyBindAction::OpenItemLocation => fl!("open-item-location"),
        KeyBindAction::OpenTerminal => fl!("open-in-terminal"),
        KeyBindAction::OpenWith => fl!("open-with"),
        KeyBindAction::Paste => fl!("paste"),
        KeyBindAction::PermanentlyDelete => fl!("permanently-delete"),
        KeyBindAction::Preview => fl!("preview"),
        KeyBindAction::Reload => fl!("reload"),
        KeyBindAction::RemoveFromRecents => fl!("remove-from-recents"),
        KeyBindAction::Rename => fl!("rename"),
        KeyBindAction::RestoreFromTrash => fl!("restore-from-trash"),
        KeyBindAction::SearchActivate => fl!("search-activate"),
        KeyBindAction::SelectFirst => fl!("select-first"),
        KeyBindAction::SelectLast => fl!("select-last"),
        KeyBindAction::SelectAll => fl!("select-all"),
        KeyBindAction::Settings => fl!("settings"),
        KeyBindAction::TabClose => fl!("tab-close"),
        KeyBindAction::TabNew => fl!("tab-new"),
        KeyBindAction::TabNext => fl!("tab-next"),
        KeyBindAction::TabPrev => fl!("tab-prev"),
        KeyBindAction::TabViewGrid => fl!("tab-view-grid"),
        KeyBindAction::TabViewList => fl!("tab-view-list"),
        KeyBindAction::ToggleShowHidden => fl!("toggle-show-hidden"),
        KeyBindAction::ToggleFoldersFirst => fl!("toggle-folders-first"),
        KeyBindAction::WindowClose => fl!("window-close"),
        KeyBindAction::WindowNew => fl!("window-new"),
        KeyBindAction::ZoomDefault => fl!("zoom-default"),
        KeyBindAction::ZoomIn => fl!("zoom-in"),
        KeyBindAction::ZoomOut => fl!("zoom-out"),
    }
}

pub struct ShortcutGroup {
    pub title: String,
    pub actions: Vec<KeyBindAction>,
}

pub fn shortcut_groups() -> Vec<ShortcutGroup> {
    let mut groups = Vec::new();
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-navigation"),
        actions: vec![
            KeyBindAction::ItemDown,
            KeyBindAction::ItemLeft,
            KeyBindAction::ItemRight,
            KeyBindAction::ItemUp,
            KeyBindAction::ItemPageDown,
            KeyBindAction::ItemPageUp,
            KeyBindAction::LocationUp,
            KeyBindAction::EditLocation,
            KeyBindAction::HistoryNext,
            KeyBindAction::HistoryPrevious,
            KeyBindAction::SearchActivate,
        ],
    });
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-view"),
        actions: vec![
            KeyBindAction::ToggleFoldersFirst,
            KeyBindAction::ToggleShowHidden,
            KeyBindAction::ZoomIn,
            KeyBindAction::ZoomDefault,
            KeyBindAction::ZoomOut,
        ],
    });
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-tabs"),
        actions: vec![
            KeyBindAction::TabClose,
            KeyBindAction::TabNew,
            KeyBindAction::TabNext,
            KeyBindAction::TabPrev,
            KeyBindAction::TabViewList,
            KeyBindAction::TabViewGrid,
        ],
    });
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-windows"),
        actions: vec![KeyBindAction::WindowClose, KeyBindAction::WindowNew],
    });
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-operations"),
        actions: vec![
            KeyBindAction::AddToSidebar,
            KeyBindAction::Compress,
            KeyBindAction::Copy,
            KeyBindAction::CopyPath,
            KeyBindAction::CopyTo,
            KeyBindAction::Cut,
            KeyBindAction::Delete,
            // KeyBindAction::Eject,
            KeyBindAction::EmptyTrash,
            // KeyBindAction::ExtractHere,
            // KeyBindAction::ExtractTo,
            KeyBindAction::Gallery,
            KeyBindAction::PermanentlyDelete,
            KeyBindAction::MoveTo,
            KeyBindAction::NewFile,
            KeyBindAction::NewFolder,
            KeyBindAction::Open,
            KeyBindAction::OpenWith,
            KeyBindAction::OpenInNewTab,
            KeyBindAction::OpenInNewWindow,
            KeyBindAction::OpenItemLocation,
            KeyBindAction::OpenTerminal,
            KeyBindAction::Preview,
            KeyBindAction::Paste,
            KeyBindAction::Reload,
            KeyBindAction::RemoveFromRecents,
            KeyBindAction::Rename,
            KeyBindAction::RestoreFromTrash,
            KeyBindAction::SelectFirst,
            KeyBindAction::SelectLast,
            KeyBindAction::SelectAll,
        ],
    });
    groups.push(ShortcutGroup {
        title: fl!("shortcut-group-context-pages"),
        actions: vec![
            KeyBindAction::About,
            KeyBindAction::KeyboardShortcuts,
            KeyBindAction::Settings,
            KeyBindAction::EditHistory,
        ],
    });
    groups
}

pub fn binding_display(binding: &Binding) -> String {
    binding
        .to_key_bind()
        .map(|key_bind| key_bind.to_string())
        .unwrap_or_else(|| binding.key.clone())
}

pub fn binding_from_key(modifiers: Modifiers, key: Key) -> Option<Binding> {
    if is_modifier_only_key(&key) {
        return None;
    }
    let key = key_to_string(&key)?;
    let mut binding_modifiers = Vec::new();
    if modifiers.control() {
        binding_modifiers.push(ModifierName::Ctrl);
    }
    if modifiers.shift() {
        binding_modifiers.push(ModifierName::Shift);
    }
    if modifiers.alt() {
        binding_modifiers.push(ModifierName::Alt);
    }
    if modifiers.logo() {
        binding_modifiers.push(ModifierName::Super);
    }
    Some(Binding {
        modifiers: binding_modifiers,
        key,
    })
}

fn insert_shortcuts(
    shortcuts: &Shortcuts,
    binds: &mut HashMap<KeyBind, Action>,
    allow_disable: bool,
    mode: &tab::Mode,
) {
    for (binding, action) in &shortcuts.0 {
        let key_bind = match binding.to_key_bind() {
            Some(key_bind) => key_bind,
            None => {
                log::warn!("invalid key binding: {:?}", binding);
                continue;
            }
        };
        if allow_disable && *action == KeyBindAction::Disable {
            binds.remove(&key_bind);
            continue;
        }
        let Some(action) = action.to_action(mode) else {
            log::warn!("unsupported shortcut action: {:?}", action);
            continue;
        };
        binds.insert(key_bind, action);
    }
}

fn fallback_shortcuts(mode: &tab::Mode) -> Shortcuts {
    let mut shortcuts = BTreeMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            shortcuts.insert(
                Binding {
                    modifiers: vec![$(ModifierName::$modifier),*],
                    key: $key.to_string(),
                },
                KeyBindAction::$action,
            );
        }};
    }
    // Common keys
    bind!([], "ArrowDown", ItemDown);
    bind!([], "ArrowLeft", ItemLeft);
    bind!([], "ArrowRight", ItemRight);
    bind!([], "ArrowUp", ItemUp);
    bind!([], "F5", Reload);
    bind!([], "Home", SelectFirst);
    bind!([], "End", SelectLast);
    bind!([], "PageDown", ItemPageDown);
    bind!([], "PageUp", ItemPageUp);
    bind!([Shift], "ArrowDown", ItemDown);
    bind!([Shift], "ArrowLeft", ItemLeft);
    bind!([Shift], "ArrowRight", ItemRight);
    bind!([Shift], "ArrowUp", ItemUp);
    bind!([Shift], "Home", SelectFirst);
    bind!([Shift], "End", SelectLast);
    bind!([Shift], "PageDown", ItemPageDown);
    bind!([Shift], "PageUp", ItemPageUp);
    bind!([Ctrl, Shift], "n", NewFolder);
    bind!([], "Enter", Open);
    bind!([Ctrl], " ", Preview);
    bind!([], " ", Gallery);

    bind!([Ctrl], "H", ToggleShowHidden);
    bind!([Ctrl], "A", SelectAll);
    // Switch view
    bind!([Ctrl], "1", TabViewList);
    bind!([Ctrl], "2", TabViewGrid);

    // App-only keys
    if matches!(mode, tab::Mode::App) {
        bind!([Ctrl], "D", AddToSidebar);
        bind!([Ctrl], "Enter", OpenInNewTab);
        bind!([Ctrl], ",", Settings);
        bind!([Ctrl], "W", TabClose);
        bind!([Ctrl], "T", TabNew);
        bind!([Ctrl], "Tab", TabNext);
        bind!([Ctrl, Shift], "Tab", TabPrev);
        bind!([Ctrl], "Q", WindowClose);
        bind!([Ctrl], "N", WindowNew);
    }

    // App and desktop only keys
    if matches!(mode, tab::Mode::App | tab::Mode::Desktop) {
        bind!([Ctrl], "C", Copy);
        bind!([Ctrl, Shift], "C", CopyPath);
        bind!([Ctrl], "X", Cut);
        bind!([], "Delete", Delete);
        bind!([Shift], "Delete", PermanentlyDelete);
        bind!([Shift], "Enter", OpenInNewWindow);
        bind!([Ctrl], "V", Paste);
        bind!([], "F2", Rename);
    }

    // App and dialog only keys
    if matches!(mode, tab::Mode::App | tab::Mode::Dialog(_)) {
        bind!([Ctrl], "=", ZoomIn);
        bind!([Ctrl], "+", ZoomIn);
        bind!([Ctrl], "0", ZoomDefault);
        bind!([Ctrl], "-", ZoomOut);
        bind!([Ctrl], "L", EditLocation);
        bind!([Alt], "ArrowRight", HistoryNext);
        bind!([Alt], "ArrowLeft", HistoryPrevious);
        bind!([], "Backspace", HistoryPrevious);
        bind!([Alt], "ArrowUp", LocationUp);
        bind!([Ctrl], "F", SearchActivate);
    }

    Shortcuts(shortcuts)
}

fn key_from_string(value: &str) -> Option<Key> {
    match value {
        "Backspace" => Some(Key::Named(Named::Backspace)),
        "Enter" => Some(Key::Named(Named::Enter)),
        "Escape" => Some(Key::Named(Named::Escape)),
        "Insert" => Some(Key::Named(Named::Insert)),
        "Delete" => Some(Key::Named(Named::Delete)),
        "Tab" => Some(Key::Named(Named::Tab)),
        "F1" => Some(Key::Named(Named::F1)),
        "F2" => Some(Key::Named(Named::F2)),
        "F3" => Some(Key::Named(Named::F3)),
        "F4" => Some(Key::Named(Named::F4)),
        "F5" => Some(Key::Named(Named::F5)),
        "F6" => Some(Key::Named(Named::F6)),
        "F7" => Some(Key::Named(Named::F7)),
        "F8" => Some(Key::Named(Named::F8)),
        "F9" => Some(Key::Named(Named::F9)),
        "F10" => Some(Key::Named(Named::F10)),
        "F11" => Some(Key::Named(Named::F11)),
        "F12" => Some(Key::Named(Named::F12)),
        "Home" => Some(Key::Named(Named::Home)),
        "End" => Some(Key::Named(Named::End)),
        "ArrowLeft" | "Left" => Some(Key::Named(Named::ArrowLeft)),
        "ArrowRight" | "Right" => Some(Key::Named(Named::ArrowRight)),
        "ArrowUp" | "Up" => Some(Key::Named(Named::ArrowUp)),
        "ArrowDown" | "Down" => Some(Key::Named(Named::ArrowDown)),
        "PageUp" => Some(Key::Named(Named::PageUp)),
        "PageDown" => Some(Key::Named(Named::PageDown)),
        "Space" | "space" => Some(Key::Character(" ".into())),
        _ if !value.is_empty() => Some(Key::Character(value.into())),
        _ => None,
    }
}

fn key_to_string(key: &Key) -> Option<String> {
    match key {
        Key::Character(c) => {
            if c == " " {
                Some("Space".to_string())
            } else if c.len() == 1 && c.chars().all(|ch| ch.is_ascii_alphabetic()) {
                Some(c.to_uppercase())
            } else {
                Some(c.to_string())
            }
        }
        Key::Named(named) => Some(format!("{named:?}")),
        _ => None,
    }
}

fn is_modifier_only_key(key: &Key) -> bool {
    matches!(
        key,
        Key::Named(
            Named::Alt
                | Named::AltGraph
                | Named::CapsLock
                | Named::Control
                | Named::Fn
                | Named::FnLock
                | Named::NumLock
                | Named::ScrollLock
                | Named::Shift
                | Named::Symbol
                | Named::SymbolLock
                | Named::Meta
                | Named::Hyper
                | Named::Super
        )
    )
}
