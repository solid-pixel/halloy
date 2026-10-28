use data::config::settings::Settings as Draft;
use data::config::settings::preferences::{Area, Field, Kind};
use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Alignment, Length};

use super::keyboard::{checkbox, pick_list};
use super::{
    Message, Settings, action, labelled, labelled_width, settings_rows, style,
};
use crate::widget::Element;
use crate::{font, theme};

impl Settings {
    pub(super) fn preferences<'a>(
        &'a self,
        draft: &'a Draft,
        area: Area,
    ) -> Element<'a, Message> {
        let groups = groups(draft, area);
        let selected =
            self.preference_group.min(groups.len().saturating_sub(1));
        let group = groups.get(selected).copied().unwrap_or("");
        let mut form =
            column![text(group).size(16).font(font::PRIMARY_BOLD.clone())]
                .spacing(12);
        if area == Area::Navigation && selected == 0 {
            form = form.push(action("Reuse the current pane for everything", Message::ReuseCurrentPane))
                .push(text("After applying, close extra panes to use one-pane navigation.").style(theme::text::secondary));
        }
        if area == Area::Chat {
            form = form.push(text("Channel-specific settings can override these global defaults.").style(theme::text::secondary));
        }
        let mut rows = vec![];
        if area == Area::Sidebar && selected == 0 {
            rows.push(labelled_width(
                "Position",
                pick_list(
                    Some(draft.appearance.sidebar_position.clone()),
                    vec![
                        "left".to_owned(),
                        "right".to_owned(),
                        "top".to_owned(),
                        "bottom".to_owned(),
                    ],
                    String::clone,
                )
                .on_select(Message::Sidebar)
                .style(style::pick_list)
                .menu_style(style::menu)
                .padding(6)
                .width(Length::Fill)
                .into(),
                160.0,
            ));
        }
        for (index, field) in draft.preferences.fields.iter().enumerate() {
            if field.spec.area == area && field.spec.group == group {
                let error = self.validation_error.as_ref().filter(|error| error.target == data::config::settings::ValidationTarget::Preference(index)).map(|error| error.message.as_str());
                rows.push(control(index, field, error));
            }
        }
        form = form.push(settings_rows(rows));
        container(
            scrollable(form.padding(iced::padding::right(12)))
                .id("settings-form-scroll")
                .style(style::scrollable)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub(super) fn groups(draft: &Draft, area: Area) -> Vec<&'static str> {
    let mut groups = vec![];
    for field in &draft.preferences.fields {
        if field.spec.area == area && !groups.contains(&field.spec.group) {
            groups.push(field.spec.group);
        }
    }
    groups
}

fn control<'a>(
    index: usize,
    field: &'a Field,
    error: Option<&'a str>,
) -> Element<'a, Message> {
    let widget = match field.spec.kind {
        Kind::Boolean if !field.is_custom() => checkbox(field.value == "true")
            .label(field.spec.label)
            .style(style::checkbox)
            .on_toggle(move |value| {
                Message::Preference(index, value.to_string())
            })
            .into(),
        Kind::Choice(choices) => {
            let mut labels = choices
                .iter()
                .map(|choice| choice.label.to_owned())
                .collect::<Vec<_>>();
            let selected = choices
                .iter()
                .find(|choice| choice.value == field.value)
                .map_or("Configured in file", |choice| choice.label)
                .to_owned();
            if field.is_custom() {
                labels.push(selected.clone());
            }
            labelled(
                field.spec.label,
                pick_list(Some(selected), labels, String::clone)
                    .on_select(move |label| {
                        Message::Preference(
                            index,
                            choices
                                .iter()
                                .find(|choice| choice.label == label)
                                .map_or_else(
                                    || field.original_value().to_owned(),
                                    |choice| choice.value.to_owned(),
                                ),
                        )
                    })
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .padding(6)
                    .width(Length::Fill)
                    .into(),
            )
        }
        Kind::Boolean => labelled(
            field.spec.label,
            pick_list(
                Some("Configured in file".to_owned()),
                vec![
                    "Configured in file".to_owned(),
                    "Enabled".to_owned(),
                    "Disabled".to_owned(),
                ],
                String::clone,
            )
            .on_select(move |label| {
                Message::Preference(
                    index,
                    match label.as_str() {
                        "Enabled" => "true".to_owned(),
                        "Disabled" => "false".to_owned(),
                        _ => field.original_value().to_owned(),
                    },
                )
            })
            .style(style::pick_list)
            .menu_style(style::menu)
            .padding(6)
            .width(Length::Fill)
            .into(),
        ),
        _ => {
            let width = match field.spec.kind {
                Kind::Integer { .. } | Kind::Float { .. } => 104.0,
                Kind::Pair => 128.0,
                _ => 240.0,
            };
            let input: Element<'_, Message> =
                text_input("Default", &field.value)
                    .id(format!("settings-preference-{index}"))
                    .style(style::input)
                    .padding(6)
                    .on_input(move |value| Message::Preference(index, value))
                    .width(Length::Fill)
                    .into();
            let (input, width) = if field.spec.help == "Pixels" {
                (
                    row![
                        container(input).width(Length::Fill.max(width)),
                        text("px").style(theme::text::secondary)
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into(),
                    width + 32.0,
                )
            } else {
                (input, width)
            };
            labelled_width(field.spec.label, input, width)
        }
    };
    // Keep the input in the same tree position when validation clears while typing.
    let mut content = column![widget].spacing(4);
    if let Some(error) = error {
        content = content.push(text(error).size(12).style(theme::text::error));
    }
    let widget: Element<'_, Message> = content.into();
    if field.spec.help.is_empty() || field.spec.help == "Pixels" {
        widget
    } else {
        column![
            widget,
            text(field.spec.help).size(12).style(theme::text::secondary)
        ]
        .spacing(4)
        .into()
    }
}
