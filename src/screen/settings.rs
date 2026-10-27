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

mod preferences;
mod style;
mod theme_browser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Networks,
    Appearance,
    Navigation,
    Chat,
    Sidebar,
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
    PreferenceGroup(usize),
    Preference(usize, String),
    ReuseCurrentPane,
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
    preference_group: usize,
    selected: Option<usize>,
    themes: Vec<String>,
    installed_themes: Vec<data::Theme>,
    browser: Option<theme_browser::Browser>,
    preview: Option<Theme>,
    pending_themes: Vec<data::appearance::catalog::Entry>,
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
        self.preview = None;
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
                preference_group: 0,
                selected,
                themes,
                installed_themes,
                browser: None,
                preview: None,
                pending_themes: vec![],
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

    pub fn accessible_theme(&self, current: &Theme) -> Theme {
        style::accessible_theme(self.preview_theme(current))
    }

    pub fn preview_theme<'a>(&'a self, current: &'a Theme) -> &'a Theme {
        self.preview.as_ref().unwrap_or(current)
    }

    fn select_theme(&mut self, target: theme_browser::Target, name: String) {
        if let Ok(draft) = &mut self.draft {
            match target {
                theme_browser::Target::Single => {
                    draft.appearance.mode = ThemeMode::Single;
                    draft.appearance.theme = name.clone();
                }
                theme_browser::Target::Light => {
                    draft.appearance.light = name.clone();
                }
                theme_browser::Target::Dark => {
                    draft.appearance.dark = name.clone();
                }
            }
        }
        self.preview = self
            .installed_themes
            .iter()
            .find(|theme| theme.name.eq_ignore_ascii_case(&name))
            .cloned()
            .map(Theme::from)
            .or_else(|| {
                self.pending_themes
                    .iter()
                    .find(|entry| {
                        entry.file_stem().is_ok_and(|stem| stem == name)
                    })
                    .map(|entry| {
                        Theme::from(data::Theme::new(name, entry.styles))
                    })
            });
        self.status = None;
        self.error = None;
    }

    fn refresh_themes(&mut self) {
        self.installed_themes =
            data::appearance::catalog::installed(&Config::themes_dir());
        self.themes = self
            .installed_themes
            .iter()
            .map(|theme| theme.name.clone())
            .collect();
        if let Ok(draft) = &self.draft {
            for name in [
                &draft.appearance.theme,
                &draft.appearance.light,
                &draft.appearance.dark,
            ] {
                if !self.themes.contains(name) {
                    self.themes.push(name.clone());
                }
            }
        }
        self.themes.sort_by_key(|name| name.to_lowercase());
        if let Some(browser) = &mut self.browser {
            browser.refresh_installed(&self.installed_themes);
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
                self.refresh_themes();
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
                    Some(theme_browser::Event::Preview(target, entry)) => {
                        if let Ok(name) = entry.file_stem() {
                            self.pending_themes.retain(|pending| {
                                pending.file_stem().ok().as_ref() != Some(&name)
                            });
                            self.pending_themes.push(entry);
                            if !self.themes.contains(&name) {
                                self.themes.push(name.clone());
                                self.themes
                                    .sort_by_key(|name| name.to_lowercase());
                            }
                            self.select_theme(target, name);
                        }
                    }
                    Some(theme_browser::Event::Use(target, name)) => {
                        self.select_theme(target, name);
                        self.browser = None;
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
            Message::Saved(result) => {
                self.refresh_themes();
                match result {
                    Ok(source) => {
                        let Ok(draft) = &self.draft else {
                            return (Task::none(), None);
                        };
                        self.restart_needed |= draft.font_changed();
                        let networks_changed = draft.networks_changed();
                        self.draft = Draft::parse(source);
                        self.pending_themes.clear();
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
                }
            }
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
                self.preview = None;
                self.pending_themes.clear();
                self.draft = Draft::load();
                self.refresh_themes();
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
                        self.preference_group = 0;
                        self.browser = None;
                        self.confirm_remove = false;
                    }
                    Message::PreferenceGroup(index) => {
                        self.preference_group = index;
                    }
                    Message::Preference(index, value) => {
                        if let Some(field) =
                            draft.preferences.fields.get_mut(index)
                        {
                            field.value = value;
                        }
                    }
                    Message::ReuseCurrentPane => {
                        draft.preferences.reuse_current_pane();
                        self.status = Some(
                            "Current-pane navigation selected. Apply to save."
                                .into(),
                        );
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
                    Message::ThemeMode(value) => {
                        draft.appearance.mode = value;
                        self.preview = None;
                        if value == ThemeMode::Single {
                            let name = draft.appearance.theme.clone();
                            self.select_theme(
                                theme_browser::Target::Single,
                                name,
                            );
                        }
                    }
                    Message::Theme(value) => {
                        self.select_theme(theme_browser::Target::Single, value);
                    }
                    Message::Light(value) => {
                        self.select_theme(theme_browser::Target::Light, value);
                    }
                    Message::Dark(value) => {
                        self.select_theme(theme_browser::Target::Dark, value);
                    }
                    Message::Sidebar(value) => {
                        draft.appearance.sidebar_position = value;
                    }
                    Message::Apply => {
                        if !draft.is_dirty()
                            || self.browser.as_ref().is_some_and(
                                theme_browser::Browser::is_downloading,
                            )
                        {
                            return (Task::none(), None);
                        }
                        if let Err(error) = draft.render() {
                            self.error = Some(error);
                        } else {
                            self.saving = true;
                            self.confirm_close = false;
                            let draft = draft.clone();
                            let pending = self
                                .pending_themes
                                .iter()
                                .filter(|entry| {
                                    entry.file_stem().is_ok_and(|name| {
                                        let used = match draft.appearance.mode {
                                            ThemeMode::Single => {
                                                name == draft.appearance.theme
                                            }
                                            ThemeMode::System => {
                                                name == draft.appearance.light
                                                    || name
                                                        == draft.appearance.dark
                                            }
                                            ThemeMode::Configured => false,
                                        };
                                        used && !self
                                            .installed_themes
                                            .iter()
                                            .any(|theme| {
                                                theme
                                                    .name
                                                    .eq_ignore_ascii_case(&name)
                                            })
                                    })
                                })
                                .cloned()
                                .collect::<Vec<_>>();
                            return (
                                Task::perform(
                                    async move {
                                        tokio::task::spawn_blocking(move || {
                                            for entry in pending {
                                                entry.install(
                                                    &Config::themes_dir(),
                                                )?;
                                            }
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
        let tabs = column![
            row![
                section_tab(
                    "Networks",
                    self.section == Section::Networks,
                    Message::Section(Section::Networks)
                ),
                section_tab(
                    "Appearance",
                    self.section == Section::Appearance,
                    Message::Section(Section::Appearance)
                ),
                section_tab(
                    "Navigation",
                    self.section == Section::Navigation,
                    Message::Section(Section::Navigation)
                ),
                section_tab(
                    "Chat display",
                    self.section == Section::Chat,
                    Message::Section(Section::Chat)
                ),
                section_tab(
                    "Sidebar",
                    self.section == Section::Sidebar,
                    Message::Section(Section::Sidebar)
                ),
            ]
            .spacing(8),
            rule::horizontal(1),
        ]
        .spacing(0);
        let body: Element<_> = match &self.draft {
            Ok(draft) => match self.section {
                Section::Networks => self.networks(draft),
                Section::Navigation => self.preferences(
                    draft,
                    data::config::settings::preferences::Area::Navigation,
                ),
                Section::Chat => self.preferences(
                    draft,
                    data::config::settings::preferences::Area::Chat,
                ),
                Section::Sidebar => self.preferences(
                    draft,
                    data::config::settings::preferences::Area::Sidebar,
                ),
                Section::Appearance => {
                    let content: Element<_> = match &self.browser {
                        Some(browser) => browser.view().map(Message::Browser),
                        None => scrollable(self.appearance(draft))
                            .style(style::scrollable)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into(),
                    };
                    container(content)
                        .padding(12)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(style::pane)
                        .into()
                }
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
        let downloading = self
            .browser
            .as_ref()
            .is_some_and(theme_browser::Browser::is_downloading);
        let mut footer = column![].spacing(10);
        if let Some(error) = &self.error {
            footer = footer.push(text(error).style(theme::text::error));
        }
        if let Some(status) = &self.status
            && !self.saving
        {
            footer = footer.push(text(status).style(theme::text::success));
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
                    } else if dirty && self.preview.is_some() {
                        "Theme preview (unsaved)"
                    } else if dirty {
                        "Unsaved changes"
                    } else if self.preview.is_some() {
                        "Saved theme selected"
                    } else {
                        "No unsaved changes"
                    })
                    .style(theme::text::secondary),
                    space::horizontal(),
                    button("Revert changes")
                        .padding([6, 10])
                        .style(style::button)
                        .on_press_maybe(
                            (dirty && !self.saving).then_some(Message::Revert)
                        ),
                    button(if self.saving { "Saving…" } else { "Apply" })
                        .style(style::primary_button)
                        .padding([6, 16])
                        .on_press_maybe(
                            (dirty && !self.saving && !downloading)
                                .then_some(Message::Apply)
                        ),
                    action("Close", Message::Close),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        let content = container(
            column![heading, tabs, body, rule::horizontal(1), footer]
                .spacing(12),
        )
        .padding(iced::Padding {
            top: 24.0,
            right: 16.0,
            bottom: 16.0,
            left: 16.0,
        })
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
            scrollable(list)
                .style(style::scrollable)
                .height(Length::Fill),
            action("Add network", Message::Add)
        ]
        .spacing(12)
        .width(Length::Fill);
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
                labelled_width(
                    "Port",
                    row![
                        container(input_field(
                            "Automatic",
                            &network.port,
                            Field::Port,
                            false
                        ))
                        .width(104),
                        checkbox(network.use_tls)
                            .label("Use TLS")
                            .on_toggle(Message::Tls)
                            .style(style::checkbox),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    260.0,
                ),
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
                    .on_toggle(Message::Autoconnect)
                    .style(style::checkbox),
                rule::horizontal(1),
                text("Authentication")
                    .font(font::PRIMARY_BOLD.clone())
                    .size(16),
                labelled(
                    "Method",
                    pick_list(
                        Some(network.authentication),
                        authentication,
                        Authentication::to_string
                    )
                    .on_select(Message::Authentication)
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .width(Length::Fill)
                    .padding(6)
                    .into()
                ),
            ]
            .spacing(10);
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
                .style(style::scrollable)
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
        row![
            container(list)
                .padding(8)
                .width(164)
                .height(Length::Fill)
                .style(style::pane),
            container(form)
                .padding(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style::pane),
        ]
        .spacing(12)
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
            labelled(
                "Mode",
                pick_list(Some(a.mode), modes, ThemeMode::to_string)
                    .on_select(Message::ThemeMode)
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .width(Length::Fill)
                    .padding(6)
                    .into()
            ),
        ]
        .spacing(10);
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
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .width(Length::Fill)
                    .padding(6)
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
                        .style(style::pick_list)
                        .menu_style(style::menu)
                        .width(Length::Fill)
                        .padding(6)
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
                        .style(style::pick_list)
                        .menu_style(style::menu)
                        .width(Length::Fill)
                        .padding(6)
                        .into(),
                    ));
            }
            ThemeMode::Configured => {
                form = form.push(text("Your theme configuration is managed in the config file. Choose a mode above to replace it.").style(theme::text::secondary));
            }
        }
        form = form.push(
            container(action(
                "Browse and download themes",
                Message::BrowseThemes,
            ))
            .padding(iced::padding::left(180)),
        );
        form = form.push(text("Theme selections preview throughout Halloy. Apply saves your choice; Revert restores the saved appearance.").style(theme::text::secondary))
            .push(rule::horizontal(1))
            .push(text("Font").size(16).font(font::PRIMARY_BOLD.clone()))
            .push(field("Font family", "Default (Iosevka Term)", &a.font_family, Field::FontFamily, false))
            .push(labelled_width("Text size", row![container(input_field("13", &a.font_size, Field::FontSize, false)).width(96), text("Line height"), container(input_field("1.3", &a.line_height, Field::LineHeight, false)).width(96)].spacing(12).align_y(Alignment::Center).into(), 320.0))
            .push(text("Restart Halloy after applying font changes. Empty fields use Halloy’s defaults.").style(theme::text::secondary))
            .push(rule::horizontal(1))
            .push(text("Layout").size(16).font(font::PRIMARY_BOLD.clone()))
            .push(field("Chat interface scale", "1.0", &a.scale_factor, Field::Scale, false))
            .push(text("Settings stays at its default scale so you can always adjust this value.").style(theme::text::secondary));
        container(form)
            .width(Length::Fill.max(740))
            .padding(iced::padding::right(12))
            .into()
    }
}

fn action<'a, M: Clone + 'a>(label: &'a str, message: M) -> Element<'a, M> {
    button(text(label))
        .padding([6, 10])
        .style(style::button)
        .on_press(message)
        .into()
}

fn section_tab<'a>(
    label: &'a str,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    column![
        button(text(label).font(if selected {
            font::PRIMARY_BOLD.clone()
        } else {
            font::PRIMARY.clone()
        }))
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |theme, status| style::section_tab(theme, status, selected))
        .on_press(message),
        container(space::horizontal())
            .height(2)
            .width(Length::Fill)
            .style(move |theme| style::tab_indicator(theme, selected)),
    ]
    .spacing(0)
    .width(match label {
        "Networks" => 104,
        "Appearance" | "Navigation" => 116,
        "Chat display" => 128,
        _ => 92,
    })
    .into()
}

fn tab<'a>(
    label: &'a str,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    button(text(label))
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme, status| style::tab(theme, status, selected))
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
    let width = match field {
        Field::Port | Field::FontSize | Field::LineHeight | Field::Scale => {
            104.0
        }
        Field::Nickname | Field::Username => 240.0,
        Field::Channels => 420.0,
        Field::Server => 360.0,
        _ => 300.0,
    };
    labelled_width(label, input_field(placeholder, value, field, secret), width)
}

fn input_field<'a>(
    placeholder: &'a str,
    value: &'a str,
    field: Field,
    secret: bool,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .style(input_style)
        .on_input(move |value| Message::Edit(field, value))
        .secure(secret)
        .padding(6)
        .width(Length::Fill)
        .into()
}

fn labelled<'a>(
    label: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    labelled_width(label, control, 320.0)
}

fn labelled_width<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    max_width: f32,
) -> Element<'a, Message> {
    row![
        text(label).width(168),
        container(control).width(Length::Fill.max(max_width))
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn input_style(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    style::input(theme, status)
}

#[cfg(test)]
mod tests {
    use data::appearance::catalog::Entry;

    use super::*;

    fn settings(source: &str) -> Settings {
        let installed_themes = vec![data::Theme::default()];
        let draft = Draft::parse(source.to_owned()).unwrap();
        let (browser, _) = theme_browser::Browser::open(
            &installed_themes,
            draft.appearance.mode,
        );
        Settings {
            window: window::Id::unique(),
            draft: Ok(draft),
            section: Section::Appearance,
            preference_group: 0,
            selected: None,
            themes: vec!["Ferra".into()],
            installed_themes,
            browser: Some(browser),
            preview: None,
            pending_themes: vec![],
            saving: false,
            status: None,
            error: None,
            confirm_close: false,
            confirm_remove: false,
            restart_needed: false,
            quit_requested: false,
        }
    }

    fn entry(id: u64, name: &str, background: iced::Color) -> Entry {
        let mut styles = data::appearance::theme::Styles::default();
        styles.general.background = background;
        Entry {
            id,
            name: name.into(),
            styles,
        }
    }

    #[test]
    fn gallery_selection_previews_uninstalled_theme_and_enables_apply() {
        let mut settings = settings("theme = 'Ferra'\n");
        let current = Theme::default();
        let _ =
            settings.update(Message::Browser(theme_browser::Message::Loaded(
                Ok(vec![entry(1, "New theme", iced::Color::WHITE)]),
            )));
        assert!(!settings.draft.as_ref().unwrap().is_dirty());
        assert!(std::ptr::eq(settings.preview_theme(&current), &current));
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(1)));
        let draft = settings.draft.as_ref().unwrap();
        assert!(draft.is_dirty());
        assert_eq!(draft.appearance.theme, "New theme");
        assert_eq!(
            settings.preview_theme(&current).styles().general.background,
            iced::Color::WHITE
        );
        assert_eq!(settings.pending_themes.len(), 1);
        assert_eq!(settings.installed_themes.len(), 1);
        assert!(draft.render().unwrap().contains("New theme"));
        let _ = settings.update(Message::Close);
        assert!(settings.confirm_close);
        let (_, event) = settings.update(Message::Discard);
        assert!(matches!(event, Some(Event::Close)));
    }

    #[test]
    fn gallery_system_selections_keep_both_slots_and_preview_last_choice() {
        let mut settings =
            settings("theme = { light = 'Ferra', dark = 'Ferra' }\n");
        let _ = settings.update(Message::Browser(
            theme_browser::Message::Loaded(Ok(vec![
                entry(1, "New light", iced::Color::WHITE),
                entry(2, "New dark", iced::Color::BLACK),
            ])),
        ));
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(1)));
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(2)));
        let draft = settings.draft.as_ref().unwrap();
        assert_eq!(draft.appearance.mode, ThemeMode::System);
        assert_eq!(draft.appearance.light, "New light");
        assert_eq!(draft.appearance.dark, "New dark");
        assert_eq!(settings.pending_themes.len(), 2);
        assert_eq!(
            settings
                .preview_theme(&Theme::default())
                .styles()
                .general
                .background,
            iced::Color::BLACK
        );
    }

    #[test]
    fn selecting_saved_theme_leaves_apply_disabled_and_saved_reload_ends_preview()
     {
        let mut settings = settings("theme = 'Ferra'\n");
        let _ = settings.update(Message::Theme("Ferra".into()));
        assert!(!settings.draft.as_ref().unwrap().is_dirty());
        assert!(settings.preview.is_some());
        settings.application_succeeded();
        let current = Theme::default();
        assert!(std::ptr::eq(settings.preview_theme(&current), &current));
    }
}
