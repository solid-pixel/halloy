//! Compact General and Commands forms. The parent maps `Message` into its own
//! message enum and supplies the shared draft and current validation error.
use data::config::settings::preferences::{Area, Kind};
use data::config::settings::remaining_general::{
    Draft, Editor, Error, Field, Mode, Target, same_literal,
};
use iced::widget::{column, container, row, rule, scrollable};
use iced::{Alignment, Length};

use super::keyboard::{checkbox, pick_list};
use super::{action, style};
use crate::widget::{Element, text, text_input};
use crate::{font, theme};

#[derive(Debug, Clone)]
pub enum Message {
    Display(super::remaining_display::Event),
    Edit(usize, String),
    Default(usize),
    Explicit(usize),
    AddAlias,
    AliasName(usize, String),
    AliasCommand(usize, String),
    RemoveAlias(usize),
    RestoreAlias(usize),
}

/// The parent should clear validation after editing, as with the existing forms.
pub(super) fn update(draft: &mut Draft, message: Message) {
    match message {
        Message::Display(_) => {}
        Message::Edit(index, value) => draft.edit(index, value),
        Message::Default(index) => draft.reset(index),
        Message::Explicit(index) => draft.set_explicit(index),
        Message::AddAlias => draft.add_alias(),
        Message::AliasName(index, name) => {
            if let Some(alias) = draft.aliases.get_mut(index) {
                alias.name = name;
            }
        }
        Message::AliasCommand(index, command) => {
            if let Some(alias) = draft.aliases.get_mut(index) {
                alias.command = command;
            }
        }
        Message::RemoveAlias(index) => {
            if let Some(alias) = draft.aliases.get_mut(index) {
                alias.removed = true;
            }
        }
        Message::RestoreAlias(index) => {
            if let Some(alias) = draft.aliases.get_mut(index) {
                alias.removed = false;
            }
        }
    }
}

pub(super) fn group_for_target(
    draft: &Draft,
    target: &Target,
) -> (Area, &'static str) {
    match *target {
        Target::Field(index) => draft
            .fields
            .get(index)
            .map_or((Area::General, "Application"), |field| {
                (field.area, field.group)
            }),
        Target::AliasName(_) | Target::AliasCommand(_) => {
            (Area::Commands, "Aliases")
        }
    }
}

pub(super) fn focus_id(target: &Target) -> String {
    match target {
        Target::Field(index) => format!("settings-general-field-{index}"),
        Target::AliasName(index) => {
            format!("settings-general-alias-name-{index}")
        }
        Target::AliasCommand(index) => {
            format!("settings-general-alias-command-{index}")
        }
    }
}

pub(super) fn view<'a>(
    draft: &'a Draft,
    area: Area,
    group: &'a str,
    error: Option<&Error>,
    display: &'a data::config::settings::remaining_display::DisplayEditors,
    display_error: Option<
        &data::config::settings::remaining_display::ValidationError,
    >,
) -> Element<'a, Message> {
    let mut content =
        column![text(group).size(16).font(font::PRIMARY_BOLD.clone())]
            .spacing(12);
    if area == Area::Commands && group == "Aliases" {
        content = content
            .push(text("Alias names are case insensitive. Commands can use $1–$9, $1-, optional arguments such as $1?, and $nick, $channel or $server.").size(12).style(theme::text::secondary))
            .push(action("Add alias", Message::AddAlias));
        if draft.aliases.is_empty() {
            content = content.push(
                text("No command aliases configured.")
                    .style(theme::text::secondary),
            );
        }
        let mut rows = Vec::new();
        for (index, alias) in draft.aliases.iter().enumerate() {
            if alias.removed {
                rows.push(
                    row![
                        text("Alias removal staged")
                            .style(theme::text::secondary),
                        alias_action(
                            "Restore",
                            index,
                            Message::RestoreAlias(index)
                        )
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                );
                continue;
            }
            let name = text_input("Alias name", &alias.name)
                .id(focus_id(&Target::AliasName(index)))
                .on_input(move |value| Message::AliasName(index, value))
                .padding(6)
                .style(style::input)
                .width(Length::Fill);
            let command = text_input("/command $1", &alias.command)
                .id(focus_id(&Target::AliasCommand(index)))
                .on_input(move |value| Message::AliasCommand(index, value))
                .padding(6)
                .style(style::input)
                .width(Length::Fill);
            rows.push(column![
                labelled("Name", name.into(), 240.0),
                labelled("Command", command.into(), 360.0),
                alias_action("Remove alias", index, Message::RemoveAlias(index)),
                error_text(error.filter(|e| matches!(e.target, Target::AliasName(i) | Target::AliasCommand(i) if i == index)))
            ].spacing(6).into());
        }
        content = content.push(grouped(rows));
    } else {
        let rows = draft
            .fields
            .iter()
            .enumerate()
            .filter(|(index, field)| {
                field.area == area
                    && !field.path.starts_with("channel_monitor.")
                    && field.group == group
                    && (visible(draft, field)
                        || error.is_some_and(|error| {
                            error.target == Target::Field(*index)
                        }))
            })
            .map(|(index, field)| {
                field_view(
                    index,
                    field,
                    error.filter(|error| error.target == Target::Field(index)),
                )
            })
            .collect();
        content = content.push(grouped(rows));
    }
    content = content.push(
        super::remaining_display::view(display, area, group, display_error)
            .map(Message::Display),
    );
    container(
        scrollable(content.padding(iced::padding::right(12)))
            .id("settings-form-scroll")
            .style(style::scrollable)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn visible(draft: &Draft, field: &Field) -> bool {
    let enabled = |path| {
        draft
            .fields
            .iter()
            .find(|field| field.path == path)
            .is_none_or(|field| field.value == "true")
    };
    if field.path.starts_with("buffer.commands.exec.")
        && field.path != "buffer.commands.exec.enabled"
    {
        return enabled("buffer.commands.exec.enabled");
    }
    if matches!(
        field.path,
        "metadata.avatar.size" | "metadata.avatar.exclude"
    ) {
        return enabled("metadata.avatar.enabled");
    }
    true
}

fn field_view<'a>(
    index: usize,
    field: &'a Field,
    error: Option<&Error>,
) -> Element<'a, Message> {
    let id = focus_id(&Target::Field(index));
    let control: Element<'_, Message> = match field.editor {
        Editor::Scalar(Kind::Boolean) => labelled(
            field.label,
            checkbox(field.value == "true")
                .id(id)
                .label(if field.value == "true" {
                    "Enabled"
                } else {
                    "Disabled"
                })
                .style(style::checkbox)
                .on_toggle(move |value| Message::Edit(index, value.to_string()))
                .into(),
            240.0,
        ),
        Editor::Scalar(Kind::Choice(choices)) => {
            // Detailed backend choices may be displayed with different TOML
            // whitespace. Compare semantic values rather than source text.
            let selected = choices
                .iter()
                .find(|choice| same_literal(choice.value, &field.value))
                .map_or("Configured in file", |choice| choice.label)
                .to_owned();
            let mut labels = choices
                .iter()
                .map(|choice| choice.label.to_owned())
                .collect::<Vec<_>>();
            if field.is_custom() {
                labels.push(selected.clone());
            }
            labelled(
                field.label,
                pick_list(Some(selected), labels, String::clone)
                    .id(id)
                    .on_select(move |label| {
                        Message::Edit(
                            index,
                            choices
                                .iter()
                                .find(|choice| choice.label == label)
                                .map_or_else(
                                    || field.value.clone(),
                                    |choice| choice.value.to_owned(),
                                ),
                        )
                    })
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .padding(6)
                    .width(Length::Fill)
                    .into(),
                240.0,
            )
        }
        _ => {
            let width = match field.editor {
                Editor::Scalar(Kind::Integer { .. } | Kind::Float { .. }) => {
                    104.0
                }
                Editor::Scalar(Kind::Pair) => 128.0,
                Editor::List(_) => 360.0,
                _ => 240.0,
            };
            labelled(
                field.label,
                text_input("Default", &field.value)
                    .id(id)
                    .on_input(move |value| Message::Edit(index, value))
                    .padding(6)
                    .style(style::input)
                    .width(Length::Fill)
                    .into(),
                width,
            )
        }
    };
    let mode = if field.mode == Mode::Default {
        "Default"
    } else {
        "Custom value"
    }
    .to_owned();
    let mode_control = labelled(
        "Value source",
        pick_list(
            Some(mode),
            vec!["Default".to_owned(), "Custom value".to_owned()],
            String::clone,
        )
        .id(format!("settings-general-source-{index}"))
        .on_select(move |value| {
            if value == "Default" {
                Message::Default(index)
            } else {
                Message::Explicit(index)
            }
        })
        .style(style::pick_list)
        .menu_style(style::menu)
        .padding(6)
        .width(Length::Fill)
        .into(),
        160.0,
    );
    let mut content = column![control, mode_control].spacing(6);
    if !field.help.is_empty() {
        content = content
            .push(text(field.help).size(12).style(theme::text::secondary));
    }
    content.push(error_text(error)).into()
}

fn alias_action(
    label: &'static str,
    index: usize,
    message: Message,
) -> Element<'static, Message> {
    super::keyboard::button(text(label))
        .id(format!("settings-general-{label}-{index}"))
        .padding([6, 10])
        .style(style::button)
        .on_press(message)
        .into()
}
fn error_text(error: Option<&Error>) -> Element<'static, Message> {
    text(error.map_or_else(String::new, |e| e.message.clone()))
        .size(12)
        .style(theme::text::error)
        .into()
}
fn labelled<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    width: f32,
) -> Element<'a, Message> {
    row![
        text(label).width(160),
        container(control).width(Length::Fill.max(width))
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}
fn grouped(rows: Vec<Element<'_, Message>>) -> Element<'_, Message> {
    let mut content = column![].spacing(0);
    for (index, element) in rows.into_iter().enumerate() {
        if index > 0 {
            content = content.push(
                container(rule::horizontal(1))
                    .padding(iced::padding::horizontal(12)),
            );
        }
        content = content
            .push(container(element).padding([8, 12]).width(Length::Fill));
    }
    container(content)
        .style(style::pane)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependent_controls_reappear_without_losing_values() {
        let source = "[buffer.commands.exec]\nenabled=false\ntimeout=123\n"
            .parse()
            .unwrap();
        let mut draft = Draft::parse(&source).unwrap();
        let timeout = draft
            .fields
            .iter()
            .position(|f| f.path == "buffer.commands.exec.timeout")
            .unwrap();
        assert!(!visible(&draft, &draft.fields[timeout]));
        let enabled = draft
            .fields
            .iter()
            .position(|f| f.path == "buffer.commands.exec.enabled")
            .unwrap();
        update(&mut draft, Message::Edit(enabled, "true".into()));
        assert!(visible(&draft, &draft.fields[timeout]));
        assert_eq!(draft.fields[timeout].value, "123");
    }
    #[test]
    fn alias_validation_targets_open_commands_aliases() {
        let draft = Draft::parse(&"".parse().unwrap()).unwrap();
        assert_eq!(
            group_for_target(&draft, &Target::AliasCommand(4)),
            (Area::Commands, "Aliases")
        );
    }
}
