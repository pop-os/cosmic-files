// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{message, Command, Core, Settings},
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme, executor,
    iced::{
        event,
        keyboard::{Event as KeyEvent, KeyCode, Modifiers},
        subscription::Subscription,
        window, Event, Length, Point,
    },
    style,
    widget::{self, segmented_button},
    Application, ApplicationExt, Element,
};
use std::{any::TypeId, env, fs, path::PathBuf, process};

use config::{AppTheme, Config, CONFIG_VERSION};
mod config;

mod menu;

mod mouse_area;

mod localize;

mod mime_icon;

use tab::{Location, Tab};
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
    Cut,
    MoveToTrash,
    NewFile,
    NewFolder,
    Paste,
    Properties,
    RestoreFromTrash,
    SelectAll,
    Settings,
    TabNew,
}

impl Action {
    pub fn message(self, entity: segmented_button::Entity) -> Message {
        match self {
            Action::Copy => Message::Copy(Some(entity)),
            Action::Cut => Message::Cut(Some(entity)),
            Action::MoveToTrash => Message::MoveToTrash(Some(entity)),
            Action::NewFile => Message::NewFile(Some(entity)),
            Action::NewFolder => Message::NewFolder(Some(entity)),
            Action::Paste => Message::Paste(Some(entity)),
            Action::Properties => Message::ToggleContextPage(ContextPage::Properties),
            Action::RestoreFromTrash => Message::RestoreFromTrash(Some(entity)),
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
    Cut(Option<segmented_button::Entity>),
    KeyModifiers(Modifiers),
    MoveToTrash(Option<segmented_button::Entity>),
    NewFile(Option<segmented_button::Entity>),
    NewFolder(Option<segmented_button::Entity>),
    Paste(Option<segmented_button::Entity>),
    RestoreFromTrash(Option<segmented_button::Entity>),
    SelectAll(Option<segmented_button::Entity>),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    TabActivate(segmented_button::Entity),
    TabNext,
    TabPrev,
    TabClose(Option<segmented_button::Entity>),
    TabContextAction(segmented_button::Entity, Action),
    TabContextMenu(segmented_button::Entity, Option<Point>),
    TabMessage(Option<segmented_button::Entity>, tab::Message),
    TabNew,
    TabRescan(segmented_button::Entity, Vec<tab::Item>),
    ToggleContextPage(ContextPage),
    WindowClose,
    WindowNew,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    Properties,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::Properties => fl!("properties"),
            Self::Settings => fl!("settings"),
        }
    }
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: segmented_button::SingleSelectModel,
    tab_model: segmented_button::Model<segmented_button::SingleSelect>,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
    modifiers: Modifiers,
}

impl App {
    fn open_tab(&mut self, location: Location) -> Command<Message> {
        let tab = Tab::new(location.clone());
        let entity = self
            .tab_model
            .insert()
            .text(tab.title())
            .data(tab)
            .closable()
            .activate()
            .id();
        Command::batch([self.update_title(), self.rescan_tab(entity, location)])
    }

    fn rescan_tab(
        &mut self,
        entity: segmented_button::Entity,
        location: Location,
    ) -> Command<Message> {
        Command::perform(
            async move {
                match tokio::task::spawn_blocking(move || location.scan()).await {
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

    fn properties(&self) -> Element<Message> {
        let mut children = Vec::new();
        let entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            if let Some(ref items) = tab.items_opt {
                for item in items.iter() {
                    if item.selected {
                        children.push(item.property_view(&self.core));
                    }
                }
            }
        }
        widget::settings::view_column(children).into()
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

        let mut nav_model = segmented_button::ModelBuilder::default();
        //TODO: Sort by name?
        for dir_opt in &[
            dirs::home_dir(),
            dirs::document_dir(),
            dirs::download_dir(),
            dirs::audio_dir(),
            dirs::picture_dir(),
            dirs::video_dir(),
        ] {
            if let Some(dir) = dir_opt {
                if let Some(file_name) = dir.file_name().and_then(|x| x.to_str()) {
                    nav_model = nav_model.insert(move |b| {
                        b.text(file_name.to_string())
                            .icon(widget::icon::icon(tab::folder_icon_symbolic(&dir, 16)).size(16))
                            .data(Location::Path(dir.clone()))
                    });
                }
            }
        }
        nav_model = nav_model.insert(|b| {
            b.text(fl!("trash"))
                .icon(
                    //TODO: dynamic empty/full icon
                    widget::icon::from_name("user-trash-full-symbolic")
                        .size(16)
                        .icon(),
                )
                .data(Location::Trash)
        });

        let mut app = App {
            core,
            nav_model: nav_model.build(),
            tab_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes,
            context_page: ContextPage::Settings,
            modifiers: Modifiers::empty(),
        };

        let mut commands = Vec::new();

        for arg in env::args().skip(1) {
            let location = match fs::canonicalize(&arg) {
                Ok(absolute) => Location::Path(absolute),
                Err(err) => {
                    log::warn!("failed to canonicalize {:?}: {}", arg, err);
                    continue;
                }
            };
            commands.push(app.open_tab(location));
        }

        if app.tab_model.iter().next().is_none() {
            commands.push(app.open_tab(Location::Path(home_dir())));
        }

        (app, Command::batch(commands))
    }

    // The default nav_bar widget needs to have its width reduced for cosmic-files
    fn nav_bar(&self) -> Option<Element<message::Message<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = crate::widget::nav_bar(nav_model, |entity| {
            message::cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        });

        if !self.core().is_condensed() {
            nav = nav.max_width(200);
        }

        Some(Element::from(nav))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Command<Self::Message> {
        let location_opt = self.nav_model.data::<Location>(entity).clone();

        if let Some(location) = location_opt {
            let message = Message::TabMessage(None, tab::Message::Location(location.clone()));
            return self.update(message);
        }

        Command::none()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        match message {
            Message::Todo => {
                log::warn!("TODO");
            }
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
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
            Message::Cut(entity_opt) => {
                log::warn!("TODO: CUT");
            }
            Message::KeyModifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::MoveToTrash(entity_opt) => {
                log::warn!("TODO: MOVE TO TRASH");
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
            Message::RestoreFromTrash(entity_opt) => {
                log::warn!("TODO: RESTORE FROM TRASH");
            }
            Message::SelectAll(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if item.hidden {
                                //TODO: option to show hidden files
                                continue;
                            }
                            item.selected = true;
                            item.click_time = None;
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
            Message::TabNext => {
                let len = self.tab_model.iter().count();
                let pos = self
                    .tab_model
                    .position(self.tab_model.active())
                    // Wraparound to 0 if i + 1 > num of tabs
                    .map(|i| (i as usize + 1) % len)
                    .expect("should always be at least one tab open");

                let entity = self.tab_model.iter().nth(pos);
                if let Some(entity) = entity {
                    return self.update(Message::TabActivate(entity));
                }
            }
            Message::TabPrev => {
                let pos = self
                    .tab_model
                    .position(self.tab_model.active())
                    .and_then(|i| (i as usize).checked_sub(1))
                    // Subtraction underflow => last tab; i.e. it wraps around
                    .unwrap_or_else(|| {
                        self.tab_model
                            .iter()
                            .count()
                            .checked_sub(1)
                            .unwrap_or_default()
                    });

                let entity = self.tab_model.iter().nth(pos);
                if let Some(entity) = entity {
                    return self.update(Message::TabActivate(entity));
                }
            }
            Message::TabClose(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

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
                // Disable side context page
                self.core.window.show_context = false;
            }
            Message::TabMessage(entity_opt, tab_message) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

                let mut update_opt = None;
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        if tab.update(tab_message, self.modifiers) {
                            update_opt = Some((tab.title(), tab.location.clone()));
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
                let location = match self.tab_model.data::<Tab>(active) {
                    Some(tab) => tab.location.clone(),
                    None => Location::Path(home_dir()),
                };
                return self.open_tab(location);
            }
            Message::TabRescan(entity, items) => match self.tab_model.data_mut::<Tab>(entity) {
                Some(tab) => {
                    tab.items_opt = Some(items);
                }
                _ => (),
            },
            //TODO: TABRELOAD
            Message::ToggleContextPage(context_page) => {
                //TODO: ensure context menus are closed
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.title());
            }
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_child) => {}
                    Err(err) => {
                        log::error!("failed to execute {:?}: {}", exe, err);
                    }
                },
                Err(err) => {
                    log::error!("failed to get current executable path: {}", err);
                }
            },
        }

        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::Properties => self.properties(),
            ContextPage::Settings => self.settings(),
        })
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![
            menu::menu_bar().into(),
            //TODO: use theme defined space?
            widget::horizontal_space(Length::Fixed(32.0)).into(),
        ]
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        let entity = self.tab_model.active();
        let tab = match self.tab_model.data::<Tab>(entity) {
            Some(some) => some,
            None => return Vec::new(),
        };

        vec![tab
            .breadcrumbs_view(&self.core)
            .map(move |message| Message::TabMessage(Some(entity), message))
            .into()]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        vec![
            //TODO: use defined space
            widget::horizontal_space(Length::Fixed(32.0)).into(),
        ]
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
                        .on_close(|entity| Message::TabClose(Some(entity))),
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
                        .map(move |message| Message::TabMessage(Some(entity), message)),
                )
                .on_press(move |_point_opt| {
                    Message::TabMessage(Some(entity), tab::Message::Click(None))
                });
                if tab.context_menu.is_some() {
                    mouse_area = mouse_area
                        .on_right_press(move |_point_opt| Message::TabContextMenu(entity, None));
                } else {
                    mouse_area = mouse_area.on_right_press(move |point_opt| {
                        Message::TabContextMenu(entity, point_opt)
                    });
                }
                let mut popover = widget::popover(mouse_area, menu::context_menu(entity, &tab));
                match tab.context_menu {
                    Some(point) => {
                        let rounded = Point::new(point.x.round(), point.y.round());
                        popover = popover.position(rounded);
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
            event::listen_with(|event, _status| match event {
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::A,
                    modifiers,
                }) => {
                    if modifiers == Modifiers::CTRL {
                        Some(Message::SelectAll(None))
                    } else {
                        None
                    }
                }
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::C,
                    modifiers,
                }) => {
                    if modifiers == Modifiers::CTRL {
                        Some(Message::Copy(None))
                    } else {
                        None
                    }
                }
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::X,
                    modifiers,
                }) => {
                    if modifiers == Modifiers::CTRL {
                        Some(Message::Cut(None))
                    } else {
                        None
                    }
                }
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::T,
                    modifiers,
                }) => {
                    if modifiers == Modifiers::CTRL {
                        Some(Message::TabNew)
                    } else {
                        None
                    }
                }
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::W,
                    modifiers: Modifiers::CTRL,
                }) => Some(Message::TabClose(None)),
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: key @ (KeyCode::PageUp | KeyCode::PageDown),
                    modifiers: Modifiers::CTRL,
                }) => match key {
                    KeyCode::PageDown => Some(Message::TabPrev),
                    KeyCode::PageUp => Some(Message::TabNext),
                    _ => None,
                },
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code: KeyCode::V,
                    modifiers,
                }) => {
                    if modifiers == Modifiers::CTRL {
                        Some(Message::Paste(None))
                    } else {
                        None
                    }
                }
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::KeyModifiers(modifiers))
                }
                _ => None,
            }),
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
