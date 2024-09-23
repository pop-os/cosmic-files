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
    config::TabConfig,
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
            .align_items(Alignment::Center)
        )
        .padding([theme::active().cosmic().spacing.space_xxxs, 16])
        .width(Length::Fill)
        .style(theme::Button::MenuItem)
    );
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
        menu_button!(
            text::body(label),
            horizontal_space(Length::Fill),
            text::body(key)
        )
        .on_press(tab::Message::ContextAction(action))
    };

    let TabConfig {
        sort_name,
        sort_direction,
        ..
    } = tab.config;
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
    let mut selected_types: Vec<Mime> = vec![];
    tab.items_opt().map(|items| {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                selected_types.push(item.mime.clone());
            }
        }
    });
    selected_types.sort_unstable();
    selected_types.dedup();

    let mut children: Vec<Element<_>> = Vec::new();
    match (&tab.mode, &tab.location) {
        (
            tab::Mode::App | tab::Mode::Desktop,
            Location::Path(_) | Location::Search(_, _) | Location::Recents,
        ) => {
            if selected > 0 {
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
                if matches!(tab.location, Location::Search(_, _)) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                // All selected items are directories
                if selected == selected_dir {
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
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar).into());
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
                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
        (
            tab::Mode::Dialog(dialog_kind),
            Location::Path(_) | Location::Search(_, _) | Location::Recents,
        ) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if matches!(tab.location, Location::Search(_, _)) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
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
        (_, Location::Network(_, _)) => {
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
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
    }

    container(column::with_children(children))
        .padding(1)
        //TODO: move style to libcosmic
        .style(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Appearance {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        }))
        .width(Length::Fixed(260.0))
        .into()
}

pub fn dialog_menu<'a>(
    tab: &Tab,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'static, Message> {
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            tab.config.sort_name == sort && tab.config.sort_direction == dir,
            Action::SetSort(sort, dir),
        )
    };

    MenuBar::new(vec![
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name(match tab.config.view {
                tab::View::Grid => "view-grid-symbolic",
                tab::View::List => "view-list-symbolic",
            }))
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
            widget::button::icon(widget::icon::from_name(if tab.config.sort_direction {
                "view-sort-ascending-symbolic"
            } else {
                "view-sort-descending-symbolic"
            }))
            .padding(8),
            menu::items(
                key_binds,
                vec![
                    sort_item(fl!("sort-a-z"), tab::HeadingOptions::Name, true),
                    sort_item(fl!("sort-z-a"), tab::HeadingOptions::Name, false),
                    sort_item(
                        fl!("sort-newest-first"),
                        tab::HeadingOptions::Modified,
                        false,
                    ),
                    sort_item(
                        fl!("sort-oldest-first"),
                        tab::HeadingOptions::Modified,
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

pub fn menu_bar<'a>(
    tab_opt: Option<&Tab>,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            tab_opt.map_or(false, |tab| {
                tab.config.sort_name == sort && tab.config.sort_direction == dir
            }),
            Action::SetSort(sort, dir),
        )
    };

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
                    menu::Item::Button(fl!("open"), Action::Open),
                    menu::Item::Button(fl!("open-with"), Action::OpenWith),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("rename"), Action::Rename),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-show-details"), Action::Preview),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("add-to-sidebar"), Action::AddToSidebar),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("move-to-trash"), Action::MoveToTrash),
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
                    menu::Item::Button(fl!("cut"), Action::Cut),
                    menu::Item::Button(fl!("copy"), Action::Copy),
                    menu::Item::Button(fl!("paste"), Action::Paste),
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
                        tab::HeadingOptions::Modified,
                        false,
                    ),
                    sort_item(
                        fl!("sort-oldest-first"),
                        tab::HeadingOptions::Modified,
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
    ];

    container(column::with_children(children))
        .padding(1)
        .style(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Appearance {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        }))
        .width(Length::Fixed(240.0))
        .into()
}
