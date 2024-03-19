use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{iced::keyboard::Key, iced_core::keyboard::key::Named};
use std::collections::HashMap;

use crate::app::Action;
use cosmic::widget::menu::key_bind::Modifier;

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

    bind!([Ctrl], Key::Character("c".into()), Copy);
    bind!([Ctrl], Key::Character("x".into()), Cut);
    bind!([Ctrl], Key::Character("l".into()), EditLocation);
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
    bind!([], Key::Named(Named::Enter), Open);
    bind!([Ctrl], Key::Character("v".into()), Paste);
    bind!([], Key::Named(Named::F2), Rename);
    bind!([Ctrl], Key::Character("a".into()), SelectAll);
    bind!([Ctrl], Key::Character("w".into()), TabClose);
    bind!([Ctrl], Key::Character("t".into()), TabNew);
    bind!([Ctrl], Key::Named(Named::Tab), TabNext);
    bind!([Ctrl, Shift], Key::Named(Named::Tab), TabPrev);
    bind!([Ctrl], Key::Character("h".into()), ToggleShowHidden);
    bind!([Ctrl], Key::Character("q".into()), WindowClose);
    bind!([Ctrl], Key::Character("n".into()), WindowNew);

    key_binds
}
