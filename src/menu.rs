// SPDX-License-Identifier: GPL-3.0-only

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
    fl,
    key_bind::KeyBind,
    tab::{self, Location, Tab},
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

    let selected = tab
        .items_opt()
        .map_or(0, |items| items.iter().filter(|x| x.selected).count());

    let mut children: Vec<Element<_>> = Vec::new();
    match tab.location {
        Location::Path(_) => {
            if selected > 0 {
                children.push(menu_item(fl!("open"), Action::Open).into());
                //TODO: Open with
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
                children.push(horizontal_rule(1).into());
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                children.push(menu_item(fl!("paste"), Action::Paste).into());
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
    //TODO: port to libcosmic
    let menu_root = |label| {
        widget::button(widget::text(label))
            .padding([4, 12])
            .style(theme::Button::MenuRoot)
    };

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
        MenuTree::new(
            menu_button!(
                widget::text(label),
                widget::horizontal_space(Length::Fill),
                widget::text(key)
            )
            .on_press(action.message(None)),
        )
    };

    MenuBar::new(vec![
        MenuTree::with_children(
            menu_root(fl!("file")),
            vec![
                menu_item(fl!("new-tab"), Action::TabNew),
                menu_item(fl!("new-window"), Action::WindowNew),
                menu_item(fl!("new-file"), Action::NewFile),
                menu_item(fl!("new-folder"), Action::NewFolder),
                menu_item(fl!("open"), Action::Open),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("rename"), Action::Rename),
                //TOOD: add to sidebar, then divider
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("move-to-trash"), Action::MoveToTrash),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("close-tab"), Action::TabClose),
                menu_item(fl!("quit"), Action::WindowClose),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("edit")),
            vec![
                menu_item(fl!("cut"), Action::Cut),
                menu_item(fl!("copy"), Action::Copy),
                menu_item(fl!("paste"), Action::Paste),
                menu_item(fl!("select-all"), Action::SelectAll),
                MenuTree::new(horizontal_rule(1)),
                //TODO: edit history
                menu_item(fl!("operations"), Action::Operations),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("view")),
            vec![
                menu_item(fl!("grid-view"), Action::TabViewGrid),
                menu_item(fl!("list-view"), Action::TabViewList),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("menu-settings"), Action::Settings),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("menu-about"), Action::About),
            ],
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
