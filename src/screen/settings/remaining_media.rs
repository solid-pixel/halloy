//! Structured rows appended to the existing preference form. The parent owns
//! scrolling, group navigation, validation routing, and the persistent footer.
use data::config::settings::remaining_media::{
    Control, Error, Media, MediaArea,
};
use iced::Length;
use iced::widget::{column, row, text, text_input};

use super::keyboard::{checkbox, pick_list};
use super::{Message, labelled_width, style};
use crate::theme;
use crate::widget::Element;

/// Merge these names with preference groups, retaining the order shown here.
pub(super) fn groups(area: MediaArea) -> &'static [&'static str] {
    match area {
        MediaArea::Previews => {
            &["Links", "Link cards", "Images", "Requests and cache"]
        }
        MediaArea::Transfers => &[
            "DCC transfers",
            "Automatic acceptance",
            "DCC connection",
            "Uploads",
        ],
    }
}

/// Append to `preferences::control` rows before calling `settings_rows`.
/// Message::MediaEdit(index, value) directly edits draft.media.fields[index].value.
pub(super) fn rows<'a>(
    media: &'a Media,
    area: MediaArea,
    group: &str,
    error: Option<&Error>,
) -> Vec<Element<'a, Message>> {
    media
        .fields
        .iter()
        .enumerate()
        .filter(|(index, field)| {
            field.area == area
                && field.group == group
                && field.control != Control::Scope
                && media.visible(*index)
        })
        .map(|(index, field)| {
            let edit = move |value| Message::MediaEdit(index, value);
            let control: Element<'a, Message> = match field.control {
                Control::ServerEnabled => checkbox(field.value == "true")
                    .label(&field.label)
                    .id(field.id())
                    .style(style::checkbox)
                    .on_toggle(move |value| edit(value.to_string()))
                    .into(),
                Control::UrlMode => labelled_width(
                    &field.label,
                    pick_list(
                        Some(field.value.clone()),
                        vec![
                            "Enabled".to_owned(),
                            "Disabled".to_owned(),
                            "Matching URLs".to_owned(),
                        ],
                        String::clone,
                    )
                    .id(field.id())
                    .on_select(edit)
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .padding(6)
                    .width(Length::Fill)
                    .into(),
                    240.0,
                ),
                Control::CacheSize | Control::TrimInterval => {
                    let special = if field.control == Control::CacheSize {
                        "unlimited"
                    } else {
                        "first-save-only"
                    };
                    let special_label = if field.control == Control::CacheSize {
                        "Unlimited"
                    } else {
                        "First save only"
                    };
                    let numeric_label = if field.control == Control::CacheSize {
                        "Limit"
                    } else {
                        "Every N saves"
                    };
                    let is_special = field.value == special;
                    let mode = pick_list(
                        Some(
                            if is_special {
                                special_label
                            } else {
                                numeric_label
                            }
                            .to_owned(),
                        ),
                        vec![
                            numeric_label.to_owned(),
                            special_label.to_owned(),
                        ],
                        String::clone,
                    )
                    .id(format!("{}-mode", field.id()))
                    .on_select(move |label| {
                        edit(if label == special_label {
                            special.to_owned()
                        } else if is_special {
                            if field.control == Control::CacheSize {
                                "500"
                            } else {
                                "32"
                            }
                            .to_owned()
                        } else {
                            field.value.clone()
                        })
                    })
                    .style(style::pick_list)
                    .menu_style(style::menu)
                    .padding(6)
                    .width(Length::Fixed(156.0));
                    let mut controls =
                        row![mode].spacing(8).align_y(iced::Alignment::Center);
                    if !is_special {
                        controls = controls.push(
                            text_input("Default", &field.value)
                                .id(field.id())
                                .on_input(edit)
                                .style(style::input)
                                .padding(6)
                                .width(Length::Fixed(104.0)),
                        );
                    }
                    labelled_width(&field.label, controls.into(), 268.0)
                }
                _ => {
                    let width = if field.control == Control::Port {
                        104.0
                    } else {
                        280.0
                    };
                    labelled_width(
                        &field.label,
                        text_input(
                            if field.control == Control::Text {
                                "Default"
                            } else {
                                "None"
                            },
                            &field.value,
                        )
                        .id(field.id())
                        .on_input(edit)
                        .style(style::input)
                        .padding(6)
                        .width(Length::Fill)
                        .into(),
                        width,
                    )
                }
            };
            // Preserve the control's tree position while its error is corrected.
            let mut content = column![control].spacing(4);
            if let Some(error) = error.filter(|error| error.field == index) {
                content = content.push(
                    text(error.message.clone())
                        .size(12)
                        .style(theme::text::error),
                );
            }
            if !field.help.is_empty() {
                content = content.push(
                    text(field.help).size(12).style(theme::text::secondary),
                );
            }
            content.into()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_structured_field_has_a_visible_navigation_group_and_unique_id() {
        let media = Media::parse(&"".parse().unwrap()).unwrap();
        let mut ids = std::collections::HashSet::new();
        for field in &media.fields {
            assert!(groups(field.area).contains(&field.group));
            assert!(
                ids.insert(field.id()),
                "duplicate field id for {}",
                field.path
            );
        }
    }
}
