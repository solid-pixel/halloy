use std::collections::HashSet;

use data::Config;
use data::appearance::catalog::{self, Entry};
use data::config::settings::ThemeMode;
use iced::widget::{
    button, column, container, pick_list, row, rule, scrollable, space, text,
    text_input,
};
use iced::{Alignment, Length, Task};

use super::{action, input_style};
use crate::widget::Element;
use crate::{font, open_url, theme};

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

    pub fn has_selection(&self) -> bool {
        self.selected().is_some()
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
            .padding(8)
            .width(160),
        ]
        .spacing(12);
        let body: Element<_> = if self.loading {
            column![
                text("Loading themes from the Halloy gallery…"),
                rule::horizontal(1),
                text("Theme names and previews will appear here.")
                    .style(secondary_text)
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
                        let mut list = column![
                            text(format!(
                                "{} {}",
                                filtered.len(),
                                if filtered.len() == 1 {
                                    "theme"
                                } else {
                                    "themes"
                                }
                            ))
                            .style(secondary_text)
                        ]
                        .spacing(4);
                        for entry in filtered {
                            let selected = self.selected == Some(entry.id);
                            list =
                                list.push(
                                    button(
                                        column![
                                            text(&entry.name),
                                            text(if self.is_installed(entry) {
                                                "Installed"
                                            } else if entry.is_light() {
                                                "Light"
                                            } else {
                                                "Dark"
                                            })
                                            .size(11)
                                            .style(theme::text::primary)
                                        ]
                                        .spacing(3),
                                    )
                                    .padding([8, 10])
                                    .width(Length::Fill)
                                    .on_press(Message::Select(entry.id))
                                    .style(move |theme, status| {
                                        theme::button::primary(
                                            theme, status, selected,
                                        )
                                    }),
                                );
                        }
                        let details = self.selected().map_or_else(
                            || text("Select a theme to preview it.").into(),
                            |entry| self.details(entry),
                        );
                        row![
                            scrollable(list).width(190).height(Length::Fill),
                            rule::vertical(1),
                            container(details).width(Length::Fill)
                        ]
                        .spacing(16)
                        .height(Length::Fill)
                        .into()
                    }
                }
            }
        };
        column![
            heading,
            text("Community themes from themes.halloy.chat · Preview uses sample messages")
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
        let mut details = column![
            text(&entry.name).size(16).font(font::PRIMARY_BOLD.clone()),
            text(if entry.is_light() { "Light theme" } else { "Dark theme" }).style(secondary_text),
            preview(entry),
            text(if self.system_mode {
                "Selecting previews throughout Halloy and chooses the matching light or dark theme. Apply saves your choice."
            } else {
                "Select a theme to preview it throughout Halloy. Apply downloads it if needed and saves your choice."
            }).style(secondary_text),
        ].spacing(12);
        if let Some(error) = &self.error {
            details = details.push(text(error).style(error_text));
        }
        if let Some(notice) = &self.notice {
            details = details.push(text(notice).style(success_text));
        }
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
            .on_press_maybe(
                self.downloading.is_none().then_some(Message::Download),
            )
            .into()
        };
        column![scrollable(details).height(Length::Fill), controls]
            .spacing(12)
            .height(Length::Fill)
            .into()
    }
}

fn preview(entry: &Entry) -> Element<'_, Message> {
    let styles = entry.styles;
    let title = container(text("#halloy").size(12))
        .padding(10)
        .width(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(styles.buffer.background_title_bar.into()),
            text_color: Some(styles.buffer.topic.color),
            ..Default::default()
        });
    let messages = column![
        row![
            text("alex").color(styles.buffer.nickname.color),
            text("Welcome to Halloy.").color(styles.text.primary.color)
        ]
        .spacing(8),
        row![
            text("you").color(styles.text.tertiary.color),
            text("This is how messages will look.")
                .color(styles.text.primary.color)
        ]
        .spacing(8),
        text("https://halloy.chat").color(styles.buffer.url.color),
    ]
    .spacing(12)
    .padding(12);
    container(column![title, messages])
        .width(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(styles.buffer.background.into()),
            border: iced::Border {
                color: styles.buffer.border_selected,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
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
