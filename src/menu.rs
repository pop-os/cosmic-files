// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::menu_tree::{menu_items, menu_root, MenuItem};
use cosmic::{
    //TODO: export iced::widget::horizontal_rule in cosmic::widget
    iced::{widget::horizontal_rule, Alignment, Background, Border, Length},
    theme,
    widget::{
        self,
        menu::{ItemHeight, ItemWidth, MenuBar, MenuTree},
    },
    Element,
};
use std::collections::HashMap;

use crate::{
    app::{Action, Message},
    config::TabConfig,
    fl,
    tab::{self, HeadingOptions, Location, Tab},
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
        Location::Path(_) => {
            if selected > 0 {
                children.push(menu_item(fl!("open"), Action::Open).into());
                if selected == 1 {
                    children.push(menu_item(fl!("open-with"), Action::OpenWith).into());
                    if selected_dir == 1 {
                        children
                            .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                    }
                }
                children.push(horizontal_rule(1).into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());
                //TODO: Print?
                children.push(horizontal_rule(1).into());
                //TODO: change to Show details
                children.push(menu_item(fl!("properties"), Action::Properties).into());
                //TODO: Add to sidebar
                children.push(horizontal_rule(1).into());
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
            } else {
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                children.push(menu_item(fl!("new-file"), Action::NewFile).into());
                children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                children.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                children.push(horizontal_rule(1).into());
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                children.push(menu_item(fl!("paste"), Action::Paste).into());
                children.push(horizontal_rule(1).into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions::Size));
            }
        }
        Location::Trash => {
            children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
            if selected > 0 {
                children.push(horizontal_rule(1).into());
                children.push(menu_item(fl!("properties"), Action::Properties).into());
                children.push(horizontal_rule(1).into());
                children
                    .push(menu_item(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
            }
            children.push(horizontal_rule(1).into());
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
        .width(Length::Fixed(240.0))
        .into()
}

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    MenuBar::new(vec![
        MenuTree::with_children(
            menu_root(fl!("file")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("new-tab"), Action::TabNew),
                    MenuItem::Button(fl!("new-window"), Action::WindowNew),
                    MenuItem::Button(fl!("new-file"), Action::NewFile),
                    MenuItem::Button(fl!("new-folder"), Action::NewFolder),
                    MenuItem::Button(fl!("open"), Action::Open),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("rename"), Action::Rename),
                    //TOOD: add to sidebar, then divider
                    MenuItem::Divider,
                    MenuItem::Button(fl!("move-to-trash"), Action::MoveToTrash),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("close-tab"), Action::TabClose),
                    MenuItem::Button(fl!("quit"), Action::WindowClose),
                ],
            ),
        ),
        MenuTree::with_children(
            menu_root(fl!("edit")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("cut"), Action::Cut),
                    MenuItem::Button(fl!("copy"), Action::Copy),
                    MenuItem::Button(fl!("paste"), Action::Paste),
                    MenuItem::Button(fl!("select-all"), Action::SelectAll),
                    MenuItem::Divider,
                    //TODO: edit history
                    MenuItem::Button(fl!("operations"), Action::Operations),
                ],
            ),
        ),
        MenuTree::with_children(
            menu_root(fl!("view")),
            menu_items(
                key_binds,
                vec![
                    MenuItem::Button(fl!("grid-view"), Action::TabViewGrid),
                    MenuItem::Button(fl!("list-view"), Action::TabViewList),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("menu-settings"), Action::Settings),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("menu-about"), Action::About),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
