//! Advanced network forms backed by document leaves, never by resolved runtime secrets.
//!
//! Apply this draft to the network table AFTER the ordinary network editor and
//! BEFORE `preserve_keyring_references` on rename. Global proxy drafts apply to
//! the document root. The owning Settings save path still performs external-edit
//! detection, full Config deserialization, backup and atomic replacement.
use toml_edit::{
    Array, DocumentMut, InlineTable, Item, Table, TableLike, Value,
};

use super::rules::{display_list, parse_list};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Key(String),
    Index(usize),
}
pub type Path = Vec<Part>;
/// Keys accepted by metadata::Key (not arbitrary IRC metadata keys).
pub const METADATA_KEYS: &[&str] = &[
    "avatar",
    "color",
    "display-name",
    "homepage",
    "pronouns",
    "status",
];
fn path(input: &str) -> Path {
    input.split('.').map(|s| Part::Key(s.to_owned())).collect()
}
fn joined(base: &[Part], suffix: &str) -> Path {
    let mut result = base.to_vec();
    result.extend(path(suffix));
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Network,
    GlobalProxy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Secret,
    List,
    Regex,
    RegexList,
    Boolean,
    Integer(i64, i64),
    Limit,
    Choice(&'static [&'static str]),
    Keyring,
    /// A reroute destination: `server`, or a channel name.
    Target,
    /// Inclusivity leaf: `*` is a string; other entries form an array.
    ScopeList,
}
#[derive(Clone)]
pub struct Field {
    pub group: String,
    pub label: String,
    pub path: Path,
    pub kind: Kind,
    /// None means inherit/unset, Some("") means an explicitly empty string/list.
    pub value: Option<String>,
    pub configured: bool,
    pub custom: bool,
    pub help: &'static str,
    original: Option<String>,
    edited: bool,
}
impl Field {
    pub fn id(&self) -> String {
        format!("settings-network-extra-{}", path_id(&self.path))
    }
    pub fn is_edited(&self) -> bool {
        self.edited
    }
    /// Existing password contents are deliberately never copied to a field.
    pub fn placeholder(&self) -> &'static str {
        if self.kind == Kind::Secret && self.configured && !self.edited {
            "Stored password kept"
        } else if self.custom {
            "Configured in file"
        } else {
            "Default"
        }
    }
    fn parsed(&self) -> Result<Option<Value>, String> {
        let Some(input) = &self.value else {
            return Ok(None);
        };
        let v = match self.kind {
            Kind::Text | Kind::Secret => Value::from(input.as_str()),
            Kind::Regex => {
                fancy_regex::Regex::new(input)
                    .map_err(|e| format!("Invalid regular expression: {e}"))?;
                Value::from(input.as_str())
            }
            Kind::List | Kind::RegexList | Kind::ScopeList => {
                if self.kind == Kind::ScopeList && input.trim() == "*" {
                    return Ok(Some(Value::from("*")));
                }
                let list = parse_list(input)?;
                if self.kind == Kind::RegexList {
                    for regex in &list {
                        fancy_regex::Regex::new(regex).map_err(|e| {
                            format!("Invalid regular expression: {e}")
                        })?;
                    }
                }
                let mut array = Array::new();
                for item in list {
                    array.push(item);
                }
                Value::Array(array)
            }
            Kind::Boolean => Value::from(match input.as_str() {
                "true" => true,
                "false" => false,
                _ => return Err("Choose Default, Enabled or Disabled.".into()),
            }),
            Kind::Limit => {
                if input == "unlimited" {
                    Value::from("unlimited")
                } else {
                    let number = input.parse::<u16>().ok().filter(|n| *n > 0).ok_or("Enter a positive count up to 65535, or choose Unlimited.")?;
                    Value::from(i64::from(number))
                }
            }
            Kind::Integer(min, max) => {
                let number = input.parse::<i64>().map_err(|_| {
                    format!("Enter a whole number from {min} to {max}.")
                })?;
                if !(min..=max).contains(&number) {
                    return Err(format!(
                        "Enter a whole number from {min} to {max}."
                    ));
                }
                Value::from(number)
            }
            Kind::Choice(choices) => {
                if !choices.contains(&input.as_str()) {
                    return Err("Choose a supported value.".into());
                }
                Value::from(input.as_str())
            }
            Kind::Keyring => match input.as_str() {
                "default" => Value::from(true),
                "disabled" => Value::from(false),
                _ => Value::from(
                    input
                        .strip_prefix("key:")
                        .filter(|key| !key.is_empty())
                        .ok_or("Enter a keyring entry name.")?,
                ),
            },
            Kind::Target => {
                if input == "server" {
                    Value::from("server")
                } else {
                    if !input.starts_with(['#', '&', '+', '!'])
                        || input.chars().any(char::is_whitespace)
                    {
                        return Err(
                            "Enter a channel name, or select Server.".into()
                        );
                    }
                    let mut table = InlineTable::new();
                    table.insert("channel", Value::from(input.as_str()));
                    Value::InlineTable(table)
                }
            }
        };
        Ok(Some(v))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rows {
    Channels,
    Queries,
    Ignore,
    QueryReroute,
    NoticeReroute,
    ChannelKeys,
    ChannelKeyrings,
    Metadata,
    Bouncer,
    Criteria,
}
#[derive(Clone)]
pub struct Collection {
    pub group: String,
    pub label: &'static str,
    pub path: Path,
    pub kind: Rows,
    pub entries: Vec<Entry>,
}
#[derive(Clone)]
pub struct Entry {
    pub label: String,
    pub path: Path,
}
#[derive(Clone)]
pub struct Selector {
    pub group: String,
    pub label: &'static str,
    pub path: Path,
    pub selected: String,
    pub choices: &'static [&'static str],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub path: Path,
    pub message: String,
}
impl ValidationError {
    pub fn id(&self) -> String {
        format!("settings-network-extra-{}", path_id(&self.path))
    }
}

// No Debug implementation: these document fragments can contain stored secrets.
#[derive(Clone)]
pub struct Draft {
    scope: Scope,
    working: DocumentMut,
    patches: Vec<Patch>,
    pub fields: Vec<Field>,
    pub collections: Vec<Collection>,
    pub selectors: Vec<Selector>,
}
#[derive(Clone)]
enum Patch {
    Set(Path, Option<Value>),
    Append(Path, Value),
    Remove(Path),
}

impl Draft {
    pub fn parse_network(source: &str) -> Result<Self, String> {
        let document =
            source.parse::<DocumentMut>().map_err(|e| e.to_string())?;
        Ok(Self::network(document.as_table()))
    }
    pub fn network(table: &dyn TableLike) -> Self {
        Self::new(table, Scope::Network)
    }
    pub fn global_proxy(document: &DocumentMut) -> Self {
        Self::new(document.as_table(), Scope::GlobalProxy)
    }
    fn new(table: &dyn TableLike, scope: Scope) -> Self {
        let mut working = DocumentMut::new();
        for (key, item) in table.iter() {
            working.as_table_mut().insert(key, item.clone());
        }
        let mut draft = Self {
            scope,
            working,
            patches: vec![],
            fields: vec![],
            collections: vec![],
            selectors: vec![],
        };
        draft.rebuild();
        draft
    }
    pub fn is_global_proxy(&self) -> bool {
        self.scope == Scope::GlobalProxy
    }
    pub fn groups(&self) -> Vec<&str> {
        let mut result = Vec::new();
        for group in self
            .fields
            .iter()
            .map(|f| f.group.as_str())
            .chain(self.selectors.iter().map(|s| s.group.as_str()))
            .chain(self.collections.iter().map(|c| c.group.as_str()))
        {
            if !result.contains(&group) {
                result.push(group);
            }
        }
        result
    }
    pub fn edit(&mut self, index: usize, value: Option<String>) {
        if let Some(field) = self.fields.get_mut(index) {
            field.edited = value != field.original
                || (field.kind == Kind::Secret && field.configured);
            field.value = value;
        }
    }
    pub fn keep(&mut self, index: usize) {
        if let Some(field) = self.fields.get_mut(index) {
            field.value.clone_from(&field.original);
            field.edited = false;
        }
    }
    pub fn is_dirty(&self) -> bool {
        !self.patches.is_empty() || self.fields.iter().any(Field::is_edited)
    }
    pub fn validation_error(&self) -> Option<ValidationError> {
        let mut candidate = self.working.clone();
        for field in self.fields.iter().filter(|f| f.edited) {
            let parsed = match field.parsed() {
                Ok(value) => value,
                Err(message) => {
                    return Some(ValidationError {
                        path: field.path.clone(),
                        message,
                    });
                }
            };
            if let Err(message) = apply(
                candidate.as_table_mut(),
                &Patch::Set(field.path.clone(), parsed),
            ) {
                return Some(ValidationError {
                    path: field.path.clone(),
                    message,
                });
            }
        }
        if !self.is_dirty() {
            return None;
        }
        // Count references without opening files, invoking commands, or resolving
        // keyring contents. Presence of a stored password is sufficient.
        for field in self.fields.iter().filter(|f| f.kind == Kind::Secret) {
            let Some(Part::Key(key)) = field.path.last() else {
                continue;
            };
            if !["password", "nick_password"].contains(&key.as_str()) {
                continue;
            }
            if field
                .path
                .iter()
                .any(|p| matches!(p, Part::Key(k) if k == "channel_keys"))
            {
                continue;
            }
            let base = &field.path[..field.path.len() - 1];
            let mut count =
                usize::from(read(candidate.as_table(), &field.path).is_some());
            for suffix in ["keyring", "file", "command"] {
                let source = read(
                    candidate.as_table(),
                    &joined(base, &format!("{key}_{suffix}")),
                );
                count += usize::from(if suffix == "keyring" {
                    source.is_some_and(|n| {
                        n.boolean() == Some(true) || n.string().is_some()
                    })
                } else {
                    source.is_some()
                });
            }
            if count > 1 {
                return Some(ValidationError { path: field.path.clone(), message: "Choose one password source: stored password, keyring, file, or command. Set the other sources to Default or Disabled.".into() });
            }
            let plain = base.last() == Some(&Part::Key("plain".into()));
            if plain && count == 0 {
                return Some(ValidationError {
                    path: field.path.clone(),
                    message: "Choose a password source for SASL PLAIN.".into(),
                });
            }
        }
        for field in self
            .fields
            .iter()
            .filter(|f| f.path.last() == Some(&Part::Key("cert".into())))
        {
            if read(candidate.as_table(), &field.path)
                .and_then(Node::string)
                .is_none_or(str::is_empty)
            {
                return Some(ValidationError {
                    path: field.path.clone(),
                    message:
                        "Enter the client certificate path for SASL EXTERNAL."
                            .into(),
                });
            }
        }
        for field in &self.fields {
            let Some(Part::Key(key)) = field.path.last() else {
                continue;
            };
            let proxy_field = field
                .path
                .iter()
                .any(|p| matches!(p, Part::Key(k) if k == "proxy"));
            if proxy_field
                && key == "host"
                && read(candidate.as_table(), &field.path)
                    .and_then(Node::string)
                    .is_none_or(str::is_empty)
            {
                return Some(ValidationError {
                    path: field.path.clone(),
                    message: "Enter a proxy hostname.".into(),
                });
            }
            if proxy_field
                && key == "port"
                && read(candidate.as_table(), &field.path)
                    .and_then(Node::integer)
                    .is_none_or(|p| !(1..=65535).contains(&p))
            {
                return Some(ValidationError {
                    path: field.path.clone(),
                    message: "Enter a proxy port from 1 to 65535.".into(),
                });
            }
            if field.path.first()
                == Some(&Part::Key("channel_keys_keyring".into()))
                && read(candidate.as_table(), &field.path).is_some_and(|n| {
                    n.boolean() == Some(true) || n.string().is_some()
                })
            {
                let stored = vec![
                    Part::Key("channel_keys".into()),
                    Part::Key(key.clone()),
                ];
                if read(candidate.as_table(), &stored).is_some() {
                    return Some(ValidationError { path: field.path.clone(), message: "This channel already has a stored key. Remove that key or disable its keyring reference.".into() });
                }
            }
        }
        for collection in self
            .collections
            .iter()
            .filter(|c| matches!(c.kind, Rows::Channels | Rows::Queries))
        {
            for entry in &collection.entries {
                let target = joined(&entry.path, "name");
                let name = read(candidate.as_table(), &target)
                    .and_then(Node::string)
                    .unwrap_or("");
                let invalid = name.is_empty()
                    || name.chars().any(char::is_whitespace)
                    || name.contains([',', ':', '\0', '\u{7}'])
                    || (collection.kind == Rows::Channels
                        && !name.starts_with(['#', '&', '+', '!']));
                if invalid {
                    return Some(ValidationError { path: target, message: if collection.kind == Rows::Channels { "Enter a channel beginning with #, &, + or !, without spaces, commas or colons." } else { "Enter a nickname without spaces, commas or colons." }.into() });
                }
            }
        }
        let serialized = candidate.to_string();
        let result = match self.scope {
            Scope::Network => {
                toml::from_str::<crate::config::Server>(&serialized).map(|_| ())
            }
            Scope::GlobalProxy => {
                #[derive(serde::Deserialize)]
                struct ProxyConfig {
                    #[serde(rename = "proxy")]
                    _proxy: Option<crate::config::Proxy>,
                }
                toml::from_str::<ProxyConfig>(&serialized).map(|_| ())
            }
        };
        result.err().map(|error| {
            let span_target = error.span().and_then(|span| {
                let parsed =
                    toml_edit::Document::parse(serialized.as_str()).ok()?;
                self.fields
                    .iter()
                    .filter_map(|field| {
                        let field_span =
                            read(parsed.as_table(), &field.path)?.span()?;
                        (field_span.start <= span.start
                            && field_span.end >= span.end)
                            .then(|| (field_span.len(), field.path.clone()))
                    })
                    .min_by_key(|(length, _)| *length)
                    .map(|(_, path)| path)
            });
            ValidationError {
                path: span_target
                    .or_else(|| {
                        self.fields
                            .iter()
                            .find(|f| f.edited)
                            .map(|f| f.path.clone())
                    })
                    .or_else(|| self.patches.last().map(|p| p.path().to_vec()))
                    .unwrap_or_default(),
                message: format!(
                    "Check this network setting: {}",
                    error.message()
                ),
            }
        })
    }
    pub fn group_for(&self, target: &[Part]) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.path == target)
            .map(|f| f.group.as_str())
            .or_else(|| {
                self.selectors
                    .iter()
                    .find(|s| s.path == target)
                    .map(|s| s.group.as_str())
            })
            .or_else(|| {
                self.collections
                    .iter()
                    .find(|c| {
                        c.path == target
                            || c.entries.iter().any(|e| e.path == target)
                    })
                    .map(|c| c.group.as_str())
            })
    }
    /// Replay only edited leaves onto the parent's current table. No document
    /// replacement, network reordering, keyring lookup or credential execution.
    pub fn render_into(
        &self,
        target: &mut dyn TableLike,
    ) -> Result<(), ValidationError> {
        if let Some(error) = self.validation_error() {
            return Err(error);
        }
        for patch in &self.patches {
            apply(target, patch).map_err(|message| ValidationError {
                path: patch.path().to_vec(),
                message,
            })?;
        }
        for field in self.fields.iter().filter(|f| f.edited) {
            let value = field.parsed().map_err(|message| ValidationError {
                path: field.path.clone(),
                message,
            })?;
            apply(target, &Patch::Set(field.path.clone(), value)).map_err(
                |message| ValidationError {
                    path: field.path.clone(),
                    message,
                },
            )?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), ValidationError> {
        let patches = self
            .fields
            .iter()
            .filter(|f| f.edited)
            .map(|f| {
                Ok(Patch::Set(
                    f.path.clone(),
                    f.parsed().map_err(|message| ValidationError {
                        path: f.path.clone(),
                        message,
                    })?,
                ))
            })
            .collect::<Result<Vec<_>, ValidationError>>()?;
        if patches.is_empty() {
            return Ok(());
        }
        let mut candidate = self.working.clone();
        for patch in &patches {
            apply(candidate.as_table_mut(), patch).map_err(|message| {
                ValidationError {
                    path: patch.path().to_vec(),
                    message,
                }
            })?;
        }
        self.working = candidate;
        self.patches.extend(patches);
        self.rebuild();
        Ok(())
    }
    fn commit(&mut self, patch: Patch) -> Result<(), ValidationError> {
        let mut candidate = self.working.clone();
        apply(candidate.as_table_mut(), &patch).map_err(|message| {
            ValidationError {
                path: patch.path().to_vec(),
                message,
            }
        })?;
        self.working = candidate;
        self.patches.push(patch);
        Ok(())
    }
    /// Switching a tagged method is explicit and removes the previous variant.
    /// NickServ remains independent of SASL.
    pub fn select(
        &mut self,
        index: usize,
        selection: &str,
    ) -> Result<(), ValidationError> {
        let Some(selector) = self.selectors.get(index).cloned() else {
            return Ok(());
        };
        if selector.selected == selection {
            return Ok(());
        }
        if !selector.choices.contains(&selection) {
            return Err(ValidationError {
                path: selector.path,
                message: "Choose a supported method.".into(),
            });
        }
        self.flush()?;
        let proxy = selector.label == "Proxy";
        let value = if selection == "none" || selection == "inherit" {
            None
        } else if selection == "tor" || selection == "server" {
            Some(Value::from(selection))
        } else {
            let mut inner = InlineTable::new();
            match selection {
                "http" | "socks5" => {
                    inner.insert("host", Value::from(""));
                    inner.insert(
                        "port",
                        Value::from(if selection == "http" {
                            8080
                        } else {
                            1080
                        }),
                    );
                }
                "external" => {
                    inner.insert("cert", Value::from(""));
                }
                "plain" => {
                    inner.insert("password_keyring", Value::from(true));
                }
                _ => {}
            }
            let mut outer = InlineTable::new();
            outer.insert(selection, Value::InlineTable(inner));
            Some(Value::InlineTable(outer))
        };
        // Filehost's literal "none" differs from absent credentials (server).
        let value = if selector.label == "Filehost credentials"
            && selection == "none"
        {
            Some(Value::from("none"))
        } else {
            value
        };
        if proxy && selection == "tor" && !cfg!(feature = "tor") {
            return Err(ValidationError {
                path: selector.path,
                message: "This Halloy build does not include Tor support."
                    .into(),
            });
        }
        self.commit(Patch::Set(selector.path, value))?;
        self.rebuild();
        Ok(())
    }
    /// Add an ordinary structured row. Map/bouncer names are literal TOML keys;
    /// dots, spaces and IRC punctuation are never split into nested paths.
    pub fn add(
        &mut self,
        index: usize,
        name: &str,
    ) -> Result<(), ValidationError> {
        let Some(collection) = self.collections.get(index).cloned() else {
            return Ok(());
        };
        self.flush()?;
        let error = |message: &str| ValidationError {
            path: collection.path.clone(),
            message: message.to_owned(),
        };
        let map = matches!(
            collection.kind,
            Rows::ChannelKeys
                | Rows::ChannelKeyrings
                | Rows::Metadata
                | Rows::Bouncer
        );
        if map {
            if name.trim().is_empty() {
                return Err(error("Enter a name for the new entry."));
            }
            if matches!(
                collection.kind,
                Rows::ChannelKeys | Rows::ChannelKeyrings
            ) && (!name.starts_with(['#', '&', '+', '!'])
                || name.chars().any(char::is_whitespace)
                || name.contains([',', ':', '\0', '\u{7}']))
            {
                return Err(error(
                    "Enter a channel beginning with #, &, + or !, without spaces, commas or colons.",
                ));
            }

            if collection.kind == Rows::Metadata
                && !METADATA_KEYS.contains(&name)
            {
                return Err(error(
                    "Choose avatar, color, display-name, homepage, pronouns or status.",
                ));
            }
            let mut destination = collection.path.clone();
            destination.push(Part::Key(name.to_owned()));
            if read(self.working.as_table(), &destination).is_some() {
                return Err(error("An entry with this name already exists."));
            }
            let value = match collection.kind {
                Rows::ChannelKeys => Value::from(""),
                Rows::ChannelKeyrings => Value::from(true),
                Rows::Metadata => Value::from(""),
                _ => Value::InlineTable(InlineTable::new()),
            };
            self.commit(Patch::Set(destination, Some(value)))?;
        } else {
            let value = match collection.kind {
                Rows::Channels | Rows::Queries => {
                    Value::from(if name.is_empty() {
                        if collection.kind == Rows::Channels {
                            "#channel"
                        } else {
                            "nickname"
                        }
                    } else {
                        name
                    })
                }
                Rows::Ignore => {
                    Value::from(if name.is_empty() { "nickname" } else { name })
                }
                Rows::Criteria => {
                    let mut row = InlineTable::new();
                    let mut users = Array::new();
                    users.push(if name.is_empty() { "nickname" } else { name });
                    row.insert("users", Value::Array(users));
                    Value::InlineTable(row)
                }
                _ => {
                    let mut row = InlineTable::new();
                    row.insert(
                        "user",
                        Value::from(if name.is_empty() {
                            "nickname"
                        } else {
                            name
                        }),
                    );
                    row.insert("target", Value::from("server"));
                    Value::InlineTable(row)
                }
            };
            self.commit(Patch::Append(collection.path, value))?;
        }
        self.rebuild();
        Ok(())
    }
    pub fn remove(&mut self, entry: &[Part]) -> Result<(), ValidationError> {
        // Callers must request confirmation before removing a bouncer or rule.
        self.flush()?;
        self.commit(Patch::Remove(entry.to_vec()))?;
        self.rebuild();
        Ok(())
    }
    /// Convert an ignore row without a raw TOML editor. The four formats are
    /// exactly the variants accepted by Server::Filters.
    pub fn ignore_kind(
        &mut self,
        entry: &[Part],
        kind: &str,
    ) -> Result<(), ValidationError> {
        if !["user", "user-channel", "regex", "regex-channel"].contains(&kind) {
            return Err(ValidationError {
                path: entry.to_vec(),
                message: "Choose an ignore rule type.".into(),
            });
        }
        self.flush()?;
        let node = read(self.working.as_table(), entry);
        let user = node
            .and_then(Node::string)
            .or_else(|| {
                node.and_then(|n| n.child(&Part::Key("user".into())))
                    .and_then(Node::string)
            })
            .unwrap_or("nickname")
            .to_owned();
        let previous_regex = node
            .and_then(|n| n.child(&Part::Key("regex".into())))
            .and_then(Node::string)
            .map(str::to_owned);
        let previous_channel = node
            .and_then(|n| n.child(&Part::Key("channel".into())))
            .and_then(Node::string)
            .unwrap_or("#channel")
            .to_owned();
        let value = if kind == "user" {
            Value::from(user)
        } else {
            let mut table = InlineTable::new();
            table.insert(
                if kind.starts_with("regex") {
                    "regex"
                } else {
                    "user"
                },
                Value::from(if kind.starts_with("regex") {
                    previous_regex.as_deref().unwrap_or("(?!)")
                } else {
                    &user
                }),
            );
            if kind.ends_with("channel") {
                table.insert("channel", Value::from(previous_channel));
            }
            Value::InlineTable(table)
        };
        self.commit(Patch::Set(entry.to_vec(), Some(value)))?;
        self.rebuild();
        Ok(())
    }
    fn rebuild(&mut self) {
        self.fields.clear();
        self.collections.clear();
        self.selectors.clear();
        if self.scope == Scope::GlobalProxy {
            self.proxy(&[], "Proxy");
        } else {
            self.network_fields(&[], None);
        }
    }
    fn field(
        &mut self,
        base: &[Part],
        group: &str,
        key: &str,
        label: &str,
        kind: Kind,
        help: &'static str,
    ) {
        let mut path = joined(base, key);
        // Keep the original alias when this leaf is edited.
        for (canonical, alias) in [
            ("automated_chathistory", "chathistory"),
            ("websocket_ping_time", "websocket_ping_interval"),
        ] {
            if key == canonical
                && read(self.working.as_table(), &path).is_none()
            {
                let alternate = joined(base, alias);
                if read(self.working.as_table(), &alternate).is_some() {
                    path = alternate;
                }
            }
        }
        let node = read(self.working.as_table(), &path);
        let virtual_value = if kind == Kind::ScopeList {
            virtual_scope(self.working.as_table(), &path)
        } else {
            None
        };
        let configured = node.is_some() || virtual_value.is_some();
        let value = if kind == Kind::Secret {
            None
        } else {
            node.and_then(|n| display(n, kind)).or_else(|| {
                virtual_value
                    .as_ref()
                    .and_then(|v| display(Node::Value(v), kind))
            })
        };
        let custom = configured && value.is_none() && kind != Kind::Secret;
        self.fields.push(Field {
            group: group.to_owned(),
            label: label.to_owned(),
            path,
            kind,
            original: value.clone(),
            value,
            configured,
            custom,
            help,
            edited: false,
        });
    }
    fn selector(
        &mut self,
        base: &[Part],
        group: &str,
        key: &str,
        label: &'static str,
        choices: &'static [&'static str],
    ) -> String {
        let path = joined(base, key);
        let selected = read(self.working.as_table(), &path).map_or_else(
            || {
                if label == "Proxy" {
                    "inherit"
                } else if label == "Filehost credentials" {
                    "server"
                } else {
                    "none"
                }
                .into()
            },
            |node| {
                if let Some(s) = node.string() {
                    s.to_owned()
                } else {
                    choices
                        .iter()
                        .find(|choice| {
                            node.child(&Part::Key((**choice).to_owned()))
                                .is_some()
                        })
                        .copied()
                        .unwrap_or("configured")
                        .to_owned()
                }
            },
        );
        self.selectors.push(Selector {
            group: group.to_owned(),
            label,
            path,
            choices,
            selected: selected.clone(),
        });
        selected
    }
    fn credentials(&mut self, base: &[Part], group: &str, prefix: &str) {
        self.field(base, group, prefix, "Password", Kind::Secret, "Existing passwords are kept. A replacement is saved in config.toml.");
        self.field(base, group, &format!("{prefix}_keyring"), "Keyring entry", Kind::Keyring, "Default entry uses the network name. No keyring contents are read or changed.");
        self.field(
            base,
            group,
            &format!("{prefix}_file"),
            "Password file",
            Kind::Text,
            "Path only; Settings does not open this file.",
        );
        self.field(
            base,
            group,
            &format!("{prefix}_file_first_line_only"),
            "Read first line only",
            Kind::Boolean,
            "",
        );
        self.field(
            base,
            group,
            &format!("{prefix}_command"),
            "Password command",
            Kind::Text,
            "Command only; Settings does not execute it.",
        );
    }
    fn sasl(&mut self, base: &[Part], group: &str, key: &str, filehost: bool) {
        let method = self.selector(
            base,
            group,
            key,
            if filehost {
                "Filehost credentials"
            } else {
                "SASL method"
            },
            if filehost {
                &["server", "none", "plain", "external"]
            } else {
                &["none", "plain", "external"]
            },
        );
        if !["plain", "external"].contains(&method.as_str()) {
            return;
        }
        let mut base = joined(base, key);
        base.push(Part::Key(method.clone()));
        if method == "plain" {
            self.field(
                &base,
                group,
                "username",
                "SASL account name",
                Kind::Text,
                "Separate from the IRC username.",
            );
            self.credentials(&base, group, "password");
        } else {
            self.field(
                &base,
                group,
                "cert",
                "Client certificate",
                Kind::Text,
                "PEM encoded X509 certificate path.",
            );
            self.field(&base, group, "key", "Private key", Kind::Text, "Optional PEM encoded PKCS#8 key path; kept separate from the certificate.");
        }
        self.field(
            &base,
            group,
            "disconnect_on_failure",
            "Disconnect if SASL fails",
            Kind::Boolean,
            "",
        );
    }
    fn proxy(&mut self, base: &[Part], group: &str) {
        let kind = self.selector(
            base,
            group,
            "proxy",
            "Proxy",
            &["inherit", "http", "socks5", "tor"],
        );
        if !["http", "socks5"].contains(&kind.as_str()) {
            return;
        }
        let mut base = joined(base, "proxy");
        base.push(Part::Key(kind));
        self.field(&base, group, "host", "Proxy host", Kind::Text, "");
        self.field(
            &base,
            group,
            "port",
            "Proxy port",
            Kind::Integer(1, 65535),
            "",
        );
        self.field(&base, group, "username", "Proxy username", Kind::Text, "");
        self.field(
            &base,
            group,
            "password",
            "Proxy password",
            Kind::Secret,
            "A replacement is saved in config.toml.",
        );
        self.field(
            &base,
            group,
            "password_keyring",
            "Proxy keyring entry",
            Kind::Keyring,
            "",
        );
    }
    fn collection(
        &mut self,
        base: &[Part],
        group: &str,
        key: &str,
        label: &'static str,
        kind: Rows,
    ) {
        let collection_path = joined(base, key);
        let entries = read(self.working.as_table(), &collection_path)
            .map(|node| node.entries(&collection_path))
            .unwrap_or_default();
        self.collections.push(Collection {
            group: group.to_owned(),
            label,
            path: collection_path,
            kind,
            entries: entries.clone(),
        });
        for entry in entries {
            let row_group = if kind == Rows::Bouncer {
                format!("Bouncer: {}", entry.label)
            } else {
                group.to_owned()
            };
            match kind {
                Rows::Channels | Rows::Queries => {
                    self.field(
                        &entry.path,
                        &row_group,
                        "name",
                        &format!("{} name", entry.label),
                        Kind::Text,
                        "",
                    );
                    self.field(
                        &entry.path,
                        &row_group,
                        "mute",
                        "Mute",
                        Kind::Boolean,
                        "",
                    );
                }
                Rows::Ignore => {
                    let node = read(self.working.as_table(), &entry.path);
                    if node.and_then(Node::string).is_some() {
                        self.direct(
                            &entry.path,
                            group,
                            "Ignore user",
                            Kind::Text,
                        );
                    } else {
                        let regex = node
                            .and_then(|n| n.child(&Part::Key("regex".into())))
                            .is_some();
                        self.field(
                            &entry.path,
                            group,
                            if regex { "regex" } else { "user" },
                            if regex { "Ignore regex" } else { "Ignore user" },
                            if regex { Kind::Regex } else { Kind::Text },
                            "",
                        );
                        self.field(
                            &entry.path,
                            group,
                            "channel",
                            "In channel",
                            Kind::Text,
                            "Default means all channels.",
                        );
                    }
                }
                Rows::QueryReroute | Rows::NoticeReroute => {
                    self.field(
                        &entry.path,
                        group,
                        "user",
                        "Sender",
                        Kind::Text,
                        "",
                    );
                    self.field(
                        &entry.path,
                        group,
                        "target",
                        "Destination",
                        Kind::Target,
                        "",
                    );
                    self.field(
                        &entry.path,
                        group,
                        "follow",
                        "Follow destination",
                        Kind::Boolean,
                        "",
                    );
                }
                Rows::ChannelKeys => {
                    self.direct(&entry.path, group, &entry.label, Kind::Secret);
                }
                Rows::ChannelKeyrings => {
                    self.direct(
                        &entry.path,
                        group,
                        &entry.label,
                        Kind::Keyring,
                    );
                }
                Rows::Metadata => {
                    self.direct(&entry.path, group, &entry.label, Kind::Text);
                }
                Rows::Criteria => {
                    for (key, label, kind) in [
                        ("user", "User", Kind::Text),
                        ("users", "Users", Kind::List),
                        ("channel", "Channel", Kind::Text),
                        ("channels", "Channels", Kind::List),
                        ("server", "Server", Kind::Text),
                        ("server_message", "Server message", Kind::Text),
                    ] {
                        self.field(&entry.path, group, key, label, kind, "Conditions within this entry must match together. Separate list entries with semicolons.");
                    }
                }
                Rows::Bouncer => {
                    self.network_fields(&entry.path, Some(&row_group));
                }
            }
        }
    }
    fn direct(
        &mut self,
        destination: &[Part],
        group: &str,
        label: &str,
        kind: Kind,
    ) {
        let node = read(self.working.as_table(), destination);
        let value = if kind == Kind::Secret {
            None
        } else {
            node.and_then(|n| display(n, kind))
        };
        self.fields.push(Field {
            group: group.into(),
            label: label.into(),
            path: destination.to_vec(),
            kind,
            value: value.clone(),
            original: value,
            configured: node.is_some(),
            custom: false,
            help: "",
            edited: false,
        });
    }
    fn network_fields(&mut self, base: &[Part], bouncer: Option<&str>) {
        macro_rules! f {
            ($g:literal,$p:literal,$l:literal,$k:expr,$h:literal) => {
                self.field(base, bouncer.unwrap_or($g), $p, $l, $k, $h);
            };
        }
        macro_rules! c {
            ($g:literal,$p:literal,$l:literal,$k:ident) => {
                self.collection(base, bouncer.unwrap_or($g), $p, $l, Rows::$k);
            };
        }
        if bouncer.is_some() {
            f!("Bouncer", "enabled", "Enable network", Kind::Boolean, "");
            c!("Bouncer", "channels", "Joined channels", Channels);
        } else {
            c!(
                "Joined channels",
                "channels",
                "Joined channels and mute settings",
                Channels
            );
            f!(
                "Identity",
                "alt_nicks",
                "Alternative nicknames",
                Kind::List,
                "Separate entries with semicolons."
            );
            f!(
                "Identity",
                "username",
                "IRC username",
                Kind::Text,
                "Defaults to your nickname. This is not the SASL account name."
            );
            f!("Identity", "realname", "Real name", Kind::Text, "");
            self.credentials(base, "Server password", "password");
            self.credentials(base, "NickServ", "nick_password");
            f!(
                "NickServ",
                "nick_identify_syntax",
                "Identify syntax",
                Kind::Choice(&["nick-password", "password-nick"]),
                ""
            );
            f!(
                "NickServ",
                "should_ghost",
                "Reclaim nickname",
                Kind::Boolean,
                ""
            );
            f!(
                "NickServ",
                "ghost_sequence",
                "Ghost commands",
                Kind::List,
                "Commands run in order; Halloy appends nickname and password."
            );
            self.sasl(base, "SASL", "sasl", false);
            c!(
                "Channel keys",
                "channel_keys",
                "Channel passwords",
                ChannelKeys
            );
            c!(
                "Channel keys",
                "channel_keys_keyring",
                "Channel keyring references",
                ChannelKeyrings
            );
            for (key, label, min, max) in [
                ("ping_time", "Ping after inactivity (seconds)", 1, i64::MAX),
                ("ping_timeout", "Ping timeout (seconds)", 1, i64::MAX),
                ("reconnect_delay", "Reconnect delay (seconds)", 1, i64::MAX),
                ("max_connection_attempts", "Connection attempts", 0, 65535),
                ("who_poll_interval", "WHO interval (seconds)", 1, 3600),
                (
                    "anti_flood.rate",
                    "Flood interval (milliseconds)",
                    100,
                    60000,
                ),
                ("anti_flood.burst", "Flood burst", 1, 32),
            ] {
                self.field(
                    base,
                    "Connection limits",
                    key,
                    label,
                    if key == "max_connection_attempts" {
                        Kind::Limit
                    } else {
                        Kind::Integer(min, max)
                    },
                    if key == "max_connection_attempts" {
                        "Choose Unlimited, or set a positive attempt count."
                    } else {
                        ""
                    },
                );
            }
            f!(
                "WebSocket and TLS",
                "use_websocket",
                "Use WebSocket",
                Kind::Boolean,
                ""
            );
            f!(
                "WebSocket and TLS",
                "websocket_path",
                "WebSocket path",
                Kind::Text,
                ""
            );
            f!(
                "WebSocket and TLS",
                "websocket_ping_time",
                "WebSocket ping (seconds)",
                Kind::Integer(1, i64::MAX),
                ""
            );
            f!(
                "WebSocket and TLS",
                "root_cert_path",
                "Root certificate",
                Kind::Text,
                "PEM certificate path."
            );
            f!(
                "WebSocket and TLS",
                "dangerously_accept_invalid_certs",
                "Skip certificate validation",
                Kind::Boolean,
                "Disables TLS certificate verification for this network."
            );
            self.proxy(base, "Proxy");
            f!(
                "Filehost",
                "filehost.enabled",
                "Use server filehost",
                Kind::Boolean,
                ""
            );
            f!(
                "Filehost",
                "filehost.override_url",
                "Filehost URL",
                Kind::Text,
                ""
            );
            self.sasl(base, "Filehost", "filehost.credentials", true);
            f!(
                "Protocol log",
                "irc_protocol_log.enabled",
                "Log IRC protocol",
                Kind::Boolean,
                ""
            );
            f!(
                "Protocol log",
                "irc_protocol_log.format",
                "Log format",
                Kind::Choice(&["halloy", "goguma"]),
                ""
            );
            f!(
                "Protocol log",
                "irc_protocol_log.timestamp",
                "Log timestamps",
                Kind::Choice(&["local", "utc"]),
                ""
            );
        }
        c!("Startup", "queries", "Initial queries", Queries);
        f!(
            "Startup",
            "umodes",
            "User modes",
            Kind::Text,
            "For example +RB-x."
        );
        f!(
            "Startup",
            "on_connect",
            "On-connect commands",
            Kind::List,
            "Separate commands with semicolons; quote commands containing a literal semicolon."
        );
        f!(
            "Presence and history",
            "who_poll_enabled",
            "Poll WHO",
            Kind::Boolean,
            ""
        );
        f!(
            "Presence and history",
            "monitor",
            "Watched nicknames",
            Kind::List,
            "Separate nicknames with semicolons."
        );
        f!(
            "Presence and history",
            "automated_chathistory",
            "Retrieve history automatically",
            Kind::Boolean,
            ""
        );
        f!(
            "Presence and history",
            "typing.share",
            "Share typing",
            Kind::Boolean,
            "Default inherits the global typing setting."
        );
        f!(
            "Presence and history",
            "typing.show",
            "Show typing",
            Kind::Boolean,
            ""
        );
        f!(
            "Sidebar and metadata",
            "order_channels_by",
            "Channel order",
            Kind::Choice(&["name", "name-and-prefix", "config"]),
            ""
        );
        f!(
            "Sidebar and metadata",
            "sidebar_visibility",
            "Sidebar visibility",
            Kind::Choice(&["collapsed", "expanded"]),
            ""
        );
        f!(
            "Sidebar and metadata",
            "icon.enabled",
            "Show server icon",
            Kind::Boolean,
            ""
        );
        f!(
            "Sidebar and metadata",
            "icon.override_url",
            "Server icon URL",
            Kind::Text,
            ""
        );
        c!("Sidebar and metadata", "metadata", "Metadata", Metadata);
        f!(
            "Capabilities and delivery",
            "do_not_request",
            "Do not request capabilities",
            Kind::List,
            "Use capability names supported by Halloy, separated with semicolons."
        );
        f!(
            "Capabilities and delivery",
            "confirm_message_delivery.enabled",
            "Confirm message delivery",
            Kind::Boolean,
            ""
        );
        for direction in ["include", "exclude"] {
            for scope in ["users", "channels", "servers", "server_messages"] {
                self.field(base, bouncer.unwrap_or("Capabilities and delivery"), &format!("confirm_message_delivery.{direction}.{scope}"), &format!("{direction} {scope}"), Kind::ScopeList, "Separate entries with semicolons; * matches every entry. Existing compound criteria are preserved.");
            }
        }
        for direction in ["include", "exclude"] {
            self.collection(
                base,
                bouncer.unwrap_or("Capabilities and delivery"),
                &format!("confirm_message_delivery.{direction}.criteria"),
                if direction == "include" {
                    "Include matching combinations"
                } else {
                    "Exclude matching combinations"
                },
                Rows::Criteria,
            );
        }
        c!(
            "Ignored messages",
            "filters.ignore",
            "Ignored users and patterns",
            Ignore
        );
        f!(
            "Ignored messages",
            "filters.regex",
            "Ignored message expressions",
            Kind::RegexList,
            "Separate expressions with semicolons; quote expressions containing a semicolon."
        );
        c!("Rerouting", "reroute.query", "Query reroutes", QueryReroute);
        c!(
            "Rerouting",
            "reroute.notice",
            "Notice reroutes",
            NoticeReroute
        );
        if bouncer.is_none() {
            c!("Bouncer networks", "networks", "Bouncer overrides", Bouncer);
        }
    }
}

impl Patch {
    fn path(&self) -> &[Part] {
        match self {
            Self::Set(path, _) | Self::Append(path, _) | Self::Remove(path) => {
                path
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Node<'a> {
    Item(&'a Item),
    Value(&'a Value),
    Table(&'a Table),
}
impl<'a> Node<'a> {
    fn span(self) -> Option<std::ops::Range<usize>> {
        match self {
            Self::Item(i) => i.span(),
            Self::Value(v) => v.span(),
            Self::Table(t) => t.span(),
        }
    }

    fn string(self) -> Option<&'a str> {
        match self {
            Self::Item(i) => i.as_str(),
            Self::Value(v) => v.as_str(),
            _ => None,
        }
    }
    fn boolean(self) -> Option<bool> {
        match self {
            Self::Item(i) => i.as_bool(),
            Self::Value(v) => v.as_bool(),
            _ => None,
        }
    }
    fn integer(self) -> Option<i64> {
        match self {
            Self::Item(i) => i.as_integer(),
            Self::Value(v) => v.as_integer(),
            _ => None,
        }
    }
    fn child(self, part: &Part) -> Option<Self> {
        match (self, part) {
            (Self::Item(item), Part::Key(key)) => {
                item.as_table_like()?.get(key).map(Self::Item)
            }
            (Self::Table(table), Part::Key(key)) => {
                table.get(key).map(Self::Item)
            }
            (Self::Value(value), Part::Key(key)) => {
                value.as_inline_table()?.get(key).map(Self::Value)
            }
            (Self::Item(item), Part::Index(index)) => {
                if let Some(a) = item.as_array() {
                    a.get(*index).map(Self::Value)
                } else {
                    item.as_array_of_tables()?.get(*index).map(Self::Table)
                }
            }
            (Self::Value(value), Part::Index(index)) => {
                value.as_array()?.get(*index).map(Self::Value)
            }
            _ => None,
        }
    }
    fn to_value(self) -> Option<Value> {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Item(item) => {
                if let Some(value) = item.as_value() {
                    Some(value.clone())
                } else {
                    item.as_table_like().map(table_value)
                }
            }
            Self::Table(table) => Some(table_value(table)),
        }
    }
    fn entries(self, prefix: &[Part]) -> Vec<Entry> {
        let mut entries = vec![];
        let mut add = |part: Part, label: String| {
            let mut path = prefix.to_vec();
            path.push(part);
            entries.push(Entry { label, path });
        };
        match self {
            Self::Item(item) => {
                if let Some(table) = item.as_table_like() {
                    for (name, _) in table.iter() {
                        add(Part::Key(name.into()), name.into());
                    }
                } else if let Some(array) = item.as_array() {
                    for (index, _) in array.iter().enumerate() {
                        add(Part::Index(index), format!("Entry {}", index + 1));
                    }
                } else if let Some(array) = item.as_array_of_tables() {
                    for (index, _) in array.iter().enumerate() {
                        add(Part::Index(index), format!("Entry {}", index + 1));
                    }
                }
            }
            Self::Value(Value::Array(array)) => {
                for (index, _) in array.iter().enumerate() {
                    add(Part::Index(index), format!("Entry {}", index + 1));
                }
            }
            Self::Value(Value::InlineTable(table)) => {
                for (name, _) in table.iter() {
                    add(Part::Key(name.into()), name.into());
                }
            }
            Self::Table(table) => {
                for (name, _) in table.iter() {
                    add(Part::Key(name.into()), name.into());
                }
            }
            _ => {}
        }
        entries
    }
}
fn table_value(table: &dyn TableLike) -> Value {
    let mut output = InlineTable::new();
    for (key, item) in table.iter() {
        if let Some(value) = Node::Item(item).to_value() {
            output.insert(key, value);
        }
    }
    Value::InlineTable(output)
}
fn read<'a>(table: &'a dyn TableLike, parts: &[Part]) -> Option<Node<'a>> {
    let (Part::Key(key), remaining) = parts.split_first()? else {
        return None;
    };
    let mut node = Node::Item(table.get(key)?);
    for part in remaining {
        // Virtual leaves preserve the supported scalar shorthand until edited.
        if let Part::Key(key) = part {
            if key == "name" && node.string().is_some() {
                continue;
            }
            if key == "rate" && node.integer().is_some() {
                continue;
            }
        }
        node = node.child(part)?;
    }
    Some(node)
}
fn display(node: Node<'_>, kind: Kind) -> Option<String> {
    match kind {
        Kind::Secret => None,
        Kind::Boolean => node.boolean().map(|v| v.to_string()),
        Kind::Integer(_, _) => node.integer().map(|v| v.to_string()),
        Kind::Limit => node.integer().map(|v| v.to_string()).or_else(|| {
            node.string()
                .filter(|s| *s == "unlimited")
                .map(str::to_owned)
        }),
        Kind::Keyring => node
            .boolean()
            .map(|b| if b { "default" } else { "disabled" }.into())
            .or_else(|| node.string().map(|s| format!("key:{s}"))),
        Kind::Target => node
            .string()
            .filter(|s| *s == "server")
            .map(str::to_owned)
            .or_else(|| {
                node.child(&Part::Key("channel".into()))?
                    .string()
                    .map(str::to_owned)
            }),
        Kind::List | Kind::RegexList | Kind::ScopeList => {
            if kind == Kind::ScopeList && node.string() == Some("*") {
                return Some("*".into());
            }
            let value = node.to_value()?;
            let list = value
                .as_array()?
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()?;
            Some(display_list(&list))
        }
        Kind::Choice(choices) => node
            .string()
            .filter(|s| choices.contains(s))
            .map(str::to_owned),
        _ => node.string().map(str::to_owned),
    }
}

/// Stable, collision-free widget IDs (including literal dots and Unicode names).
pub fn path_id(path: &[Part]) -> String {
    path.iter()
        .map(|part| match part {
            Part::Key(key) => format!("k{}:{}", key.len(), key),
            Part::Index(index) => format!("i{index}"),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn apply(table: &mut dyn TableLike, patch: &Patch) -> Result<(), String> {
    mutate_table(table, patch.path(), patch)
}
fn replacement(patch: &Patch) -> Option<Value> {
    if let Patch::Set(_, value) = patch {
        value.clone()
    } else {
        None
    }
}
fn mutate_table(
    table: &mut dyn TableLike,
    parts: &[Part],
    patch: &Patch,
) -> Result<(), String> {
    let Some((Part::Key(key), remaining)) = parts.split_first() else {
        return Err("Expected a named configuration field.".into());
    };
    if remaining.is_empty() {
        match patch {
            Patch::Remove(_) | Patch::Set(_, None) => {
                table.remove(key);
            }
            Patch::Set(_, Some(value)) => {
                super::set(table, key, Item::Value(value.clone()));
            }
            Patch::Append(_, value) => {
                if !table.contains_key(key) {
                    table.insert(key, Item::Value(Value::Array(Array::new())));
                }
                let item = table.get_mut(key).ok_or("Missing collection.")?;
                if let Some(array) = item.as_array_mut() {
                    array.push(value.clone());
                } else if let Some(array) = item.as_array_of_tables_mut() {
                    let inline = value
                        .as_inline_table()
                        .ok_or("Expected a structured row.")?;
                    let mut row = Table::new();
                    for (key, value) in inline.iter() {
                        row.insert(key, Item::Value(value.clone()));
                    }
                    array.push(row);
                } else {
                    return Err("This collection has an unsupported configuration format.".into());
                }
            }
        }
        return Ok(());
    }
    if !table.contains_key(key) {
        if matches!(patch, Patch::Set(_, None) | Patch::Remove(_)) {
            return Ok(());
        }
        table.insert(key, Item::Table(Table::new()));
    }
    let item = table.get_mut(key).ok_or("Missing configuration field.")?;
    if let Some(table) = item.as_table_like_mut() {
        return mutate_table(table, remaining, patch);
    }
    if let Some(array) = item.as_array_of_tables_mut() {
        let Some((Part::Index(index), tail)) = remaining.split_first() else {
            return Err("Expected a row number.".into());
        };
        if tail.is_empty() && matches!(patch, Patch::Remove(_)) {
            if *index < array.len() {
                array.remove(*index);
            }
            return Ok(());
        }
        return mutate_table(
            array.get_mut(*index).ok_or("This row no longer exists.")?,
            tail,
            patch,
        );
    }
    mutate_value(
        item.as_value_mut()
            .ok_or("This configuration field cannot be edited here.")?,
        remaining,
        patch,
    )
}
fn mutate_value(
    value: &mut Value,
    parts: &[Part],
    patch: &Patch,
) -> Result<(), String> {
    let Some((part, tail)) = parts.split_first() else {
        *value = replacement(patch).ok_or("Cannot remove the root value.")?;
        return Ok(());
    };
    match part {
        Part::Key(key) => {
            // Muteable string and anti_flood integer shorthand.
            if value.as_str().is_some() && key == "name" && tail.is_empty() {
                if let Some(next) = replacement(patch) {
                    replace_value(value, next);
                } else {
                    return Err("Enter a name, or remove this entry.".into());
                }
                return Ok(());
            }
            if !value.is_inline_table() {
                let location =
                    &patch.path()[..patch.path().len() - parts.len()];
                let Some(table) = expand_shorthand(location, value) else {
                    return Err("This value uses an unsupported format. Keep it, or choose a replacement before editing nested fields.".into());
                };
                replace_value(value, Value::InlineTable(table));
            }
            let table =
                value.as_inline_table_mut().ok_or("Expected a table.")?;
            if tail.is_empty() {
                match patch {
                    Patch::Set(_, None) | Patch::Remove(_) => {
                        table.remove(key);
                    }
                    Patch::Set(_, Some(next)) => {
                        if let Some(old) = table.get_mut(key) {
                            replace_value(old, next.clone());
                        } else {
                            table.insert(key, next.clone());
                        }
                    }
                    Patch::Append(_, row) => {
                        if !table.contains_key(key) {
                            table.insert(key, Value::Array(Array::new()));
                        }
                        table
                            .get_mut(key)
                            .and_then(Value::as_array_mut)
                            .ok_or("Expected a list.")?
                            .push(row.clone());
                    }
                }
            } else {
                if !table.contains_key(key) {
                    if matches!(patch, Patch::Set(_, None) | Patch::Remove(_)) {
                        return Ok(());
                    }
                    table.insert(key, Value::InlineTable(InlineTable::new()));
                }
                mutate_value(
                    table.get_mut(key).ok_or("Missing field.")?,
                    tail,
                    patch,
                )?;
            }
        }
        Part::Index(index) => {
            let array = value.as_array_mut().ok_or("Expected a list.")?;
            if tail.is_empty() {
                match patch {
                    Patch::Remove(_) | Patch::Set(_, None) => {
                        if *index < array.len() {
                            array.remove(*index);
                        }
                    }
                    Patch::Set(_, Some(next)) => {
                        replace_value(
                            array
                                .get_mut(*index)
                                .ok_or("This row no longer exists.")?,
                            next.clone(),
                        );
                    }
                    Patch::Append(_, _) => {
                        return Err("Cannot append to a single row.".into());
                    }
                }
            } else {
                mutate_value(
                    array
                        .get_mut(*index)
                        .ok_or("This row no longer exists.")?,
                    tail,
                    patch,
                )?;
            }
        }
    }
    Ok(())
}
fn replace_value(old: &mut Value, mut next: Value) {
    *next.decor_mut() = old.decor().clone();
    *old = next;
}

fn virtual_scope(table: &dyn TableLike, destination: &[Part]) -> Option<Value> {
    let (last, parent) = destination.split_last()?;
    let Part::Key(key) = last else {
        return None;
    };
    let original = read(table, parent)?.to_value()?;
    expand_shorthand(parent, &original)?.get(key).cloned()
}
fn expand_shorthand(location: &[Part], value: &Value) -> Option<InlineTable> {
    let muted_row = location.iter().any(
        |p| matches!(p, Part::Key(k) if k == "channels" || k == "queries"),
    ) && matches!(location.last(), Some(Part::Index(_)));
    let flood =
        matches!(location.last(), Some(Part::Key(k)) if k == "anti_flood");
    let scopes = matches!(location.last(), Some(Part::Key(k)) if k == "include" || k == "exclude");
    let mut table = InlineTable::new();
    if muted_row {
        table.insert("name", Value::from(value.as_str()?));
    } else if flood {
        table.insert("rate", Value::from(value.as_integer()?));
    } else if scopes {
        let legacy = value
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>());
        if matches!(value.as_str(), Some("*" | "all"))
            || legacy
                .as_ref()
                .is_some_and(|a| a.iter().any(|s| ["*", "all"].contains(s)))
        {
            for key in ["users", "channels", "servers", "server_messages"] {
                table.insert(key, Value::from("*"));
            }
        } else {
            let list = legacy?;
            let mut users = Array::new();
            let mut channels = Array::new();
            for name in list {
                if irc::proto::is_channel(
                    name,
                    irc::proto::DEFAULT_CHANNEL_PREFIXES,
                ) {
                    channels.push(name);
                } else {
                    users.push(name);
                }
            }
            if !users.is_empty() {
                table.insert("users", Value::Array(users));
            }
            if !channels.is_empty() {
                table.insert("channels", Value::Array(channels));
            }
        }
    } else {
        return None;
    }
    Some(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(source: &str) -> (DocumentMut, Draft) {
        let document: DocumentMut = source.parse().unwrap();
        let draft = Draft::network(document.as_table());
        (document, draft)
    }
    fn edit(draft: &mut Draft, key: &str, value: Option<&str>) {
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == path(key))
            .unwrap_or_else(|| panic!("Missing field {key}"));
        draft.edit(index, value.map(str::to_owned));
    }
    fn render(draft: &Draft, mut document: DocumentMut) -> DocumentMut {
        draft.render_into(document.as_table_mut()).unwrap();
        document
    }
    fn collection(draft: &Draft, key: &str) -> usize {
        draft
            .collections
            .iter()
            .position(|c| c.path == path(key))
            .unwrap()
    }
    #[test]
    fn untouched_network_is_identical_and_passwords_are_not_fields() {
        let source = "# header\nnick = 'nick' # alias\npassword = 'fixture password'\nchannels = [{name='#rust', mute=true}]\n[sasl.plain]\npassword_command = 'never execute this fixture'\n";
        let (document, draft) = draft(source);
        assert!(!draft.is_dirty());
        assert!(
            draft
                .fields
                .iter()
                .filter(|f| f.kind == Kind::Secret)
                .all(|f| f.value.is_none() && f.original.is_none())
        );
        assert_eq!(render(&draft, document).to_string(), source);
    }
    #[test]
    fn irc_identity_is_separate_from_sasl_and_preserves_comments() {
        let (document, mut draft) = draft(
            "nick='nick'\nusername='irc' # retain\n[sasl.plain]\nusername='account'\npassword_keyring='entry'\n",
        );
        edit(&mut draft, "username", Some("new-irc"));
        edit(&mut draft, "realname", Some(""));
        let result = render(&draft, document);
        assert_eq!(result["username"].as_str(), Some("new-irc"));
        assert_eq!(result["realname"].as_str(), Some(""));
        assert_eq!(
            result["sasl"]["plain"]["username"].as_str(),
            Some("account")
        );
        assert!(result.to_string().contains("# retain"));
        assert_eq!(result["nick"].as_str(), Some("nick"));
    }
    #[test]
    fn unset_false_and_empty_are_distinct() {
        let (document, mut draft) =
            draft("typing = {share=true, show=true}\nrealname='name'\n");
        edit(&mut draft, "typing.share", None);
        edit(&mut draft, "typing.show", Some("false"));
        edit(&mut draft, "realname", Some(""));
        let result = render(&draft, document);
        assert!(result["typing"].get("share").is_none());
        assert_eq!(result["typing"]["show"].as_bool(), Some(false));
        assert_eq!(result["realname"].as_str(), Some(""));
    }
    #[test]
    fn aliases_and_untargeted_values_survive() {
        let (document, mut draft) = draft(
            "chathistory=true # legacy\nwebsocket_ping_interval=60\ncustom='keep'\n",
        );
        edit(&mut draft, "chathistory", Some("false"));
        edit(&mut draft, "websocket_ping_interval", Some("90"));
        let result = render(&draft, document);
        assert_eq!(result["chathistory"].as_bool(), Some(false));
        assert!(result.get("automated_chathistory").is_none());
        assert_eq!(result["websocket_ping_interval"].as_integer(), Some(90));
        assert!(result.get("websocket_ping_time").is_none());
        assert!(result.to_string().contains("# legacy"));
        assert_eq!(result["custom"].as_str(), Some("keep"));
    }
    #[test]
    fn duplicate_credential_sources_have_a_field_target() {
        let (_, mut draft) = draft("password_command='fixture-command'\n");
        edit(&mut draft, "password_keyring", Some("key:example"));
        let error = draft.validation_error().unwrap();
        assert_eq!(error.path, path("password"));
        assert!(error.message.contains("one password source"));
        edit(&mut draft, "password_command", None);
        assert!(draft.validation_error().is_none());
    }
    #[test]
    fn external_certificates_and_independent_nickserv_are_supported() {
        let (document, mut draft) = draft(
            "nick_password_keyring='nick-entry'\n[sasl.plain]\npassword_keyring='sasl-entry'\n",
        );
        let selector = draft
            .selectors
            .iter()
            .position(|s| s.label == "SASL method")
            .unwrap();
        draft.select(selector, "external").unwrap();
        assert!(
            draft
                .validation_error()
                .unwrap()
                .message
                .contains("certificate path")
        );
        edit(&mut draft, "sasl.external.cert", Some("~/cert.pem"));
        edit(&mut draft, "sasl.external.key", Some("~/key.pem"));
        let result = render(&draft, document);
        assert!(result["sasl"].get("plain").is_none());
        assert_eq!(
            result["nick_password_keyring"].as_str(),
            Some("nick-entry")
        );
    }
    #[test]
    fn scalar_flood_rate_is_preserved_when_editing_burst() {
        let (document, mut draft) = draft("anti_flood=3500 # comment\n");
        edit(&mut draft, "anti_flood.burst", Some("12"));
        let result = render(&draft, document);
        assert_eq!(result["anti_flood"]["rate"].as_integer(), Some(3500));
        assert_eq!(result["anti_flood"]["burst"].as_integer(), Some(12));
        assert!(result.to_string().contains("# comment"));
    }
    #[test]
    fn timing_and_flood_ranges_match_server_deserializers() {
        let (_, mut draft) = draft("");
        for (key, bad, good) in [
            ("who_poll_interval", "0", "1"),
            ("anti_flood.rate", "99", "100"),
            ("anti_flood.burst", "33", "32"),
            ("max_connection_attempts", "65536", "unlimited"),
            ("ping_time", "0", "1"),
        ] {
            edit(&mut draft, key, Some(bad));
            assert_eq!(draft.validation_error().unwrap().path, path(key));
            edit(&mut draft, key, Some(good));
            assert!(draft.validation_error().is_none());
        }
    }
    #[test]
    fn mute_changes_preserve_unedited_channel_entries_and_order() {
        let source = "channels = ['#first', { name = '#second', mute = true }] # retain\n";
        let (document, mut draft) = draft(source);
        let mut target = path("channels");
        target.push(Part::Index(0));
        target.push(Part::Key("mute".into()));
        let index = draft.fields.iter().position(|f| f.path == target).unwrap();
        draft.edit(index, Some("true".into()));
        let result = render(&draft, document);
        let array = result["channels"].as_array().unwrap();
        assert_eq!(
            array
                .get(0)
                .unwrap()
                .as_inline_table()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str(),
            Some("#first")
        );
        assert_eq!(
            array.get(1).unwrap().to_string(),
            " { name = '#second', mute = true }"
        );
        assert!(result.to_string().contains("# retain"));
    }
    #[test]
    fn dotted_channel_keys_are_literal_and_secrets_remain_hidden() {
        let (document, mut draft) =
            draft("[channel_keys]\n'#a.b'='fixture' # keep\n");
        let target =
            vec![Part::Key("channel_keys".into()), Part::Key("#a.b".into())];
        let field = draft.fields.iter().find(|f| f.path == target).unwrap();
        assert!(field.configured && field.value.is_none());
        let index = collection(&draft, "channel_keys_keyring");
        draft.add(index, "#other.channel").unwrap();
        let result = render(&draft, document);
        assert_eq!(result["channel_keys"]["#a.b"].as_str(), Some("fixture"));
        assert_eq!(
            result["channel_keys_keyring"]["#other.channel"].as_bool(),
            Some(true)
        );
        assert!(result.to_string().contains("# keep"));
    }
    #[test]
    fn ignore_rules_are_structured_and_invalid_regexes_are_targeted() {
        let (document, mut draft) =
            draft("[filters]\nignore=['fixture-user']\n");
        let target = vec![
            Part::Key("filters".into()),
            Part::Key("ignore".into()),
            Part::Index(0),
        ];
        draft.ignore_kind(&target, "regex-channel").unwrap();
        let mut regex = target.clone();
        regex.push(Part::Key("regex".into()));
        let index = draft.fields.iter().position(|f| f.path == regex).unwrap();
        draft.edit(index, Some("[".into()));
        assert_eq!(draft.validation_error().unwrap().path, regex);
        draft.edit(index, Some("^spam".into()));
        let result = render(&draft, document);
        assert_eq!(
            result["filters"]["ignore"]
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_inline_table()
                .unwrap()
                .get("regex")
                .unwrap()
                .as_str(),
            Some("^spam")
        );
    }
    #[test]
    fn array_of_table_reroutes_keep_other_rows_and_comments() {
        let (document, mut draft) = draft(
            "[[reroute.query]]\nuser='first' # keep first\ntarget='server'\n[[reroute.query]]\nuser='second' # keep second\ntarget={channel='#room'}\n",
        );
        let mut destination = path("reroute.query");
        destination.push(Part::Index(1));
        destination.push(Part::Key("follow".into()));
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == destination)
            .unwrap();
        draft.edit(index, Some("true".into()));
        let result = render(&draft, document);
        assert_eq!(
            result["reroute"]["query"]
                .as_array_of_tables()
                .unwrap()
                .get(1)
                .unwrap()["follow"]
                .as_bool(),
            Some(true)
        );
        assert!(result.to_string().contains("# keep first"));
        assert!(result.to_string().contains("# keep second"));
    }
    #[test]
    fn additions_then_removals_use_current_stable_row_indices() {
        let (document, mut draft) = draft("queries=['one', 'two']\n");
        draft.add(collection(&draft, "queries"), "three").unwrap();
        let mut remove = path("queries");
        remove.push(Part::Index(0));
        draft.remove(&remove).unwrap();
        let mut destination = path("queries");
        destination.push(Part::Index(0));
        destination.push(Part::Key("name".into()));
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == destination)
            .unwrap();
        draft.edit(index, Some("changed-two".into()));
        let result = render(&draft, document);
        let queries = result["queries"].as_array().unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries.get(0).unwrap().as_str(), Some("changed-two"));
        assert_eq!(queries.get(1).unwrap().as_str(), Some("three"));
    }
    #[test]
    fn delivery_legacy_scopes_keep_untargeted_conditions() {
        let (document, mut draft) = draft(
            "[confirm_message_delivery]\nexclude=['#room', 'sender']\ninclude={channels=['#allowed'], criteria=[{users=['a'],channels=['#a']}]}\n",
        );
        edit(
            &mut draft,
            "confirm_message_delivery.exclude.channels",
            Some("#different"),
        );
        let result = render(&draft, document);
        assert_eq!(
            result["confirm_message_delivery"]["exclude"]["users"]
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_str(),
            Some("sender")
        );
        assert!(
            result
                .to_string()
                .contains("criteria=[{users=['a'],channels=['#a']} ]")
                || result
                    .to_string()
                    .contains("criteria=[{users=['a'],channels=['#a']}]")
        );
    }
    #[test]
    fn bouncer_names_with_dots_are_literal_and_unsupported_fields_are_absent() {
        let (document, mut draft) = draft("");
        draft
            .add(collection(&draft, "networks"), "Libera.chat")
            .unwrap();
        let base = vec![
            Part::Key("networks".into()),
            Part::Key("Libera.chat".into()),
        ];
        let target = joined(&base, "enabled");
        let index = draft.fields.iter().position(|f| f.path == target).unwrap();
        draft.edit(index, Some("false".into()));
        assert!(
            !draft
                .fields
                .iter()
                .any(|f| f.path == joined(&base, "filehost.enabled"))
        );
        let result = render(&draft, document);
        assert_eq!(
            result["networks"]["Libera.chat"]["enabled"].as_bool(),
            Some(false)
        );
        assert!(result["networks"].get("Libera").is_none());
    }
    #[test]
    fn global_proxy_switches_method_without_touching_other_config() {
        let source = "# header\n[proxy.http]\nhost='old'\nport=8080\npassword_keyring='proxy-entry'\n[servers.example]\nserver='example.org' # retain\n";
        let mut document: DocumentMut = source.parse().unwrap();
        let mut draft = Draft::global_proxy(&document);
        draft.select(0, "socks5").unwrap();
        edit(&mut draft, "proxy.socks5.host", Some("localhost"));
        edit(
            &mut draft,
            "proxy.socks5.password_keyring",
            Some("key:socks-entry"),
        );
        draft.render_into(document.as_table_mut()).unwrap();
        assert!(document["proxy"].get("http").is_none());
        assert_eq!(
            document["proxy"]["socks5"]["port"].as_integer(),
            Some(1080)
        );
        assert_eq!(
            document["servers"]["example"]["server"].as_str(),
            Some("example.org")
        );
        assert!(document.to_string().contains("# retain"));
    }
    #[test]
    fn per_network_filehost_accepts_its_actual_credential_variants() {
        let (document, mut draft) = draft("[filehost]\ncredentials='none'\n");
        let selector = draft
            .selectors
            .iter()
            .position(|s| s.label == "Filehost credentials")
            .unwrap();
        draft.select(selector, "plain").unwrap();
        edit(
            &mut draft,
            "filehost.credentials.plain.username",
            Some("upload-user"),
        );
        edit(
            &mut draft,
            "filehost.credentials.plain.password_keyring",
            Some("key:upload-entry"),
        );
        let result = render(&draft, document);
        assert_eq!(
            result["filehost"]["credentials"]["plain"]["username"].as_str(),
            Some("upload-user")
        );
    }
    #[test]
    fn keyboard_ids_do_not_collide_for_literal_dots_or_array_entries() {
        assert_ne!(
            path_id(&path("networks.a.b")),
            path_id(&[Part::Key("networks".into()), Part::Key("a.b".into())])
        );
        assert_ne!(
            path_id(&[Part::Key("1".into())]),
            path_id(&[Part::Index(1)])
        );
    }
    #[test]
    fn generic_config_errors_target_the_offending_leaf_after_other_edits() {
        let (_, mut draft) = draft("");
        edit(&mut draft, "realname", Some("Fixture name"));
        edit(
            &mut draft,
            "do_not_request",
            Some("invented-invalid-capability"),
        );
        let error = draft.validation_error().unwrap();
        assert_eq!(error.path, path("do_not_request"));
    }
    #[test]
    fn channel_key_sources_cannot_overlap_and_zero_attempts_is_invalid() {
        let (_, mut draft) = draft("[channel_keys]\n'#room'='fixture'\n");
        draft
            .add(collection(&draft, "channel_keys_keyring"), "#room")
            .unwrap();
        assert!(
            draft
                .validation_error()
                .unwrap()
                .message
                .contains("already has a stored key")
        );
        let (_, mut limit_draft) = super::tests::draft("");
        edit(&mut limit_draft, "max_connection_attempts", Some("0"));
        assert_eq!(
            limit_draft.validation_error().unwrap().path,
            path("max_connection_attempts")
        );
    }
    #[test]
    fn delivery_combinations_edit_individual_conditions_and_preserve_aliases() {
        let source = "[confirm_message_delivery]\ninclude = { criteria = [{user='one', users=['two'], channel='#room', server='network'}] }\n";
        let (document, mut draft) = draft(source);
        let mut destination = path("confirm_message_delivery.include.criteria");
        destination.push(Part::Index(0));
        destination.push(Part::Key("users".into()));
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == destination)
            .unwrap();
        draft.edit(index, Some("three; four".into()));
        let result = render(&draft, document);
        let criteria =
            result["confirm_message_delivery"]["include"]["criteria"]
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_inline_table()
                .unwrap();
        assert_eq!(criteria.get("user").unwrap().as_str(), Some("one"));
        assert_eq!(criteria.get("channel").unwrap().as_str(), Some("#room"));
        assert_eq!(criteria.get("server").unwrap().as_str(), Some("network"));
        assert_eq!(criteria.get("users").unwrap().as_array().unwrap().len(), 2);
    }
}
