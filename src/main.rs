// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{message, Command, Core, Settings},
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme, executor,
    iced::{subscription::Subscription, widget::row, window, Alignment, Length, Point},
    style,
    widget::{self, segmented_button},
    Application, ApplicationExt, Element,
};
use std::{any::TypeId, env, path::PathBuf, process, time::Instant};

use config::{AppTheme, Config, CONFIG_VERSION};
mod config;

mod menu;

mod mouse_area;

mod localize;

mod mime_icon;

use tab::Tab;
mod tab;

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(unix, not(target_os = "redox")))]
    match fork::daemon(true, true) {
        Ok(fork::Fork::Child) => (),
        Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
        Err(err) => {
            eprintln!("failed to daemonize: {:?}", err);
            process::exit(1);
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let (config_handler, config) = match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    log::info!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            log::error!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());

    #[cfg(target_os = "redox")]
    {
        // Redox does not support resize if doing CSDs
        settings = settings.client_decorations(false);
    }

    //TODO: allow size limits on iced_winit
    //settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(200.0));

    let flags = Flags {
        config_handler,
        config,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

fn home_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home,
        None => {
            log::warn!("failed to locate home directory");
            PathBuf::from("/")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Flags {
    config_handler: Option<cosmic_config::Config>,
    config: Config,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Copy,
    NewFile,
    NewFolder,
    Paste,
    SelectAll,
    Settings,
    TabNew,
}

impl Action {
    pub fn message(self, entity: segmented_button::Entity) -> Message {
        match self {
            Action::Copy => Message::Copy(Some(entity)),
            Action::NewFile => Message::NewFile(Some(entity)),
            Action::NewFolder => Message::NewFolder(Some(entity)),
            Action::Paste => Message::Paste(Some(entity)),
            Action::SelectAll => Message::SelectAll(Some(entity)),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::TabNew => Message::TabNew,
        }
    }
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    Todo,
    AppTheme(AppTheme),
    Config(Config),
    Copy(Option<segmented_button::Entity>),
    NewFile(Option<segmented_button::Entity>),
    NewFolder(Option<segmented_button::Entity>),
    Paste(Option<segmented_button::Entity>),
    SelectAll(Option<segmented_button::Entity>),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    TabActivate(segmented_button::Entity),
    TabClose(segmented_button::Entity),
    TabContextAction(segmented_button::Entity, Action),
    TabContextMenu(segmented_button::Entity, Option<Point>),
    TabMessage(segmented_button::Entity, tab::Message),
    TabNew,
    TabRescan(segmented_button::Entity, Vec<tab::Item>),
    ToggleContextPage(ContextPage),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::Settings => fl!("settings"),
        }
    }
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    tab_model: segmented_button::Model<segmented_button::SingleSelect>,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
}

impl App {
    fn open_tab<P: Into<PathBuf>>(&mut self, path: P) -> Command<Message> {
        let path = path.into();
        let tab = Tab::new(path.clone());
        let entity = self
            .tab_model
            .insert()
            .text(tab.title())
            .data(tab)
            .closable()
            .activate()
            .id();
        Command::batch([self.update_title(), self.rescan_tab(entity, path)])
    }

    fn rescan_tab<P: Into<PathBuf>>(
        &mut self,
        entity: segmented_button::Entity,
        path: P,
    ) -> Command<Message> {
        let path = path.into();
        Command::perform(
            async move {
                match tokio::task::spawn_blocking(move || tab::rescan(path)).await {
                    Ok(items) => message::app(Message::TabRescan(entity, items)),
                    Err(err) => {
                        log::warn!("failed to rescan: {}", err);
                        message::none()
                    }
                }
            },
            |x| x,
        )
    }

    fn update_config(&mut self) -> Command<Message> {
        cosmic::app::command::set_theme(self.config.app_theme.theme())
    }

    fn save_config(&mut self) -> Command<Message> {
        match self.config_handler {
            Some(ref config_handler) => match self.config.write_entry(&config_handler) {
                Ok(()) => {}
                Err(err) => {
                    log::error!("failed to save config: {}", err);
                }
            },
            None => {}
        }
        self.update_config()
    }

    fn update_title(&mut self) -> Command<Message> {
        let (header_title, window_title) = match self.tab_model.text(self.tab_model.active()) {
            Some(tab_title) => (
                tab_title.to_string(),
                format!("{tab_title} — COSMIC File Manager"),
            ),
            None => (String::new(), "COSMIC File Manager".to_string()),
        };
        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::view_section(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    move |index| {
                        Message::AppTheme(match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        })
                    },
                )),
            )
            .into()])
        .into()
    }
}

/// Implement [`Application`] to integrate with COSMIC.
impl Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received
    type Flags = Flags;

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "com.system76.CosmicFiles";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        let mut app = App {
            core,
            tab_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes,
            context_page: ContextPage::Settings,
        };

        let mut commands = Vec::new();

        for arg in env::args().skip(1) {
            commands.push(app.open_tab(arg));
        }

        if app.tab_model.iter().next().is_none() {
            commands.push(app.open_tab(home_dir()));
        }

        (app, Command::batch(commands))
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Todo => {
                log::warn!("TODO");
            }
            Message::AppTheme(app_theme) => {
                self.config.app_theme = app_theme;
                return self.save_config();
            }
            Message::Config(config) => {
                if config != self.config {
                    log::info!("update config");
                    //TODO: update syntax theme by clearing tabs, only if needed
                    self.config = config;
                    return self.update_config();
                }
            }
            Message::Copy(entity_opt) => {
                log::warn!("TODO: COPY");
            }
            Message::NewFile(entity_opt) => {
                log::warn!("TODO: NEW FILE");
            }
            Message::NewFolder(entity_opt) => {
                log::warn!("TODO: NEW FOLDER");
            }
            Message::Paste(entity_opt) => {
                log::warn!("TODO: PASTE");
            }
            Message::SelectAll(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        let select_time = Instant::now();
                        for item in items.iter_mut() {
                            item.select_time = Some(select_time);
                        }
                    }
                }
            }
            Message::SystemThemeModeChange(_theme_mode) => {
                return self.update_config();
            }
            Message::TabActivate(entity) => {
                self.tab_model.activate(entity);
                return self.update_title();
            }
            Message::TabClose(entity) => {
                // Activate closest item
                if let Some(position) = self.tab_model.position(entity) {
                    if position > 0 {
                        self.tab_model.activate_position(position - 1);
                    } else {
                        self.tab_model.activate_position(position + 1);
                    }
                }

                // Remove item
                self.tab_model.remove(entity);

                // If that was the last tab, close window
                if self.tab_model.iter().next().is_none() {
                    return window::close(window::Id::MAIN);
                }

                return self.update_title();
            }
            Message::TabContextAction(entity, action) => {
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        // Close context menu
                        {
                            tab.context_menu = None;
                        }
                        // Run action's message
                        return self.update(action.message(entity));
                    }
                    _ => {}
                }
            }
            Message::TabContextMenu(entity, position_opt) => {
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        // Update context menu position
                        tab.context_menu = position_opt;
                    }
                    _ => {}
                }
            }
            Message::TabMessage(entity, tab_message) => {
                let mut update_opt = None;
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        if tab.update(tab_message) {
                            update_opt = Some((tab.title(), tab.path.clone()));
                        }
                    }
                    _ => (),
                }
                if let Some((tab_title, tab_path)) = update_opt {
                    self.tab_model.text_set(entity, tab_title);
                    return Command::batch([
                        self.update_title(),
                        self.rescan_tab(entity, tab_path),
                    ]);
                }
            }
            Message::TabNew => {
                let active = self.tab_model.active();
                let path = match self.tab_model.data::<Tab>(active) {
                    Some(tab) => tab.path.clone(),
                    None => home_dir(),
                };
                return self.open_tab(path);
            }
            Message::TabRescan(entity, items) => match self.tab_model.data_mut::<Tab>(entity) {
                Some(tab) => {
                    tab.items_opt = Some(items);
                }
                _ => (),
            },
            //TODO: TABRELOAD
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.title());
            }
        }

        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::Settings => self.settings(),
        })
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        let active = self.tab_model.active();
        vec![row![
            widget::button(widget::icon::from_name("list-add-symbolic").size(16).icon())
                .on_press(Message::TabNew)
                .padding(space_xxs)
                .style(style::Button::Icon),
            widget::button(widget::icon::from_name("go-home-symbolic").size(16).icon())
                .on_press(Message::TabMessage(active, tab::Message::Home))
                .padding(space_xxs)
                .style(style::Button::Icon),
            widget::button(widget::icon::from_name("go-up-symbolic").size(16).icon())
                .on_press(Message::TabMessage(active, tab::Message::Parent))
                .padding(space_xxs)
                .style(style::Button::Icon)
        ]
        .align_items(Alignment::Center)
        .into()]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        vec![row![widget::button(
            widget::icon::from_name("preferences-system-symbolic")
                .size(16)
                .icon()
        )
        .on_press(Message::ToggleContextPage(ContextPage::Settings))
        .padding(space_xxs)
        .style(style::Button::Icon)]
        .align_items(Alignment::Center)
        .into()]
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(1);

        if self.tab_model.iter().count() > 1 {
            tab_column = tab_column.push(
                widget::container(
                    widget::view_switcher::horizontal(&self.tab_model)
                        .button_height(32)
                        .button_spacing(space_xxs)
                        .on_activate(Message::TabActivate)
                        .on_close(Message::TabClose),
                )
                .style(style::Container::Background)
                .width(Length::Fill),
            );
        }

        let entity = self.tab_model.active();
        match self.tab_model.data::<Tab>(entity) {
            Some(tab) => {
                let mut mouse_area = mouse_area::MouseArea::new(
                    tab.view(self.core())
                        .map(move |message| Message::TabMessage(entity, message)),
                )
                .on_press(move |_point_opt| Message::TabMessage(entity, tab::Message::Click(None)));
                if tab.context_menu.is_some() {
                    mouse_area = mouse_area
                        .on_right_press(move |_point_opt| Message::TabContextMenu(entity, None));
                } else {
                    mouse_area = mouse_area.on_right_press(move |point_opt| {
                        Message::TabContextMenu(entity, point_opt)
                    });
                }
                let mut popover = widget::popover(mouse_area, menu::context_menu(entity));
                match tab.context_menu {
                    Some(point) => {
                        popover = popover.position(point);
                    }
                    None => {
                        popover = popover.show_popup(false);
                    }
                }
                tab_column = tab_column.push(popover);
            }
            None => {
                //TODO
            }
        }

        let content: Element<_> = tab_column.into();

        // Uncomment to debug layout:
        //content.explain(cosmic::iced::Color::WHITE)
        content
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        Subscription::batch([
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|(_, res)| match res {
                Ok(config) => Message::Config(config),
                Err((errs, config)) => {
                    log::info!("errors loading config: {:?}", errs);
                    Message::Config(config)
                }
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|(_, u)| match u {
                Ok(t) => Message::SystemThemeModeChange(t),
                Err((errs, t)) => {
                    log::info!("errors loading theme mode: {:?}", errs);
                    Message::SystemThemeModeChange(t)
                }
            }),
        ])
    }
}
