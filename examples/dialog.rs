use cosmic::{
    app::{self, Command, Core, Settings},
    executor,
    iced::{subscription::Subscription, window},
    widget, Application, Element,
};
use cosmic_files::dialog::{Dialog, DialogKind, DialogMessage, DialogResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let settings = Settings::default();
    app::run::<App>(settings, ())?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum Message {
    DialogMessage(DialogMessage),
    DialogOpen,
    DialogResult(DialogResult),
    DialogSave,
}

pub struct App {
    core: Core,
    dialog_opt: Option<Dialog<Message>>,
    result_opt: Option<DialogResult>,
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.system76.CosmicFilesDialogExample";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                core,
                dialog_opt: None,
                result_opt: None,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DialogMessage(dialog_message) => {
                if let Some(dialog) = &mut self.dialog_opt {
                    return dialog.update(dialog_message);
                }
            }
            Message::DialogOpen => {
                if self.dialog_opt.is_none() {
                    let (dialog, command) = Dialog::new(
                        DialogKind::OpenFile,
                        None,
                        Message::DialogMessage,
                        Message::DialogResult,
                    );
                    self.dialog_opt = Some(dialog);
                    return command;
                }
            }
            Message::DialogResult(result) => {
                self.dialog_opt = None;
                self.result_opt = Some(result);
            }
            Message::DialogSave => {
                if self.dialog_opt.is_none() {
                    let (dialog, command) = Dialog::new(
                        DialogKind::SaveFile,
                        Some("README.md".into()),
                        Message::DialogMessage,
                        Message::DialogResult,
                    );
                    self.dialog_opt = Some(dialog);
                    return command;
                }
            }
        }

        Command::none()
    }

    fn view_window(&self, window_id: window::Id) -> Element<Message> {
        match &self.dialog_opt {
            Some(dialog) => dialog.view(window_id),
            None => widget::text("No dialog").into(),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = widget::column().spacing(8);
        {
            let mut button = widget::button(widget::text("Open Dialog"));
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen);
            }
            column = column.push(button);
        }
        {
            let mut button = widget::button(widget::text("Save Dialog"));
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogSave);
            }
            column = column.push(button);
        }
        if let Some(result) = &self.result_opt {
            match result {
                DialogResult::Cancel => {
                    column = column.push(widget::text("Cancel"));
                }
                DialogResult::Open(paths) => {
                    for path in paths.iter() {
                        column = column.push(widget::text(format!("{}", path.display())));
                    }
                }
            }
        }
        column.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.dialog_opt {
            Some(dialog) => dialog.subscription(),
            None => Subscription::none(),
        }
    }
}
