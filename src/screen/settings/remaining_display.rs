//! Compact structured editors appended to the selected preference group.
//! All controls use Settings' keyboard-aware buttons, checkboxes and pickers.
use data::config::settings::preferences::Area;
use data::config::settings::remaining_display::{
    DisplayEditors, OpacityKind, OpacityMode, RepeatMode, Target,
    ThemeListMode, ValidationError,
};
use iced::widget::{column, container, row, rule};
use iced::{Alignment, Length};

use super::keyboard::{button, checkbox, pick_list};
use super::style;
use crate::widget::{Element, text, text_input};
use crate::{font, theme};

#[derive(Debug, Clone, Copy)]
pub enum CriterionField {
    Users,
    User,
    Channels,
    Channel,
    Server,
    ServerMessage,
}
#[derive(Debug, Clone, Copy)]
pub enum ColorField {
    Mode,
    Regex,
    Nicknames,
    IgnoreCase,
    Color,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSlot {
    Random,
    Light,
    Dark,
}
#[derive(Debug, Clone)]
pub enum Event {
    ListValue(usize, usize, String),
    ListAdd(usize),
    ListRemove(usize, usize),
    ListMove(usize, usize, bool),
    Scope(usize, usize, String),
    Criterion(usize, usize, CriterionField, String),
    CriterionAdd(usize),
    CriterionRemove(usize, usize),
    Color(usize, ColorField, String),
    ColorAdd,
    ColorRemove(usize),
    ColorMove(usize, bool),
    ThemeMode(ThemeListMode),
    ThemeValue(ThemeSlot, usize, String),
    ThemeAdd(ThemeSlot, String),
    ThemeRemove(ThemeSlot, usize),
    OpacityMode(usize, OpacityMode),
    OpacityAlpha(usize, String),
    OfflineColor(usize, String),
    RepeatMode(usize, RepeatMode),
    RepeatSeconds(usize, String),
    RepeatAfterPreviews(usize, bool),
    OptionalText(usize, String),
    BacklogShow(bool),
    BacklogText(String),
}
fn remove<T>(rows: &mut Vec<T>, index: usize) {
    if index < rows.len() {
        rows.remove(index);
    }
}
fn reorder<T>(rows: &mut [T], index: usize, up: bool) {
    let next = if up {
        index.checked_sub(1)
    } else {
        index.checked_add(1)
    };
    if let Some(next) =
        next.filter(|next| index < rows.len() && *next < rows.len())
    {
        rows.swap(index, next);
    }
}
fn theme_list(
    editor: &mut DisplayEditors,
    slot: ThemeSlot,
) -> &mut Vec<String> {
    match slot {
        ThemeSlot::Random => &mut editor.themes.random,
        ThemeSlot::Light => &mut editor.themes.light,
        ThemeSlot::Dark => &mut editor.themes.dark,
    }
}
pub(super) fn update(editor: &mut DisplayEditors, event: Event) {
    match event {
        Event::ListValue(list, row, value) => {
            if let Some(entry) = editor
                .lists
                .get_mut(list)
                .and_then(|list| list.entries.get_mut(row))
            {
                *entry = value;
            }
        }
        Event::ListAdd(index) => {
            if let Some(list) = editor.lists.get_mut(index)
                && list.fixed_len.is_none()
            {
                list.entries
                    .push(list.options.first().copied().unwrap_or("").into());
            }
        }
        Event::ListRemove(index, row) => {
            if let Some(list) = editor.lists.get_mut(index)
                && list.fixed_len.is_none()
            {
                remove(&mut list.entries, row);
            }
        }
        Event::ListMove(index, row, up) => {
            if let Some(list) = editor.lists.get_mut(index)
                && list.fixed_len.is_none()
            {
                reorder(&mut list.entries, row, up);
            }
        }
        Event::Scope(filter, scope, value) => {
            if let Some(field) = editor
                .filters
                .get_mut(filter)
                .and_then(|filter| filter.scopes.get_mut(scope))
            {
                *field = value;
            }
        }
        Event::Criterion(filter, row, field, value) => {
            if let Some(criterion) = editor
                .filters
                .get_mut(filter)
                .and_then(|filter| filter.criteria.get_mut(row))
            {
                let values = &mut criterion.values;
                *match field {
                    CriterionField::Users => &mut values.users,
                    CriterionField::User => &mut values.user,
                    CriterionField::Channels => &mut values.channels,
                    CriterionField::Channel => &mut values.channel,
                    CriterionField::Server => &mut values.server,
                    CriterionField::ServerMessage => &mut values.server_message,
                } = value;
            }
        }
        Event::CriterionAdd(filter) => editor.add_criterion(filter),
        Event::CriterionRemove(filter, row) => {
            if let Some(filter) = editor.filters.get_mut(filter) {
                remove(&mut filter.criteria, row);
            }
        }
        Event::Color(index, field, value) => {
            if let Some(rule) = editor.colors.get_mut(index) {
                match field {
                    ColorField::Mode => {
                        rule.values.regex_mode = value == "Regular expression";
                    }
                    ColorField::Regex => rule.values.regex = value,
                    ColorField::Nicknames => rule.values.nicknames = value,
                    ColorField::IgnoreCase => {
                        rule.values.ignore_case = value == "true";
                    }
                    ColorField::Color => rule.values.color = value,
                }
            }
        }
        Event::ColorAdd => editor.add_color(),
        Event::ColorRemove(index) => remove(&mut editor.colors, index),
        Event::ColorMove(index, up) => reorder(&mut editor.colors, index, up),
        Event::ThemeMode(mode) => editor.themes.mode = mode,
        Event::ThemeValue(slot, row, value) => {
            if let Some(entry) = theme_list(editor, slot).get_mut(row) {
                *entry = value;
            }
        }
        Event::ThemeAdd(slot, value) => theme_list(editor, slot).push(value),
        Event::ThemeRemove(slot, row) => remove(theme_list(editor, slot), row),
        Event::OpacityMode(index, mode) => {
            if let Some(opacity) = editor.opacities.get_mut(index) {
                opacity.mode = mode;
            }
        }
        Event::OpacityAlpha(index, value) => {
            if let Some(opacity) = editor.opacities.get_mut(index) {
                opacity.alpha = value;
            }
        }
        Event::OfflineColor(index, value) => {
            if let Some(opacity) = editor.opacities.get_mut(index) {
                opacity.offline_color = value;
            }
        }
        Event::RepeatMode(index, mode) => {
            if let Some(repeat) = editor.repeats.get_mut(index) {
                repeat.mode = mode;
            }
        }
        Event::RepeatSeconds(index, value) => {
            if let Some(repeat) = editor.repeats.get_mut(index) {
                repeat.seconds = value;
            }
        }
        Event::RepeatAfterPreviews(index, value) => {
            if let Some(repeat) = editor.repeats.get_mut(index) {
                repeat.show_after_previews = value;
            }
        }
        Event::OptionalText(index, value) => {
            if let Some(field) = editor.optional_text.get_mut(index) {
                field.value = value;
            }
        }
        Event::BacklogShow(show) => editor.backlog.show = show,
        Event::BacklogText(value) => editor.backlog.text = value,
    }
}
fn action<'a>(
    label: &'a str,
    id: String,
    message: Event,
) -> Element<'a, Event> {
    button(text(label))
        .id(id)
        .padding([6, 10])
        .style(style::button)
        .on_press(message)
        .into()
}
fn labelled<'a>(
    label: &'a str,
    widget: Element<'a, Event>,
    width: f32,
) -> Element<'a, Event> {
    row![
        text(label).width(160),
        container(widget).width(Length::Fill.max(width))
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}
fn input<'a>(
    id: String,
    value: &'a str,
    message: impl Fn(String) -> Event + 'a,
) -> Element<'a, Event> {
    text_input("", value)
        .id(id)
        .padding(6)
        .style(style::input)
        .on_input(message)
        .width(Length::Fill)
        .into()
}
fn picker<'a>(
    id: String,
    current: &str,
    mut options: Vec<String>,
    message: impl Fn(String) -> Event + 'a,
) -> Element<'a, Event> {
    if !options.iter().any(|option| option == current) {
        options.push(current.into());
    }
    pick_list(Some(current.to_owned()), options, String::clone)
        .id(id)
        .on_select(message)
        .padding(6)
        .style(style::pick_list)
        .menu_style(style::menu)
        .width(Length::Fill)
        .into()
}
fn group<'a>(rows: Vec<Element<'a, Event>>) -> Element<'a, Event> {
    let mut content = column![].spacing(0);
    for (index, widget) in rows.into_iter().enumerate() {
        if index > 0 {
            content = content.push(
                container(rule::horizontal(1))
                    .padding(iced::padding::horizontal(12)),
            );
        }
        content = content
            .push(container(widget).padding([8, 12]).width(Length::Fill));
    }
    container(content)
        .width(Length::Fill)
        .style(style::pane)
        .into()
}
fn error(
    target: Target,
    validation: Option<&ValidationError>,
) -> Option<Element<'static, Event>> {
    validation
        .filter(|error| error.target == target)
        .map(|error| {
            text(error.message.clone())
                .size(12)
                .style(theme::text::error)
                .into()
        })
}

fn error_at(
    id: &str,
    editor: &DisplayEditors,
    validation: Option<&ValidationError>,
) -> Option<Element<'static, Event>> {
    validation
        .filter(|error| editor.focus_id(error.target) == id)
        .map(|error| {
            text(error.message.clone())
                .size(12)
                .style(theme::text::error)
                .into()
        })
}

/// Append this below the ordinary controls for the selected area/group. It
/// returns a column with flat rows and leaves scrolling/footer to Settings.
pub(super) fn view<'a>(
    editor: &'a DisplayEditors,
    area: Area,
    selected_group: &str,
    validation: Option<&ValidationError>,
) -> Element<'a, Event> {
    let mut sections = column![].spacing(12);
    sections = sections.push(optional_text(
        editor,
        Some(area),
        selected_group,
        validation,
    ));
    for (index, list) in editor.lists.iter().enumerate().filter(|(_, list)| {
        list.area == Some(area) && list.group == selected_group
    }) {
        let mut rows =
            vec![text(list.label).font(font::PRIMARY_BOLD.clone()).into()];
        for (entry, value) in list.entries.iter().enumerate() {
            let id = if entry == 0 {
                Target::List(index).input_id()
            } else {
                format!("settings-display-list-{index}-{entry}")
            };
            let control = if list.options.is_empty() {
                input(id.clone(), value, move |value| {
                    Event::ListValue(index, entry, value)
                })
            } else {
                picker(
                    id.clone(),
                    value,
                    list.options.iter().map(|value| (*value).into()).collect(),
                    move |value| Event::ListValue(index, entry, value),
                )
            };
            let mut controls =
                row![container(control).width(Length::Fill.max(320.0))]
                    .spacing(6)
                    .align_y(Alignment::Center);
            if list.fixed_len.is_none() {
                controls = controls
                    .push(action(
                        "Move up",
                        format!("display-list-up-{index}-{entry}"),
                        Event::ListMove(index, entry, true),
                    ))
                    .push(action(
                        "Move down",
                        format!("display-list-down-{index}-{entry}"),
                        Event::ListMove(index, entry, false),
                    ))
                    .push(action(
                        "Remove",
                        format!("display-list-remove-{index}-{entry}"),
                        Event::ListRemove(index, entry),
                    ));
            }
            rows.push(controls.into());
            if let Some(error) = error_at(&id, editor, validation) {
                rows.push(error);
            }
        }
        if list.fixed_len.is_none() {
            rows.push(action(
                "Add entry",
                if list.entries.is_empty() {
                    Target::List(index).input_id()
                } else {
                    format!("display-list-add-{index}")
                },
                Event::ListAdd(index),
            ));
        }
        if list.entries.is_empty()
            && let Some(error) = error(Target::List(index), validation)
        {
            rows.push(error);
        }
        sections = sections.push(group(rows));
    }
    for (index, opacity) in
        editor.opacities.iter().enumerate().filter(|(_, opacity)| {
            opacity.area == area && opacity.group == selected_group
        })
    {
        let label = match opacity.kind {
            OpacityKind::Away => "Away nickname appearance",
            OpacityKind::Offline => "Offline nickname appearance",
            OpacityKind::Activity => "Activity opacity",
        };
        let mut choices = vec![
            ("Dimmed", OpacityMode::Dimmed),
            ("Solid", OpacityMode::Solid),
            ("Custom opacity", OpacityMode::Custom),
        ];
        if opacity.kind == OpacityKind::Activity {
            choices.insert(0, ("Use default", OpacityMode::Default));
        }
        if opacity.kind == OpacityKind::Offline {
            choices.push(("No special appearance", OpacityMode::Hidden));
        }
        let current = choices
            .iter()
            .find(|(_, mode)| *mode == opacity.mode)
            .map_or("Use default", |(label, _)| *label);
        let mut rows = vec![labelled(
            label,
            picker(
                Target::Opacity(index).input_id(),
                current,
                choices.iter().map(|(label, _)| (*label).into()).collect(),
                move |label| {
                    Event::OpacityMode(
                        index,
                        choices
                            .iter()
                            .find(|(choice, _)| *choice == label)
                            .map_or(OpacityMode::Default, |(_, mode)| *mode),
                    )
                },
            ),
            240.0,
        )];
        if opacity.kind == OpacityKind::Offline
            && opacity.mode != OpacityMode::Hidden
        {
            rows.push(labelled(
                "Offline color",
                picker(
                    format!("display-opacity-color-{index}"),
                    &opacity.offline_color,
                    vec!["theme".into(), "nickname".into()],
                    move |value| Event::OfflineColor(index, value),
                ),
                180.0,
            ));
        }
        if opacity.mode == OpacityMode::Custom {
            rows.push(labelled(
                "Opacity (0–1)",
                input(
                    format!("display-opacity-alpha-{index}"),
                    &opacity.alpha,
                    move |value| Event::OpacityAlpha(index, value),
                ),
                104.0,
            ));
        }
        if let Some(error) = error(Target::Opacity(index), validation) {
            rows.push(error);
        }
        sections = sections.push(group(rows));
    }
    for (index, repeat) in
        editor.repeats.iter().enumerate().filter(|(_, repeat)| {
            repeat.area == area && repeat.group == selected_group
        })
    {
        let current = match repeat.mode {
            RepeatMode::Never => "Show all",
            RepeatMode::Always => "Hide repeats",
            RepeatMode::Smart => "Hide within a time window",
        };
        let mut rows = vec![labelled(
            repeat.label,
            picker(
                Target::Repeat(index).input_id(),
                current,
                vec![
                    "Show all".into(),
                    "Hide repeats".into(),
                    "Hide within a time window".into(),
                ],
                move |value| {
                    Event::RepeatMode(
                        index,
                        match value.as_str() {
                            "Hide repeats" => RepeatMode::Always,
                            "Hide within a time window" => RepeatMode::Smart,
                            _ => RepeatMode::Never,
                        },
                    )
                },
            ),
            260.0,
        )];
        if repeat.mode == RepeatMode::Smart {
            rows.push(labelled(
                "Window (seconds)",
                input(
                    format!("display-repeat-seconds-{index}"),
                    &repeat.seconds,
                    move |value| Event::RepeatSeconds(index, value),
                ),
                104.0,
            ));
        }
        rows.push(
            checkbox(repeat.show_after_previews)
                .label("Show again after previews")
                .style(style::checkbox)
                .on_toggle(move |value| {
                    Event::RepeatAfterPreviews(index, value)
                })
                .into(),
        );
        if let Some(error) = error(Target::Repeat(index), validation) {
            rows.push(error);
        }
        sections = sections.push(group(rows));
    }
    if area == Area::Chat && selected_group == "Messages" {
        let mut rows = vec![
            checkbox(editor.backlog.show)
                .label("Show backlog separator text")
                .style(style::checkbox)
                .on_toggle(Event::BacklogShow)
                .into(),
        ];
        if editor.backlog.show {
            rows.push(labelled(
                "Backlog separator text",
                input(
                    Target::Backlog.input_id(),
                    &editor.backlog.text,
                    Event::BacklogText,
                ),
                240.0,
            ));
        }
        sections = sections.push(group(rows));
    }
    if area == Area::Chat && selected_group == "Names and users" {
        sections = sections.push(color_rules(editor, validation));
    }
    for (index, filter) in
        editor.filters.iter().enumerate().filter(|(_, filter)| {
            filter.area == area && filter.group == selected_group
        })
    {
        let mut rows = vec![text(&filter.label).font(font::PRIMARY_BOLD.clone()).into(), text("Separate names with semicolons; * includes the whole scope. Include takes priority over Exclude.").size(12).style(theme::text::secondary).into()];
        for (scope, label) in
            ["Users", "Channels", "Servers", "Server-message kinds"]
                .iter()
                .enumerate()
        {
            let id = if scope == 0 {
                Target::Filter(index).input_id()
            } else {
                format!("display-filter-{index}-{scope}")
            };
            rows.push(labelled(
                label,
                input(id.clone(), &filter.scopes[scope], move |value| {
                    Event::Scope(index, scope, value)
                }),
                320.0,
            ));
            if let Some(error) = error_at(&id, editor, validation) {
                rows.push(error);
            }
        }
        for (criterion, item) in filter.criteria.iter().enumerate() {
            rows.push(
                row![
                    text(format!("Condition {}", criterion + 1))
                        .font(font::PRIMARY_BOLD.clone()),
                    action(
                        "Remove condition",
                        format!("display-criterion-remove-{index}-{criterion}"),
                        Event::CriterionRemove(index, criterion)
                    )
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .into(),
            );
            rows.push(text("All nonempty fields in one condition must match. An empty condition matches everything.").size(12).style(theme::text::secondary).into());
            for (field_index, (label, value, field)) in [
                ("Users", &item.values.users, CriterionField::Users),
                ("Single user", &item.values.user, CriterionField::User),
                ("Channels", &item.values.channels, CriterionField::Channels),
                (
                    "Single channel",
                    &item.values.channel,
                    CriterionField::Channel,
                ),
                ("Server", &item.values.server, CriterionField::Server),
                (
                    "Server-message kind",
                    &item.values.server_message,
                    CriterionField::ServerMessage,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let id = format!(
                    "display-criterion-{index}-{criterion}-{field_index}"
                );
                rows.push(labelled(
                    label,
                    input(id.clone(), value, move |value| {
                        Event::Criterion(index, criterion, field, value)
                    }),
                    320.0,
                ));
                if let Some(error) = error_at(&id, editor, validation) {
                    rows.push(error);
                }
            }
        }
        rows.push(action(
            "Add condition",
            format!("display-criterion-add-{index}"),
            Event::CriterionAdd(index),
        ));

        sections = sections.push(group(rows));
    }
    sections.into()
}
fn color_rules<'a>(
    editor: &'a DisplayEditors,
    validation: Option<&ValidationError>,
) -> Element<'a, Event> {
    let mut rows = vec![text("Nickname color overrides").font(font::PRIMARY_BOLD.clone()).into(), text("The first matching rule wins. Names use semicolons; colors use #RRGGBB.").size(12).style(theme::text::secondary).into()];
    for (index, rule) in editor.colors.iter().enumerate() {
        let values = &rule.values;
        rows.push(
            row![
                text(format!("Rule {}", index + 1))
                    .font(font::PRIMARY_BOLD.clone()),
                action(
                    "Move up",
                    format!("display-color-up-{index}"),
                    Event::ColorMove(index, true)
                ),
                action(
                    "Move down",
                    format!("display-color-down-{index}"),
                    Event::ColorMove(index, false)
                ),
                action(
                    "Remove rule",
                    format!("display-color-remove-{index}"),
                    Event::ColorRemove(index)
                )
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        );
        rows.push(labelled(
            "Match",
            picker(
                if index == 0 {
                    Target::Colors.input_id()
                } else {
                    format!("display-color-mode-{index}")
                },
                if values.regex_mode {
                    "Regular expression"
                } else {
                    "Nicknames"
                },
                vec!["Nicknames".into(), "Regular expression".into()],
                move |value| Event::Color(index, ColorField::Mode, value),
            ),
            240.0,
        ));
        if values.regex_mode {
            rows.push(labelled(
                "Regular expression",
                input(
                    format!("display-color-regex-{index}"),
                    &values.regex,
                    move |value| Event::Color(index, ColorField::Regex, value),
                ),
                320.0,
            ));
            if let Some(error) = error_at(
                &format!("display-color-regex-{index}"),
                editor,
                validation,
            ) {
                rows.push(error);
            }
        } else {
            rows.push(labelled(
                "Nicknames",
                input(
                    format!("display-color-nicks-{index}"),
                    &values.nicknames,
                    move |value| {
                        Event::Color(index, ColorField::Nicknames, value)
                    },
                ),
                320.0,
            ));
            if let Some(error) = error_at(
                &format!("display-color-nicks-{index}"),
                editor,
                validation,
            ) {
                rows.push(error);
            }
            rows.push(
                checkbox(values.ignore_case)
                    .label("Ignore case")
                    .style(style::checkbox)
                    .on_toggle(move |value| {
                        Event::Color(
                            index,
                            ColorField::IgnoreCase,
                            value.to_string(),
                        )
                    })
                    .into(),
            );
        }
        rows.push(labelled(
            "Color",
            input(
                format!("display-color-value-{index}"),
                &values.color,
                move |value| Event::Color(index, ColorField::Color, value),
            ),
            128.0,
        ));
        if let Some(error) = error_at(
            &format!("display-color-value-{index}"),
            editor,
            validation,
        ) {
            rows.push(error);
        }
    }
    rows.push(action(
        "Add color rule",
        if editor.colors.is_empty() {
            Target::Colors.input_id()
        } else {
            "display-color-add".into()
        },
        Event::ColorAdd,
    ));

    group(rows)
}

/// Pass `themes_for(Single/Light/Dark)` from Settings so brightness filtering
/// remains identical to the existing single-theme menus at c11bb65b.
pub(super) fn theme_lists<'a>(
    editor: &'a DisplayEditors,
    all: Vec<String>,
    light: Vec<String>,
    dark: Vec<String>,
    validation: Option<&ValidationError>,
) -> Element<'a, Event> {
    let current = match editor.themes.mode {
        ThemeListMode::Keep => "Use theme controls above",
        ThemeListMode::Random => "Random theme list",
        ThemeListMode::Adaptive => "Random light and dark lists",
    };
    let mut rows = vec![labelled(
        "Randomized themes",
        picker(
            "display-theme-mode".into(),
            current,
            vec![
                "Use theme controls above".into(),
                "Random theme list".into(),
                "Random light and dark lists".into(),
            ],
            |value| {
                Event::ThemeMode(match value.as_str() {
                    "Random theme list" => ThemeListMode::Random,
                    "Random light and dark lists" => ThemeListMode::Adaptive,
                    _ => ThemeListMode::Keep,
                })
            },
        ),
        280.0,
    )];
    for (slot, label, values, choices) in [
        (ThemeSlot::Random, "Themes", &editor.themes.random, all),
        (
            ThemeSlot::Light,
            "Light themes",
            &editor.themes.light,
            light,
        ),
        (ThemeSlot::Dark, "Dark themes", &editor.themes.dark, dark),
    ] {
        if !matches!(
            (editor.themes.mode, slot),
            (ThemeListMode::Random, ThemeSlot::Random)
                | (ThemeListMode::Adaptive, ThemeSlot::Light | ThemeSlot::Dark)
        ) {
            continue;
        }
        rows.push(text(label).font(font::PRIMARY_BOLD.clone()).into());
        for (index, value) in values.iter().enumerate() {
            rows.push(
                row![
                    container(theme_picker(
                        format!("display-theme-{slot:?}-{index}"),
                        value,
                        choices.clone(),
                        move |value| Event::ThemeValue(slot, index, value)
                    ))
                    .width(Length::Fill.max(320.0)),
                    action(
                        "Remove",
                        format!("display-theme-remove-{slot:?}-{index}"),
                        Event::ThemeRemove(slot, index)
                    )
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        if let Some(first) = choices.first() {
            rows.push(action(
                "Add theme",
                format!("display-theme-add-{slot:?}"),
                Event::ThemeAdd(slot, first.clone()),
            ));
        } else {
            rows.push(text("No matching installed themes. Download a theme from the browser.").size(12).style(theme::text::secondary).into());
        }
    }
    if let Some(error) = error(Target::Themes, validation) {
        rows.push(error);
    }
    group(rows)
}

/// Append to Appearance's Font group alongside the scalar font specs.
pub(super) fn appearance_font<'a>(
    editor: &'a DisplayEditors,
    validation: Option<&ValidationError>,
) -> Element<'a, Event> {
    optional_text(editor, None, "Font", validation)
}
fn optional_text<'a>(
    editor: &'a DisplayEditors,
    area: Option<Area>,
    selected_group: &str,
    validation: Option<&ValidationError>,
) -> Element<'a, Event> {
    let mut rows = vec![];
    for (index, field) in
        editor
            .optional_text
            .iter()
            .enumerate()
            .filter(|(_, field)| {
                field.area == area && field.group == selected_group
            })
    {
        let mut content = column![
            labelled(
                field.label,
                input(
                    Target::OptionalText(index).input_id(),
                    &field.value,
                    move |value| Event::OptionalText(index, value)
                ),
                240.0
            ),
            text(field.help).size(12).style(theme::text::secondary)
        ]
        .spacing(4);
        if let Some(error) = error(Target::OptionalText(index), validation) {
            content = content.push(error);
        }
        rows.push(content.into());
    }
    if rows.is_empty() {
        column![].into()
    } else {
        group(rows)
    }
}

fn theme_picker<'a>(
    id: String,
    current: &str,
    options: Vec<String>,
    message: impl Fn(String) -> Event + 'a,
) -> Element<'a, Event> {
    // Keep an older missing/mismatched selection visible without adding it to
    // the menu's filtered choices. This matches the incumbent theme menus.
    pick_list(Some(current.to_owned()), options, String::clone)
        .id(id)
        .on_select(message)
        .padding(6)
        .style(style::pick_list)
        .menu_style(style::menu)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use data::config::settings::remaining_display::ThemeListMode;

    use super::*;

    fn editor() -> DisplayEditors {
        DisplayEditors::parse(&"".parse().unwrap()).unwrap()
    }
    #[test]
    fn fixed_completion_pair_cannot_be_added_removed_or_reordered() {
        let mut editor = editor();
        let index = editor
            .lists
            .iter()
            .position(|list| list.fixed_len == Some(2))
            .unwrap();
        let initial = editor.lists[index].entries.clone();
        update(&mut editor, Event::ListRemove(index, 0));
        update(&mut editor, Event::ListAdd(index));
        update(&mut editor, Event::ListMove(index, 0, false));
        assert_eq!(editor.lists[index].entries, initial);
        update(&mut editor, Event::ListValue(index, 0, "; ".into()));
        assert_eq!(editor.lists[index].entries[0], "; ");
    }
    #[test]
    fn stale_row_events_and_boundary_moves_are_harmless() {
        let mut editor = editor();
        update(&mut editor, Event::ColorRemove(99));
        update(&mut editor, Event::ColorMove(99, false));
        update(&mut editor, Event::ListRemove(99, 99));
        update(&mut editor, Event::CriterionRemove(99, 99));
        update(&mut editor, Event::Scope(0, 99, "ignored".into()));
        assert!(!editor.is_dirty());
    }
    #[test]
    fn theme_add_and_remove_updates_only_the_requested_slot() {
        let mut editor = editor();
        update(&mut editor, Event::ThemeMode(ThemeListMode::Adaptive));
        update(
            &mut editor,
            Event::ThemeAdd(ThemeSlot::Light, "Light".into()),
        );
        update(&mut editor, Event::ThemeAdd(ThemeSlot::Dark, "Dark".into()));
        update(&mut editor, Event::ThemeRemove(ThemeSlot::Light, 0));
        assert!(editor.themes.light.is_empty());
        assert_eq!(editor.themes.dark, ["Dark"]);
    }
}
