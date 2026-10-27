use std::collections::HashSet;

use data::Config;
use data::appearance::catalog::{self, Entry};
use data::config::settings::ThemeMode;
use iced::widget::{
    button, column, pick_list, row, rule, scrollable, space, text, text_input,
};
use iced::{Alignment, Length, Task};

use super::{action, input_style, style};
use crate::widget::Element;
use crate::{font, open_url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Light,
    Dark,
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::All => "All themes",
            Self::Light => "Light themes",
            Self::Dark => "Dark themes",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Single,
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<Vec<Entry>, String>),
    Search(String),
    Filter(Filter),
    Select(u64),
    Download,
    Downloaded(u64, Result<data::Theme, String>),
    Use(Target),
    Back,
    Retry,
    Website,
}

pub enum Event {
    Back,
    Preview(Target, Entry),
    Installed(data::Theme),
    Use(Target, String),
}

pub struct Browser {
    entries: Result<Vec<Entry>, String>,
    loading: bool,
    query: String,
    filter: Filter,
    selected: Option<u64>,
    installed: HashSet<String>,
    downloading: Option<u64>,
    notice: Option<String>,
    error: Option<String>,
    system_mode: bool,
}

impl Browser {
    pub fn open(
        installed: &[data::Theme],
        mode: ThemeMode,
    ) -> (Self, Task<Message>) {
        (
            Self {
                entries: Ok(vec![]),
                loading: true,
                query: String::new(),
                filter: Filter::All,
                selected: None,
                installed: installed
                    .iter()
                    .map(|theme| theme.name.to_lowercase())
                    .collect(),
                downloading: None,
                notice: None,
                error: None,
                system_mode: mode == ThemeMode::System,
            },
            Task::perform(catalog::fetch(), Message::Loaded),
        )
    }

    pub fn refresh_installed(&mut self, themes: &[data::Theme]) {
        self.installed = themes
            .iter()
            .map(|theme| theme.name.to_lowercase())
            .collect();
    }

    fn matches(&self, entry: &Entry) -> bool {
        entry
            .name
            .to_lowercase()
            .contains(&self.query.trim().to_lowercase())
            && match self.filter {
                Filter::All => true,
                Filter::Light => entry.is_light(),
                Filter::Dark => !entry.is_light(),
            }
    }

    fn selected(&self) -> Option<&Entry> {
        self.entries.as_ref().ok()?.iter().find(|entry| {
            Some(entry.id) == self.selected && self.matches(entry)
        })
    }

    pub fn is_downloading(&self) -> bool {
        self.downloading.is_some()
    }

    fn retain_selection(&mut self) {
        if self.selected().is_none() {
            self.selected = None;
        }
    }

    fn is_installed(&self, entry: &Entry) -> bool {
        entry
            .file_stem()
            .is_ok_and(|name| self.installed.contains(&name.to_lowercase()))
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::Loaded(result) => {
                self.loading = false;
                self.entries = result;
                self.retain_selection();
            }
            Message::Search(query) => {
                self.query = query;
                self.notice = None;
                self.error = None;
                self.retain_selection();
            }
            Message::Filter(filter) => {
                self.filter = filter;
                self.notice = None;
                self.error = None;
                self.retain_selection();
            }
            Message::Select(id) => {
                self.selected = Some(id);
                self.error = None;
                self.notice = None;
                if let Some(entry) = self.selected().cloned() {
                    let target = if self.system_mode {
                        if entry.is_light() {
                            Target::Light
                        } else {
                            Target::Dark
                        }
                    } else {
                        Target::Single
                    };
                    return (Task::none(), Some(Event::Preview(target, entry)));
                }
            }
            Message::Retry => {
                self.loading = true;
                return (
                    Task::perform(catalog::fetch(), Message::Loaded),
                    None,
                );
            }
            Message::Website => {
                let _ = open_url::open(data::environment::THEME_WEBSITE);
            }
            Message::Back => return (Task::none(), Some(Event::Back)),
            Message::Use(target) => {
                if let Some(entry) =
                    self.selected().filter(|entry| self.is_installed(entry))
                    && let Ok(name) = entry.file_stem()
                {
                    return (Task::none(), Some(Event::Use(target, name)));
                }
            }
            Message::Download => {
                if self.downloading.is_some() {
                    return (Task::none(), None);
                }
                if let Some(entry) = self
                    .selected()
                    .filter(|entry| !self.is_installed(entry))
                    .cloned()
                {
                    let id = entry.id;
                    self.downloading = Some(id);
                    self.error = None;
                    self.notice = None;
                    return (
                        Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    entry.install(&Config::themes_dir())
                                })
                                .await
                                .map_err(|error| error.to_string())
                                .and_then(|result| result)
                            },
                            move |result| Message::Downloaded(id, result),
                        ),
                        None,
                    );
                }
            }
            Message::Downloaded(id, result) => {
                if self.downloading != Some(id) {
                    return (Task::none(), None);
                }
                self.downloading = None;
                match result {
                    Ok(theme) => {
                        self.installed.insert(theme.name.to_lowercase());
                        self.notice = Some(format!(
                            "{} downloaded. It is now available in Appearance.",
                            theme.name
                        ));
                        return (Task::none(), Some(Event::Installed(theme)));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
        }
        (Task::none(), None)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let heading = row![
            action("Back", Message::Back),
            text("Browse themes")
                .size(16)
                .font(font::PRIMARY_BOLD.clone()),
            space::horizontal(),
            action("Gallery website", Message::Website)
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        let controls = row![
            text_input("Search themes…", &self.query)
                .style(input_style)
                .padding(8)
                .on_input(Message::Search),
            pick_list(
                Some(self.filter),
                vec![Filter::All, Filter::Light, Filter::Dark],
                Filter::to_string
            )
            .on_select(Message::Filter)
            .style(style::pick_list)
            .menu_style(style::menu)
            .padding(8)
            .width(160),
        ]
        .spacing(12);
        let body: Element<_> = if self.loading {
            column![
                text("Loading themes from the Halloy gallery…"),
                rule::horizontal(1),
                text("Theme names will appear here.").style(secondary_text)
            ]
            .spacing(16)
            .into()
        } else {
            match &self.entries {
                Err(error) => column![
                    text("Could not load themes")
                        .font(font::PRIMARY_BOLD.clone()),
                    text(error).style(error_text),
                    action("Try again", Message::Retry)
                ]
                .spacing(12)
                .into(),
                Ok(entries) => {
                    let filtered = entries
                        .iter()
                        .filter(|entry| self.matches(entry))
                        .collect::<Vec<_>>();
                    if filtered.is_empty() {
                        column![
                            text("No themes match your search."),
                            action(
                                "Clear search",
                                Message::Search(String::new())
                            )
                        ]
                        .spacing(12)
                        .into()
                    } else {
                        let mut list = column![].spacing(4);
                        for entry in &filtered {
                            let selected = self.selected == Some(entry.id);
                            list = list.push(
                                button(
                                    row![
                                        text(&entry.name).width(Length::Fill),
                                        text(if entry.is_light() {
                                            "Light"
                                        } else {
                                            "Dark"
                                        })
                                        .width(80),
                                        text(if self.is_installed(entry) {
                                            "Installed"
                                        } else {
                                            ""
                                        })
                                        .width(100),
                                    ]
                                    .spacing(12)
                                    .align_y(Alignment::Center),
                                )
                                .padding([8, 10])
                                .width(Length::Fill)
                                .on_press(Message::Select(entry.id))
                                .style(
                                    move |theme, status| {
                                        style::tab(theme, status, selected)
                                    },
                                ),
                            );
                        }
                        let mut body = column![
                            text(format!(
                                "{} {}",
                                filtered.len(),
                                if filtered.len() == 1 {
                                    "theme"
                                } else {
                                    "themes"
                                }
                            ))
                            .style(secondary_text),
                            scrollable(list)
                                .style(style::scrollable)
                                .height(Length::Fill),
                        ]
                        .spacing(12)
                        .height(Length::Fill);
                        if let Some(entry) = self.selected() {
                            body = body
                                .push(rule::horizontal(1))
                                .push(self.details(entry));
                        }
                        body.into()
                    }
                }
            }
        };
        column![
            heading,
            text(
                "Select to preview throughout Halloy. Apply saves your choice."
            )
            .style(secondary_text),
            controls,
            body
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn details<'a>(&'a self, entry: &'a Entry) -> Element<'a, Message> {
        let installed = self.is_installed(entry);
        let controls: Element<_> = if installed {
            if self.system_mode {
                row![
                    action("Use as light", Message::Use(Target::Light)),
                    action("Use as dark", Message::Use(Target::Dark))
                ]
                .spacing(8)
                .into()
            } else {
                text("Installed").style(secondary_text).into()
            }
        } else {
            button(if self.downloading == Some(entry.id) {
                "Downloading…"
            } else {
                "Download theme"
            })
            .padding([8, 12])
            .style(style::button)
            .on_press_maybe(
                self.downloading.is_none().then_some(Message::Download),
            )
            .into()
        };
        let mut details = column![
            row![
                text(&entry.name)
                    .size(16)
                    .font(font::PRIMARY_BOLD.clone())
                    .width(Length::Fill),
                controls,
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        ]
        .spacing(8);
        if self.system_mode {
            details = details.push(text("Light and dark themes update the matching system appearance.").style(secondary_text));
        }
        if let Some(error) = &self.error {
            details = details.push(text(error).style(error_text));
        }
        if let Some(notice) = &self.notice {
            details = details.push(text(notice).style(success_text));
        }
        details.into()
    }
}

fn readable_text(
    theme: &crate::Theme,
    color: iced::Color,
) -> iced::widget::text::Style {
    let background = theme.styles().general.background;
    let primary = theme.styles().text.primary.color;
    let color = (0u8..=10)
        .map(|step| color.mix(primary, f32::from(step) / 10.0))
        .find(|candidate| candidate.relative_contrast(background) >= 4.5)
        .unwrap_or(primary);
    iced::widget::text::Style { color: Some(color) }
}

fn secondary_text(theme: &crate::Theme) -> iced::widget::text::Style {
    readable_text(theme, theme.styles().text.secondary.color)
}
fn success_text(theme: &crate::Theme) -> iced::widget::text::Style {
    readable_text(theme, theme.styles().text.success.color)
}
fn error_text(theme: &crate::Theme) -> iced::widget::text::Style {
    readable_text(theme, theme.styles().text.error.color)
}
