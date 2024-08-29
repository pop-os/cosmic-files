use cosmic::{
    iced::keyboard::Key,
    iced_core::keyboard::key::Named,
    widget::menu::key_bind::{KeyBind, Modifier},
};
use std::collections::HashMap;

use crate::app::Action;

//TODO: load from config
pub fn key_binds() -> HashMap<KeyBind, Action> {
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

    bind!([Ctrl], Key::Character("d".into()), AddToSidebar);
    bind!([Ctrl], Key::Character("D".into()), AddToSidebar);
    bind!([Ctrl], Key::Character("c".into()), Copy);
    bind!([Ctrl], Key::Character("C".into()), Copy);
    bind!([Ctrl], Key::Character("x".into()), Cut);
    bind!([Ctrl], Key::Character("X".into()), Cut);
    bind!([Ctrl], Key::Character("l".into()), EditLocation);
    bind!([Ctrl], Key::Character("L".into()), EditLocation);
    bind!([Alt], Key::Named(Named::ArrowRight), HistoryNext);
    bind!([Alt], Key::Named(Named::ArrowLeft), HistoryPrevious);
    // Catch arrow keys
    bind!([], Key::Named(Named::ArrowDown), ItemDown);
    bind!([], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([], Key::Named(Named::ArrowRight), ItemRight);
    bind!([], Key::Named(Named::ArrowUp), ItemUp);
    // We also need to catch these when shift is held
    bind!([Shift], Key::Named(Named::ArrowDown), ItemDown);
    bind!([Shift], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([Shift], Key::Named(Named::ArrowRight), ItemRight);
    bind!([Shift], Key::Named(Named::ArrowUp), ItemUp);
    bind!([Alt], Key::Named(Named::ArrowUp), LocationUp);
    bind!([], Key::Named(Named::Delete), MoveToTrash);
    bind!([Ctrl, Shift], Key::Character("N".into()), NewFolder);
    bind!([Ctrl, Shift], Key::Character("n".into()), NewFolder);
    bind!([], Key::Named(Named::Enter), Open);
    bind!([Ctrl], Key::Named(Named::Enter), OpenInNewTab);
    bind!([Shift], Key::Named(Named::Enter), OpenInNewWindow);
    bind!([Ctrl], Key::Character("v".into()), Paste);
    bind!([Ctrl], Key::Character("V".into()), Paste);
    bind!([], Key::Named(Named::Space), Properties);
    bind!([], Key::Named(Named::F2), Rename);
    bind!([Ctrl], Key::Character("f".into()), SearchActivate);
    bind!([Ctrl], Key::Character("F".into()), SearchActivate);
    bind!([Ctrl], Key::Character("a".into()), SelectAll);
    bind!([Ctrl], Key::Character("A".into()), SelectAll);
    bind!([Ctrl], Key::Character(",".into()), Settings);
    bind!([Ctrl], Key::Character("w".into()), TabClose);
    bind!([Ctrl], Key::Character("W".into()), TabClose);
    bind!([Ctrl], Key::Character("t".into()), TabNew);
    bind!([Ctrl], Key::Character("T".into()), TabNew);
    bind!([Ctrl], Key::Named(Named::Tab), TabNext);
    bind!([Ctrl, Shift], Key::Named(Named::Tab), TabPrev);
    bind!([Ctrl], Key::Character("h".into()), ToggleShowHidden);
    bind!([Ctrl], Key::Character("H".into()), ToggleShowHidden);
    bind!([Ctrl], Key::Character("q".into()), WindowClose);
    bind!([Ctrl], Key::Character("Q".into()), WindowClose);
    bind!([Ctrl], Key::Character("n".into()), WindowNew);
    bind!([Ctrl], Key::Character("N".into()), WindowNew);
    bind!([Ctrl], Key::Character("=".into()), ZoomIn);
    bind!([Ctrl], Key::Character("+".into()), ZoomIn);
    bind!([Ctrl], Key::Character("0".into()), ZoomDefault);
    bind!([Ctrl], Key::Character("-".into()), ZoomOut);

    key_binds
}
