use data::config::settings::rules::{Area, Kind};
use data::config::settings::{Settings as Draft, ValidationTarget};
use iced::Length;
use iced::widget::{column, container, row, scrollable, text, text_input};

use super::keyboard::{checkbox, pick_list};
use super::{
    Message, Section, Settings, action, keyboard, labelled_width,
    settings_rows, style,
};
use crate::widget::Element;
use crate::{font, theme};

pub(super) fn area(section: Section) -> Option<Area> {
    match section {
        Section::Notifications => Some(Area::Notifications),
        Section::Highlights => Some(Area::Highlights),
        Section::Shortcuts => Some(Area::Keyboard),
        _ => None,
    }
}
pub(super) fn section(area: Area) -> Section {
    match area {
        Area::Notifications => Section::Notifications,
        Area::Highlights => Section::Highlights,
        Area::Keyboard => Section::Shortcuts,
    }
}
impl Settings {
    pub(super) fn rules<'a>(
        &'a self,
        draft: &'a Draft,
    ) -> Element<'a, Message> {
        let area = area(self.section).unwrap();
        let mut content = column![].spacing(12);
        if area == Area::Notifications {
            content = content.push(action(
                "Add channel rule",
                Message::RuleAdd(area, false),
            ));
        }
        if area == Area::Highlights {
            content = content.push(
                row![
                    action("Add word rule", Message::RuleAdd(area, false)),
                    action("Add regex rule", Message::RuleAdd(area, true))
                ]
                .spacing(8),
            );
        }
        if let Some(group) = draft
            .rules
            .groups
            .get(self.rule_group)
            .filter(|g| g.area == area && !g.removed)
        {
            content = content.push(
                text(group.label())
                    .size(16)
                    .font(font::PRIMARY_BOLD.clone()),
            );
            let help = match area {
                Area::Notifications => {
                    "Sounds: built-in names or files in the sounds folder. Separate entries with semicolons. Empty means silent. Delay limits how often this event notifies."
                }
                Area::Highlights => {
                    if group.is_highlight() {
                        "Separate words, phrases and sounds with semicolons. An empty sound list uses the highlight notification sound. Regex flags such as (?i) control case."
                    } else {
                        "Nickname matching uses the server’s case mapping. Exclude * to disable it everywhere, then include the channels, users or servers you want."
                    }
                }
                Area::Keyboard => {
                    "Use command+k or ctrl+shift+k. Separate alternative bindings with semicolons. Clear disables an action; Default restores Halloy’s platform default. Explicit bindings can override Cmd/Ctrl+, for Settings."
                }
            };
            content =
                content.push(text(help).size(12).style(theme::text::secondary));
            if area == Area::Keyboard && group.title == "Message focus" {
                content=content.push(text("These actions run during message selection. Plain arrow keys are built in. Direction bindings also enter message selection from chat.").size(12).style(theme::text::secondary));
            }
            let mut rows = vec![];
            for (field_index, field) in
                group.fields.iter().enumerate().filter(|(i, field)| {
                    group.visible(*i)
                        && !(group.conditions.is_some()
                            && field.path.first().is_some_and(|key| {
                                matches!(key.as_str(), "include" | "exclude")
                            }))
                })
            {
                let group_index = self.rule_group;
                let message = move |value| {
                    Message::RuleEdit(group_index, field_index, value)
                };
                let id = format!("settings-rule-{group_index}-{field_index}");
                let control: Element<'_, Message> = match field.kind {
                    Kind::Boolean => checkbox(field.value == "true")
                        .label(&field.label)
                        .id(id)
                        .style(style::checkbox)
                        .on_toggle(move |v| message(v.to_string()))
                        .into(),
                    Kind::MatchType => labelled_width(
                        &field.label,
                        pick_list(
                            Some(field.value.clone()),
                            vec!["Words".into(), "Regular expression".into()],
                            String::clone,
                        )
                        .on_select(message)
                        .style(style::pick_list)
                        .menu_style(style::menu)
                        .padding(6)
                        .width(Length::Fill)
                        .into(),
                        240.0,
                    ),
                    _ => {
                        let input = text_input(
                            if field.kind == Kind::Binding {
                                "Disabled"
                            } else {
                                "Default"
                            },
                            &field.value,
                        )
                        .id(id)
                        .style(style::input)
                        .padding(6)
                        .on_input(message)
                        .width(Length::Fill);
                        let width = if field.kind == Kind::Number {
                            104.0
                        } else {
                            280.0
                        };
                        let editor: Element<'_, Message> = if field.kind
                            == Kind::Binding
                        {
                            column![
                                input,
                                row![
                                    action(
                                        "Clear",
                                        Message::RuleEdit(
                                            group_index,
                                            field_index,
                                            String::new()
                                        )
                                    ),
                                    action(
                                        "Default",
                                        Message::RuleReset(
                                            group_index,
                                            field_index
                                        )
                                    ),
                                    text(if field.reset {
                                        "Default on Apply"
                                    } else {
                                        ""
                                    })
                                    .size(12)
                                    .style(theme::text::secondary)
                                ]
                                .spacing(6)
                                .align_y(iced::Alignment::Center)
                            ]
                            .spacing(4)
                            .into()
                        } else if field.path == ["sound"] {
                            let mut choices =
                                vec!["Add built-in sound".to_owned()];
                            choices.extend(
                                [
                                    "bloop", "bonk", "dong", "drop", "peck",
                                    "ring", "sing", "squeak", "tweep",
                                    "whistle", "zone",
                                ]
                                .map(str::to_owned),
                            );
                            column![
                                input,
                                pick_list(
                                    Some("Add built-in sound".to_owned()),
                                    choices,
                                    String::clone
                                )
                                .on_select(move |sound| {
                                    if sound == "Add built-in sound" {
                                        message(field.value.clone())
                                    } else {
                                        message(
                                            if field.value.trim().is_empty() {
                                                sound
                                            } else {
                                                format!(
                                                    "{}; {sound}",
                                                    field.value
                                                )
                                            },
                                        )
                                    }
                                })
                                .style(style::pick_list)
                                .menu_style(style::menu)
                                .padding(6)
                                .width(Length::Fill)
                            ]
                            .spacing(4)
                            .into()
                        } else {
                            input.into()
                        };
                        labelled_width(&field.label, editor, width)
                    }
                };
                let mut row = column![control].spacing(4);
                if let Some(error) =
                    self.validation_error.as_ref().filter(|e| {
                        e.target
                            == (ValidationTarget::Rule {
                                group: group_index,
                                field: field_index,
                            })
                    })
                {
                    row = row.push(
                        text(&error.message).size(12).style(theme::text::error),
                    );
                }
                rows.push(row.into());
            }
            let fields: Element<'_, Message> = settings_rows(rows);
            content = content.push(if self.confirm_rule_remove {
                keyboard::inactive(fields)
            } else {
                fields
            });
            if let Some(conditions) = &group.conditions {
                let error = self.validation_error.as_ref().and_then(|error| match error.target {
                    ValidationTarget::RuleCondition { group, target } if group == self.rule_group => Some(data::config::settings::remaining_display::ValidationError { target, message: error.message.clone() }), _ => None,
                });
                let group_index = self.rule_group;
                let conditions = super::remaining_display::view(
                    conditions,
                    data::config::settings::preferences::Area::General,
                    "Conditions",
                    error.as_ref(),
                )
                .map(move |event| Message::RuleCondition(group_index, event));
                content = content.push(if self.confirm_rule_remove {
                    keyboard::inactive(conditions)
                } else {
                    conditions
                });
            }
            if area != Area::Keyboard {
                content=content.push(text("Conditions use semicolon-separated names, or * for all. Includes take priority over excludes; to allow only listed names, exclude * first. Each compound condition requires all its filled fields to match. Quote an entry containing a semicolon.").size(12).style(theme::text::secondary));
            }
            if group.removable && !self.confirm_rule_remove {
                content =
                    content.push(action("Remove rule", Message::RuleRemove));
            }
        } else {
            content = content.push(
                text("Add a rule to begin.").style(theme::text::secondary),
            );
        }
        let content: Element<'_, Message> = content.into();
        // Keep only confirmation choices interactive, including the add buttons.
        let content = if self.confirm_rule_remove {
            column![
                keyboard::inactive(content),
                text("Remove this rule from the draft?"),
                row![
                    action("Keep rule", Message::RuleCancelRemove),
                    action("Remove rule", Message::RuleConfirmRemove)
                ]
                .spacing(8)
            ]
            .spacing(12)
            .into()
        } else {
            content
        };
        container(
            scrollable(
                container(content)
                    .padding(iced::padding::right(12))
                    .width(Length::Fill),
            )
            .id("settings-form-scroll")
            .style(style::scrollable)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
