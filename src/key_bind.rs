use cosmic::iced::core::keyboard::key::Named;
use cosmic::iced::keyboard::Key;
use cosmic::widget::menu::key_bind::{KeyBind, Modifier};
use serde::de::Error as _;
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use crate::app::Action;
use crate::tab::{self, HeadingOptions};

/// Default key bindings for a tab mode.
///
/// User overrides are applied on top of these by [`key_binds_with_overrides`].
pub fn key_binds(mode: &tab::Mode) -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    // Common keys
    bind!([], Key::Named(Named::ArrowDown), ItemDown);
    bind!([], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([], Key::Named(Named::ArrowRight), ItemRight);
    bind!([], Key::Named(Named::ArrowUp), ItemUp);
    bind!([], Key::Named(Named::F5), Reload);
    bind!([], Key::Named(Named::Home), SelectFirst);
    bind!([], Key::Named(Named::End), SelectLast);
    bind!([], Key::Named(Named::PageDown), ItemPageDown);
    bind!([], Key::Named(Named::PageUp), ItemPageUp);
    bind!([Shift], Key::Named(Named::ArrowDown), ItemDown);
    bind!([Shift], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([Shift], Key::Named(Named::ArrowRight), ItemRight);
    bind!([Shift], Key::Named(Named::ArrowUp), ItemUp);
    bind!([Shift], Key::Named(Named::Home), SelectFirst);
    bind!([Shift], Key::Named(Named::End), SelectLast);
    bind!([Shift], Key::Named(Named::PageDown), ItemPageDown);
    bind!([Shift], Key::Named(Named::PageUp), ItemPageUp);
    bind!([Ctrl, Shift], Key::Character("n".into()), NewFolder);
    bind!([], Key::Named(Named::Enter), Open);
    bind!([Ctrl], Key::Character(" ".into()), Preview);
    bind!([], Key::Character(" ".into()), Gallery);

    bind!([Ctrl], Key::Character("h".into()), ToggleShowHidden);
    bind!([Ctrl], Key::Character("a".into()), SelectAll);
    bind!([Ctrl], Key::Character("=".into()), ZoomIn);
    bind!([Ctrl], Key::Character("+".into()), ZoomIn);
    bind!([Ctrl], Key::Character("0".into()), ZoomDefault);
    bind!([Ctrl], Key::Character("-".into()), ZoomOut);
    // Switch view
    bind!([Ctrl], Key::Character("1".into()), TabViewList);
    bind!([Ctrl], Key::Character("2".into()), TabViewGrid);

    // App-only keys
    if matches!(mode, tab::Mode::App) {
        bind!([Ctrl], Key::Character("d".into()), AddToSidebar);
        bind!([Ctrl], Key::Named(Named::Enter), OpenInNewTab);
        bind!([Ctrl], Key::Character(",".into()), Settings);
        bind!([Ctrl], Key::Character("w".into()), TabClose);
        bind!([Ctrl], Key::Character("t".into()), TabNew);
        bind!([Ctrl], Key::Named(Named::Tab), TabNext);
        bind!([Ctrl, Shift], Key::Named(Named::Tab), TabPrev);
        bind!([Ctrl], Key::Character("q".into()), WindowClose);
        bind!([Ctrl], Key::Character("n".into()), WindowNew);
    }

    // App and desktop only keys
    if matches!(mode, tab::Mode::App | tab::Mode::Desktop) {
        bind!([Ctrl], Key::Character("c".into()), Copy);
        bind!([Ctrl, Shift], Key::Character("c".into()), CopyPath);
        bind!([Ctrl], Key::Character("x".into()), Cut);
        bind!([], Key::Named(Named::Delete), Delete);
        bind!([Shift], Key::Named(Named::Delete), PermanentlyDelete);
        bind!([Shift], Key::Named(Named::Enter), OpenInNewWindow);
        bind!([Ctrl], Key::Character("v".into()), Paste);
        bind!([], Key::Named(Named::F2), Rename);
    }

    // App and dialog only keys
    if matches!(mode, tab::Mode::App | tab::Mode::Dialog(_)) {
        bind!([Ctrl], Key::Character("l".into()), EditLocation);
        bind!([Alt], Key::Named(Named::ArrowRight), HistoryNext);
        bind!([Alt], Key::Named(Named::ArrowLeft), HistoryPrevious);
        bind!([], Key::Named(Named::Backspace), HistoryPrevious);
        bind!([Alt], Key::Named(Named::ArrowUp), LocationUp);
        bind!([Ctrl], Key::Character("f".into()), SearchActivate);
    }

    key_binds
}

/// Default key bindings with the user's configured overrides applied on top.
///
/// Every default stays in place unless the configuration names the same binding, so a partial or
/// malformed configuration can never leave the application without shortcuts.
pub fn key_binds_with_overrides(
    mode: &tab::Mode,
    overrides: &Shortcuts,
) -> HashMap<KeyBind, Action> {
    let mut key_binds = key_binds(mode);
    for (binding, action) in overrides.iter() {
        match action {
            KeyBindAction::Action(action) => {
                key_binds.insert(binding.key_bind().clone(), *action);
            }
            KeyBindAction::Disable => {
                key_binds.remove(binding.key_bind());
            }
        }
    }
    key_binds
}

/// Error returned when a key binding or an action cannot be parsed from its textual form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// A [`KeyBind`] that can be read from and written to the configuration as a string.
///
/// The textual form is a list of modifiers followed by a key, joined by `+`, for example
/// `Ctrl+Shift+n`, `Alt+ArrowUp` or `Ctrl+Space`. Parsing is case insensitive; printing is
/// canonical, so a parsed binding always prints back to the same string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Binding(KeyBind);

impl Binding {
    /// Wrap a [`KeyBind`], normalizing the modifier list so that equal bindings compare equal.
    pub fn new(key_bind: KeyBind) -> Self {
        let mut modifiers = key_bind.modifiers;
        modifiers.sort_unstable();
        modifiers.dedup();
        Self(KeyBind {
            modifiers,
            key: key_bind.key,
        })
    }

    pub fn key_bind(&self) -> &KeyBind {
        &self.0
    }

    pub fn into_key_bind(self) -> KeyBind {
        self.0
    }
}

impl From<KeyBind> for Binding {
    fn from(key_bind: KeyBind) -> Self {
        Self::new(key_bind)
    }
}

impl From<Binding> for KeyBind {
    fn from(binding: Binding) -> Self {
        binding.0
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.0.modifiers {
            write!(f, "{modifier:?}+")?;
        }
        match key_to_string(&self.0.key) {
            Some(key) => f.write_str(&key),
            // Keys without a textual form are not round-trippable; show something for logs.
            None => write!(f, "{:?}", self.0.key),
        }
    }
}

impl FromStr for Binding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Vec::new();
        let mut rest = s.trim();
        // Modifiers are consumed from the left so that a `+` key (as in `Ctrl++`) still parses.
        while let Some(index) = rest.find('+') {
            let Some(modifier) = modifier_from_str(rest[..index].trim()) else {
                break;
            };
            modifiers.push(modifier);
            rest = rest[index + 1..].trim_start();
        }
        if rest.is_empty() {
            return Err(ParseError(format!("no key in key binding {s:?}")));
        }
        let key = key_from_str(rest)
            .ok_or_else(|| ParseError(format!("unknown key {rest:?} in key binding {s:?}")))?;
        Ok(Self::new(KeyBind { modifiers, key }))
    }
}

impl Serialize for Binding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if key_to_string(&self.0.key).is_none() {
            return Err(S::Error::custom(format!(
                "key binding {self} has no textual form"
            )));
        }
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Binding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

/// What a configured key binding does: run an action, or suppress a default binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyBindAction {
    Action(Action),
    /// Remove the default binding for this key without replacing it.
    Disable,
}

/// The textual form of [`KeyBindAction::Disable`].
pub const DISABLE: &str = "Disable";

impl fmt::Display for KeyBindAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action(action) => f.write_str(&action.config_name()),
            Self::Disable => f.write_str(DISABLE),
        }
    }
}

impl FromStr for KeyBindAction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case(DISABLE) {
            return Ok(Self::Disable);
        }
        Action::from_config_name(s)
            .map(Self::Action)
            .ok_or_else(|| ParseError(format!("unknown action {s:?}")))
    }
}

impl Serialize for KeyBindAction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for KeyBindAction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

/// User configured key bindings, keyed by their textual form in the configuration file.
///
/// Deserialization is deliberately lenient: an entry that names an unknown action or an
/// unparseable binding is logged and skipped so that one typo cannot discard the rest.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Shortcuts(pub BTreeMap<Binding, KeyBindAction>);

impl Shortcuts {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Shortcuts {
    type Target = BTreeMap<Binding, KeyBindAction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<(Binding, KeyBindAction)> for Shortcuts {
    fn from_iter<T: IntoIterator<Item = (Binding, KeyBindAction)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Serialize for Shortcuts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = BTreeMap::new();
        for (binding, action) in &self.0 {
            if key_to_string(&binding.key_bind().key).is_none() {
                return Err(S::Error::custom(format!(
                    "key binding {binding} has no textual form"
                )));
            }
            map.insert(binding.to_string(), action.to_string());
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Shortcuts {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = BTreeMap::<String, String>::deserialize(deserializer)?;
        let mut shortcuts = BTreeMap::new();
        for (key, value) in raw {
            let binding = match Binding::from_str(&key) {
                Ok(binding) => binding,
                Err(err) => {
                    log::warn!("ignoring key binding {key:?}: {err}");
                    continue;
                }
            };
            let action = match KeyBindAction::from_str(&value) {
                Ok(action) => action,
                Err(err) => {
                    log::warn!("ignoring key binding {key:?}: {err}");
                    continue;
                }
            };
            shortcuts.insert(binding, action);
        }
        Ok(Self(shortcuts))
    }
}

fn modifier_from_str(s: &str) -> Option<Modifier> {
    // The names accepted here are the ones users are likely to reach for on any platform.
    Some(match s.to_ascii_lowercase().as_str() {
        "super" | "logo" | "meta" | "cmd" | "command" | "win" | "windows" => Modifier::Super,
        "ctrl" | "control" => Modifier::Ctrl,
        "alt" | "option" | "opt" => Modifier::Alt,
        "shift" => Modifier::Shift,
        _ => return None,
    })
}

/// Spelling used for the space bar, which is a character key rather than a named key.
const SPACE: &str = "Space";

fn key_to_string(key: &Key) -> Option<String> {
    match key {
        Key::Character(c) if c.as_str() == " " => Some(SPACE.to_string()),
        Key::Character(c) => Some(c.to_lowercase()),
        Key::Named(named) => named_key_name(*named).map(str::to_string),
        Key::Unidentified => None,
    }
}

fn key_from_str(s: &str) -> Option<Key> {
    if s.eq_ignore_ascii_case(SPACE) {
        return Some(Key::Character(" ".into()));
    }
    if let Some(named) = named_key_from_str(s) {
        return Some(Key::Named(named));
    }
    // Anything else is a character key. Casing is irrelevant: `KeyBind::matches` compares
    // character keys case insensitively.
    Some(Key::Character(s.to_lowercase().into()))
}

fn heading_to_name(heading: &HeadingOptions) -> &'static str {
    // `HeadingOptions: Display` is translated, so it cannot be used here.
    match heading {
        HeadingOptions::Name => "Name",
        HeadingOptions::Modified => "Modified",
        HeadingOptions::Size => "Size",
        HeadingOptions::TrashedOn => "TrashedOn",
    }
}

fn heading_from_name(name: &str) -> Option<HeadingOptions> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "name" => HeadingOptions::Name,
        "modified" => HeadingOptions::Modified,
        "size" => HeadingOptions::Size,
        "trashedon" => HeadingOptions::TrashedOn,
        _ => return None,
    })
}

/// Split `Name(arg, arg)` into its name and its arguments.
fn split_call(s: &str) -> Option<(&str, Vec<&str>)> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    if close < open {
        return None;
    }
    let name = s[..open].trim();
    let args = s[open + 1..close].trim();
    let args = if args.is_empty() {
        Vec::new()
    } else {
        args.split(',').map(str::trim).collect()
    };
    Some((name, args))
}

macro_rules! unit_actions {
    ($($name:ident),* $(,)?) => {
        /// Every [`Action`] that carries no payload, with its configuration name.
        const UNIT_ACTIONS: &[(&str, Action)] = &[$((stringify!($name), Action::$name)),*];

        /// The configuration name of a payload-free action.
        ///
        /// This match is exhaustive on purpose: a new [`Action`] variant fails to compile until it
        /// is given a name here.
        fn unit_action_name(action: &Action) -> Option<&'static str> {
            match action {
                $(Action::$name => Some(stringify!($name)),)*
                Action::RunContextAction(..) | Action::SetSort(..) | Action::ToggleSort(..) => None,
                #[cfg(feature = "desktop")]
                Action::ExecEntryAction(..) => None,
            }
        }
    };
}

unit_actions![
    About,
    AddToSidebar,
    Compress,
    Copy,
    CopyPath,
    CopyTo,
    Cut,
    CosmicSettingsDesktop,
    CosmicSettingsDisplays,
    CosmicSettingsWallpaper,
    DesktopViewOptions,
    Delete,
    EditHistory,
    EditLocation,
    Eject,
    EmptyTrash,
    ExtractHere,
    ExtractTo,
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
    Recents,
];

impl Action {
    /// Stable name of this action in the configuration file.
    pub fn config_name(&self) -> String {
        match self {
            Action::RunContextAction(index) => format!("RunContextAction({index})"),
            Action::SetSort(heading, ascending) => {
                format!("SetSort({}, {})", heading_to_name(heading), ascending)
            }
            Action::ToggleSort(heading) => format!("ToggleSort({})", heading_to_name(heading)),
            #[cfg(feature = "desktop")]
            Action::ExecEntryAction(index) => format!("ExecEntryAction({index})"),
            other => unit_action_name(other).unwrap_or_default().to_string(),
        }
    }

    /// Parse an action from its configuration name, or `None` if the name is not recognized.
    pub fn from_config_name(name: &str) -> Option<Self> {
        let name = name.trim();
        if let Some((call, args)) = split_call(name) {
            return match (call, args.as_slice()) {
                ("RunContextAction", [index]) => {
                    Some(Action::RunContextAction(index.parse().ok()?))
                }
                #[cfg(feature = "desktop")]
                ("ExecEntryAction", [index]) => Some(Action::ExecEntryAction(index.parse().ok()?)),
                ("SetSort", [heading, ascending]) => Some(Action::SetSort(
                    heading_from_name(heading)?,
                    ascending.parse().ok()?,
                )),
                ("ToggleSort", [heading]) => Some(Action::ToggleSort(heading_from_name(heading)?)),
                _ => None,
            };
        }
        UNIT_ACTIONS
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            .map(|(_, action)| *action)
    }
}

macro_rules! named_keys {
    ($($name:ident),* $(,)?) => {
        fn named_key_name(named: Named) -> Option<&'static str> {
            match named {
                $(Named::$name => Some(stringify!($name)),)*
            }
        }

        fn named_key_from_str(s: &str) -> Option<Named> {
            $(if s.eq_ignore_ascii_case(stringify!($name)) {
                return Some(Named::$name);
            })*
            None
        }

        #[cfg(test)]
        const ALL_NAMED_KEYS: &[Named] = &[$(Named::$name),*];
    };
}

named_keys![
    Alt,
    AltGraph,
    CapsLock,
    Control,
    Fn,
    FnLock,
    NumLock,
    ScrollLock,
    Shift,
    Symbol,
    SymbolLock,
    Meta,
    Hyper,
    Super,
    Enter,
    Tab,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    End,
    Home,
    PageDown,
    PageUp,
    Backspace,
    Clear,
    Copy,
    CrSel,
    Cut,
    Delete,
    EraseEof,
    ExSel,
    Insert,
    Paste,
    Redo,
    Undo,
    Accept,
    Again,
    Attn,
    Cancel,
    ContextMenu,
    Escape,
    Execute,
    Find,
    Help,
    Pause,
    Play,
    Props,
    Select,
    ZoomIn,
    ZoomOut,
    BrightnessDown,
    BrightnessUp,
    Eject,
    LogOff,
    Power,
    PowerOff,
    PrintScreen,
    Hibernate,
    Standby,
    WakeUp,
    AllCandidates,
    Alphanumeric,
    CodeInput,
    Compose,
    Convert,
    FinalMode,
    GroupFirst,
    GroupLast,
    GroupNext,
    GroupPrevious,
    ModeChange,
    NextCandidate,
    NonConvert,
    PreviousCandidate,
    Process,
    SingleCandidate,
    HangulMode,
    HanjaMode,
    JunjaMode,
    Eisu,
    Hankaku,
    Hiragana,
    HiraganaKatakana,
    KanaMode,
    KanjiMode,
    Katakana,
    Romaji,
    Zenkaku,
    ZenkakuHankaku,
    Soft1,
    Soft2,
    Soft3,
    Soft4,
    ChannelDown,
    ChannelUp,
    Close,
    MailForward,
    MailReply,
    MailSend,
    MediaClose,
    MediaFastForward,
    MediaPause,
    MediaPlay,
    MediaPlayPause,
    MediaRecord,
    MediaRewind,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    New,
    Open,
    Print,
    Save,
    SpellCheck,
    Key11,
    Key12,
    AudioBalanceLeft,
    AudioBalanceRight,
    AudioBassBoostDown,
    AudioBassBoostToggle,
    AudioBassBoostUp,
    AudioFaderFront,
    AudioFaderRear,
    AudioSurroundModeNext,
    AudioTrebleDown,
    AudioTrebleUp,
    AudioVolumeDown,
    AudioVolumeUp,
    AudioVolumeMute,
    MicrophoneToggle,
    MicrophoneVolumeDown,
    MicrophoneVolumeUp,
    MicrophoneVolumeMute,
    SpeechCorrectionList,
    SpeechInputToggle,
    LaunchApplication1,
    LaunchApplication2,
    LaunchCalendar,
    LaunchContacts,
    LaunchMail,
    LaunchMediaPlayer,
    LaunchMusicPlayer,
    LaunchPhone,
    LaunchScreenSaver,
    LaunchSpreadsheet,
    LaunchWebBrowser,
    LaunchWebCam,
    LaunchWordProcessor,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    AppSwitch,
    Call,
    Camera,
    CameraFocus,
    EndCall,
    GoBack,
    GoHome,
    HeadsetHook,
    LastNumberRedial,
    Notification,
    MannerMode,
    VoiceDial,
    TV,
    TV3DMode,
    TVAntennaCable,
    TVAudioDescription,
    TVAudioDescriptionMixDown,
    TVAudioDescriptionMixUp,
    TVContentsMenu,
    TVDataService,
    TVInput,
    TVInputComponent1,
    TVInputComponent2,
    TVInputComposite1,
    TVInputComposite2,
    TVInputHDMI1,
    TVInputHDMI2,
    TVInputHDMI3,
    TVInputHDMI4,
    TVInputVGA1,
    TVMediaContext,
    TVNetwork,
    TVNumberEntry,
    TVPower,
    TVRadioService,
    TVSatellite,
    TVSatelliteBS,
    TVSatelliteCS,
    TVSatelliteToggle,
    TVTerrestrialAnalog,
    TVTerrestrialDigital,
    TVTimer,
    AVRInput,
    AVRPower,
    ColorF0Red,
    ColorF1Green,
    ColorF2Yellow,
    ColorF3Blue,
    ColorF4Grey,
    ColorF5Brown,
    ClosedCaptionToggle,
    Dimmer,
    DisplaySwap,
    DVR,
    Exit,
    FavoriteClear0,
    FavoriteClear1,
    FavoriteClear2,
    FavoriteClear3,
    FavoriteRecall0,
    FavoriteRecall1,
    FavoriteRecall2,
    FavoriteRecall3,
    FavoriteStore0,
    FavoriteStore1,
    FavoriteStore2,
    FavoriteStore3,
    Guide,
    GuideNextDay,
    GuidePreviousDay,
    Info,
    InstantReplay,
    Link,
    ListProgram,
    LiveContent,
    Lock,
    MediaApps,
    MediaAudioTrack,
    MediaLast,
    MediaSkipBackward,
    MediaSkipForward,
    MediaStepBackward,
    MediaStepForward,
    MediaTopMenu,
    NavigateIn,
    NavigateNext,
    NavigateOut,
    NavigatePrevious,
    NextFavoriteChannel,
    NextUserProfile,
    OnDemand,
    Pairing,
    PinPDown,
    PinPMove,
    PinPToggle,
    PinPUp,
    PlaySpeedDown,
    PlaySpeedReset,
    PlaySpeedUp,
    RandomToggle,
    RcLowBattery,
    RecordSpeedNext,
    RfBypass,
    ScanChannelsToggle,
    ScreenModeNext,
    Settings,
    SplitScreenToggle,
    STBInput,
    STBPower,
    Subtitle,
    Teletext,
    VideoModeNext,
    Wink,
    ZoomToggle,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{Error as ValueError, MapDeserializer};

    fn parse(s: &str) -> Binding {
        Binding::from_str(s).expect("binding should parse")
    }

    fn shortcuts_from(entries: &[(&str, &str)]) -> Result<Shortcuts, ValueError> {
        Shortcuts::deserialize(MapDeserializer::new(
            entries
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        ))
    }

    #[test]
    fn every_action_name_round_trips() {
        for (name, action) in UNIT_ACTIONS {
            assert_eq!(&action.config_name(), name);
            assert_eq!(Action::from_config_name(name), Some(*action));
        }
        // 63 payload-free variants plus the four parameterized ones below.
        assert_eq!(UNIT_ACTIONS.len(), 63);
    }

    #[test]
    fn parameterized_action_names_round_trip() {
        // `mut` is only needed when the desktop feature adds another variant below.
        #[allow(unused_mut)]
        let mut actions = vec![
            Action::RunContextAction(0),
            Action::RunContextAction(7),
            Action::ToggleSort(HeadingOptions::Name),
            Action::ToggleSort(HeadingOptions::TrashedOn),
            Action::SetSort(HeadingOptions::Modified, true),
            Action::SetSort(HeadingOptions::Size, false),
        ];
        #[cfg(feature = "desktop")]
        actions.push(Action::ExecEntryAction(2));

        for action in actions {
            let name = action.config_name();
            assert_eq!(
                Action::from_config_name(&name),
                Some(action),
                "{name} did not round trip"
            );
        }
    }

    #[test]
    fn unknown_action_names_are_rejected() {
        for name in [
            "",
            "Nope",
            "SetSort(Nope, true)",
            "SetSort(Name)",
            "ToggleSort(Nope)",
            "RunContextAction(x)",
            "RunContextAction(",
        ] {
            assert_eq!(Action::from_config_name(name), None, "{name:?}");
        }
    }

    #[test]
    fn binding_strings_round_trip() {
        for s in [
            "ctrl+shift+n",
            "Ctrl+n",
            "ctrl++",
            "ctrl+-",
            "Ctrl + Shift + Tab",
            "alt+ArrowUp",
            "F5",
            "ctrl+Space",
            "super+shift+Delete",
            "Backspace",
        ] {
            let binding = parse(s);
            assert_eq!(
                parse(&binding.to_string()),
                binding,
                "{s} did not round trip"
            );
        }
    }

    #[test]
    fn binding_parsing_is_canonical() {
        assert_eq!(parse("shift+ctrl+n"), parse("Ctrl+Shift+N"));
        assert_eq!(parse("ctrl+ctrl+n"), parse("ctrl+n"));
        assert_eq!(parse("cmd+n"), parse("super+n"));
        assert_eq!(parse("ctrl+shift+n").to_string(), "Ctrl+Shift+n");
        assert_eq!(parse("ctrl+Space").to_string(), "Ctrl+Space");
        assert_eq!(
            parse("ctrl++").into_key_bind(),
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("+".into()),
            }
        );
    }

    #[test]
    fn malformed_bindings_are_rejected() {
        for s in ["", "   ", "ctrl+", "ctrl + shift + "] {
            assert!(Binding::from_str(s).is_err(), "{s:?} should not parse");
        }
    }

    #[test]
    fn every_named_key_round_trips() {
        for named in ALL_NAMED_KEYS {
            let key = Key::Named(*named);
            let printed = key_to_string(&key).expect("named key should print");
            assert_eq!(key_from_str(&printed), Some(key.clone()), "{printed}");
        }
    }

    #[test]
    fn default_bindings_round_trip() {
        for mode in [tab::Mode::App, tab::Mode::Desktop] {
            for key_bind in key_binds(&mode).into_keys() {
                let binding = Binding::new(key_bind.clone());
                assert_eq!(
                    parse(&binding.to_string()).into_key_bind(),
                    key_bind,
                    "{binding} did not round trip"
                );
            }
        }
    }

    #[test]
    fn empty_overrides_keep_the_defaults() {
        let defaults = key_binds(&tab::Mode::App);
        let merged = key_binds_with_overrides(&tab::Mode::App, &Shortcuts::new());
        assert_eq!(defaults, merged);
    }

    #[test]
    fn overrides_take_precedence_over_defaults() {
        let shortcuts = shortcuts_from(&[
            // Rebind a key that has a default.
            ("ctrl+t", "WindowNew"),
            // Add a key that has no default.
            ("ctrl+shift+p", "Preview"),
        ])
        .expect("shortcuts should deserialize");

        let merged = key_binds_with_overrides(&tab::Mode::App, &shortcuts);
        assert_eq!(
            merged.get(parse("ctrl+t").key_bind()),
            Some(&Action::WindowNew)
        );
        assert_eq!(
            merged.get(parse("ctrl+shift+p").key_bind()),
            Some(&Action::Preview)
        );
        // Untouched defaults survive.
        assert_eq!(
            merged.get(parse("ctrl+w").key_bind()),
            Some(&Action::TabClose)
        );
    }

    #[test]
    fn disable_removes_a_default_binding() {
        let shortcuts = shortcuts_from(&[("ctrl+w", "Disable")]).expect("should deserialize");
        let merged = key_binds_with_overrides(&tab::Mode::App, &shortcuts);
        assert_eq!(merged.get(parse("ctrl+w").key_bind()), None);
        assert!(!merged.is_empty());
    }

    #[test]
    fn disabling_a_binding_that_has_no_default_is_harmless() {
        let shortcuts = shortcuts_from(&[("ctrl+shift+j", "Disable")]).expect("should deserialize");
        let merged = key_binds_with_overrides(&tab::Mode::App, &shortcuts);
        assert_eq!(merged, key_binds(&tab::Mode::App));
    }

    #[test]
    fn mode_specific_defaults_are_still_mode_specific() {
        let shortcuts = shortcuts_from(&[("ctrl+t", "TabNew")]).expect("should deserialize");
        let desktop = key_binds_with_overrides(&tab::Mode::Desktop, &shortcuts);
        // The override applies, but app-only defaults do not leak into desktop mode.
        assert_eq!(
            desktop.get(parse("ctrl+t").key_bind()),
            Some(&Action::TabNew)
        );
        assert_eq!(desktop.get(parse("ctrl+w").key_bind()), None);
    }

    #[test]
    fn malformed_entries_are_skipped_without_dropping_the_rest() {
        let shortcuts = shortcuts_from(&[
            ("ctrl+shift+p", "Preview"),
            ("", "Copy"),
            ("ctrl+", "Copy"),
            ("ctrl+q", "NoSuchAction"),
            ("ctrl+e", "SetSort(Nope, true)"),
            ("ctrl+g", "Gallery"),
        ])
        .expect("malformed entries should not fail the whole map");

        assert_eq!(shortcuts.len(), 2);
        assert_eq!(
            shortcuts.get(&parse("ctrl+shift+p")),
            Some(&KeyBindAction::Action(Action::Preview))
        );
        assert_eq!(
            shortcuts.get(&parse("ctrl+g")),
            Some(&KeyBindAction::Action(Action::Gallery))
        );

        // The defaults for the skipped entries are untouched.
        let merged = key_binds_with_overrides(&tab::Mode::App, &shortcuts);
        assert_eq!(
            merged.get(parse("ctrl+q").key_bind()),
            Some(&Action::WindowClose)
        );
    }

    #[test]
    fn shortcuts_round_trip_through_serde() {
        let shortcuts: Shortcuts = [
            (
                parse("ctrl+shift+n"),
                KeyBindAction::Action(Action::NewFile),
            ),
            (
                parse("ctrl+e"),
                KeyBindAction::Action(Action::SetSort(HeadingOptions::Size, false)),
            ),
            (parse("ctrl+w"), KeyBindAction::Disable),
        ]
        .into_iter()
        .collect();

        let entries: BTreeMap<String, String> = shortcuts
            .iter()
            .map(|(binding, action)| (binding.to_string(), action.to_string()))
            .collect();
        let parsed: Shortcuts =
            Shortcuts::deserialize(MapDeserializer::<_, ValueError>::new(entries.into_iter()))
                .expect("should deserialize");
        assert_eq!(parsed, shortcuts);
    }
}
