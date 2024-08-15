// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    //TODO: export iced::widget::horizontal_rule in cosmic::widget
    iced::{widget::horizontal_rule, Alignment, Background, Border, Length},
    theme,
    widget,
    widget::container,
    widget::menu::{self, key_bind::KeyBind, ItemHeight, ItemWidth, MenuBar},
    Element,
};
use std::collections::HashMap;

use crate::{
    app::{Action, Message},
    config::TabConfig,
    fl,
    tab::{self, HeadingOptions, Location, LocationMenuAction, Tab},
};

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        widget::button(
            widget::Row::with_children(
                vec![$(Element::from($x)),+]
            )
            .align_items(Alignment::Center)
        )
        .height(Length::Fixed(32.0))
        .padding([4, 16])
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
            widget::text(label),
            widget::horizontal_space(Length::Fill),
            widget::text(key)
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
    tab.items_opt().map(|items| {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
            }
        }
    });

    let mut children: Vec<Element<_>> = Vec::new();
    match tab.location {
        Location::Path(_) | Location::Search(_, _) | Location::Bookmarks => {
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
                // All selected items are directories
                if selected == selected_dir {
                    children.push(menu_item(fl!("open-in-new-tab"), Action::OpenInNewTab).into());
                    children
                        .push(menu_item(fl!("open-in-new-window"), Action::OpenInNewWindow).into());
                }
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());
                //TODO: Print?
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("show-details"), Action::Properties).into());
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar).into());
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
                //TODO this should be add or remove if it already exist not only add
                children.push(menu_item(fl!("add_remove_bookmark"), Action::AddBookmark).into());
            } else {
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                children.push(menu_item(fl!("new-file"), Action::NewFile).into());
                children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                children.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                children.push(menu_item(fl!("paste"), Action::Paste).into());
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
        Location::Trash => {
            children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
            if selected > 0 {
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children.push(menu_item(fl!("show-details"), Action::Properties).into());
                children.push(container(horizontal_rule(1)).padding([0, 8]).into());
                children
                    .push(menu_item(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
            }
            children.push(container(horizontal_rule(1)).padding([0, 8]).into());
            // TODO: Nested menu
            children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
            children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
            children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
        }
    }

    widget::container(widget::column::with_children(children))
        .padding(1)
        //TODO: move style to libcosmic
        .style(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            widget::container::Appearance {
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

pub fn menu_bar<'a>(
    tab_opt: Option<&Tab>,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    MenuBar::new(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("new-tab"), Action::TabNew),
                    menu::Item::Button(fl!("new-window"), Action::WindowNew),
                    menu::Item::Button(fl!("new-file"), Action::NewFile),
                    menu::Item::Button(fl!("new-folder"), Action::NewFolder),
                    menu::Item::Button(fl!("open"), Action::Open),
                    menu::Item::Button(fl!("open-with"), Action::OpenWith),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("rename"), Action::Rename),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-show-details"), Action::Properties),
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
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}

pub fn location_context_menu<'a>(ancestor_index: usize) -> Element<'a, tab::Message> {
    let children = vec![
        menu_button!(widget::text(fl!("open-in-new-tab")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::OpenInNewTab(ancestor_index),
            ))
            .into(),
        menu_button!(widget::text(fl!("open-in-new-window")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::OpenInNewWindow(ancestor_index),
            ))
            .into(),
        container(horizontal_rule(1)).padding([0, 8]).into(),
        menu_button!(widget::text(fl!("show-details")))
            .on_press(tab::Message::LocationMenuAction(
                LocationMenuAction::Properties(ancestor_index),
            ))
            .into(),
    ];

    widget::container(widget::column::with_children(children))
        .padding(1)
        .style(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            widget::container::Appearance {
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
