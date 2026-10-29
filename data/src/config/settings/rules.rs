//! Notification, highlight, and shortcut drafts. Keep original items and update
//! only edited leaves, including inline tables and arrays of highlight tables.
use toml_edit::{
    Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, TableLike,
    Value, value,
};

use super::{child_table, set};
use crate::config::inclusivities::{Inclusivities, Inclusivity};
use crate::config::keys::Keyboard;
use crate::shortcut::KeyBind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Notifications,
    Highlights,
    Keyboard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Boolean,
    Text,
    List,
    Number,
    Binding,
    MatchType,
    Channel,
}
#[derive(Debug, Clone)]
pub struct Field {
    pub label: String,
    pub path: Vec<String>,
    pub kind: Kind,
    pub value: String,
    original: String,
    pub default: String,
    pub reset: bool,
    present: bool,
}
#[derive(Debug, Clone)]
pub struct Group {
    pub area: Area,
    pub title: String,
    pub fields: Vec<Field>,
    pub conditions: Option<super::remaining_display::DisplayEditors>,
    pub removable: bool,
    pub removed: bool,
    path: Vec<String>,
    original: Option<Item>,
    highlight: bool,
}
#[derive(Debug, Clone)]
pub struct Rules {
    pub groups: Vec<Group>,
}

const EVENTS: &[(&str, &str)] = &[
    ("direct_message", "Direct messages"),
    ("highlight", "Highlights"),
    ("reaction", "Reactions"),
    ("file_transfer_request", "File transfers"),
    ("connected", "Connected"),
    ("disconnected", "Disconnected"),
    ("reconnected", "Reconnected"),
    ("monitored_online", "Monitored online"),
    ("monitored_offline", "Monitored offline"),
];

fn lookup<'a>(mut item: &'a Item, path: &[String]) -> Option<&'a Item> {
    for key in path {
        item = item.as_table_like()?.get(key)?;
    }
    Some(item)
}
fn string_list(item: Option<&Item>) -> Vec<String> {
    match item {
        Some(item) if item.is_str() => vec![item.as_str().unwrap().into()],
        Some(item) => item
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        None => vec![],
    }
}
/// Semicolons separate entries; quoted TOML strings allow literal semicolons,
/// quotes, leading/trailing spaces, or an empty string without losing data.
pub fn display_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| {
            if s.is_empty() || s.contains([';', '"']) || s.trim() != s {
                serde_json::to_string(s).expect("strings serialize")
            } else {
                s.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}
pub fn parse_list(input: &str) -> Result<Vec<String>, String> {
    if input.trim().is_empty() {
        return Ok(vec![]);
    }
    let mut parts = vec![];
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if c == ';' && !quoted {
            parts.push(&input[start..i]);
            start = i + 1;
        }
    }
    if quoted {
        return Err("Close the quoted entry with a double quote.".into());
    }
    parts.push(&input[start..]);
    parts.into_iter().map(|part| {
        let part = part.trim();
        if part.starts_with('"') {
            part.parse::<Value>().ok().and_then(|v| v.as_str().map(str::to_owned)).ok_or_else(|| "Use a valid quoted string, or separate entries with semicolons.".into())
        } else if part.is_empty() { Err("Remove the empty entry between semicolons.".into()) }
        else { Ok(part.to_owned()) }
    }).collect()
}
fn array(strings: Vec<String>) -> Item {
    let mut a = Array::new();
    for s in strings {
        a.push(s);
    }
    value(a)
}
impl Field {
    fn new(
        label: &str,
        path: &str,
        kind: Kind,
        default: &str,
        source: &Item,
    ) -> Self {
        let path = path.split('.').map(str::to_owned).collect::<Vec<_>>();
        let item = lookup(source, &path);
        let text = match kind {
            Kind::List | Kind::Binding => {
                item.map(|i| display_list(&string_list(Some(i))))
            }
            Kind::Boolean => {
                item.and_then(Item::as_bool).map(|v| v.to_string())
            }
            Kind::Number => {
                item.and_then(Item::as_integer).map(|v| v.to_string())
            }
            _ => item.and_then(Item::as_str).map(str::to_owned),
        }
        .unwrap_or_else(|| default.into());
        Self {
            label: label.into(),
            path,
            kind,
            original: text.clone(),
            value: text,
            default: default.into(),
            reset: false,
            present: item.is_some(),
        }
    }
    fn changed(&self) -> bool {
        self.reset && self.present || self.value != self.original
    }
    fn render(&self) -> Result<Option<Item>, String> {
        if self.reset {
            return Ok(None);
        }
        Ok(match self.kind {
            Kind::Boolean => Some(value(
                self.value
                    .parse::<bool>()
                    .map_err(|_| "Choose enabled or disabled.")?,
            )),
            Kind::Number => {
                if self.value.is_empty() {
                    None
                } else {
                    Some(value(i64::from(self.value.parse::<u32>().map_err(|_| "Enter a delay from 0 to 4294967295 milliseconds, or leave it empty for 500.")?)))
                }
            }
            Kind::List | Kind::Binding => {
                let items = parse_list(&self.value)?;
                if self.kind == Kind::Binding {
                    for binding in &items {
                        let _: KeyBind = toml::Value::String(binding.clone()).try_into().map_err(|error: toml::de::Error| format!("Invalid binding ‘{binding}’: {error} Use a key such as command+k or ctrl+shift+k."))?;
                    }
                    Some(if items.is_empty() {
                        value("none")
                    } else {
                        array(items)
                    })
                } else if items.is_empty() {
                    None
                } else {
                    Some(array(items))
                }
            }
            Kind::MatchType | Kind::Channel => None,
            Kind::Text => {
                if self.value.is_empty() {
                    None
                } else {
                    Some(value(&self.value))
                }
            }
        })
    }
}
impl Group {
    pub fn label(&self) -> String {
        if self.highlight {
            self.fields
                .iter()
                .find(|f| f.path == ["name"])
                .filter(|f| !f.value.is_empty())
                .map_or_else(|| self.title.clone(), |f| f.value.clone())
        } else if self.removable {
            self.fields.first().map_or_else(
                || self.title.clone(),
                |f| {
                    if f.value.is_empty() {
                        "New channel".into()
                    } else {
                        f.value.clone()
                    }
                },
            )
        } else {
            self.title.clone()
        }
    }
    pub fn is_highlight(&self) -> bool {
        self.highlight
    }
    pub fn visible(&self, field: usize) -> bool {
        if !self.highlight {
            return true;
        }
        let mode = &self.fields[1].value;
        match self.fields[field].path.first().map(String::as_str) {
            Some("words" | "case_insensitive") => mode == "Words",
            Some("regex") => mode == "Regular expression",
            _ => true,
        }
    }
    fn changed(&self) -> bool {
        self.removed
            || self.original.is_none() && self.removable
            || self.fields.iter().any(Field::changed)
            || self
                .conditions
                .as_ref()
                .is_some_and(super::remaining_display::DisplayEditors::is_dirty)
    }
    fn render(&self) -> Result<Item, (usize, String)> {
        if !self.changed()
            && let Some(original) = &self.original
        {
            return Ok(original.clone());
        }
        let mut item = self
            .original
            .clone()
            .unwrap_or_else(|| Item::Table(Table::new()));
        let table = item.as_table_like_mut().ok_or((
            0,
            "Edit this unsupported table in the configuration file.".into(),
        ))?;
        if self.highlight
            && (self.original.is_none() || self.fields[1].changed())
        {
            if self.fields[1].value == "Words" {
                table.remove("regex");
            } else {
                table.remove("words");
            }
        }
        for (index, f) in self.fields.iter().enumerate() {
            if !self.visible(index)
                || matches!(f.kind, Kind::MatchType | Kind::Channel)
            {
                continue;
            }
            if !f.changed() {
                continue;
            }
            let next = f
                .render()
                .map_err(|e| (index, format!("{}: {e}", f.label)))?;
            let mut parent: &mut dyn TableLike = &mut *table;
            for key in &f.path[..f.path.len() - 1] {
                // Legacy lists and all-wildcards have the same semantic model.
                // Project only on an explicit edit, leaving all other leaves intact.
                if matches!(key.as_str(), "include" | "exclude")
                    && parent
                        .get(key)
                        .is_some_and(|i| i.as_table_like().is_none())
                {
                    let old = parent.get(key).unwrap();
                    let inclusivities: Inclusivities =
                        toml::from_str::<toml::Value>(&format!(
                            "conditions = {old}"
                        ))
                        .map_err(|e| (index, e.to_string()))?
                        .get("conditions")
                        .unwrap()
                        .clone()
                        .try_into()
                        .map_err(|e: toml::de::Error| (index, e.to_string()))?;
                    let mut converted = InlineTable::new();
                    for (name, values) in [
                        ("users", inclusivities.users),
                        ("channels", inclusivities.channels),
                        ("servers", inclusivities.servers),
                        ("server_messages", inclusivities.server_messages),
                    ] {
                        if let Some(values) = values {
                            converted.insert(
                                name,
                                match values {
                                    Inclusivity::All => Value::from("*"),
                                    Inclusivity::Any(items) => {
                                        array(items).into_value().unwrap()
                                    }
                                },
                            );
                        }
                    }
                    set(parent, key, value(converted));
                }
                parent = child_table(parent, key).map_err(|e| (index, e))?;
            }
            let key = f.path.last().unwrap();
            if let Some(mut next) = next {
                if matches!(key.as_str(), "users" | "channels" | "servers")
                    && matches!(f.value.trim(), "*" | "all")
                {
                    next = value("*");
                }
                set(parent, key, next);
            } else {
                parent.remove(key);
            }
        }
        // A new rule, or changing its type, must write its active pattern even
        // when its initial empty draft has not otherwise changed.
        if self.highlight {
            let words = self.fields[1].value == "Words";
            let index = if words { 2 } else { 3 };
            let f = &self.fields[index];
            let pattern_changed = self.original.is_none()
                || self.fields[1].changed()
                || f.changed();
            if pattern_changed && f.value.trim().is_empty() {
                return Err((
                    index,
                    if words {
                        "Enter at least one word or phrase."
                    } else {
                        "Enter a regular expression."
                    }
                    .into(),
                ));
            }
            if !words && pattern_changed {
                fancy_regex::Regex::new(&f.value).map_err(|e| {
                    (index, format!("Invalid regular expression: {e}"))
                })?;
            }
            if self.original.is_none() || self.fields[1].changed() {
                set(
                    table,
                    if words { "words" } else { "regex" },
                    f.render().map_err(|e| (index, e))?.unwrap(),
                );
            }
        }
        if let Some(conditions) = &self.conditions
            && conditions.is_dirty()
        {
            let mut document = DocumentMut::new();
            let table = item.as_table_like().unwrap();
            for (key, value) in table.iter() {
                document
                    .as_table_mut()
                    .insert_formatted(table.key(key).unwrap(), value.clone());
            }
            conditions
                .render_into(&mut document)
                .map_err(|message| (0, message))?;
            for filter in
                conditions.filters.iter().filter(|filter| filter.is_dirty())
            {
                if let Some(value) = document.remove(&filter.path) {
                    set(item.as_table_like_mut().unwrap(), &filter.path, value);
                }
            }
        }
        Ok(item)
    }
}

fn condition_editor(
    source: &Item,
    fields: &[Field],
) -> Option<super::remaining_display::DisplayEditors> {
    fields
        .iter()
        .any(|field| {
            field.path.first().is_some_and(|key| {
                matches!(key.as_str(), "include" | "exclude")
            })
        })
        .then(|| {
            super::remaining_display::DisplayEditors::conditions(source)
                .expect("validated conditions")
        })
}

fn conditions(fields: &mut Vec<Field>, source: &Item) {
    for (key, prefix) in [("exclude", "Exclude"), ("include", "Include")] {
        let projected = lookup(source, &[key.into()])
            .filter(|i| i.as_table_like().is_none())
            .and_then(|i| {
                toml::from_str::<toml::Value>(&format!("v = {i}"))
                    .ok()?
                    .get("v")?
                    .clone()
                    .try_into::<Inclusivities>()
                    .ok()
            });
        for (leaf, label) in [
            ("channels", "channels"),
            ("users", "users"),
            ("servers", "servers"),
        ] {
            let mut field = Field::new(
                &format!("{prefix} {label}"),
                &format!("{key}.{leaf}"),
                Kind::List,
                "",
                source,
            );
            if let Some(projected) = &projected {
                let values = match leaf {
                    "channels" => &projected.channels,
                    "users" => &projected.users,
                    _ => &projected.servers,
                };
                field.value = match values {
                    Some(Inclusivity::All) => "*".into(),
                    Some(Inclusivity::Any(values)) => display_list(values),
                    None => String::new(),
                };
                field.original = field.value.clone();
            }
            fields.push(field);
        }
    }
}
fn notification(
    path: Vec<String>,
    title: String,
    original: Option<Item>,
    channel: bool,
) -> Group {
    let empty = Item::Table(Table::new());
    let source = original.as_ref().unwrap_or(&empty);
    let mut fields = vec![];
    if channel {
        let mut f = Field::new("Channel", "", Kind::Channel, "", source);
        f.value = path.last().unwrap().clone();
        f.original = f.value.clone();
        fields.push(f);
    }
    for (path, label) in [
        ("show_toast", "Show a system notification"),
        ("request_attention", "Request attention"),
        ("show_content", "Show message content"),
    ] {
        fields.push(Field::new(label, path, Kind::Boolean, "false", source));
    }
    fields.push(Field::new("Sounds", "sound", Kind::List, "", source));
    fields.push(Field::new(
        "Delay (ms)",
        "delay",
        Kind::Number,
        "500",
        source,
    ));
    if channel
        || matches!(
            path.last().map(String::as_str),
            Some(
                "direct_message"
                    | "highlight"
                    | "file_transfer_request"
                    | "reaction"
            )
        )
    {
        conditions(&mut fields, source);
    }
    Group {
        area: Area::Notifications,
        title,
        conditions: condition_editor(source, &fields),
        fields,
        removable: channel,
        removed: false,
        path,
        original,
        highlight: false,
    }
}
fn highlight(original: Option<Item>, number: usize, regex: bool) -> Group {
    let empty = Item::Table(Table::new());
    let source = original.as_ref().unwrap_or(&empty);
    let words = source
        .as_table_like()
        .is_some_and(|t| t.contains_key("words"));
    let mode = if words || original.is_none() && !regex {
        "Words"
    } else {
        "Regular expression"
    };
    let mut fields = vec![
        Field::new("Rule name", "name", Kind::Text, "", source),
        Field::new("Match", "", Kind::MatchType, mode, source),
        Field::new("Words or phrases", "words", Kind::List, "", source),
        Field::new("Regular expression", "regex", Kind::Text, "", source),
        Field::new(
            "Ignore case",
            "case_insensitive",
            Kind::Boolean,
            "false",
            source,
        ),
        Field::new("Sounds", "sound", Kind::List, "", source),
    ];
    conditions(&mut fields, source);
    Group {
        area: Area::Highlights,
        title: format!("Rule {number}"),
        conditions: condition_editor(source, &fields),
        fields,
        removable: true,
        removed: false,
        path: vec![],
        original,
        highlight: true,
    }
}
impl Rules {
    pub fn parse(document: &DocumentMut) -> Self {
        let root = document.as_item();
        let mut groups = EVENTS
            .iter()
            .map(|(key, label)| {
                let path = vec!["notifications".into(), (*key).into()];
                notification(
                    path.clone(),
                    (*label).into(),
                    lookup(root, &path).cloned(),
                    false,
                )
            })
            .collect::<Vec<_>>();
        if let Some(channels) =
            lookup(root, &["notifications".into(), "channel".into()])
                .and_then(Item::as_table_like)
        {
            for (key, item) in channels.iter() {
                groups.push(notification(
                    vec!["notifications".into(), "channel".into(), key.into()],
                    key.into(),
                    Some(item.clone()),
                    true,
                ));
            }
        }
        let path = vec!["highlights".into(), "nickname".into()];
        let original = lookup(root, &path).cloned();
        let empty = Item::Table(Table::new());
        let source = original.as_ref().unwrap_or(&empty);
        let mut fields = vec![Field::new(
            "Ignore nickname case",
            "case_insensitive",
            Kind::Boolean,
            "true",
            source,
        )];
        conditions(&mut fields, source);
        groups.push(Group {
            area: Area::Highlights,
            title: "Nickname".into(),
            conditions: condition_editor(source, &fields),
            fields,
            removable: false,
            removed: false,
            path,
            original,
            highlight: false,
        });
        if let Some(matches) =
            lookup(root, &["highlights".into(), "match".into()])
        {
            if let Some(tables) = matches.as_array_of_tables() {
                for (i, table) in tables.iter().enumerate() {
                    groups.push(highlight(
                        Some(Item::Table(table.clone())),
                        i + 1,
                        false,
                    ));
                }
            } else if let Some(array) = matches.as_array() {
                for (i, item) in array.iter().enumerate() {
                    groups.push(highlight(
                        Some(Item::Value(item.clone())),
                        i + 1,
                        false,
                    ));
                }
            }
        }
        let keyboard = lookup(root, &["keyboard".into()])
            .cloned()
            .unwrap_or_else(|| Item::Table(Table::new()));
        let defaults = Keyboard::default();
        macro_rules! keys {
            ($title:literal, $($key:ident),+ $(,)?) => {{
                let mut fields = vec![];
                $(let default = display_list(&defaults.$key.iter().map(|k|k.as_config_string().replace('↑',"up").replace('↓',"down").replace('←',"left").replace('→',"right")).collect::<Vec<_>>());
                let path = if stringify!($key) == "highlights" && keyboard.as_table_like().is_some_and(|t| !t.contains_key("highlights") && t.contains_key("highlight")) {"highlight"} else {stringify!($key)};
                fields.push(Field::new(&stringify!($key).replace('_'," "),path,Kind::Binding,&default,&keyboard));)+
                groups.push(Group{conditions:None,area:Area::Keyboard,title:$title.into(),fields,removable:false,removed:false,path:vec!["keyboard".into()],original:Some(keyboard.clone()),highlight:false});
            }};
        }
        keys!(
            "Panes and navigation",
            move_up,
            move_down,
            move_left,
            move_right,
            new_horizontal_buffer,
            new_vertical_buffer,
            close_buffer,
            maximize_buffer,
            restore_buffer,
            cycle_next_buffer,
            cycle_previous_buffer,
            leave_buffer,
            cycle_next_unread_buffer,
            cycle_previous_unread_buffer
        );
        keys!(
            "Tools and display",
            toggle_nick_list,
            toggle_topic,
            toggle_sidebar,
            toggle_fullscreen,
            command_bar,
            reload_configuration,
            file_transfers,
            logs,
            theme_editor,
            highlights,
            search,
            quit_application,
            config_editor_save,
            open_config_editor,
            open_config_file,
            show_muted_buffers,
            hide_muted_buffers
        );
        keys!(
            "Scrolling and read state",
            scroll_up_page,
            scroll_down_page,
            scroll_to_top,
            scroll_to_bottom,
            scroll_to_backlog,
            mark_as_read
        );
        keys!(
            "Message focus",
            focus_up,
            focus_down,
            focus_left,
            focus_right,
            focus_activate,
            focus_activate_alt,
            focus_reply,
            focus_react,
            focus_redact
        );
        Self { groups }
    }
    pub fn add(&mut self, area: Area, regex: bool) -> usize {
        let group = match area {
            Area::Highlights => highlight(
                None,
                self.groups.iter().filter(|g| g.highlight).count() + 1,
                regex,
            ),
            _ => notification(
                vec!["notifications".into(), "channel".into(), String::new()],
                "New channel".into(),
                None,
                true,
            ),
        };
        self.groups.push(group);
        self.groups.len() - 1
    }
    pub fn is_dirty(&self) -> bool {
        self.groups
            .iter()
            .any(|g| g.changed() && !(g.removed && g.original.is_none()))
    }
    pub fn render_into(
        &self,
        document: &mut DocumentMut,
    ) -> Result<(), (usize, usize, String)> {
        let mut channels = std::collections::HashSet::new();
        for (index, g) in self.groups.iter().enumerate().filter(|(_, g)| {
            g.area == Area::Notifications && g.removable && !g.removed
        }) {
            let name = &g.fields[0].value;
            if name.is_empty()
                || !name.starts_with(['#', '&', '+', '!'])
                || name.chars().any(char::is_whitespace)
                || name.contains([',', ':', '\0', '\u{7}'])
            {
                return Err((index,0,"Enter a channel name beginning with #, &, + or !, without spaces, commas or colons.".into()));
            }
            if !channels.insert(name) {
                return Err((
                    index,
                    0,
                    format!("A notification rule for {name} already exists."),
                ));
            }
        }
        // Remove/rename channel entries before inserting replacements (allows swaps).
        for g in self.groups.iter().filter(|g| {
            g.area == Area::Notifications && g.removable && g.changed()
        }) {
            if g.original.is_some()
                && (g.removed || g.path.last() != Some(&g.fields[0].value))
                && let Some(table) = document
                    .get_mut("notifications")
                    .and_then(Item::as_table_like_mut)
                    .and_then(|t| t.get_mut("channel"))
                    .and_then(Item::as_table_like_mut)
            {
                table.remove(g.path.last().unwrap());
            }
        }
        for (index, g) in self
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.highlight && !g.removed && g.changed())
        {
            if g.area == Area::Keyboard {
                // Groups share one table; merge only their changed leaves.
                let parent = child_table(document.as_table_mut(), "keyboard")
                    .map_err(|e| (index, 0, e))?;
                for (field, f) in
                    g.fields.iter().enumerate().filter(|(_, f)| f.changed())
                {
                    if let Some(item) = f.render().map_err(|e| {
                        (index, field, format!("{}: {e}", f.label))
                    })? {
                        set(parent, &f.path[0], item);
                    } else {
                        parent.remove(&f.path[0]);
                    }
                }
                continue;
            }
            let item = g.render().map_err(|(f, e)| (index, f, e))?;
            let mut path = g.path.clone();
            if g.removable {
                *path.last_mut().unwrap() = g.fields[0].value.clone();
            }
            let mut parent: &mut dyn TableLike = document.as_table_mut();
            for part in &path[..path.len() - 1] {
                parent =
                    child_table(parent, part).map_err(|e| (index, 0, e))?;
            }
            set(parent, path.last().unwrap(), item);
        }
        if self.groups.iter().any(|g| {
            g.highlight && g.changed() && !(g.removed && g.original.is_none())
        }) {
            let parent = child_table(document.as_table_mut(), "highlights")
                .map_err(|e| (0, 0, e))?;
            let inline = parent.get("match").is_some_and(Item::is_array);
            if inline {
                let mut items = parent
                    .get("match")
                    .and_then(Item::as_array)
                    .unwrap()
                    .clone();
                items.clear();
                for (index, g) in self
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.highlight && !g.removed)
                {
                    let item = g.render().map_err(|(f, e)| (index, f, e))?;
                    let v = match item {
                        Item::Table(t) => {
                            Value::InlineTable(t.into_inline_table())
                        }
                        Item::Value(v) => v,
                        _ => unreachable!(),
                    };
                    items.push_formatted(v);
                }
                set(parent, "match", value(items));
            } else {
                let mut tables = ArrayOfTables::new();
                for (index, g) in self
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.highlight && !g.removed)
                {
                    let item = g.render().map_err(|(f, e)| (index, f, e))?;
                    let t = match item {
                        Item::Table(t) => t,
                        Item::Value(Value::InlineTable(t)) => t.into_table(),
                        _ => unreachable!(),
                    };
                    tables.push(t);
                }
                if tables.is_empty() {
                    parent.remove("match");
                } else {
                    set(parent, "match", Item::ArrayOfTables(tables));
                }
            }
        }
        Ok(())
    }
    pub fn validation_error(
        &self,
        source: &DocumentMut,
    ) -> Option<(usize, usize, String)> {
        let mut document = source.clone();
        if let Err(e) = self.render_into(&mut document) {
            return Some(e);
        }
        if let Err(error) = super::Config::validate(&document.to_string()) {
            // Runtime validation checks conflicts across every keyboard group.
            // Locate an edited field in the reported actions, including aliases.
            let message = error.to_string();
            let target = self
                .groups
                .iter()
                .enumerate()
                .flat_map(|(g, group)| {
                    group
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| f.changed())
                        .map(move |(f, field)| (g, f, field))
                })
                .find(|(_, _, f)| {
                    f.kind == Kind::Binding
                        && message.contains(match f.path[0].as_str() {
                            "toggle_nick_list" => "toggle_nicklist",
                            "highlight" => "highlights",
                            path => path,
                        })
                });
            let target = target.or_else(|| {
                self.groups
                    .iter()
                    .enumerate()
                    .find(|(_, g)| g.changed())
                    .map(|(g, group)| (g, 0, &group.fields[0]))
            });
            return target.map(|(g, f, _)| (g, f, message));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{Settings, ValidationTarget};

    fn group(settings: &Settings, title: &str) -> usize {
        settings
            .rules
            .groups
            .iter()
            .position(|g| g.title == title)
            .unwrap()
    }
    fn edit(settings: &mut Settings, group: usize, path: &str, text: &str) {
        settings.rules.groups[group]
            .fields
            .iter_mut()
            .find(|f| f.path.join(".") == path)
            .unwrap()
            .value = text.into();
    }
    #[test]
    fn notifications_keep_quoted_keys_inline_tables_comments_and_conditions() {
        let source = "# header\n[notifications]\ndirect_message = { sound = ['peck', 'dong'], exclude = { criteria = [{ user = 'bot', server = 'Libera' }], channels = ['#quiet'] } } # inline\n[notifications.channel.\"#a.b\"]\n\"show_toast\" = true # toast\nsound = 'zone'\n[servers.test]\nserver = 'localhost'\nnickname = 'test'\nautoconnect = false\n";
        let mut s = Settings::parse(source.into()).unwrap();
        assert_eq!(s.render().unwrap(), source);
        let g = group(&s, "#a.b");
        edit(&mut s, g, "show_toast", "false");
        let g = group(&s, "Direct messages");
        edit(&mut s, g, "sound", "bonk; custom.wav");
        let rendered = s.render().unwrap();
        assert!(rendered.contains("\"show_toast\" = false # toast"));
        assert!(
            rendered
                .contains("criteria = [{ user = 'bot', server = 'Libera' }]")
        );
        assert!(rendered.contains("channels = ['#quiet']"));
        assert!(rendered.contains("# inline"));
        assert!(rendered.contains("[notifications.channel.\"#a.b\"]"));
        let clean = Settings::parse(rendered).unwrap();
        assert!(!clean.is_dirty());
    }
    #[test]
    fn conditions_convert_legacy_without_losing_other_scopes() {
        let mut s=Settings::parse("[notifications.highlight]\nexclude = ['NickServ', '#quiet'] # legacy\n".into()).unwrap();
        let g = group(&s, "Highlights");
        edit(&mut s, g, "exclude.servers", "Libera");
        let rendered = s.render().unwrap();
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let exclude = &parsed["notifications"]["highlight"]["exclude"];
        assert_eq!(exclude["users"][0].as_str(), Some("NickServ"));
        assert_eq!(exclude["channels"][0].as_str(), Some("#quiet"));
        assert_eq!(exclude["servers"][0].as_str(), Some("Libera"));
        assert!(rendered.contains("# legacy"));
        let mut s =
            Settings::parse("[highlights.nickname]\nexclude = '*'\n".into())
                .unwrap();
        let g = group(&s, "Nickname");
        edit(&mut s, g, "exclude.channels", "");
        let rendered = s.render().unwrap();
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(
            parsed["highlights"]["nickname"]["exclude"]["users"].as_str(),
            Some("*")
        );
        assert_eq!(
            parsed["highlights"]["nickname"]["exclude"]["servers"].as_str(),
            Some("*")
        );
    }
    #[test]
    fn named_highlights_add_edit_remove_and_switch_match_type() {
        let source = "# keep\n[[highlights.match]]\nwords = ['boat'] # pattern\nname = 'Boats'\nexclude = { criteria = [{ channel = '#a', server = 'Libera' }] }\n[[highlights.match]]\nregex = 'cargo' # second\n";
        let mut s = Settings::parse(source.into()).unwrap();
        let g = group(&s, "Rule 1");
        edit(&mut s, g, "name", "Ships");
        let new = s.rules.add(Area::Highlights, true);
        edit(&mut s, new, "regex", r"(?i)\bcaptain\b");
        let rendered = s.render().unwrap();
        assert!(rendered.contains("words = ['boat'] # pattern"));
        assert!(rendered.contains("regex = 'cargo' # second"));
        assert!(
            rendered
                .contains("criteria = [{ channel = '#a', server = 'Libera' }]")
        );
        s.rules.groups[g].fields[1].value = "Regular expression".into();
        edit(&mut s, g, "regex", "ship.*");
        let removed = group(&s, "Rule 2");
        s.rules.groups[removed].removed = true;
        let rendered = s.render().unwrap();
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let rules = parsed["highlights"]["match"].as_array().unwrap();
        assert_eq!(rules.len(), 2);
        assert!(rules[0].get("words").is_none());
        assert_eq!(rules[0]["regex"].as_str(), Some("ship.*"));
        assert!(!Settings::parse(rendered).unwrap().is_dirty());
    }
    #[test]
    fn inline_highlights_and_untouched_empty_word_rules_survive() {
        let source = "[highlights]\nmatch = [{ words = [], name = 'Empty' }, { regex = 'ok', sound = 'peck' }] # list\n";
        let mut s = Settings::parse(source.into()).unwrap();
        let g = group(&s, "Rule 2");
        edit(&mut s, g, "sound", "dong");
        let rendered = s.render().unwrap();
        assert!(rendered.contains("words = [], name = 'Empty'"));
        assert!(rendered.contains("# list"));
        assert!(rendered.contains("match = ["));
    }
    #[test]
    fn invalid_rules_focus_the_actual_field() {
        let mut s = Settings::parse(String::new()).unwrap();
        let g = s.rules.add(Area::Highlights, true);
        edit(&mut s, g, "regex", "[");
        let e = s.validation_error().unwrap();
        assert_eq!(e.target, ValidationTarget::Rule { group: g, field: 3 });
        assert!(e.message.contains("regular expression"));
        edit(&mut s, g, "regex", "valid");
        let n = group(&s, "Direct messages");
        edit(&mut s, n, "delay", "-1");
        let e = s.validation_error().unwrap();
        assert_eq!(e.target, ValidationTarget::Rule { group: n, field: 4 });
    }
    #[test]
    fn channel_rules_validate_duplicates_rename_and_cancel_new_rules() {
        let mut s = Settings::parse(
            "[notifications.channel.'#old']\nshow_toast = true # keep\n".into(),
        )
        .unwrap();
        let old = group(&s, "#old");
        s.rules.groups[old].fields[0].value = "#new.with.dot".into();
        let rendered = s.render().unwrap();
        assert!(!rendered.contains("'#old'"));
        assert!(rendered.contains("# keep"));
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(
            parsed["notifications"]["channel"]["#new.with.dot"]["show_toast"]
                .as_bool(),
            Some(true)
        );
        let new = s.rules.add(Area::Notifications, false);
        s.rules.groups[new].fields[0].value = "#new.with.dot".into();
        assert!(
            s.validation_error()
                .unwrap()
                .message
                .contains("already exists")
        );
        s.rules.groups[new].removed = true;
        assert!(s.render().is_ok());
        let mut clean = Settings::parse(String::new()).unwrap();
        let new = clean.rules.add(Area::Highlights, false);
        clean.rules.groups[new].removed = true;
        assert!(!clean.is_dirty());
    }
    #[test]
    fn keyboard_conflicts_are_checked_on_the_combined_draft_and_can_be_resolved()
     {
        let mut s = Settings::parse(String::new()).unwrap();
        let g = group(&s, "Tools and display");
        edit(&mut s, g, "logs", "command+k");
        let e = s.validation_error().unwrap();
        assert!(e.message.contains("command_bar"));
        assert!(e.message.contains("logs"));
        let f = s.rules.groups[g]
            .fields
            .iter()
            .position(|f| f.path == ["logs"])
            .unwrap();
        assert_eq!(e.target, ValidationTarget::Rule { group: g, field: f });
        edit(&mut s, g, "command_bar", "");
        assert!(s.render().is_ok());
        edit(&mut s, g, "logs", "invalidmodifier+k");
        assert!(
            s.validation_error()
                .unwrap()
                .message
                .contains("Invalid binding")
        );
    }
    #[test]
    fn keyboard_clear_reset_aliases_and_settings_override_use_runtime_models() {
        let mut s = Settings::parse(
            "[keyboard]\n\"highlight\" = 'command+9' # alias\n".into(),
        )
        .unwrap();
        let g = group(&s, "Tools and display");
        let f = s.rules.groups[g]
            .fields
            .iter()
            .position(|f| f.path == ["highlight"])
            .unwrap();
        s.rules.groups[g].fields[f].reset = true;
        s.rules.groups[g].fields[f].value =
            s.rules.groups[g].fields[f].default.clone();
        edit(&mut s, g, "open_config_editor", "command+,");
        edit(&mut s, g, "logs", "");
        let rendered = s.render().unwrap();
        let k: Keyboard =
            toml::from_str::<toml::Value>(&rendered).unwrap()["keyboard"]
                .clone()
                .try_into()
                .unwrap();
        assert!(!k.settings_shortcut_available());
        assert!(k.logs.iter().next().is_none());
        assert_eq!(
            k.highlights.primary(),
            Keyboard::default().highlights.primary()
        );
        assert!(!rendered.contains("highlight\""));
        let mut clean = Settings::parse(String::new()).unwrap();
        let g = group(&clean, "Message focus");
        let f = clean.rules.groups[g]
            .fields
            .iter()
            .position(|f| f.path == ["focus_activate"])
            .unwrap();
        clean.rules.groups[g].fields[f].reset = true;
        assert!(!clean.is_dirty());
    }
    #[test]
    fn all_keyboard_actions_and_notification_events_round_trip_through_runtime()
    {
        let s = Settings::parse(String::new()).unwrap();
        let mut count = 0;
        for (g, group) in s.rules.groups.iter().enumerate() {
            for (f, _) in group
                .fields
                .iter()
                .enumerate()
                .filter(|(_, f)| f.kind == Kind::Binding)
            {
                count += 1;
                let mut edited = s.clone();
                edited.rules.groups[g].fields[f].original =
                    "force write".into();
                let rendered = edited.render().unwrap();
                super::super::Config::validate(&rendered).unwrap();
            }
            if group.area == Area::Notifications {
                let mut edited = s.clone();
                edit(&mut edited, g, "show_toast", "true");
                edit(&mut edited, g, "sound", "peck; dong");
                assert!(edited.render().is_ok());
            }
        }
        assert_eq!(count, 46);
    }
    #[test]
    fn semicolon_list_round_trips_commas_paths_quotes_and_literal_separators() {
        let values = vec![
            "command+,".into(),
            "a;b".into(),
            " leading ".into(),
            "quote\"here".into(),
            r"C:\sound.wav".into(),
            String::new(),
        ];
        assert_eq!(parse_list(&display_list(&values)).unwrap(), values);
        assert!(parse_list("a;;b").is_err());
        assert!(parse_list("\"unterminated").is_err());
    }
    #[test]
    fn multiline_inline_rule_arrays_keep_trailing_comments_and_order() {
        let source = "[highlights]\nmatch = [\n # first\n { words = ['one'], name = 'One' },\n # second\n { regex = 'two', name = 'Two' },\n # trailing note\n] # array\n";
        let mut s = Settings::parse(source.into()).unwrap();
        let g = group(&s, "Rule 2");
        edit(&mut s, g, "name", "Second");
        let rendered = s.render().unwrap();
        for comment in ["# first", "# second", "# trailing note", "# array"] {
            assert!(rendered.contains(comment), "{rendered}");
        }
        assert!(
            rendered.find("name = 'One'").unwrap()
                < rendered.find("Second").unwrap()
        );
    }
    #[test]
    fn absent_defaults_and_existing_empty_patterns_are_not_rewritten() {
        let mut s = Settings::parse(String::new()).unwrap();
        let g = group(&s, "Direct messages");
        edit(&mut s, g, "sound", "peck");
        let rendered = s.render().unwrap();
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(
            parsed["notifications"]["direct_message"]
                .as_table()
                .unwrap()
                .len(),
            1
        );
        let mut s = Settings::parse("[[highlights.match]]\nwords = [] # intentionally empty\nname = 'Empty'\n".into()).unwrap();
        let g = group(&s, "Rule 1");
        edit(&mut s, g, "name", "Renamed");
        assert!(
            s.render()
                .unwrap()
                .contains("words = [] # intentionally empty")
        );
    }
}
