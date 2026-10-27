use data::config::settings::Settings as Draft;
use data::config::settings::preferences::{Area, Field, Kind};
use iced::Length;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text,
    text_input,
};

use super::{Message, Settings, action, labelled, style};
use crate::widget::Element;
use crate::{font, theme};

impl Settings {
    pub(super) fn preferences<'a>(
        &'a self,
        draft: &'a Draft,
        area: Area,
    ) -> Element<'a, Message> {
        let mut groups = vec![];
        for field in &draft.preferences.fields {
            if field.spec.area == area && !groups.contains(&field.spec.group) {
                groups.push(field.spec.group);
            }
        }
        let selected =
            self.preference_group.min(groups.len().saturating_sub(1));
        let mut navigation = column![].spacing(4);
        for (index, group) in groups.iter().enumerate() {
            navigation = navigation.push(
                button(text(if *group == "Highlights and typing" {
                    "Highlights & typing"
                } else {
                    group
                }))
                .padding([8, 10])
                .width(Length::Fill)
                .style(move |theme, status| {
                    style::tab(theme, status, index == selected)
                })
                .on_press(Message::PreferenceGroup(index)),
            );
        }
        let group = groups.get(selected).copied().unwrap_or("");
        let mut form =
            column![text(group).size(16).font(font::PRIMARY_BOLD.clone())]
                .spacing(16);
        if area == Area::Navigation && selected == 0 {
            form = form.push(action("Reuse the current pane for everything", Message::ReuseCurrentPane))
                .push(text("After applying, close extra panes to use one-pane navigation.").style(theme::text::secondary));
        }
        if area == Area::Chat {
            form = form.push(text("Channel-specific settings can override these global defaults.").style(theme::text::secondary));
        }
        if area == Area::Sidebar && selected == 0 {
            form = form.push(labelled(
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
                .padding(8)
                .width(Length::Fill)
                .into(),
            ));
        }
        for (index, field) in draft.preferences.fields.iter().enumerate() {
            if field.spec.area == area && field.spec.group == group {
                form = form.push(control(index, field));
            }
        }
        row![
            container(
                scrollable(navigation)
                    .style(style::scrollable)
                    .height(Length::Fill)
            )
            .padding(8)
            .width(172)
            .height(Length::Fill)
            .style(style::pane),
            container(
                scrollable(form.padding(iced::padding::right(12)))
                    .style(style::scrollable)
                    .height(Length::Fill)
            )
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::pane),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    }
}

fn control(index: usize, field: &Field) -> Element<'_, Message> {
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
                    .padding(8)
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
            .padding(8)
            .width(Length::Fill)
            .into(),
        ),
        _ => labelled(
            field.spec.label,
            text_input("Default", &field.value)
                .style(style::input)
                .padding(8)
                .on_input(move |value| Message::Preference(index, value))
                .into(),
        ),
    };
    if field.spec.help.is_empty() {
        widget
    } else {
        column![
            widget,
            text(field.spec.help).size(12).style(theme::text::secondary)
        ]
        .spacing(6)
        .into()
    }
}
