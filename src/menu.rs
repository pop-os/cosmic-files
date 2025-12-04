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
        self, Row, button, column, container, divider, horizontal_space,
        menu::{self, ItemHeight, ItemWidth, MenuBar, key_bind::KeyBind},
        responsive_menu_bar, text,
    },
};
use i18n_embed::LanguageLoader;
use mime_guess::Mime;
use std::{collections::HashMap, sync::LazyLock};

use crate::{
    app::{Action, Message},
    config::Config,
    fl,
    tab::{self, HeadingOptions, Location, LocationMenuAction, Tab},
};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        button::custom(
            Row::with_children(
                [$(Element::from($x)),+]
            )
            .height(Length::Fixed(24.0))
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

pub fn context_menu<'a>(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
    modifiers: &Modifiers,
) -> Element<'a, tab::Message> {
    let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };
    fn key_style(theme: &cosmic::Theme) -> TextStyle {
        let mut color = theme.cosmic().background.component.on;
        color.alpha *= 0.75;
        TextStyle {
            color: Some(color.into()),
        }
    }

    let menu_item = |label, action| {
        let key = find_key(&action);
        menu_button!(
            text::body(label),
            horizontal_space(),
            text::body(key).class(theme::Text::Custom(key_style))
        )
        .on_press(tab::Message::ContextAction(action))
    };

    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, variant| {
        menu_item(
            format!(
                "{} {}",
                label,
                match (sort_name == variant, sort_direction) {
                    (true, true) => "\u{2B07}",
                    (true, false) => "\u{2B06}",
                    _ => "",
                }
            ),
            Action::ToggleSort(variant),
        )
        .into()
    };

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_trash_only = false;
    let mut selected_desktop_entry = None;
    let mut selected_types: Vec<Mime> = vec![];
    let mut selected_mount_point = 0;
    if let Some(items) = tab.items_opt() {
        for item in items {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_mount_point += i32::from(item.is_mount_point);
                    selected_dir += 1;
                }
                match &item.location_opt {
                    Some(Location::Trash) => selected_trash_only = true,
                    Some(Location::Path(path)) => {
                        if selected == 1
                            && path.extension().and_then(|s| s.to_str()) == Some("desktop")
                        {
                            selected_desktop_entry = Some(&**path);
                        }
                    }
                    _ => (),
                }
                selected_types.push(item.mime.clone());
            }
        }
    }
    selected_types.sort_unstable();
    selected_types.dedup();
    selected_trash_only = selected_trash_only && selected == 1;
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

    let mut children: Vec<Element<_>> = Vec::new();
    match (&tab.mode, &tab.location) {
        (
            tab::Mode::App | tab::Mode::Desktop,
            Location::Desktop(..)
            | Location::Path(..)
            | Location::Search(..)
            | Location::Recents
            | Location::Network(_, _, Some(_)),
        ) => {
            if selected_trash_only {
                children.push(menu_item(fl!("open"), Action::Open).into());
                if !trash::os_limited::is_empty().unwrap_or(true) {
                    children.push(menu_item(fl!("empty-trash"), Action::EmptyTrash).into());
                }
            } else if let Some(entry) = selected_desktop_entry {
                children.push(menu_item(fl!("open"), Action::Open).into());
                #[cfg(feature = "desktop")]
                {
                    children.extend(entry.desktop_actions.into_iter().enumerate().map(
                        |(i, action)| menu_item(action.name, Action::ExecEntryAction(i)).into(),
                    ));
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                if modifiers.shift() && !modifiers.control() {
                    children.push(menu_item(fl!("copy-path"), Action::CopyPath).into());
                } else {
                    children.push(menu_item(fl!("copy"), Action::Copy).into());
                }
                // Should this simply bypass trash and remove the shortcut?
                children.push(menu_item(fl!("move-to-trash"), Action::Delete).into());
            } else if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if selected == 1 {
                    children.push(menu_item(fl!("menu-open-with"), Action::OpenWith).into());
                    if selected_dir == 1 {
                        children
                            .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                    }
                }
                if matches!(tab.location, Location::Search(..) | Location::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                // All selected items are directories
                if selected == selected_dir && matches!(tab.mode, tab::Mode::App) {
                    children.push(menu_item(fl!("open-in-new-tab"), Action::OpenInNewTab).into());
                    children
                        .push(menu_item(fl!("open-in-new-window"), Action::OpenInNewWindow).into());
                }
                children.push(divider::horizontal::light().into());
                if selected_mount_point == 0 {
                    children.push(menu_item(fl!("rename"), Action::Rename).into());
                    children.push(menu_item(fl!("cut"), Action::Cut).into());
                }
                if modifiers.shift() && !modifiers.control() {
                    children.push(menu_item(fl!("copy-path"), Action::CopyPath).into());
                } else {
                    children.push(menu_item(fl!("copy"), Action::Copy).into());
                }

                children.push(divider::horizontal::light().into());
                let supported_archive_types = crate::archive::SUPPORTED_ARCHIVE_TYPES;
                selected_types.retain(|t| supported_archive_types.iter().copied().all(|m| *t != m));
                if selected_types.is_empty() {
                    children.push(menu_item(fl!("extract-here"), Action::ExtractHere).into());
                    children.push(menu_item(fl!("extract-to"), Action::ExtractTo).into());
                }
                children.push(menu_item(fl!("compress"), Action::Compress).into());
                children.push(divider::horizontal::light().into());

                //TODO: Print?
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                if matches!(tab.mode, tab::Mode::App) {
                    children.push(divider::horizontal::light().into());
                    children.push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar).into());
                }
                children.push(divider::horizontal::light().into());
                if matches!(tab.location, Location::Recents) {
                    children.push(
                        menu_item(fl!("remove-from-recents"), Action::RemoveFromRecents).into(),
                    );
                    children.push(divider::horizontal::light().into());
                }
                if selected_mount_point == 0 {
                    if modifiers.shift() && !modifiers.control() {
                        children.push(
                            menu_item(fl!("delete-permanently"), Action::PermanentlyDelete).into(),
                        );
                    } else {
                        children.push(menu_item(fl!("move-to-trash"), Action::Delete).into());
                    }
                } else if selected == 1 {
                    children.push(menu_item(fl!("eject"), Action::Eject).into());
                }
            } else {
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                children.push(menu_item(fl!("new-file"), Action::NewFile).into());
                children.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                children.push(divider::horizontal::light().into());
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                children.push(menu_item(fl!("paste"), Action::Paste).into());

                //TODO: only show if cosmic-settings is found?
                if matches!(tab.mode, tab::Mode::Desktop) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("change-wallpaper"), Action::CosmicSettingsWallpaper).into(),
                    );
                    children.push(
                        menu_item(fl!("desktop-appearance"), Action::CosmicSettingsDesktop).into(),
                    );
                    children.push(
                        menu_item(fl!("display-settings"), Action::CosmicSettingsDisplays).into(),
                    );
                }

                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
                if matches!(tab.location, Location::Desktop(..)) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("desktop-view-options"), Action::DesktopViewOptions).into(),
                    );
                }
            }
        }
        (
            tab::Mode::Dialog(dialog_kind),
            Location::Desktop(..)
            | Location::Path(..)
            | Location::Search(..)
            | Location::Recents
            | Location::Network(_, _, Some(_)),
        ) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if matches!(tab.location, Location::Search(..) | Location::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
            } else {
                if dialog_kind.save() {
                    children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                }
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
        (_, Location::Network(..)) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
            } else {
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
        (_, Location::Trash) => {
            if tab.mode.multiple() {
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
            }
            if !children.is_empty() {
                children.push(divider::horizontal::light().into());
            }
            if selected > 0 {
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                children.push(divider::horizontal::light().into());
                children
                    .push(menu_item(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("delete-permanently"), Action::Delete).into());
            } else {
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-trashed"), HeadingOptions::TrashedOn));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
    }

    container(column::with_children(children))
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
        .width(Length::Fixed(360.0))
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
    let in_trash = tab.location == Location::Trash;

    let mut selected_gallery = 0;
    if let Some(items) = tab.items_opt() {
        for item in items {
            if item.selected && item.can_gallery() {
                selected_gallery += 1;
            }
        }
    }

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

pub fn menu_bar<'a>(
    core: &Core,
    tab_opt: Option<&Tab>,
    config: &Config,
    modifiers: &Modifiers,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
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
    let in_trash = tab_opt.is_some_and(|tab| tab.location == Location::Trash);

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_gallery = 0;
    if let Some(items) = tab_opt.and_then(|tab| tab.items_opt()) {
        for item in items {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                if item.can_gallery() {
                    selected_gallery += 1;
                }
            }
        }
    }

    let (delete_item, delete_item_action) = if in_trash || modifiers.shift() {
        (fl!("delete-permanently"), Action::Delete)
    } else {
        (fl!("move-to-trash"), Action::Delete)
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
                        menu_button_optional(delete_item, delete_item_action, selected > 0),
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
                        menu_button_optional(fl!("paste"), Action::Paste, selected > 0),
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

pub fn location_context_menu<'a>(ancestor_index: usize) -> Element<'a, tab::Message> {
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
        .width(Length::Fixed(360.0))
        .into()
}
