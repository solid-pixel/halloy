use data::Config;
use data::config::settings::{
    Authentication, Network, Settings as Draft, ThemeMode, ValidationError,
    ValidationTarget,
};
use iced::widget::{
    column, container, responsive, row, rule, scrollable, space, text,
    text_input,
};
use iced::{Alignment, Length, Size, Task, Vector};

use crate::widget::{Element, key_press};
use crate::{Theme, font, open_url, theme, window};

mod keyboard;
use keyboard::{button, checkbox, pick_list};
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

impl Field {
    fn id(self) -> String {
        format!("settings-field-{self:?}")
    }
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
    validation_error: Option<ValidationError>,
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
                validation_error: None,
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
        let name = self
            .installed_themes
            .iter()
            .find(|theme| theme.name == name)
            .or_else(|| {
                self.installed_themes
                    .iter()
                    .find(|theme| theme.name.eq_ignore_ascii_case(&name))
            })
            .map_or(name.clone(), |theme| theme.name.clone());
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
            .find(|theme| theme.name == name)
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
        proxy: Option<&data::config::Proxy>,
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
                let (browser, task) = theme_browser::Browser::open(
                    &self.installed_themes,
                    mode,
                    proxy.cloned(),
                );
                self.browser = Some(browser);
                return (
                    Task::batch([
                        task.map(Message::Browser),
                        keyboard::focus("settings-gallery-search"),
                    ]),
                    None,
                );
            }
            Message::Browser(message) => {
                let Some(browser) = &mut self.browser else {
                    return (Task::none(), None);
                };
                let (task, event) = browser.update(message, proxy.cloned());
                let mut focus = Task::none();
                match event {
                    Some(theme_browser::Event::Back) => {
                        self.browser = None;
                        focus = keyboard::focus(
                            "settings-action-Browse and download themes",
                        );
                    }
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
                            if !self
                                .themes
                                .iter()
                                .any(|theme| theme.eq_ignore_ascii_case(&name))
                            {
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
                return (
                    Task::batch([task.map(Message::Browser), focus]),
                    None,
                );
            }

            Message::Focus(previous) => {
                return (keyboard::step(previous), None);
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
                        self.validation_error = None;
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
                    return (keyboard::step(false), None);
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
                self.validation_error = None;
                self.status = None;
                self.confirm_close = false;
                self.confirm_remove = false;
            }
            message => {
                let Ok(draft) = &mut self.draft else {
                    return (Task::none(), None);
                };
                self.status = None;
                match message {
                    Message::Section(section) => {
                        self.section = section;
                        self.preference_group = 0;
                        self.browser = None;
                        self.confirm_remove = false;
                        return (
                            keyboard::focus(format!(
                                "settings-section-{section:?}"
                            )),
                            None,
                        );
                    }
                    Message::PreferenceGroup(index) => {
                        self.preference_group = index;
                        return (
                            keyboard::focus(format!("settings-group-{index}")),
                            None,
                        );
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
                        return (
                            keyboard::focus(format!(
                                "settings-network-{index}"
                            )),
                            None,
                        );
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
                        return (keyboard::focus(Field::Name.id()), None);
                    }
                    Message::Remove => {
                        self.confirm_remove = true;
                        return (keyboard::step(false), None);
                    }
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
                        if let Some(error) = draft.validation_error() {
                            let task = self.reveal_validation(error);
                            return (task, None);
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
        if self.validation_error.is_some() {
            self.validation_error =
                self.draft.as_ref().ok().and_then(Draft::validation_error);
            self.error = self
                .validation_error
                .as_ref()
                .map(|error| error.message.clone());
        }
        (Task::none(), None)
    }

    fn reveal_validation(&mut self, error: ValidationError) -> Task<Message> {
        use data::config::settings::preferences::Area;
        let id = match error.target {
            ValidationTarget::Network { index, field } => {
                self.section = Section::Networks;
                self.selected = Some(index);
                match field {
                    "name" => Field::Name,
                    "server" => Field::Server,
                    "nickname" => Field::Nickname,
                    "port" => Field::Port,
                    "channels" => Field::Channels,
                    "password" => Field::Password,
                    _ => Field::Name,
                }
                .id()
            }
            ValidationTarget::Appearance(field) => {
                self.section = Section::Appearance;
                match field {
                    "font_size" => Field::FontSize,
                    "line_height" => Field::LineHeight,
                    _ => Field::Scale,
                }
                .id()
            }
            ValidationTarget::Preference(index) => {
                if let Ok(draft) = &self.draft
                    && let Some(field) = draft.preferences.fields.get(index)
                {
                    self.section = match field.spec.area {
                        Area::Navigation => Section::Navigation,
                        Area::Chat => Section::Chat,
                        Area::Sidebar => Section::Sidebar,
                    };
                    self.preference_group =
                        preferences::groups(draft, field.spec.area)
                            .iter()
                            .position(|group| *group == field.spec.group)
                            .unwrap_or(0);
                }
                format!("settings-preference-{index}")
            }
        };
        self.browser = None;
        self.confirm_close = false;
        self.confirm_remove = false;
        self.error = Some(error.message.clone());
        self.validation_error = Some(error);
        keyboard::focus(id)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let heading = row![
            text("Settings").size(20).font(font::PRIMARY_BOLD.clone()),
            space::horizontal(),
            action("Open config file", Message::OpenFile)
        ]
        .align_y(Alignment::Center);
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
                            .id("settings-form-scroll")
                            .style(style::scrollable)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into(),
                    };
                    container(content)
                        .padding(iced::padding::right(8))
                        .width(Length::Fill)
                        .height(Length::Fill)
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
        let confirming = self.confirm_close || self.confirm_remove;
        let heading: Element<_> = if confirming {
            keyboard::inactive(heading)
        } else {
            heading.into()
        };
        let navigation = if confirming {
            keyboard::inactive(self.navigation())
        } else {
            self.navigation()
        };
        let body = if self.confirm_close {
            keyboard::inactive(body)
        } else {
            body
        };
        let footer: Element<_> = if self.confirm_remove {
            keyboard::inactive(footer)
        } else {
            footer.into()
        };
        let content = container(
            column![
                heading,
                row![navigation, body].spacing(16).height(Length::Fill),
                rule::horizontal(1),
                footer
            ]
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
        .style(theme::container::root)
        .id(keyboard::ROOT);
        let escape = self.escape_message();
        keyboard::shortcuts(content, move |key, modifiers| {
            if key.as_ref() == key_press::Key::Named(key_press::Named::Escape) {
                Some(escape.clone())
            } else if key.as_ref()
                == key_press::Key::Named(key_press::Named::Tab)
            {
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

    fn escape_message(&self) -> Message {
        if self.confirm_close {
            Message::KeepEditing
        } else if self.confirm_remove {
            Message::CancelRemove
        } else if self.browser.is_some() {
            Message::Browser(theme_browser::Message::Back)
        } else {
            Message::Close
        }
    }

    fn navigation(&self) -> Element<'_, Message> {
        let mut navigation = column![].spacing(4);
        for (section, label) in [
            (Section::Networks, "Networks"),
            (Section::Appearance, "Appearance"),
            (Section::Navigation, "Navigation"),
            (Section::Chat, "Chat display"),
            (Section::Sidebar, "Sidebar"),
        ] {
            let selected = self.section == section;
            navigation = navigation.push(section_tab(
                label,
                selected,
                Message::Section(section),
            ));
            if !selected {
                continue;
            }
            let Ok(draft) = &self.draft else {
                continue;
            };
            let mut children = column![].spacing(2);
            if let Some(area) = preference_area(section) {
                for (index, group) in
                    preferences::groups(draft, area).iter().enumerate()
                {
                    let label = if *group == "Highlights and typing" {
                        "Highlights & typing"
                    } else {
                        group
                    };
                    children = children.push(tab(
                        label,
                        self.preference_group == index,
                        Message::PreferenceGroup(index),
                    ));
                }
            } else {
                continue;
            }
            navigation =
                navigation.push(container(children).padding(iced::Padding {
                    left: 12.0,
                    bottom: 8.0,
                    ..Default::default()
                }));
        }
        container(
            scrollable(navigation)
                .id("settings-navigation-scroll")
                .style(style::scrollable)
                .height(Length::Fill),
        )
        .padding(8)
        .width(184)
        .height(Length::Fill)
        .style(style::pane)
        .into()
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
        let list = if self.confirm_remove {
            keyboard::inactive(list)
        } else {
            list.into()
        };
        let add = if self.confirm_remove {
            keyboard::inactive(action("Add network", Message::Add))
        } else {
            action("Add network", Message::Add)
        };
        row![
            container(column![
                scrollable(list).id("settings-networks-scroll").style(style::scrollable).height(Length::Fill),
                add,
            ].spacing(12))
                .padding(8)
                .width(144)
                .height(Length::Fill)
                .style(style::pane),
            responsive(move |size| self.network_form(draft, size.width < 440.0)),
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn network_form<'a>(
        &'a self,
        draft: &'a Draft,
        stacked: bool,
    ) -> Element<'a, Message> {
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
            let connection = vec![
                self.field_with_layout(
                    "Network name",
                    "e.g. Libera",
                    &network.name,
                    Field::Name,
                    false,
                    stacked,
                ),
                self.field_with_layout(
                    "Server",
                    "irc.libera.chat",
                    &network.server,
                    Field::Server,
                    false,
                    stacked,
                ),
                self.mark_error(
                    Field::Port,
                    labelled_with_layout(
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
                        stacked,
                    ),
                ),
                self.field_with_layout(
                    "Nickname",
                    "Your nickname",
                    &network.nickname,
                    Field::Nickname,
                    false,
                    stacked,
                ),
                self.field_with_layout(
                    "Join channels on connect",
                    "#halloy, #rust",
                    &network.channels,
                    Field::Channels,
                    false,
                    stacked,
                ),
                checkbox(network.autoconnect)
                    .label("Connect automatically on launch")
                    .on_toggle(Message::Autoconnect)
                    .style(style::checkbox)
                    .into(),
            ];
            let mut authentication_rows = vec![labelled_with_layout(
                "Method",
                pick_list(
                    Some(network.authentication),
                    authentication,
                    Authentication::to_string,
                )
                .on_select(Message::Authentication)
                .style(style::pick_list)
                .menu_style(style::menu)
                .width(Length::Fill)
                .padding(6)
                .into(),
                320.0,
                stacked,
            )];
            match network.authentication {
                Authentication::Sasl => {
                    authentication_rows.push(self.field_with_layout(
                        "Account name",
                        "Use nickname",
                        &network.username,
                        Field::Username,
                        false,
                        stacked,
                    ));
                }
                Authentication::Advanced => {
                    authentication_rows.push(text("Certificate, keyring and command settings are managed in the config file. Choosing another method replaces this network’s authentication settings.").style(theme::text::secondary).into());
                }
                _ => {}
            }
            if matches!(
                network.authentication,
                Authentication::Sasl | Authentication::NickServ
            ) {
                authentication_rows.push(column![self.field_with_layout("Password", "Account password", &network.password, Field::Password, true, stacked), text("Passwords entered here are stored in config.toml. Use the config file for keyring or password-command authentication.").size(12).style(theme::text::secondary)].spacing(4).into());
            }
            let mut form = column![
                text(&network.name)
                    .font(font::PRIMARY_BOLD.clone())
                    .size(16),
                settings_group("Connection", connection),
                settings_group("Authentication", authentication_rows),
            ]
            .spacing(16);
            form = form.push(text("Existing connections use changed server and authentication settings after reconnecting.").style(theme::text::secondary));
            if self.confirm_remove {
                let background: Element<_> = keyboard::inactive(form);
                form = column![background]
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
                .id("settings-form-scroll")
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
        container(form)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn field<'a>(
        &'a self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        field: Field,
        secret: bool,
    ) -> Element<'a, Message> {
        self.field_with_layout(label, placeholder, value, field, secret, false)
    }

    fn field_with_layout<'a>(
        &'a self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        field: Field,
        secret: bool,
        stacked: bool,
    ) -> Element<'a, Message> {
        self.mark_error(
            field,
            field_with_layout(
                label,
                placeholder,
                value,
                field,
                secret,
                stacked,
            ),
        )
    }

    fn mark_error<'a>(
        &'a self,
        field: Field,
        control: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let name = match field {
            Field::Name => "name",
            Field::Server => "server",
            Field::Port => "port",
            Field::Nickname => "nickname",
            Field::Channels => "channels",
            Field::Username => "username",
            Field::Password => "password",
            Field::FontFamily => "font_family",
            Field::FontSize => "font_size",
            Field::LineHeight => "line_height",
            Field::Scale => "scale_factor",
        };
        let error = self.validation_error.as_ref().filter(|error| match error
            .target
        {
            ValidationTarget::Network { index, field } => {
                self.selected == Some(index) && field == name
            }
            ValidationTarget::Appearance(field) => field == name,
            _ => false,
        });
        let mut content = column![control].spacing(4);
        if let Some(error) = error {
            content = content
                .push(text(&error.message).size(12).style(theme::text::error));
        }
        content.into()
    }

    fn appearance<'a>(&'a self, draft: &'a Draft) -> Element<'a, Message> {
        let a = &draft.appearance;
        let mut modes = vec![ThemeMode::Single, ThemeMode::System];
        if draft.has_configured_theme() {
            modes.push(ThemeMode::Configured);
        }
        let mut theme_rows = vec![labelled(
            "Mode",
            pick_list(Some(a.mode), modes, ThemeMode::to_string)
                .on_select(Message::ThemeMode)
                .style(style::pick_list)
                .menu_style(style::menu)
                .width(Length::Fill)
                .padding(6)
                .into(),
        )];
        match a.mode {
            ThemeMode::Single => {
                theme_rows.push(labelled(
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
                theme_rows.push(labelled(
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
                ));
                theme_rows.push(labelled(
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
                theme_rows.push(text("Your theme configuration is managed in the config file. Choose a mode above to replace it.").style(theme::text::secondary).into());
            }
        }
        theme_rows.push(column![
            action("Browse and download themes", Message::BrowseThemes),
            text("Theme selections preview throughout Halloy. Apply saves your choice; Revert restores the saved appearance.").size(12).style(theme::text::secondary)
        ].spacing(6).into());
        let form = column![
            text("Appearance").size(16).font(font::PRIMARY_BOLD.clone()),
            settings_group("Theme", theme_rows),
            settings_group("Font", vec![
                self.field("Font family", "Default (Iosevka Term)", &a.font_family, Field::FontFamily, false),
                column![self.mark_error(Field::LineHeight, self.mark_error(Field::FontSize, labelled_width("Text size", row![container(input_field("13", &a.font_size, Field::FontSize, false)).width(88), text("Line height"), container(input_field("1.3", &a.line_height, Field::LineHeight, false)).width(88)].spacing(12).align_y(Alignment::Center).into(), 320.0))), text("Restart Halloy after applying font changes. Empty fields use Halloy’s defaults.").size(12).style(theme::text::secondary)].spacing(4).into(),
            ]),
            settings_group("Layout", vec![
                column![self.field("Chat interface scale", "1.0", &a.scale_factor, Field::Scale, false), text("Settings stays at its default scale so you can always adjust this value.").size(12).style(theme::text::secondary)].spacing(4).into(),
            ]),
        ].spacing(16);
        container(form)
            .width(Length::Fill.max(740))
            .padding(iced::padding::right(12))
            .into()
    }
}

fn preference_area(
    section: Section,
) -> Option<data::config::settings::preferences::Area> {
    use data::config::settings::preferences::Area;
    match section {
        Section::Navigation => Some(Area::Navigation),
        Section::Chat => Some(Area::Chat),
        Section::Sidebar => Some(Area::Sidebar),
        _ => None,
    }
}

fn settings_group<'a>(
    title: &'a str,
    rows: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(title).font(font::PRIMARY_BOLD.clone()),
        settings_rows(rows)
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}

fn settings_rows<'a>(rows: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    let mut content = column![].spacing(0);
    for (index, row) in rows.into_iter().enumerate() {
        if index > 0 {
            content = content.push(
                container(rule::horizontal(1))
                    .padding(iced::padding::horizontal(12)),
            );
        }
        content =
            content.push(container(row).padding([8, 12]).width(Length::Fill));
    }
    container(content)
        .width(Length::Fill)
        .style(style::pane)
        .into()
}

fn action<'a, M: Clone + 'a>(label: &'a str, message: M) -> Element<'a, M> {
    button(text(label))
        .id(format!("settings-action-{label}"))
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
    button(text(label).font(if selected {
        font::PRIMARY_BOLD.clone()
    } else {
        font::PRIMARY.clone()
    }))
    .id(match message {
        Message::Section(section) => format!("settings-section-{section:?}"),
        _ => unreachable!(),
    })
    .padding([6, 8])
    .width(Length::Fill)
    .style(move |theme, status| style::section_tab(theme, status, selected))
    .on_press(message)
    .into()
}

fn tab<'a>(
    label: &'a str,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    button(text(label))
        .id(match message {
            Message::Select(index) => format!("settings-network-{index}"),
            Message::PreferenceGroup(index) => {
                format!("settings-group-{index}")
            }
            _ => unreachable!(),
        })
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme, status| style::tab(theme, status, selected))
        .on_press(message)
        .into()
}

fn field_with_layout<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    field: Field,
    secret: bool,
    stacked: bool,
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
    labelled_with_layout(
        label,
        input_field(placeholder, value, field, secret),
        width,
        stacked,
    )
}

fn input_field<'a>(
    placeholder: &'a str,
    value: &'a str,
    field: Field,
    secret: bool,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .id(field.id())
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
    labelled_with_layout(label, control, max_width, false)
}

fn labelled_with_layout<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    max_width: f32,
    stacked: bool,
) -> Element<'a, Message> {
    if stacked {
        return column![
            text(label),
            container(control).width(Length::Fill.max(max_width))
        ]
        .spacing(4)
        .width(Length::Fill)
        .into();
    }
    row![
        text(label).width(160),
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
            None,
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
            validation_error: None,
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
            settings.update(
                Message::Browser(theme_browser::Message::Loaded(Ok(vec![
                    entry(1, "New theme", iced::Color::WHITE),
                ]))),
                None,
            );
        assert!(!settings.draft.as_ref().unwrap().is_dirty());
        assert!(std::ptr::eq(settings.preview_theme(&current), &current));
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(1)), None);
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
        let _ = settings.update(Message::Close, None);
        assert!(settings.confirm_close);
        let (_, event) = settings.update(Message::Discard, None);
        assert!(matches!(event, Some(Event::Close)));
    }

    #[test]
    fn gallery_system_selections_keep_both_slots_and_preview_last_choice() {
        let mut settings =
            settings("theme = { light = 'Ferra', dark = 'Ferra' }\n");
        let _ = settings.update(
            Message::Browser(theme_browser::Message::Loaded(Ok(vec![
                entry(1, "New light", iced::Color::WHITE),
                entry(2, "New dark", iced::Color::BLACK),
            ]))),
            None,
        );
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(1)), None);
        let _ = settings
            .update(Message::Browser(theme_browser::Message::Select(2)), None);
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
        let _ = settings.update(Message::Theme("Ferra".into()), None);
        assert!(!settings.draft.as_ref().unwrap().is_dirty());
        assert!(settings.preview.is_some());
        settings.application_succeeded();
        let current = Theme::default();
        assert!(std::ptr::eq(settings.preview_theme(&current), &current));
    }
    #[test]
    fn gallery_uses_installed_filename_casing_for_every_theme_slot() {
        let mut settings = settings("theme = 'Ferra'\n");
        settings.installed_themes.push(data::Theme::new(
            "dracula".into(),
            data::appearance::theme::Styles::default(),
        ));
        for target in [
            theme_browser::Target::Single,
            theme_browser::Target::Light,
            theme_browser::Target::Dark,
        ] {
            settings.select_theme(target, "Dracula".into());
        }
        let draft = settings.draft.as_ref().unwrap();
        assert_eq!(draft.appearance.theme, "dracula");
        assert_eq!(draft.appearance.light, "dracula");
        assert_eq!(draft.appearance.dark, "dracula");
        assert!(draft.render().unwrap().contains("dracula"));
        assert!(settings.preview.is_some());
    }

    #[test]
    fn apply_reveals_invalid_appearance_and_navigation_retains_error_until_fixed()
     {
        let mut settings = settings("theme = 'Ferra'\n");
        let _ =
            settings.update(Message::Edit(Field::FontSize, "0".into()), None);
        let _ = settings.update(Message::Section(Section::Navigation), None);
        let _ = settings.update(Message::Apply, None);
        assert_eq!(settings.section, Section::Appearance);
        assert!(settings.browser.is_none());
        assert_eq!(
            settings.validation_error.as_ref().unwrap().target,
            ValidationTarget::Appearance("font_size")
        );
        let error = settings.error.clone();
        let _ = settings.update(Message::Section(Section::Sidebar), None);
        assert_eq!(settings.error, error);
        let _ =
            settings.update(Message::Edit(Field::FontSize, "14".into()), None);
        assert!(settings.error.is_none());
        assert!(settings.validation_error.is_none());
    }

    #[test]
    fn apply_selects_invalid_network_and_preference_group() {
        let mut settings = settings(
            "[servers.Review]\nserver = 'localhost'\nnickname = 'review'\nautoconnect = false\n",
        );
        settings.selected = Some(0);
        let _ = settings.update(Message::Edit(Field::Port, "0".into()), None);
        let _ = settings.update(Message::Apply, None);
        assert_eq!(settings.section, Section::Networks);
        assert_eq!(settings.selected, Some(0));
        assert_eq!(
            settings.validation_error.as_ref().unwrap().target,
            ValidationTarget::Network {
                index: 0,
                field: "port"
            }
        );
        let _ =
            settings.update(Message::Edit(Field::Port, "6667".into()), None);
        let index = settings
            .draft
            .as_ref()
            .unwrap()
            .preferences
            .fields
            .iter()
            .position(|field| field.spec.path == "buffer.timestamp.format")
            .unwrap();
        let _ = settings.update(Message::Preference(index, "%Q".into()), None);
        let _ = settings.update(Message::Apply, None);
        assert_eq!(settings.section, Section::Chat);
        assert_eq!(settings.preference_group, 0);
        assert_eq!(
            settings.validation_error.as_ref().unwrap().target,
            ValidationTarget::Preference(index)
        );
    }
    #[test]
    fn escape_cancels_confirmation_before_leaving_gallery() {
        let mut settings = settings("theme = 'Ferra'\n");
        settings.confirm_close = true;
        assert!(matches!(settings.escape_message(), Message::KeepEditing));
        let message = settings.escape_message();
        let _ = settings.update(message, None);
        assert!(!settings.confirm_close);
        assert!(settings.browser.is_some());
        assert!(matches!(
            settings.escape_message(),
            Message::Browser(theme_browser::Message::Back)
        ));
    }

    #[test]
    fn distinct_case_sensitive_theme_names_preview_the_exact_saved_theme() {
        let mut settings = settings("theme = 'Ferra'\n");
        let mut light = data::appearance::theme::Styles::default();
        light.general.background = iced::Color::WHITE;
        let mut dark = light;
        dark.general.background = iced::Color::BLACK;
        settings.installed_themes = vec![
            data::Theme::new("Dracula".into(), light),
            data::Theme::new("dracula".into(), dark),
        ];
        settings.select_theme(theme_browser::Target::Single, "dracula".into());
        assert_eq!(
            settings.draft.as_ref().unwrap().appearance.theme,
            "dracula"
        );
        assert_eq!(
            settings
                .preview
                .as_ref()
                .unwrap()
                .styles()
                .general
                .background,
            iced::Color::BLACK
        );
    }
}
