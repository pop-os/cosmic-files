use cosmic::{
    app::Core,
    cosmic_theme,
    iced::{Alignment, Length, Point},
    theme, widget, Element,
};
use std::{
    cmp::Ordering,
    fs,
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

use crate::mime_icon::mime_icon;

const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Click(usize),
    Parent,
}

pub struct Item {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub icon: widget::icon::Icon,
    pub select_time: Option<Instant>,
}

pub struct Tab {
    pub path: PathBuf,
    //TODO
    pub context_menu: Option<Point>,
    pub items: Vec<Item>,
}

impl Tab {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        //TODO: store absolute path
        let mut tab = Self {
            path: path.into(),
            context_menu: None,
            items: Vec::new(),
        };
        tab.rescan();
        tab
    }

    pub fn rescan(&mut self) {
        self.items.clear();
        match fs::read_dir(&self.path) {
            Ok(entries) => {
                for entry_res in entries {
                    let entry = match entry_res {
                        Ok(ok) => ok,
                        Err(err) => {
                            log::warn!("failed to read entry in {:?}: {}", self.path, err);
                            continue;
                        }
                    };

                    let name = match entry.file_name().into_string() {
                        Ok(some) => some,
                        Err(name_os) => {
                            log::warn!(
                                "failed to parse entry in {:?}: {:?} is not valid UTF-8",
                                self.path,
                                name_os,
                            );
                            continue;
                        }
                    };

                    let path = entry.path();
                    let is_dir = path.is_dir();
                    //TODO: configurable size
                    let icon_size = 32;
                    let icon = if is_dir {
                        widget::icon::from_name("folder").size(icon_size).icon()
                    } else {
                        mime_icon(&path, icon_size)
                    };

                    self.items.push(Item {
                        name,
                        path,
                        is_dir,
                        icon,
                        select_time: None,
                    });
                }
            }
            Err(err) => {
                log::warn!("failed to read directory {:?}: {}", self.path, err);
            }
        }
        self.items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
    }

    pub fn title(&self) -> String {
        //TODO: better title
        format!("{}", self.path.display())
    }

    pub fn update(&mut self, message: Message) -> bool {
        let mut cd = None;
        match message {
            Message::Click(click_i) => {
                for (i, item) in self.items.iter_mut().enumerate() {
                    if i == click_i {
                        if let Some(select_time) = item.select_time {
                            if select_time.elapsed() < DOUBLE_CLICK_DURATION {
                                if item.is_dir {
                                    cd = Some(item.path.clone());
                                } else {
                                    //TODO: support multiple OS
                                    process::Command::new("xdg-open").arg(&item.path).spawn();
                                }
                            }
                        }
                        //TODO: prevent triple-click and beyond from opening file
                        item.select_time = Some(Instant::now());
                    } else {
                        item.select_time = None;
                    }
                }
            }
            Message::Parent => {
                if let Some(parent) = self.path.parent() {
                    cd = Some(parent.to_owned());
                }
            }
        }
        if let Some(path) = cd {
            self.path = path;
            self.rescan();
            true
        } else {
            false
        }
    }

    pub fn view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        let mut column = widget::column();
        for (i, item) in self.items.iter().enumerate() {
            if item.name.starts_with(".") {
                //TODO: SHOW HIDDEN OPTION
                continue;
            }

            column = column.push(
                widget::button(
                    widget::row::with_children(vec![
                        item.icon.clone().into(),
                        widget::text(item.name.clone()).into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs),
                )
                //TODO: improve style
                .style(if item.select_time.is_some() {
                    theme::Button::Standard
                } else {
                    theme::Button::AppletMenu
                })
                .width(Length::Fill)
                .on_press(Message::Click(i)),
            );
        }
        widget::scrollable(column.width(Length::Fill)).into()
    }
}
