use data::Config;
use data::config::settings::{
    Authentication, Network, Settings as Draft, ThemeMode,
};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable,
    space, text, text_input,
};
use iced::{Alignment, Length, Size, Task, Vector};

use crate::widget::{Element, key_press};
use crate::{Theme, font, open_url, theme, window};

mod theme_browser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Networks,
    Appearance,
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Name,
    Server,
    Port,
    Nickname,
    Channels,
    Username,
    Password,
    FontFamily,
    FontSize,
    LineHeight,
    Scale,
}

#[derive(Debug, Clone)]
pub enum Message {
    Section(Section),
    Select(usize),
    Add,
    Remove,
    ConfirmRemove,
    CancelRemove,
    Edit(Field, String),
    Tls(bool),
    Autoconnect(bool),
    Authentication(Authentication),
    ThemeMode(ThemeMode),
    Theme(String),
    BrowseThemes,
    Browser(theme_browser::Message),
    Light(String),
    Dark(String),
    Sidebar(String),
    Apply,
    Saved(Result<String, String>),
    Revert,
    Close,
    Quit,
    Discard,
    KeepEditing,
    OpenFile,
    Focus(bool),
}

pub enum Event {
    Close,
    Quit,
    Saved,
}

pub struct Settings {
    pub window: window::Id,
    draft: Result<Draft, String>,
    section: Section,
    selected: Option<usize>,
    themes: Vec<String>,
    installed_themes: Vec<data::Theme>,
    browser: Option<theme_browser::Browser>,
    saving: bool,
    pub status: Option<String>,
    error: Option<String>,
    confirm_close: bool,
    confirm_remove: bool,
    restart_needed: bool,
    quit_requested: bool,
}

impl Settings {
    pub fn application_failed(&mut self, error: String) {
        self.saving = false;
        self.status = None;
        self.error = Some(error);
    }

    pub fn application_succeeded(&mut self) {
        self.saving = false;
    }
    pub fn open(
        main: &window::Window,
        config: &Config,
    ) -> (Self, Task<window::Id>) {
        let size_scale = theme::scale_for_font_size(&config.font);
        let (window, task) = window::open(window::Settings {
            size: Size::new(880.0, 720.0) * size_scale,
            min_size: Some(Size::new(740.0, 560.0) * size_scale),
            position: main
                .position
                .map(|point| {
                    window::Position::Specific(point + Vector::new(24.0, 24.0))
                })
                .unwrap_or_default(),
            exit_on_close_request: false,
            ..window::settings(config)
        });
        let draft = Draft::load();
        let selected = draft
            .as_ref()
            .ok()
            .filter(|draft| !draft.networks.is_empty())
            .map(|_| 0);
        let installed_themes =
            data::appearance::catalog::installed(&Config::themes_dir());
        let mut themes = installed_themes
            .iter()
            .map(|theme| theme.name.clone())
            .collect::<Vec<_>>();
        if let Ok(draft) = &draft {
            for name in [
                &draft.appearance.theme,
                &draft.appearance.light,
                &draft.appearance.dark,
            ] {
                if !themes.contains(name) {
                    themes.push(name.clone());
                }
            }
        }
        themes.sort_by_key(|name| name.to_lowercase());
        themes.dedup();
        (
            Self {
                window,
                draft,
                section: Section::Networks,
                selected,
                themes,
                installed_themes,
                browser: None,
                saving: false,
                status: None,
                error: None,
                confirm_close: false,
                confirm_remove: false,
                restart_needed: false,
                quit_requested: false,
            },
            task,
        )
    }

    pub fn preview_theme(&self, _config: &Config, current: &Theme) -> Theme {
        let Ok(draft) = &self.draft else {
            return current.clone();
        };
        if draft.appearance.mode == ThemeMode::Single {
            self.installed_themes
                .iter()
                .find(|theme| theme.name == draft.appearance.theme)
                .cloned()
                .map_or_else(|| current.clone(), Theme::from)
        } else {
            current.clone()
        }
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        if self.saving
            && !matches!(
                message,
                Message::Saved(_)
                    | Message::Browser(
                        theme_browser::Message::Loaded(_)
                            | theme_browser::Message::Downloaded(_, _)
                    )
            )
        {
            return (Task::none(), None);
        }
        match message {
            Message::BrowseThemes => {
                let mode = self
                    .draft
                    .as_ref()
                    .map_or(ThemeMode::Single, |draft| draft.appearance.mode);
                let (browser, task) =
                    theme_browser::Browser::open(&self.installed_themes, mode);
                self.browser = Some(browser);
                return (task.map(Message::Browser), None);
            }
            Message::Browser(message) => {
                let Some(browser) = &mut self.browser else {
                    return (Task::none(), None);
                };
                let (task, event) = browser.update(message);
                match event {
                    Some(theme_browser::Event::Back) => self.browser = None,
                    Some(theme_browser::Event::Installed(theme)) => {
                        if !self.themes.contains(&theme.name) {
                            self.themes.push(theme.name.clone());
                            self.themes.sort_by_key(|name| name.to_lowercase());
                        }
                        self.installed_themes.push(theme);
                    }
                    Some(theme_browser::Event::Use(target, name)) => {
                        if let Ok(draft) = &mut self.draft {
                            match target {
                                theme_browser::Target::Single => {
                                    draft.appearance.mode = ThemeMode::Single;
                                    draft.appearance.theme = name;
                                }
                                theme_browser::Target::Light => {
                                    draft.appearance.light = name;
                                }
                                theme_browser::Target::Dark => {
                                    draft.appearance.dark = name;
                                }
                            }
                        }
                        self.browser = None;
                        self.status = None;
                        self.error = None;
                    }
                    None => {}
                }
                return (task.map(Message::Browser), None);
            }

            Message::Focus(previous) => {
                return (
                    if previous {
                        iced::widget::operation::focus_previous()
                    } else {
                        iced::widget::operation::focus_next()
                    },
                    None,
                );
            }
            Message::Saved(result) => match result {
                Ok(source) => {
                    let Ok(draft) = &self.draft else {
                        return (Task::none(), None);
                    };
                    self.restart_needed |= draft.font_changed();
                    let networks_changed = draft.networks_changed();
                    self.draft = Draft::parse(source);
                    self.status = Some(if self.restart_needed {
                            "Saved. Restart Halloy to apply font changes."
                        } else if networks_changed {
                            "Saved. Reconnect existing networks to use connection changes."
                        } else { "Settings applied." }.into());
                    self.error = None;
                    return (Task::none(), Some(Event::Saved));
                }
                Err(error) => {
                    self.saving = false;
                    self.error = Some(error);
                }
            },
            Message::Close | Message::Quit => {
                self.quit_requested = matches!(message, Message::Quit);
                if self.draft.as_ref().is_ok_and(Draft::is_dirty) {
                    self.confirm_close = true;
                    self.confirm_remove = false;
                } else {
                    return (
                        Task::none(),
                        Some(if self.quit_requested {
                            Event::Quit
                        } else {
                            Event::Close
                        }),
                    );
                }
            }
            Message::Discard => {
                return (
                    Task::none(),
                    Some(if self.quit_requested {
                        Event::Quit
                    } else {
                        Event::Close
                    }),
                );
            }
            Message::KeepEditing => {
                self.confirm_close = false;
                self.quit_requested = false;
            }
            Message::OpenFile => {
                let _ = open_url::open(Config::path());
            }
            Message::Revert => {
                self.browser = None;
                self.draft = Draft::load();
                self.selected = self
                    .draft
                    .as_ref()
                    .ok()
                    .filter(|draft| !draft.networks.is_empty())
                    .map(|_| 0);
                self.error = None;
                self.status = None;
                self.confirm_close = false;
                self.confirm_remove = false;
            }
            message => {
                let Ok(draft) = &mut self.draft else {
                    return (Task::none(), None);
                };
                self.error = None;
                self.status = None;
                match message {
                    Message::Section(section) => {
                        self.section = section;
                        self.browser = None;
                        self.confirm_remove = false;
                    }
                    Message::Select(index) => {
                        self.selected = Some(index);
                        self.confirm_remove = false;
                    }
                    Message::Add => {
                        let mut number = 1;
                        while draft.networks.iter().any(|network| {
                            network.name == format!("Network {number}")
                        }) {
                            number += 1;
                        }
                        draft
                            .networks
                            .push(Network::new(format!("Network {number}")));
                        self.selected = Some(draft.networks.len() - 1);
                        self.confirm_remove = false;
                    }
                    Message::Remove => self.confirm_remove = true,
                    Message::CancelRemove => self.confirm_remove = false,
                    Message::ConfirmRemove => {
                        if let Some(index) = self
                            .selected
                            .filter(|index| *index < draft.networks.len())
                        {
                            draft.networks.remove(index);
                            self.selected = (!draft.networks.is_empty())
                                .then(|| index.min(draft.networks.len() - 1));
                        }
                        self.confirm_remove = false;
                    }
                    Message::Edit(field, value) => match field {
                        Field::FontFamily => {
                            draft.appearance.font_family = value;
                        }
                        Field::FontSize => draft.appearance.font_size = value,
                        Field::LineHeight => {
                            draft.appearance.line_height = value;
                        }
                        Field::Scale => draft.appearance.scale_factor = value,
                        field => {
                            if let Some(network) = self
                                .selected
                                .and_then(|index| draft.networks.get_mut(index))
                            {
                                *match field {
                                    Field::Name => &mut network.name,
                                    Field::Server => &mut network.server,
                                    Field::Port => &mut network.port,
                                    Field::Nickname => &mut network.nickname,
                                    Field::Channels => &mut network.channels,
                                    Field::Username => &mut network.username,
                                    Field::Password => &mut network.password,
                                    _ => unreachable!(),
                                } = value;
                            }
                        }
                    },
                    Message::Tls(value) => {
                        if let Some(network) = self
                            .selected
                            .and_then(|index| draft.networks.get_mut(index))
                        {
                            network.use_tls = value;
                        }
                    }
                    Message::Autoconnect(value) => {
                        if let Some(network) = self
                            .selected
                            .and_then(|index| draft.networks.get_mut(index))
                        {
                            network.autoconnect = value;
                        }
                    }
                    Message::Authentication(value) => {
                        if let Some(network) = self
                            .selected
                            .and_then(|index| draft.networks.get_mut(index))
                        {
                            if network.authentication != value {
                                network.password.clear();
                                network.username.clear();
                            }
                            network.authentication = value;
                        }
                    }
                    Message::ThemeMode(value) => draft.appearance.mode = value,
                    Message::Theme(value) => draft.appearance.theme = value,
                    Message::Light(value) => draft.appearance.light = value,
                    Message::Dark(value) => draft.appearance.dark = value,
                    Message::Sidebar(value) => {
                        draft.appearance.sidebar_position = value;
                    }
                    Message::Apply => {
                        if !draft.is_dirty() {
                            return (Task::none(), None);
                        }
                        if let Err(error) = draft.render() {
                            self.error = Some(error);
                        } else {
                            self.saving = true;
                            self.confirm_close = false;
                            let draft = draft.clone();
                            return (
                                Task::perform(
                                    async move {
                                        tokio::task::spawn_blocking(move || {
                                            draft.save_to(&Config::path())
                                        })
                                        .await
                                        .map_err(|error| error.to_string())
                                        .and_then(|result| result)
                                    },
                                    Message::Saved,
                                ),
                                None,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        (Task::none(), None)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let heading = row![
            text("Settings").size(20).font(font::PRIMARY_BOLD.clone()),
            space::horizontal(),
            action("Open config file", Message::OpenFile)
        ]
        .align_y(Alignment::Center);
        let tabs = row![
            tab(
                "Networks",
                self.section == Section::Networks,
                Message::Section(Section::Networks)
            ),
            tab(
                "Appearance",
                self.section == Section::Appearance,
                Message::Section(Section::Appearance)
            )
        ]
        .spacing(6);
        let body: Element<_> = match &self.draft {
            Ok(draft) => match self.section {
                Section::Networks => self.networks(draft),
                Section::Appearance => match &self.browser {
                    Some(browser) => browser.view().map(Message::Browser),
                    None => scrollable(self.appearance(draft))
                        .height(Length::Fill)
                        .into(),
                },
            },
            Err(error) => column![
                text("Settings could not be loaded")
                    .font(font::PRIMARY_BOLD.clone()),
                text(error).style(theme::text::error),
                text("Fix the config file, then reload Settings."),
                action("Reload Settings", Message::Revert)
            ]
            .spacing(12)
            .into(),
        };
        let dirty = self.draft.as_ref().is_ok_and(Draft::is_dirty);
        let mut footer = column![].spacing(10);
        if let Some(error) = &self.error {
            footer = footer.push(text(error).style(theme::text::error));
        }
        if let Some(status) = &self.status
            && !self.saving
        {
            footer = footer.push(text(status).style(theme::text::tertiary));
        }
        if self.confirm_close {
            footer = footer.push(
                row![
                    text("Discard unsaved changes?"),
                    space::horizontal(),
                    action("Keep editing", Message::KeepEditing),
                    action("Discard changes", Message::Discard)
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        } else {
            footer = footer.push(
                row![
                    text(if self.saving {
                        "Saving…"
                    } else if dirty {
                        "Unsaved changes"
                    } else {
                        "Changes are saved to config.toml"
                    })
                    .style(theme::text::secondary),
                    space::horizontal(),
                    button("Revert changes")
                        .padding([8, 12])
                        .style(|theme, status| theme::button::secondary(
                            theme, status, false
                        ))
                        .on_press_maybe(
                            (dirty && !self.saving).then_some(Message::Revert)
                        ),
                    button(if self.saving { "Saving…" } else { "Apply" })
                        .padding([8, 20])
                        .on_press_maybe(
                            (dirty && !self.saving).then_some(Message::Apply)
                        ),
                    action("Close", Message::Close),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        let content = container(
            column![
                heading,
                tabs,
                rule::horizontal(1),
                body,
                rule::horizontal(1),
                footer
            ]
            .spacing(16),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::container::root);
        let content = key_press(
            content,
            key_press::Key::Named(key_press::Named::Escape),
            key_press::Modifiers::default(),
            if self.browser.is_some() {
                Message::Browser(theme_browser::Message::Back)
            } else {
                Message::Close
            },
        );
        crate::widget::on_key(content, |key, modifiers| {
            if key.as_ref() == key_press::Key::Named(key_press::Named::Tab) {
                Some(Message::Focus(modifiers.shift()))
            } else if modifiers.command()
                && key.as_ref() == key_press::Key::Character("s")
            {
                Some(Message::Apply)
            } else {
                None
            }
        })
    }

    fn networks<'a>(&'a self, draft: &'a Draft) -> Element<'a, Message> {
        let mut list = column![].spacing(4);
        for (index, network) in draft.networks.iter().enumerate() {
            list = list.push(tab(
                &network.name,
                self.selected == Some(index),
                Message::Select(index),
            ));
        }
        let list = column![
            scrollable(list).height(Length::Fill),
            action("Add network", Message::Add)
        ]
        .spacing(12)
        .width(170);
        let form: Element<_> = if let Some(network) =
            self.selected.and_then(|index| draft.networks.get(index))
        {
            let mut authentication = vec![
                Authentication::None,
                Authentication::Sasl,
                Authentication::NickServ,
            ];
            if draft.has_advanced_authentication(network) {
                authentication.push(Authentication::Advanced);
            }
            let mut form = column![
                text("Connection").font(font::PRIMARY_BOLD.clone()).size(16),
                field(
                    "Network name",
                    "e.g. Libera",
                    &network.name,
                    Field::Name,
                    false
                ),
                field(
                    "Server",
                    "irc.libera.chat",
                    &network.server,
                    Field::Server,
                    false
                ),
                row![
                    container(field(
                        "Port",
                        "Automatic",
                        &network.port,
                        Field::Port,
                        false
                    ))
                    .width(150),
                    container(
                        checkbox(network.use_tls)
                            .label("Use TLS")
                            .on_toggle(Message::Tls)
                    )
                    .padding(iced::padding::top(25))
                ]
                .spacing(16),
                field(
                    "Nickname",
                    "Your nickname",
                    &network.nickname,
                    Field::Nickname,
                    false
                ),
                field(
                    "Join channels on connect",
                    "#halloy, #rust",
                    &network.channels,
                    Field::Channels,
                    false
                ),
                checkbox(network.autoconnect)
                    .label("Connect automatically on launch")
                    .on_toggle(Message::Autoconnect),
                rule::horizontal(1),
                text("Authentication")
                    .font(font::PRIMARY_BOLD.clone())
                    .size(16),
                pick_list(
                    Some(network.authentication),
                    authentication,
                    Authentication::to_string
                )
                .on_select(Message::Authentication)
                .width(Length::Fill)
                .padding(8),
            ]
            .spacing(12);
            match network.authentication {
                Authentication::Sasl => {
                    form = form.push(field(
                        "Account name",
                        "Use nickname",
                        &network.username,
                        Field::Username,
                        false,
                    ));
                }
                Authentication::Advanced => {
                    form = form.push(text("Certificate, keyring and command settings are managed in the config file. Choosing another method replaces this network’s authentication settings.").style(theme::text::secondary));
                }
                _ => {}
            }
            if matches!(
                network.authentication,
                Authentication::Sasl | Authentication::NickServ
            ) {
                form = form.push(field("Password", "Account password", &network.password, Field::Password, true))
                    .push(text("Passwords entered here are stored in config.toml. Use the config file for keyring or password-command authentication.").style(theme::text::secondary));
            }
            form = form.push(text("Existing connections use changed server and authentication settings after reconnecting.").style(theme::text::secondary));
            if self.confirm_remove {
                form = form
                    .push(text(format!(
                        "Remove ‘{}’? Applying will disconnect it.",
                        network.name
                    )))
                    .push(
                        row![
                            action("Keep network", Message::CancelRemove),
                            action("Remove network", Message::ConfirmRemove)
                        ]
                        .spacing(8),
                    );
            } else {
                form = form.push(action("Remove network…", Message::Remove));
            }
            scrollable(form.padding(iced::padding::right(12)))
                .height(Length::Fill)
                .into()
        } else {
            container(
                column![
                    text("Add your first network")
                        .font(font::PRIMARY_BOLD.clone())
                        .size(16),
                    text("Choose a server and nickname to get started."),
                    action("Add network", Message::Add)
                ]
                .spacing(12),
            )
            .center(Length::Fill)
            .into()
        };
        row![list, rule::vertical(1), container(form).width(Length::Fill)]
            .spacing(20)
            .height(Length::Fill)
            .into()
    }

    fn appearance<'a>(&'a self, draft: &'a Draft) -> Element<'a, Message> {
        let a = &draft.appearance;
        let mut modes = vec![ThemeMode::Single, ThemeMode::System];
        if draft.has_configured_theme() {
            modes.push(ThemeMode::Configured);
        }
        let mut form = column![
            text("Theme").size(16).font(font::PRIMARY_BOLD.clone()),
            pick_list(Some(a.mode), modes, ThemeMode::to_string)
                .on_select(Message::ThemeMode)
                .width(Length::Fill)
                .padding(8),
        ]
        .spacing(12);
        match a.mode {
            ThemeMode::Single => {
                form = form.push(labelled(
                    "Theme",
                    pick_list(
                        Some(a.theme.clone()),
                        self.themes.clone(),
                        String::clone,
                    )
                    .on_select(Message::Theme)
                    .width(Length::Fill)
                    .padding(8)
                    .into(),
                ));
            }
            ThemeMode::System => {
                form = form
                    .push(labelled(
                        "Light appearance",
                        pick_list(
                            Some(a.light.clone()),
                            self.themes.clone(),
                            String::clone,
                        )
                        .on_select(Message::Light)
                        .width(Length::Fill)
                        .padding(8)
                        .into(),
                    ))
                    .push(labelled(
                        "Dark appearance",
                        pick_list(
                            Some(a.dark.clone()),
                            self.themes.clone(),
                            String::clone,
                        )
                        .on_select(Message::Dark)
                        .width(Length::Fill)
                        .padding(8)
                        .into(),
                    ));
            }
            ThemeMode::Configured => {
                form = form.push(text("Your theme configuration is managed in the config file. Choose a mode above to replace it.").style(theme::text::secondary));
            }
        }
        form = form
            .push(action("Browse and download themes", Message::BrowseThemes));
        form = form.push(text("One-theme selections preview in this window. Apply to use them throughout Halloy.").style(theme::text::secondary))
            .push(rule::horizontal(1))
            .push(text("Font").size(16).font(font::PRIMARY_BOLD.clone()))
            .push(field("Font family", "Default (Iosevka Term)", &a.font_family, Field::FontFamily, false))
            .push(row![field("Text size", "13", &a.font_size, Field::FontSize, false), field("Line height", "1.3", &a.line_height, Field::LineHeight, false)].spacing(16))
            .push(text("Restart Halloy after applying font changes. Empty fields use Halloy’s defaults.").style(theme::text::secondary))
            .push(rule::horizontal(1))
            .push(text("Layout").size(16).font(font::PRIMARY_BOLD.clone()))
            .push(field("Chat interface scale", "1.0", &a.scale_factor, Field::Scale, false))
            .push(text("Settings stays at its default scale so you can always adjust this value.").style(theme::text::secondary))
            .push(labelled("Sidebar position", pick_list(Some(a.sidebar_position.clone()), vec!["left".to_owned(), "right".to_owned(), "top".to_owned(), "bottom".to_owned()], String::clone).on_select(Message::Sidebar).width(Length::Fill).padding(8).into()));
        container(form)
            .width(Length::Fill.max(600))
            .padding(iced::padding::right(12))
            .into()
    }
}

fn action<'a, M: Clone + 'a>(label: &'a str, message: M) -> Element<'a, M> {
    button(text(label))
        .padding([8, 12])
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(message)
        .into()
}

fn tab<'a>(
    label: &'a str,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    button(text(label))
        .padding([8, 12])
        .width(Length::Fill)
        .style(move |theme, status| {
            theme::button::primary(theme, status, selected)
        })
        .on_press(message)
        .into()
}

fn field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    field: Field,
    secret: bool,
) -> Element<'a, Message> {
    labelled(
        label,
        text_input(placeholder, value)
            .style(input_style)
            .on_input(move |value| Message::Edit(field, value))
            .secure(secret)
            .padding(8)
            .into(),
    )
}

fn labelled<'a>(
    label: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    column![text(label), control]
        .spacing(6)
        .width(Length::Fill)
        .into()
}

fn input_style(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let mut style = theme::text_input::primary(theme, status);
    style.placeholder = style
        .placeholder
        .mix(theme.styles().text.primary.color, 0.4);
    style
}
