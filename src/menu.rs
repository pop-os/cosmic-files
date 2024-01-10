// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    //TODO: export iced::widget::horizontal_rule in cosmic::widget
    iced::{widget::horizontal_rule, Alignment, Background, Length},
    theme,
    widget::{
        self,
        menu::{ItemHeight, ItemWidth, MenuBar, MenuTree},
        segmented_button,
    },
    Element,
};

use crate::{fl, tab, Action, ContextPage, Location, Message, Tab};

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

pub fn context_menu<'a>(entity: segmented_button::Entity, tab: &Tab) -> Element<'a, Message> {
    let menu_action = |label, action| {
        menu_button!(widget::text(label)).on_press(Message::TabContextAction(entity, action))
    };

    let selected = tab
        .items_opt
        .as_ref()
        .map_or(0, |items| items.iter().filter(|x| x.selected).count());

    let mut children: Vec<Element<_>> = Vec::new();
    match tab.location {
        Location::Path(_) => {
            children.push(menu_action(fl!("new-file"), Action::NewFile).into());
            children.push(menu_action(fl!("new-folder"), Action::NewFolder).into());
            children.push(horizontal_rule(1).into());
            if selected > 0 {
                children.push(menu_action(fl!("cut"), Action::Cut).into());
                children.push(menu_action(fl!("copy"), Action::Copy).into());
                children.push(menu_action(fl!("paste"), Action::Paste).into());
            }
            children.push(menu_action(fl!("select-all"), Action::SelectAll).into());
            if selected > 0 {
                children.push(horizontal_rule(1).into());
                children.push(menu_action(fl!("move-to-trash"), Action::MoveToTrash).into());
            }
        }
        Location::Trash => {
            children.push(menu_action(fl!("select-all"), Action::SelectAll).into());
            if selected > 0 {
                children.push(horizontal_rule(1).into());
                children
                    .push(menu_action(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
            }
        }
    }
    children.push(horizontal_rule(1).into());
    children.push(menu_action(fl!("properties"), Action::Properties).into());

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
                border_radius: 8.0.into(),
                border_width: 1.0,
                border_color: component.divider.into(),
            }
        }))
        .width(Length::Fixed(240.0))
        .into()
}

pub fn menu_bar<'a>() -> Element<'a, Message> {
    //TODO: port to libcosmic
    let menu_root = |label| {
        widget::button(widget::text(label))
            .padding([4, 12])
            .style(theme::Button::MenuRoot)
    };

    let find_key = |message: &Message| -> String {
        //TODO: hotkey config
        String::new()
    };

    let menu_item = |label, message| {
        let key = find_key(&message);
        MenuTree::new(
            menu_button!(
                widget::text(label),
                widget::horizontal_space(Length::Fill),
                widget::text(key)
            )
            .on_press(message),
        )
    };

    MenuBar::new(vec![
        MenuTree::with_children(
            menu_root(fl!("file")),
            vec![
                menu_item(fl!("new-tab"), Message::TabNew),
                menu_item(fl!("new-window"), Message::WindowNew),
                menu_item(fl!("new-file"), Message::NewFile(None)),
                menu_item(fl!("new-folder"), Message::NewFolder(None)),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("close-tab"), Message::TabClose(None)),
                MenuTree::new(horizontal_rule(1)),
                menu_item(fl!("quit"), Message::WindowClose),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("edit")),
            vec![
                menu_item(fl!("cut"), Message::Cut(None)),
                menu_item(fl!("copy"), Message::Copy(None)),
                menu_item(fl!("paste"), Message::Paste(None)),
                menu_item(fl!("select-all"), Message::SelectAll(None)),
            ],
        ),
        MenuTree::with_children(
            menu_root(fl!("view")),
            vec![
                menu_item(
                    fl!("grid-view"),
                    Message::TabMessage(None, tab::Message::View(tab::View::Grid)),
                ),
                menu_item(
                    fl!("list-view"),
                    Message::TabMessage(None, tab::Message::View(tab::View::List)),
                ),
                MenuTree::new(horizontal_rule(1)),
                menu_item(
                    fl!("menu-settings"),
                    Message::ToggleContextPage(ContextPage::Settings),
                ),
            ],
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}
