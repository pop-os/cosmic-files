use cosmic::widget::menu::key_bind::KeyBind;
use std::collections::HashMap;

use crate::app::Action;
use crate::shortcuts::ShortcutsConfig;
use crate::tab;

pub fn key_binds(shortcuts: &ShortcutsConfig, mode: &tab::Mode) -> HashMap<KeyBind, Action> {
    shortcuts.key_binds(mode)
}
