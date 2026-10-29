//! Selected-network advanced form. Parent maps `Message` and retains the footer,
//! network list, main navigation, scrolling, keyboard root and save operation.
use std::collections::BTreeMap;

use data::config::settings::remaining_networks::{
    Draft, Field, Kind, METADATA_KEYS, Part, Path, Rows, ValidationError,
    path_id,
};
use iced::widget::{column, container, row, text, text_input};
use iced::{Alignment, Length};

use super::keyboard::{button, pick_list};
use super::style;
use crate::widget::Element;
use crate::{font, theme};

#[derive(Clone)]
pub enum Message {
    Group(String),
    Edit(usize, Option<String>),
    Keep(usize),
    Select(usize, String),
    EntryName(String, String),
    Add(usize),
    IgnoreKind(Path, String),
    Remove(Path),
    CancelRemove,
    ConfirmRemove,
}
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edit(index, _) => f
                .debug_tuple("Edit")
                .field(index)
                .field(&"<redacted>")
                .finish(),
            Self::Group(value) => f.debug_tuple("Group").field(value).finish(),
            Self::Keep(index) => f.debug_tuple("Keep").field(index).finish(),
            Self::Select(index, value) => {
                f.debug_tuple("Select").field(index).field(value).finish()
            }
            Self::EntryName(path, _) => {
                f.debug_tuple("EntryName").field(path).finish()
            }
            Self::Add(index) => f.debug_tuple("Add").field(index).finish(),
            Self::IgnoreKind(path, kind) => {
                f.debug_tuple("IgnoreKind").field(path).field(kind).finish()
            }
            Self::Remove(path) => f.debug_tuple("Remove").field(path).finish(),
            Self::CancelRemove => f.write_str("CancelRemove"),
            Self::ConfirmRemove => f.write_str("ConfirmRemove"),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub group: String,
    pub error: Option<ValidationError>,
    entry_names: BTreeMap<String, String>,
    pending_remove: Option<Path>,
}
impl State {
    /// The return value is a focus ID for the owning Settings keyboard task.
    pub fn update(
        &mut self,
        draft: &mut Draft,
        message: Message,
    ) -> Option<String> {
        let restore_focus = match &message {
            Message::CancelRemove => self.pending_remove.as_ref().map(|path| {
                format!(
                    "settings-network-extra-action-{}-remove",
                    path_id(path)
                )
            }),
            Message::ConfirmRemove => {
                Some("settings-network-extra-group".into())
            }
            _ => None,
        };
        let result = match message {
            Message::Group(group) => {
                self.group = group;
                Ok(())
            }
            Message::Edit(index, value) => {
                draft.edit(index, value);
                Ok(())
            }
            Message::Keep(index) => {
                draft.keep(index);
                Ok(())
            }
            Message::Select(index, value) => draft.select(index, &value),
            Message::EntryName(key, value) => {
                self.entry_names.insert(key, value);
                Ok(())
            }
            Message::Add(index) => {
                let key = draft
                    .collections
                    .get(index)
                    .map(|c| path_id(&c.path))
                    .unwrap_or_default();
                let name =
                    self.entry_names.get(&key).cloned().unwrap_or_default();
                let result = draft.add(index, &name);
                if result.is_ok() {
                    self.entry_names.remove(&key);
                }
                result
            }
            Message::IgnoreKind(path, value) => {
                draft.ignore_kind(&path, &value)
            }
            Message::Remove(path) => {
                self.pending_remove = Some(path);
                Ok(())
            }
            Message::CancelRemove => {
                self.pending_remove = None;
                Ok(())
            }
            Message::ConfirmRemove => {
                if let Some(path) = self.pending_remove.take() {
                    draft.remove(&path)
                } else {
                    Ok(())
                }
            }
        };
        if let Err(error) = result {
            return Some(self.reveal(draft, error));
        }
        // Keep an existing validation explanation until that field is corrected.
        if self.error.is_some() {
            self.error = draft.validation_error();
        }
        restore_focus
    }
    pub fn reveal(&mut self, draft: &Draft, error: ValidationError) -> String {
        if let Some(group) = draft.group_for(&error.path) {
            self.group = group.to_owned();
        }
        let id = draft
            .collections
            .iter()
            .find(|c| c.path == error.path)
            .map_or_else(
                || error.id(),
                |c| {
                    let key = path_id(&c.path);
                    if matches!(
                        c.kind,
                        Rows::Metadata
                            | Rows::ChannelKeys
                            | Rows::ChannelKeyrings
                            | Rows::Bouncer
                    ) {
                        format!("settings-network-extra-{key}-new")
                    } else {
                        format!("settings-network-extra-action-{key}-add")
                    }
                },
            );
        let id = if let Some(field) =
            draft.fields.iter().find(|f| f.path == error.path)
        {
            match field.kind {
                Kind::Keyring
                    if field
                        .value
                        .as_deref()
                        .is_some_and(|v| v.starts_with("key:")) =>
                {
                    format!("{id}-name")
                }
                Kind::Limit
                    if field
                        .value
                        .as_deref()
                        .is_some_and(|v| v != "unlimited") =>
                {
                    format!("{id}-count")
                }
                Kind::Target
                    if field
                        .value
                        .as_deref()
                        .is_some_and(|v| v != "server") =>
                {
                    format!("{id}-channel")
                }
                _ => id,
            }
        } else {
            id
        };
        self.error = Some(error);
        id
    }
    /// Append this element inside the parent's selected network form. It has no
    /// independent scroll view and no second action footer.
    pub fn view<'a>(
        &'a self,
        draft: &'a Draft,
        stacked: bool,
    ) -> Element<'a, Message> {
        let groups = draft.groups();
        let selected = groups
            .iter()
            .find(|g| **g == self.group)
            .copied()
            .or_else(|| groups.first().copied())
            .unwrap_or("Proxy");
        let mut content = column![
            text(if draft.is_global_proxy() {
                "Global proxy"
            } else {
                "More network settings"
            })
            .size(16)
            .font(font::PRIMARY_BOLD.clone()),
            labelled(
                "Group",
                picker(
                    "settings-network-extra-group".into(),
                    Some(selected.into()),
                    groups.iter().map(|g| (*g).to_owned()).collect(),
                    Message::Group
                ),
                320.0,
                stacked
            ),
        ]
        .spacing(12);
        let mut rows = vec![];
        for (index, selector) in draft
            .selectors
            .iter()
            .enumerate()
            .filter(|(_, s)| s.group == selected)
        {
            let global = draft.is_global_proxy();
            let mut options = selector
                .choices
                .iter()
                .map(|s| method_label(s, global).to_owned())
                .collect::<Vec<_>>();
            let selected_method =
                method_label(&selector.selected, global).to_owned();
            if !options.contains(&selected_method) {
                options.push(selected_method.clone());
            }
            rows.push(labelled(
                selector.label,
                picker(
                    format!(
                        "settings-network-extra-{}",
                        path_id(&selector.path)
                    ),
                    Some(selected_method),
                    options,
                    move |label| {
                        Message::Select(
                            index,
                            selector
                                .choices
                                .iter()
                                .find(|s| method_label(s, global) == label)
                                .map_or_else(
                                    || selector.selected.clone(),
                                    |s| (*s).to_owned(),
                                ),
                        )
                    },
                ),
                260.0,
                stacked,
            ));
        }
        for (index, field) in draft
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.group == selected)
        {
            let in_collection = draft
                .collections
                .iter()
                .filter(|c| c.group == selected)
                .flat_map(|c| c.entries.iter())
                .any(|e| field.path.starts_with(&e.path));
            if !in_collection {
                rows.push(self.control(index, field, stacked));
            }
        }
        if !rows.is_empty() {
            content = content.push(group_rows(rows));
        }
        for (index, collection) in draft
            .collections
            .iter()
            .enumerate()
            .filter(|(_, c)| c.group == selected)
        {
            let mut rows = vec![];
            for entry in &collection.entries {
                let mut entry_rows = column![
                    text(&entry.label).font(font::PRIMARY_BOLD.clone())
                ]
                .spacing(8);
                if collection.kind == Rows::Bouncer {
                    entry_rows = entry_rows.push(action(
                        "Edit override",
                        format!("{}-edit", path_id(&entry.path)),
                        Message::Group(format!("Bouncer: {}", entry.label)),
                    ));
                } else {
                    if collection.kind == Rows::Ignore {
                        let fields = draft
                            .fields
                            .iter()
                            .filter(|f| f.path.starts_with(&entry.path))
                            .collect::<Vec<_>>();
                        let regex =
                            fields.iter().any(|f| f.kind == Kind::Regex);
                        let channel = fields.iter().any(|f| {
                            f.path.last() == Some(&Part::Key("channel".into()))
                                && f.configured
                        });
                        let selected = match (regex, channel) {
                            (false, false) => "user",
                            (false, true) => "user-channel",
                            (true, false) => "regex",
                            (true, true) => "regex-channel",
                        };
                        let entry_path = entry.path.clone();
                        entry_rows = entry_rows.push(labelled(
                            "Ignore rule",
                            picker(
                                format!("{}-kind", path_id(&entry.path)),
                                Some(selected.into()),
                                [
                                    "user",
                                    "user-channel",
                                    "regex",
                                    "regex-channel",
                                ]
                                .iter()
                                .map(|s| (*s).into())
                                .collect(),
                                move |value| {
                                    Message::IgnoreKind(
                                        entry_path.clone(),
                                        value,
                                    )
                                },
                            ),
                            220.0,
                            stacked,
                        ));
                    }
                    for (field_index, field) in
                        draft.fields.iter().enumerate().filter(|(_, f)| {
                            f.group == selected
                                && f.path.starts_with(&entry.path)
                        })
                    {
                        entry_rows = entry_rows.push(self.control(
                            field_index,
                            field,
                            stacked,
                        ));
                    }
                }
                entry_rows = entry_rows.push(action(
                    "Remove entry…",
                    format!("{}-remove", path_id(&entry.path)),
                    Message::Remove(entry.path.clone()),
                ));
                rows.push(entry_rows.into());
            }
            let key = path_id(&collection.path);
            let name = self.entry_names.get(&key).map_or("", String::as_str);
            let input_key = key.clone();
            let named = matches!(
                collection.kind,
                Rows::Metadata
                    | Rows::ChannelKeys
                    | Rows::ChannelKeyrings
                    | Rows::Bouncer
            );
            let mut add_row = column![].spacing(8);
            if collection.kind == Rows::Metadata {
                let options = METADATA_KEYS
                    .iter()
                    .filter(|key| {
                        !collection
                            .entries
                            .iter()
                            .any(|entry| entry.label == **key)
                    })
                    .map(|key| (*key).to_owned())
                    .collect::<Vec<_>>();
                add_row = add_row.push(labelled(
                    "New metadata",
                    picker(
                        format!("settings-network-extra-{key}-new"),
                        if name.is_empty() {
                            None
                        } else {
                            Some(name.to_owned())
                        },
                        options,
                        move |value| {
                            Message::EntryName(input_key.clone(), value)
                        },
                    ),
                    260.0,
                    stacked,
                ));
            } else if named {
                add_row = add_row.push(labelled(
                    "New entry name",
                    text_input("Name", name)
                        .id(format!("settings-network-extra-{key}-new"))
                        .on_input(move |value| {
                            Message::EntryName(input_key.clone(), value)
                        })
                        .style(style::input)
                        .padding(6)
                        .into(),
                    300.0,
                    stacked,
                ));
            }
            add_row = add_row.push(action(
                "Add entry",
                format!("{key}-add"),
                Message::Add(index),
            ));
            rows.push(add_row.into());
            content = content.push(
                column![
                    text(collection.label).font(font::PRIMARY_BOLD.clone()),
                    group_rows(rows)
                ]
                .spacing(8),
            );
        }
        if let Some(error) = &self.error {
            content = content
                .push(text(&error.message).size(12).style(theme::text::error));
        }
        if let Some(path) = &self.pending_remove {
            // Parent's keyboard root must restrict focus to these two IDs while
            // confirmation is active, just like its existing removal prompt.
            let background: Element<'_, Message> =
                super::keyboard::inactive(content);
            return column![
                background,
                text("Remove this entry when changes are applied?"),
                row![
                    action(
                        "Keep entry",
                        "confirm-keep".into(),
                        Message::CancelRemove
                    ),
                    action(
                        "Remove entry",
                        format!("confirm-{}", path_id(path)),
                        Message::ConfirmRemove
                    )
                ]
                .spacing(8)
            ]
            .spacing(12)
            .into();
        }
        content.into()
    }
    pub fn confirmation_open(&self) -> bool {
        self.pending_remove.is_some()
    }
    pub fn confirmation_ids(&self) -> Option<[String; 2]> {
        self.pending_remove.as_ref().map(|path| {
            [
                "settings-network-extra-action-confirm-keep".into(),
                format!(
                    "settings-network-extra-action-confirm-{}",
                    path_id(path)
                ),
            ]
        })
    }
    fn control<'a>(
        &'a self,
        index: usize,
        field: &'a Field,
        stacked: bool,
    ) -> Element<'a, Message> {
        let width = match field.kind {
            Kind::Integer(_, _) => 112.0,
            Kind::Boolean => 180.0,
            _ => 320.0,
        };
        let value = field.value.as_deref();
        let id = field.id();
        let control = match field.kind {
            Kind::Boolean => {
                let selected = match value {
                    Some("true") => "Enabled",
                    Some("false") => "Disabled",
                    _ if field.custom => "Configured in file",
                    _ => "Default",
                };
                picker(
                    id,
                    Some(selected.into()),
                    vec!["Default".into(), "Enabled".into(), "Disabled".into()],
                    move |choice| {
                        Message::Edit(
                            index,
                            match choice.as_str() {
                                "Enabled" => Some("true".into()),
                                "Disabled" => Some("false".into()),
                                _ => None,
                            },
                        )
                    },
                )
            }
            Kind::Choice(choices) => {
                let mut options = vec!["Default".into()];
                options.extend(choices.iter().map(|s| (*s).to_owned()));
                picker(
                    id,
                    Some(
                        value
                            .unwrap_or(if field.custom {
                                "Configured in file"
                            } else {
                                "Default"
                            })
                            .into(),
                    ),
                    options,
                    move |value| {
                        Message::Edit(
                            index,
                            if value == "Default" {
                                None
                            } else {
                                Some(value)
                            },
                        )
                    },
                )
            }
            Kind::Limit => {
                let mode = match value {
                    None => "Default",
                    Some("unlimited") => "Unlimited",
                    _ => "Limit",
                };
                let mut controls = column![picker(
                    id.clone(),
                    Some(mode.into()),
                    vec!["Default".into(), "Unlimited".into(), "Limit".into()],
                    move |choice| Message::Edit(
                        index,
                        match choice.as_str() {
                            "Unlimited" => Some("unlimited".into()),
                            "Limit" => Some("10".into()),
                            _ => None,
                        }
                    )
                )]
                .spacing(6);
                if mode == "Limit" {
                    controls = controls.push(
                        text_input("Attempts", value.unwrap_or(""))
                            .id(format!("{id}-count"))
                            .on_input(move |value| {
                                Message::Edit(index, Some(value))
                            })
                            .style(style::input)
                            .padding(6),
                    );
                }
                controls.into()
            }
            Kind::Keyring => {
                let mode = match value {
                    Some("default") => "Default entry",
                    Some("disabled") => "Disabled",
                    Some(_) => "Named entry",
                    None => "Default",
                };
                let mut controls = column![picker(
                    id.clone(),
                    Some(mode.into()),
                    vec![
                        "Default".into(),
                        "Disabled".into(),
                        "Default entry".into(),
                        "Named entry".into()
                    ],
                    move |value| Message::Edit(
                        index,
                        match value.as_str() {
                            "Disabled" => Some("disabled".into()),
                            "Default entry" => Some("default".into()),
                            "Named entry" => Some("key:".into()),
                            _ => None,
                        }
                    )
                )]
                .spacing(6);
                if mode == "Named entry" {
                    controls = controls.push(
                        text_input(
                            "Keyring entry name",
                            value
                                .and_then(|v| v.strip_prefix("key:"))
                                .unwrap_or(""),
                        )
                        .id(format!("{id}-name"))
                        .on_input(move |value| {
                            Message::Edit(index, Some(format!("key:{value}")))
                        })
                        .style(style::input)
                        .padding(6),
                    );
                }
                controls.into()
            }
            Kind::Target => {
                let channel = value.is_some_and(|v| v != "server");
                let mut controls = column![picker(
                    id.clone(),
                    Some(if channel { "Channel" } else { "Server" }.into()),
                    vec!["Server".into(), "Channel".into()],
                    move |value| Message::Edit(
                        index,
                        Some(
                            if value == "Server" {
                                "server"
                            } else {
                                "#channel"
                            }
                            .into()
                        )
                    )
                )]
                .spacing(6);
                if channel {
                    controls = controls.push(
                        text_input("#channel", value.unwrap_or(""))
                            .id(format!("{id}-channel"))
                            .on_input(move |value| {
                                Message::Edit(index, Some(value))
                            })
                            .style(style::input)
                            .padding(6),
                    );
                }
                controls.into()
            }
            _ => text_input(field.placeholder(), value.unwrap_or(""))
                .id(id)
                .on_input(move |value| Message::Edit(index, Some(value)))
                .secure(field.kind == Kind::Secret)
                .style(style::input)
                .padding(6)
                .width(Length::Fill)
                .into(),
        };
        let mut controls = row![container(control).width(Length::Fill)]
            .spacing(8)
            .align_y(Alignment::Center);
        let reset = field.configured || field.value.is_some();
        if reset {
            controls = controls.push(action(
                "Default",
                format!("{}-default", field.id()),
                Message::Edit(index, None),
            ));
        }
        if field.is_edited() {
            controls = controls.push(action(
                "Keep",
                format!("{}-keep", field.id()),
                Message::Keep(index),
            ));
        }
        let mut result = column![labelled(
            &field.label,
            controls.into(),
            width
                + if reset { 72.0 } else { 0.0 }
                + if field.is_edited() { 56.0 } else { 0.0 },
            stacked
        )]
        .spacing(4);
        if !field.help.is_empty() {
            result = result
                .push(text(field.help).size(12).style(theme::text::secondary));
        }
        if let Some(error) =
            self.error.as_ref().filter(|e| e.path == field.path)
        {
            result = result
                .push(text(&error.message).size(12).style(theme::text::error));
        }
        result.into()
    }
}
fn method_label(value: &str, global: bool) -> &str {
    match value {
        "inherit" if global => "No proxy",
        "inherit" => "Use global proxy",
        "http" => "HTTP",
        "socks5" => "SOCKS5",
        "tor" => "Tor",
        "none" => "None",
        "plain" => "SASL PLAIN",
        "external" => "SASL EXTERNAL",
        "server" => "Use server credentials",
        _ => "Configured in file",
    }
}
fn picker<'a>(
    id: String,
    selected: Option<String>,
    options: Vec<String>,
    select: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    pick_list(selected, options, String::clone)
        .id(id)
        .on_select(select)
        .style(style::pick_list)
        .menu_style(style::menu)
        .padding(6)
        .width(Length::Fill)
        .into()
}
fn action<'a>(
    label: &'a str,
    id: String,
    message: Message,
) -> Element<'a, Message> {
    button(text(label))
        .id(format!("settings-network-extra-action-{id}"))
        .on_press(message)
        .style(style::button)
        .padding([6, 10])
        .into()
}
fn labelled<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    width: f32,
    stacked: bool,
) -> Element<'a, Message> {
    let control = container(control).width(Length::Fill.max(width));
    if stacked {
        column![text(label), control].spacing(5).into()
    } else {
        row![container(text(label)).width(180), control]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }
}
fn group_rows<'a>(rows: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    let mut content = column![].spacing(0);
    for (index, item) in rows.into_iter().enumerate() {
        if index > 0 {
            content = content.push(
                container(iced::widget::rule::horizontal(1))
                    .padding(iced::padding::horizontal(12)),
            );
        }
        content =
            content.push(container(item).padding([8, 12]).width(Length::Fill));
    }
    container(content)
        .width(Length::Fill)
        .style(style::pane)
        .into()
}

#[cfg(test)]
mod tests {

    use super::*;
    fn draft(source: &str) -> Draft {
        Draft::parse_network(source).unwrap()
    }
    #[test]
    fn removal_waits_for_confirmation_and_exposes_focus_limits() {
        let mut draft = draft("[filters]\nignore=['fixture']\n");
        let collection = draft
            .collections
            .iter()
            .find(|c| c.kind == Rows::Ignore)
            .unwrap();
        let path = collection.entries[0].path.clone();
        let mut state = State::default();
        state.update(&mut draft, Message::Remove(path));
        assert!(state.confirmation_open());
        assert_eq!(
            state.confirmation_ids().unwrap()[0],
            "settings-network-extra-action-confirm-keep"
        );
        assert!(!draft.is_dirty());
        state.update(&mut draft, Message::ConfirmRemove);
        assert!(!state.confirmation_open());
        assert!(
            draft
                .collections
                .iter()
                .find(|c| c.kind == Rows::Ignore)
                .unwrap()
                .entries
                .is_empty()
        );
    }
    #[test]
    fn validation_selects_group_and_the_actual_composite_input() {
        let mut draft = draft("");
        let index = draft
            .fields
            .iter()
            .position(|f| f.kind == Kind::Limit)
            .unwrap();
        draft.edit(index, Some("0".into()));
        let error = draft.validation_error().unwrap();
        let mut state = State::default();
        let focus = state.reveal(&draft, error);
        assert_eq!(state.group, "Connection limits");
        assert!(focus.ends_with("-count"));
        state
            .update(&mut draft, Message::Edit(index, Some("unlimited".into())));
        assert!(state.error.is_none());
    }
    #[test]
    fn name_collision_focuses_the_collection_name_without_changing_rows() {
        let mut draft = draft("[metadata]\n'pronouns'='they/them'\n");
        let index = draft
            .collections
            .iter()
            .position(|c| c.kind == Rows::Metadata)
            .unwrap();
        let key = path_id(&draft.collections[index].path);
        let mut state = State::default();
        state.update(&mut draft, Message::EntryName(key, "pronouns".into()));
        let focus = state.update(&mut draft, Message::Add(index)).unwrap();
        assert_eq!(state.group, "Sidebar and metadata");
        assert!(focus.ends_with("-new"));
        assert!(!draft.is_dirty());
    }
}
