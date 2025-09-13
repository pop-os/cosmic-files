use cosmic::{
    Application, Element,
    app::{self, Core, Settings, Task},
    executor,
    iced::{Subscription, window},
    widget,
};
use cosmic_files::dialog::{
    Dialog, DialogChoice, DialogChoiceOption, DialogFilter, DialogFilterPattern, DialogKind,
    DialogMessage, DialogResult, DialogSettings,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let settings = Settings::default();
    app::run::<App>(settings, ())?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum Message {
    DialogMessage(DialogMessage),
    DialogOpen(DialogKind),
    DialogResult(DialogResult),
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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        (
            Self {
                core,
                dialog_opt: None,
                result_opt: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DialogMessage(dialog_message) => {
                if let Some(dialog) = &mut self.dialog_opt {
                    return dialog.update(dialog_message);
                }
            }
            Message::DialogOpen(dialog_kind) => {
                if self.dialog_opt.is_none() {
                    let (mut dialog, task) = Dialog::new(
                        DialogSettings::new().kind(dialog_kind),
                        Message::DialogMessage,
                        Message::DialogResult,
                    );
                    let mut tasks = vec![task];
                    dialog.set_choices(vec![
                        DialogChoice::ComboBox {
                            id: "example-combobox".into(),
                            label: "Combobox".into(),
                            options: vec![
                                DialogChoiceOption {
                                    id: "foo".into(),
                                    label: "foo".into(),
                                },
                                DialogChoiceOption {
                                    id: "bar".into(),
                                    label: "bar".into(),
                                },
                            ],
                            selected: Some(0),
                        },
                        DialogChoice::CheckBox {
                            id: "example-checkbox".into(),
                            label: "Checkbox".into(),
                            value: false,
                        },
                    ]);
                    tasks.push(dialog.set_filters(
                        vec![DialogFilter {
                            label: "Any file".into(),
                            patterns: vec![DialogFilterPattern::Glob("*".into())],
                        }],
                        Some(0),
                    ));
                    self.dialog_opt = Some(dialog);
                    return Task::batch(tasks);
                }
            }
            Message::DialogResult(result) => {
                self.dialog_opt = None;
                self.result_opt = Some(result);
            }
        }

        Task::none()
    }

    fn view_window(&self, window_id: window::Id) -> Element<'_, Message> {
        match &self.dialog_opt {
            Some(dialog) => dialog.view(window_id),
            None => widget::text::body("No dialog").into(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut column = widget::column().spacing(8).padding(8);
        {
            let mut button = widget::button::standard("Open File");
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen(DialogKind::OpenFile));
            }
            column = column.push(button);
        }
        {
            let mut button = widget::button::standard("Open Multiple Files");
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen(DialogKind::OpenMultipleFiles));
            }
            column = column.push(button);
        }
        {
            let mut button = widget::button::standard("Open Folder");
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen(DialogKind::OpenFolder));
            }
            column = column.push(button);
        }
        {
            let mut button = widget::button::standard("Open Multiple Folders");
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen(DialogKind::OpenMultipleFolders));
            }
            column = column.push(button);
        }
        {
            let mut button = widget::button::standard("Save File");
            if self.dialog_opt.is_none() {
                button = button.on_press(Message::DialogOpen(DialogKind::SaveFile {
                    filename: "test".to_string(),
                }));
            }
            column = column.push(button);
        }
        if let Some(result) = &self.result_opt {
            match result {
                DialogResult::Cancel => {
                    column = column.push(widget::text::body("Cancel"));
                }
                DialogResult::Open(paths) => {
                    for path in paths.iter() {
                        column = column.push(widget::text::body(format!("{}", path.display())));
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
