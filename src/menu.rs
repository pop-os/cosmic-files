// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Element,
    app::Core,
    iced::{
        Alignment, Background, Border, Length, advanced::widget::text::Style as TextStyle,
        keyboard::Modifiers,
    },
    theme,
    widget::{
        self, Row, button, column, container, divider,
        menu::{self, ItemHeight, ItemWidth, MenuBar, key_bind::KeyBind},
        responsive_menu_bar, space, text,
    },
};
use i18n_embed::LanguageLoader;
use rustc_hash::FxHashSet;
use std::{collections::HashMap, sync::LazyLock};

use crate::{
    app::{Action, Message},
    config::{Config, ContextActionPreset},
    fl,
    tab::{self, HeadingOptions, Location, LocationMenuAction, SearchLocation, Tab},
    trash::{Trash, TrashExt},
};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        button::custom(
            Row::with_children(
                [$(Element::from($x)),+]
            )
            .height(24.0)
            .align_y(Alignment::Center)
        )
        .padding([theme::active().cosmic().spacing.space_xxs, 16])
        .width(Length::Fill)
        .class(theme::Button::MenuItem)
    );
}

const fn menu_button_optional(
    label: String,
    action: Action,
    enabled: bool,
) -> menu::Item<Action, String> {
    if enabled {
        menu::Item::Button(label, None, action)
    } else {
        menu::Item::ButtonDisabled(label, None, action)
    }
}

pub fn context_menu(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    clipboard_paste_available: bool,
    context_actions: &[ContextActionPreset],
) -> Element<'static, tab::Message> {
    let find_key = |action: Action| -> String {
        key_binds
            .iter()
            .find(|&(_, &key_action)| action == key_action)
            .map_or_else(String::new, |(key_bind, _)| key_bind.to_string())
    };
    fn key_style(theme: &cosmic::Theme) -> TextStyle {
        let mut color = theme.cosmic().background.component.on;
        color.alpha *= 0.75;
        TextStyle {
            color: Some(color.into()),
        }
    }
    fn disabled_style(theme: &cosmic::Theme) -> TextStyle {
        let mut color = theme.cosmic().background.component.on;
        color.alpha *= 0.5;
        TextStyle {
            color: Some(color.into()),
        }
    }

    let menu_item = |label, action| {
        let key = find_key(action);
        menu_button!(
            text::body(label),
            space::horizontal(),
            text::body(key).class(theme::Text::Custom(key_style))
        )
        .on_press(tab::Message::ContextAction(action))
    };

    let menu_item_disabled = |label, action: Action| {
        let key = find_key(action);
        menu_button!(
            text::body(label).class(theme::Text::Custom(disabled_style)),
            space::horizontal(),
            text::body(key).class(theme::Text::Custom(disabled_style))
        )
    };

    // Allow paste when clipboard has data and we're in a location that supports it
    let can_paste = clipboard_paste_available && tab.location.supports_paste();

    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |mut label: String, variant| {
        label.push_str(match (sort_name == variant, sort_direction) {
            (true, true) => " \u{2B07}",
            (true, false) => " \u{2B06}",
            _ => " ",
        });
        menu_item(label, Action::ToggleSort(variant))
    };

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_trash_only = false;
    let mut selected_desktop_entry = None;
    let mut selected_types = FxHashSet::default();
    let mut selected_mount_point = 0;
    if let Some(items) = tab.items_opt() {
        for item in items.iter().filter(|&item| item.selected) {
            selected += 1;
            if item.metadata.is_dir() {
                selected_mount_point += i32::from(item.is_mount_point);
                selected_dir += 1;
            }
            match &item.location_opt {
                Some(Location::Trash | Location::Search(SearchLocation::Trash, ..)) => {
                    selected_trash_only = true;
                }
                Some(Location::Path(path)) => {
                    if selected == 1
                        && path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("desktop"))
                    {
                        selected_desktop_entry = Some(path);
                    }
                }
                _ => (),
            }
            selected_types.insert(&item.mime);
        }
    }
    selected_trash_only = selected_trash_only && selected == 1;
    let context_action_items = |selected: usize, selected_dir: usize| {
        context_actions
            .iter()
            .enumerate()
            .filter(|(_, action)| action.matches_selection(selected, selected_dir))
            .map(|(i, action)| menu_item(action.name.clone(), Action::RunContextAction(i)).into())
            .collect::<Vec<Element<'static, tab::Message>>>()
    };
    // Parse the desktop entry if it is the only selection
    #[cfg(feature = "desktop")]
    let selected_desktop_entry = selected_desktop_entry.and_then(|path| {
        if selected == 1 {
            let lang_id = crate::localize::LANGUAGE_LOADER.current_language();
            let language = lang_id.language.as_str();
            // Cache?
            cosmic::desktop::load_desktop_file(&[language.into()], path.into())
        } else {
            None
        }
    });

    let column = match (&tab.mode, &tab.location) {
        (
            tab::Mode::App | tab::Mode::Desktop,
            Location::Desktop(..)
            | Location::Path(..)
            | Location::Search(SearchLocation::Path(..) | SearchLocation::Recents, ..)
            | Location::Recents
            | Location::Network(_, _, Some(_)),
        ) => {
            if selected_trash_only {
                let mut column = column::with_capacity(2);
                column = column.push(menu_item(fl!("open"), Action::Open));
                if !Trash::is_empty() {
                    column = column.push(menu_item(fl!("empty-trash"), Action::EmptyTrash));
                }
                column
            } else if let Some(entry) = selected_desktop_entry {
                #[cfg(feature = "desktop")]
                let mut column = column::with_capacity(6 + entry.desktop_actions.len());
                #[cfg(not(feature = "desktop"))]
                let mut column = column::with_capacity(6);
                column = column.push(menu_item(fl!("open"), Action::Open));
                #[cfg(feature = "desktop")]
                {
                    column = column.extend(entry.desktop_actions.into_iter().enumerate().map(
                        |(i, action)| menu_item(action.name, Action::ExecEntryAction(i)).into(),
                    ));
                }
                column = column
                    .push(divider::horizontal::light())
                    .push(menu_item(fl!("rename"), Action::Rename))
                    .push(menu_item(fl!("cut"), Action::Cut))
                    .push(if modifiers.shift() && !modifiers.control() {
                        menu_item(fl!("copy-path"), Action::CopyPath)
                    } else {
                        menu_item(fl!("copy"), Action::Copy)
                    })
                    // Should this simply bypass trash and remove the shortcut?
                    .push(menu_item(fl!("move-to-trash"), Action::Delete));
                let action_items = context_action_items(selected, selected_dir);
                if !action_items.is_empty() {
                    column = column
                        .push(divider::horizontal::light())
                        .extend(action_items);
                }
                column
            } else if selected > 0 {
                let mut column = column::with_capacity(27);
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    column = column.push(menu_item(fl!("open"), Action::Open));
                }
                if selected == 1 {
                    column = column.push(menu_item(fl!("menu-open-with"), Action::OpenWith));
                    if selected_dir == 1 {
                        column =
                            column.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal));
                    }
                }
                if tab.location.is_recents() || matches!(tab.location, Location::Search(..)) {
                    column = column.push(menu_item(
                        fl!("open-item-location"),
                        Action::OpenItemLocation,
                    ));
                }
                // All selected items are directories
                if selected == selected_dir && matches!(tab.mode, tab::Mode::App) {
                    column = column
                        .push(menu_item(fl!("open-in-new-tab"), Action::OpenInNewTab))
                        .push(menu_item(
                            fl!("open-in-new-window"),
                            Action::OpenInNewWindow,
                        ));
                }
                let action_items = context_action_items(selected, selected_dir);
                if !action_items.is_empty() {
                    column = column
                        .push(divider::horizontal::light())
                        .extend(action_items);
                }
                column = column.push(divider::horizontal::light());
                if selected_mount_point == 0 {
                    column = column
                        .push(menu_item(fl!("rename"), Action::Rename))
                        .push(menu_item(fl!("cut"), Action::Cut));
                }
                column = column.push(if modifiers.shift() && !modifiers.control() {
                    menu_item(fl!("copy-path"), Action::CopyPath)
                } else {
                    menu_item(fl!("copy"), Action::Copy)
                });
                if selected_mount_point == 0 {
                    column = column.push(menu_item(fl!("move-to"), Action::MoveTo));
                }
                column = column.push(menu_item(fl!("copy-to"), Action::CopyTo));

                column = column.push(divider::horizontal::light());
                let supported_archive_types = crate::archive::SUPPORTED_ARCHIVE_TYPES;
                selected_types.retain(|&t| supported_archive_types.iter().all(|&m| *t != m));
                if selected_types.is_empty() {
                    column = column
                        .push(menu_item(fl!("extract-here"), Action::ExtractHere))
                        .push(menu_item(fl!("extract-to"), Action::ExtractTo));
                }
                column = column
                    .push(menu_item(fl!("compress"), Action::Compress))
                    .push(divider::horizontal::light())
                    //TODO: Print?
                    .push(menu_item(fl!("show-details"), Action::Preview));
                if matches!(tab.mode, tab::Mode::App) {
                    column = column
                        .push(divider::horizontal::light())
                        .push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar));
                }
                column = column.push(divider::horizontal::light());
                if tab.location.is_recents() {
                    column = column
                        .push(menu_item(
                            fl!("remove-from-recents"),
                            Action::RemoveFromRecents,
                        ))
                        .push(divider::horizontal::light());
                }
                if selected_mount_point == 0 {
                    if modifiers.shift() && !modifiers.control() {
                        column = column.push(menu_item(
                            fl!("delete-permanently"),
                            Action::PermanentlyDelete,
                        ));
                    } else {
                        column = column.push(menu_item(fl!("move-to-trash"), Action::Delete));
                    }
                } else if selected == 1 {
                    column = column.push(menu_item(fl!("eject"), Action::Eject));
                }
                column
            } else {
                let mut column = column::with_capacity(16);
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                if tab.location != Location::Recents {
                    column = column
                        .push(menu_item(fl!("new-folder"), Action::NewFolder))
                        .push(menu_item(fl!("new-file"), Action::NewFile))
                        .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal))
                        .push(divider::horizontal::light());
                }

                if tab.mode.multiple() {
                    column = column.push(menu_item(fl!("select-all"), Action::SelectAll));
                }
                column = column.push(if can_paste {
                    menu_item(fl!("paste"), Action::Paste)
                } else {
                    menu_item_disabled(fl!("paste"), Action::Paste)
                });

                //TODO: only show if cosmic-settings is found?
                if matches!(tab.mode, tab::Mode::Desktop) {
                    column = column
                        .push(divider::horizontal::light())
                        .push(menu_item(
                            fl!("change-wallpaper"),
                            Action::CosmicSettingsWallpaper,
                        ))
                        .push(menu_item(
                            fl!("desktop-appearance"),
                            Action::CosmicSettingsDesktop,
                        ))
                        .push(menu_item(
                            fl!("display-settings"),
                            Action::CosmicSettingsDisplays,
                        ));
                }

                column = column
                    .push(divider::horizontal::light())
                    // TODO: Nested menu
                    .push(sort_item(fl!("sort-by-name"), HeadingOptions::Name))
                    .push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified))
                    .push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
                if matches!(tab.location, Location::Desktop(..)) {
                    column = column.push(divider::horizontal::light()).push(menu_item(
                        fl!("desktop-view-options"),
                        Action::DesktopViewOptions,
                    ));
                }
                column
            }
        }
        (
            tab::Mode::Dialog(dialog_kind),
            Location::Desktop(..)
            | Location::Path(..)
            | Location::Search(SearchLocation::Path(..) | SearchLocation::Recents, ..)
            | Location::Recents
            | Location::Network(_, _, Some(_)),
        ) => {
            if selected > 0 {
                let mut column = column::with_capacity(4);
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    column = column.push(menu_item(fl!("open"), Action::Open));
                }
                if matches!(tab.location, Location::Search(..)) || tab.location.is_recents() {
                    column = column.push(menu_item(
                        fl!("open-item-location"),
                        Action::OpenItemLocation,
                    ));
                }
                column = column
                    .push(divider::horizontal::light())
                    .push(menu_item(fl!("show-details"), Action::Preview));
                column
            } else {
                let mut column = column::with_capacity(6);
                let mut column_has_children = false;
                if dialog_kind.save() {
                    column_has_children = true;
                    column = column.push(menu_item(fl!("new-folder"), Action::NewFolder));
                }
                if tab.mode.multiple() {
                    column_has_children = true;
                    column = column.push(menu_item(fl!("select-all"), Action::SelectAll));
                }
                if column_has_children {
                    column = column.push(divider::horizontal::light());
                }
                column = column
                    .push(sort_item(fl!("sort-by-name"), HeadingOptions::Name))
                    .push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified))
                    .push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
                column
            }
        }
        (_, Location::Network(..)) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    column::with_capacity(1).push(menu_item(fl!("open"), Action::Open))
                } else {
                    column::Column::new()
                }
            } else {
                let mut column = column::with_capacity(5);
                let mut column_has_children = false;
                if tab.mode.multiple() {
                    column_has_children = true;
                    column = column.push(menu_item(fl!("select-all"), Action::SelectAll));
                }
                if column_has_children {
                    column = column.push(divider::horizontal::light());
                }
                column = column
                    .push(sort_item(fl!("sort-by-name"), HeadingOptions::Name))
                    .push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified))
                    .push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
                column
            }
        }
        (_, Location::Trash | Location::Search(SearchLocation::Trash, ..)) => {
            let mut column = column::with_capacity(7);
            let mut column_has_children = false;
            if tab.mode.multiple() {
                column_has_children = true;
                column = column.push(menu_item(fl!("select-all"), Action::SelectAll));
            }
            if column_has_children {
                column = column.push(divider::horizontal::light());
            }
            if selected > 0 {
                column = column
                    .push(menu_item(fl!("show-details"), Action::Preview))
                    .push(divider::horizontal::light())
                    .push(menu_item(
                        fl!("restore-from-trash"),
                        Action::RestoreFromTrash,
                    ))
                    .push(divider::horizontal::light())
                    .push(menu_item(fl!("delete-permanently"), Action::Delete));
            } else {
                // TODO: Nested menu
                column = column
                    .push(sort_item(fl!("sort-by-name"), HeadingOptions::Name))
                    .push(sort_item(fl!("sort-by-trashed"), HeadingOptions::TrashedOn))
                    .push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
            column
        }
    };

    container(column)
        .padding(1)
        //TODO: move style to libcosmic
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(360.0)
        .into()
}

pub fn dialog_menu(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
    show_details: bool,
) -> Element<'static, Message> {
    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            None,
            sort_name == sort && sort_direction == dir,
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab.location.is_trash();

    let selected_gallery = tab.items_opt().map_or(0, |items| {
        items
            .iter()
            .filter(|&item| item.selected && item.can_gallery())
            .count()
    });

    MenuBar::new(vec![
        menu::Tree::with_children(
            Element::from(
                widget::button::icon(widget::icon::from_name(match tab.config.view {
                    tab::View::Grid => "view-grid-symbolic",
                    tab::View::List => "view-list-symbolic",
                }))
                // This prevents the button from being shown as insensitive
                .on_press(Message::None)
                .padding(8),
            ),
            menu::items(
                key_binds,
                vec![
                    menu::Item::CheckBox(
                        fl!("grid-view"),
                        None,
                        matches!(tab.config.view, tab::View::Grid),
                        Action::TabViewGrid,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-view"),
                        None,
                        matches!(tab.config.view, tab::View::List),
                        Action::TabViewList,
                    ),
                ],
            ),
        ),
        menu::Tree::with_children(
            Element::from(
                widget::button::icon(widget::icon::from_name(if sort_direction {
                    "view-sort-ascending-symbolic"
                } else {
                    "view-sort-descending-symbolic"
                }))
                // This prevents the button from being shown as insensitive
                .on_press(Message::None)
                .padding(8),
            ),
            menu::items(
                key_binds,
                vec![
                    sort_item(fl!("sort-a-z"), tab::HeadingOptions::Name, true),
                    sort_item(fl!("sort-z-a"), tab::HeadingOptions::Name, false),
                    sort_item(
                        fl!("sort-newest-first"),
                        if in_trash {
                            tab::HeadingOptions::TrashedOn
                        } else {
                            tab::HeadingOptions::Modified
                        },
                        false,
                    ),
                    sort_item(
                        fl!("sort-oldest-first"),
                        if in_trash {
                            tab::HeadingOptions::TrashedOn
                        } else {
                            tab::HeadingOptions::Modified
                        },
                        true,
                    ),
                    sort_item(
                        fl!("sort-smallest-to-largest"),
                        tab::HeadingOptions::Size,
                        true,
                    ),
                    sort_item(
                        fl!("sort-largest-to-smallest"),
                        tab::HeadingOptions::Size,
                        false,
                    ),
                    //TODO: sort by type
                ],
            ),
        ),
        menu::Tree::with_children(
            Element::from(
                widget::button::icon(widget::icon::from_name("view-more-symbolic"))
                    // This prevents the button from being shown as insensitive
                    .on_press(Message::None)
                    .padding(8),
            ),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), None, Action::ZoomIn),
                    menu::Item::Button(fl!("default-size"), None, Action::ZoomDefault),
                    menu::Item::Button(fl!("zoom-out"), None, Action::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("show-hidden-files"),
                        None,
                        tab.config.show_hidden,
                        Action::ToggleShowHidden,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-directories-first"),
                        None,
                        tab.config.folders_first,
                        Action::ToggleFoldersFirst,
                    ),
                    menu::Item::CheckBox(fl!("show-details"), None, show_details, Action::Preview),
                    menu::Item::Divider,
                    menu_button_optional(
                        fl!("gallery-preview"),
                        Action::Gallery,
                        selected_gallery > 0,
                    ),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(360))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .into()
}

pub fn menu_bar(
    core: &Core,
    tab_opt: Option<&Tab>,
    config: &Config,
    modifiers: Modifiers,
    key_binds: &HashMap<KeyBind, Action>,
    clipboard_paste_available: bool,
) -> Element<'static, Message> {
    let sort_options = tab_opt.map(Tab::sort_options);
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            None,
            sort_options.is_some_and(|(sort_name, sort_direction, _)| {
                sort_name == sort && sort_direction == dir
            }),
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab_opt.is_some_and(|tab| tab.location.is_trash());

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_gallery = 0;
    if let Some(items) = tab_opt.and_then(Tab::items_opt) {
        for item in items.iter().filter(|&item| item.selected) {
            selected += 1;
            if item.metadata.is_dir() {
                selected_dir += 1;
            }
            if item.can_gallery() {
                selected_gallery += 1;
            }
        }
    }

    // Allow paste when clipboard has data and we're in a location that supports it
    let can_paste =
        clipboard_paste_available && tab_opt.is_some_and(|tab| tab.location.supports_paste());

    let delete_item = if in_trash || modifiers.shift() {
        fl!("delete-permanently")
    } else {
        fl!("move-to-trash")
    };

    responsive_menu_bar()
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(360))
        .spacing(theme::active().cosmic().spacing.space_xxxs.into())
        .into_element(
            core,
            key_binds,
            MENU_ID.clone(),
            Message::Surface,
            vec![
                (
                    fl!("file"),
                    vec![
                        menu::Item::Button(fl!("new-tab"), None, Action::TabNew),
                        menu::Item::Button(fl!("new-window"), None, Action::WindowNew),
                        menu::Item::Button(fl!("new-folder"), None, Action::NewFolder),
                        menu::Item::Button(fl!("new-file"), None, Action::NewFile),
                        menu_button_optional(
                            fl!("open"),
                            Action::Open,
                            (selected > 0 && selected_dir == 0)
                                || (selected_dir == 1 && selected == 1),
                        ),
                        menu_button_optional(
                            fl!("menu-open-with"),
                            Action::OpenWith,
                            selected == 1,
                        ),
                        menu::Item::Divider,
                        menu_button_optional(fl!("rename"), Action::Rename, selected > 0),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("reload-folder"), None, Action::Reload),
                        menu::Item::Divider,
                        menu_button_optional(
                            fl!("add-to-sidebar"),
                            Action::AddToSidebar,
                            selected > 0,
                        ),
                        menu::Item::Divider,
                        menu_button_optional(
                            fl!("restore-from-trash"),
                            Action::RestoreFromTrash,
                            selected > 0 && in_trash,
                        ),
                        menu_button_optional(delete_item, Action::Delete, selected > 0),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("close-tab"), None, Action::TabClose),
                        menu::Item::Button(fl!("quit"), None, Action::WindowClose),
                    ],
                ),
                (
                    (fl!("edit")),
                    vec![
                        menu_button_optional(fl!("cut"), Action::Cut, selected > 0),
                        menu_button_optional(fl!("copy"), Action::Copy, selected > 0),
                        menu_button_optional(fl!("move-to"), Action::MoveTo, selected > 0),
                        menu_button_optional(fl!("copy-to"), Action::CopyTo, selected > 0),
                        menu_button_optional(fl!("paste"), Action::Paste, can_paste),
                        menu::Item::Button(fl!("select-all"), None, Action::SelectAll),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("history"), None, Action::EditHistory),
                    ],
                ),
                (
                    (fl!("view")),
                    vec![
                        menu::Item::Button(fl!("zoom-in"), None, Action::ZoomIn),
                        menu::Item::Button(fl!("default-size"), None, Action::ZoomDefault),
                        menu::Item::Button(fl!("zoom-out"), None, Action::ZoomOut),
                        menu::Item::Divider,
                        menu::Item::CheckBox(
                            fl!("grid-view"),
                            None,
                            tab_opt.is_some_and(|tab| matches!(tab.config.view, tab::View::Grid)),
                            Action::TabViewGrid,
                        ),
                        menu::Item::CheckBox(
                            fl!("list-view"),
                            None,
                            tab_opt.is_some_and(|tab| matches!(tab.config.view, tab::View::List)),
                            Action::TabViewList,
                        ),
                        menu::Item::Divider,
                        menu::Item::CheckBox(
                            fl!("show-hidden-files"),
                            None,
                            tab_opt.is_some_and(|tab| tab.config.show_hidden),
                            Action::ToggleShowHidden,
                        ),
                        menu::Item::CheckBox(
                            fl!("list-directories-first"),
                            None,
                            tab_opt.is_some_and(|tab| tab.config.folders_first),
                            Action::ToggleFoldersFirst,
                        ),
                        menu::Item::CheckBox(
                            fl!("show-details"),
                            None,
                            config.show_details,
                            Action::Preview,
                        ),
                        menu::Item::Divider,
                        menu_button_optional(
                            fl!("gallery-preview"),
                            Action::Gallery,
                            selected_gallery > 0,
                        ),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-settings"), None, Action::Settings),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-about"), None, Action::About),
                    ],
                ),
                (
                    (fl!("sort")),
                    vec![
                        sort_item(fl!("sort-a-z"), tab::HeadingOptions::Name, true),
                        sort_item(fl!("sort-z-a"), tab::HeadingOptions::Name, false),
                        sort_item(
                            fl!("sort-newest-first"),
                            if in_trash {
                                tab::HeadingOptions::TrashedOn
                            } else {
                                tab::HeadingOptions::Modified
                            },
                            false,
                        ),
                        sort_item(
                            fl!("sort-oldest-first"),
                            if in_trash {
                                tab::HeadingOptions::TrashedOn
                            } else {
                                tab::HeadingOptions::Modified
                            },
                            true,
                        ),
                        sort_item(
                            fl!("sort-smallest-to-largest"),
                            tab::HeadingOptions::Size,
                            true,
                        ),
                        sort_item(
                            fl!("sort-largest-to-smallest"),
                            tab::HeadingOptions::Size,
                            false,
                        ),
                        //TODO: sort by type
                    ],
                ),
            ],
        )
}

pub fn location_context_menu(ancestor_index: usize) -> Element<'static, tab::Message> {
    //TODO: only add some of these when in App mode
    let children = [
        menu_button!(text::body(fl!("open-in-new-tab")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::OpenInNewTab(ancestor_index),
            ))
            .into(),
        menu_button!(text::body(fl!("open-in-new-window")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::OpenInNewWindow(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("show-details")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::Preview(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("add-to-sidebar")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::AddToSidebar(ancestor_index),
            ))
            .into(),
    ];

    container(column::with_children(children))
        .padding(1)
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(360.0)
        .into()
}
