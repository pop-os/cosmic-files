// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{Alignment, Background, Border, Length},
    theme,
    widget::{
        self, button, column, container, divider, horizontal_space,
        menu::{self, key_bind::KeyBind, ItemHeight, ItemWidth, MenuBar},
        text, Row,
    },
    Element,
};
use mime_guess::Mime;
use std::collections::HashMap;

use crate::{
    app::{Action, Message},
    config::Config,
    fl,
    tab::{self, HeadingOptions, Location, LocationMenuAction, Tab},
};

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        button::custom(
            Row::with_children(
                vec![$(Element::from($x)),+]
            )
            .height(Length::Fixed(24.0))
            .align_y(Alignment::Center)
        )
        .padding([theme::active().cosmic().spacing.space_xxxs, 16])
        .width(Length::Fill)
        .class(theme::Button::MenuItem)
    );
}

fn menu_button_optional(
    label: String,
    action: Action,
    enabled: bool,
) -> menu::Item<Action, String> {
    if enabled {
        menu::Item::Button(label, action)
    } else {
        menu::Item::ButtonDisabled(label, action)
    }
}

pub fn context_menu<'a>(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, tab::Message> {
    let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds.iter() {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };

    let menu_item = |label, action| {
        let key = find_key(&action);
        menu_button!(text::body(label), horizontal_space(), text::body(key))
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
    let mut selected_types: Vec<Mime> = vec![];
    tab.items_opt().map(|items| {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                if item.location_opt == Some(Location::Trash) {
                    selected_trash_only = true;
                }
                selected_types.push(item.mime.clone());
            }
        }
    });
    selected_types.sort_unstable();
    selected_types.dedup();
    selected_trash_only = selected_trash_only && selected == 1;

    let mut children: Vec<Element<_>> = Vec::new();
    match (&tab.mode, &tab.location) {
        (
            tab::Mode::App | tab::Mode::Desktop,
            Location::Desktop(..) | Location::Path(..) | Location::Search(..) | Location::Recents,
        ) => {
            if selected_trash_only {
                children.push(menu_item(fl!("open"), Action::Open).into());
                if tab::trash_entries() > 0 {
                    children.push(menu_item(fl!("empty-trash"), Action::EmptyTrash).into());
                }
            } else if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if selected == 1 {
                    children.push(menu_item(fl!("open-with"), Action::OpenWith).into());
                    if selected_dir == 1 {
                        children
                            .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                    }
                }
                if matches!(tab.location, Location::Search(..)) {
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
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());

                children.push(divider::horizontal::light().into());
                let supported_archive_types = [
                    "application/gzip",
                    "application/x-compressed-tar",
                    "application/x-tar",
                    "application/zip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip-compressed-tar",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz-compressed-tar",
                ]
                .iter()
                .filter_map(|mime_type| mime_type.parse::<Mime>().ok())
                .collect::<Vec<_>>();
                selected_types.retain(|t| !supported_archive_types.contains(t));
                if selected_types.is_empty() {
                    children.push(menu_item(fl!("extract-here"), Action::ExtractHere).into());
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
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
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
                        menu_item(fl!("desktop-appearance"), Action::CosmicSettingsAppearance)
                            .into(),
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
            Location::Desktop(..) | Location::Path(..) | Location::Search(..) | Location::Recents,
        ) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if matches!(tab.location, Location::Search(..)) {
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
                    radius: cosmic.radius_s().into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(280.0))
        .into()
}

pub fn dialog_menu<'a>(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
    show_details: bool,
) -> Element<'static, Message> {
    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            sort_name == sort && sort_direction == dir,
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab.location == Location::Trash;

    let mut selected_gallery = 0;
    tab.items_opt().map(|items| {
        for item in items.iter() {
            if item.selected {
                if item.can_gallery() {
                    selected_gallery += 1;
                }
            }
        }
    });

    MenuBar::new(vec![
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name(match tab.config.view {
                tab::View::Grid => "view-grid-symbolic",
                tab::View::List => "view-list-symbolic",
            }))
            // This prevents the button from being shown as insensitive
            .on_press(Message::None)
            .padding(8),
            menu::items(
                key_binds,
                vec![
                    menu::Item::CheckBox(
                        fl!("grid-view"),
                        matches!(tab.config.view, tab::View::Grid),
                        Action::TabViewGrid,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-view"),
                        matches!(tab.config.view, tab::View::List),
                        Action::TabViewList,
                    ),
                ],
            ),
        ),
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name(if sort_direction {
                "view-sort-ascending-symbolic"
            } else {
                "view-sort-descending-symbolic"
            }))
            // This prevents the button from being shown as insensitive
            .on_press(Message::None)
            .padding(8),
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
            widget::button::icon(widget::icon::from_name("view-more-symbolic"))
                // This prevents the button from being shown as insensitive
                .on_press(Message::None)
                .padding(8),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), Action::ZoomIn),
                    menu::Item::Button(fl!("default-size"), Action::ZoomDefault),
                    menu::Item::Button(fl!("zoom-out"), Action::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("show-hidden-files"),
                        tab.config.show_hidden,
                        Action::ToggleShowHidden,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-directories-first"),
                        tab.config.folders_first,
                        Action::ToggleFoldersFirst,
                    ),
                    menu::Item::CheckBox(fl!("show-details"), show_details, Action::Preview),
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
    .item_width(ItemWidth::Uniform(240))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .into()
}

pub fn menu_bar<'a>(
    tab_opt: Option<&Tab>,
    config: &Config,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    let sort_options = tab_opt.map(|tab| tab.sort_options());
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            sort_options.map_or(false, |(sort_name, sort_direction, _)| {
                sort_name == sort && sort_direction == dir
            }),
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab_opt.map_or(false, |tab| tab.location == Location::Trash);

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_gallery = 0;
    tab_opt.and_then(|tab| tab.items_opt()).map(|items| {
        for item in items.iter() {
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
    });

    MenuBar::new(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("new-tab"), Action::TabNew),
                    menu::Item::Button(fl!("new-window"), Action::WindowNew),
                    menu::Item::Button(fl!("new-folder"), Action::NewFolder),
                    menu::Item::Button(fl!("new-file"), Action::NewFile),
                    menu_button_optional(
                        fl!("open"),
                        Action::Open,
                        (selected > 0 && selected_dir == 0) || (selected_dir == 1 && selected == 1),
                    ),
                    menu_button_optional(fl!("open-with"), Action::OpenWith, selected == 1),
                    menu::Item::Divider,
                    menu_button_optional(fl!("rename"), Action::Rename, selected > 0),
                    menu::Item::Divider,
                    menu_button_optional(fl!("add-to-sidebar"), Action::AddToSidebar, selected > 0),
                    menu::Item::Divider,
                    menu_button_optional(fl!("move-to-trash"), Action::MoveToTrash, selected > 0),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("close-tab"), Action::TabClose),
                    menu::Item::Button(fl!("quit"), Action::WindowClose),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("edit")),
            menu::items(
                key_binds,
                vec![
                    menu_button_optional(fl!("cut"), Action::Cut, selected > 0),
                    menu_button_optional(fl!("copy"), Action::Copy, selected > 0),
                    menu_button_optional(fl!("paste"), Action::Paste, selected > 0),
                    menu::Item::Button(fl!("select-all"), Action::SelectAll),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("history"), Action::EditHistory),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), Action::ZoomIn),
                    menu::Item::Button(fl!("default-size"), Action::ZoomDefault),
                    menu::Item::Button(fl!("zoom-out"), Action::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("grid-view"),
                        tab_opt.map_or(false, |tab| matches!(tab.config.view, tab::View::Grid)),
                        Action::TabViewGrid,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-view"),
                        tab_opt.map_or(false, |tab| matches!(tab.config.view, tab::View::List)),
                        Action::TabViewList,
                    ),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("show-hidden-files"),
                        tab_opt.map_or(false, |tab| tab.config.show_hidden),
                        Action::ToggleShowHidden,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-directories-first"),
                        tab_opt.map_or(false, |tab| tab.config.folders_first),
                        Action::ToggleFoldersFirst,
                    ),
                    menu::Item::CheckBox(fl!("show-details"), config.show_details, Action::Preview),
                    menu::Item::Divider,
                    menu_button_optional(
                        fl!("gallery-preview"),
                        Action::Gallery,
                        selected_gallery > 0,
                    ),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-settings"), Action::Settings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-about"), Action::About),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("sort")),
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
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .into()
}

pub fn location_context_menu<'a>(ancestor_index: usize) -> Element<'a, tab::Message> {
    //TODO: only add some of these when in App mode
    let children = vec![
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
                    radius: cosmic.radius_s().into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(240.0))
        .into()
}
