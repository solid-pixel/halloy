//! Preview and transfer controls. Simple leaves extend Preferences; Media holds
//! union values, scope lists, and the atomic optional DCC server configuration.
use std::net::IpAddr;

use toml_edit::{
    Array, DocumentMut, InlineTable, Item, TableLike, Value, value,
};

use super::preferences::{Area, Choice, Kind, Spec};
use super::rules::{display_list, parse_list};
use super::{child_table, set};
use crate::config::inclusivities::{Inclusivities, Inclusivity};

const HIDE_URL: &[Choice] = &[
    Choice {
        label: "Only when message is a URL",
        value: "\"contains-only-url\"",
    },
    Choice {
        label: "When URL ends the message",
        value: "\"trailing\"",
    },
    Choice {
        label: "Never",
        value: "\"never\"",
    },
];
const CARD_ACTION: &[Choice] = &[
    Choice {
        label: "Open page in browser",
        value: "\"open-url\"",
    },
    Choice {
        label: "Open image in browser",
        value: "\"open-image-url\"",
    },
    Choice {
        label: "Preview image in Halloy",
        value: "\"preview\"",
    },
];
const IMAGE_ACTION: &[Choice] = &[
    Choice {
        label: "Open image in browser",
        value: "\"open-url\"",
    },
    CARD_ACTION[2],
];
const ANIMATION: &[Choice] = &[
    Choice {
        label: "Animate on hover",
        value: "\"animate-on-hover\"",
    },
    Choice {
        label: "Animate only in image preview",
        value: "\"animate-on-preview-action\"",
    },
    Choice {
        label: "Disabled",
        value: "\"disabled\"",
    },
];

/// Call with Area::Previews and Area::Transfers after adding those shared enum
/// variants. Taking the areas as arguments keeps this module registration small.
pub fn schema(previews: Area, transfers: Area) -> Vec<Spec> {
    let mut specs = vec![];
    macro_rules! s {
        ($area:expr,$group:literal,$path:literal,$label:literal,$kind:expr,$default:literal,$help:literal) => {
            specs.push(Spec {
                area: $area,
                group: $group,
                path: $path,
                label: $label,
                kind: $kind,
                default: $default,
                help: $help,
                aliases: &[],
            });
        };
    }
    macro_rules! b {
        ($a:expr,$g:literal,$p:literal,$l:literal,$d:literal) => {
            s!($a, $g, $p, $l, Kind::Boolean, $d, "");
        };
    }
    macro_rules! n {
        ($a:expr,$g:literal,$p:literal,$l:literal,$d:literal,$help:literal) => {
            s!(
                $a,
                $g,
                $p,
                $l,
                Kind::Integer {
                    min: 0,
                    max: i64::MAX,
                    optional: false
                },
                $d,
                $help
            );
        };
    }
    macro_rules! px {
        ($p:literal,$l:literal,$d:literal,$g:literal) => {
            s!(
                previews,
                $g,
                $p,
                $l,
                Kind::Float {
                    min: f64::from(f32::MIN_POSITIVE),
                    max: f64::from(f32::MAX),
                    optional: false
                },
                $d,
                "Pixels"
            );
        };
    }
    n!(
        previews,
        "Links",
        "preview.max_per_message",
        "Previews per message",
        "1",
        ""
    );
    b!(
        previews,
        "Link cards",
        "preview.card.show_image",
        "Show card image",
        "true"
    );
    b!(
        previews,
        "Link cards",
        "preview.card.round_image_corners",
        "Round card image corners",
        "true"
    );
    s!(
        previews,
        "Link cards",
        "preview.card.hide_url",
        "Hide URL",
        Kind::Choice(HIDE_URL),
        "\"never\"",
        ""
    );
    s!(
        previews,
        "Link cards",
        "preview.card.image_action",
        "Click card image",
        Kind::Choice(CARD_ACTION),
        "\"open-url\"",
        ""
    );
    px!(
        "preview.card.max_width",
        "Maximum card width",
        "400.0",
        "Link cards"
    );
    px!(
        "preview.card.description_max_height",
        "Description height",
        "100.0",
        "Link cards"
    );
    px!(
        "preview.card.image_max_height",
        "Card image height",
        "200.0",
        "Link cards"
    );
    b!(
        previews,
        "Images",
        "preview.image.round_corners",
        "Round image corners",
        "true"
    );
    s!(
        previews,
        "Images",
        "preview.image.hide_url",
        "Hide URL",
        Kind::Choice(HIDE_URL),
        "\"contains-only-url\"",
        ""
    );
    // ImageAction uses open-url for opening the image itself.
    s!(
        previews,
        "Images",
        "preview.image.action",
        "Click image",
        Kind::Choice(IMAGE_ACTION),
        "\"preview\"",
        ""
    );
    s!(
        previews,
        "Images",
        "preview.image.animation",
        "GIF animation",
        Kind::Choice(ANIMATION),
        "\"animate-on-hover\"",
        "Also controls GIFs in link cards."
    );
    px!(
        "preview.image.max_width",
        "Maximum image width",
        "550.0",
        "Images"
    );
    px!(
        "preview.image.max_height",
        "Maximum image height",
        "350.0",
        "Images"
    );
    n!(
        previews,
        "Requests and cache",
        "preview.request.timeout_ms",
        "Timeout (ms)",
        "10000",
        ""
    );
    n!(
        previews,
        "Requests and cache",
        "preview.request.max_image_size",
        "Maximum image bytes",
        "10485760",
        ""
    );
    n!(
        previews,
        "Requests and cache",
        "preview.request.max_scrape_size",
        "Maximum metadata bytes",
        "512000",
        ""
    );
    n!(
        previews,
        "Requests and cache",
        "preview.request.concurrency",
        "Concurrent requests",
        "4",
        ""
    );
    n!(
        previews,
        "Requests and cache",
        "preview.request.delay_ms",
        "Request delay (ms)",
        "500",
        ""
    );
    b!(
        transfers,
        "DCC transfers",
        "file_transfer.enabled",
        "Enable DCC transfers",
        "true"
    );
    b!(
        transfers,
        "DCC transfers",
        "file_transfer.passive",
        "Use passive transfers",
        "true"
    );
    n!(
        transfers,
        "DCC transfers",
        "file_transfer.timeout",
        "Acceptance timeout (seconds)",
        "300",
        ""
    );
    b!(
        transfers,
        "DCC transfers",
        "file_transfer.send_completion_message",
        "Send completion message",
        "true"
    );
    b!(
        transfers,
        "Automatic acceptance",
        "file_transfer.auto_accept.enabled",
        "Automatically accept files",
        "false"
    );
    b!(
        transfers,
        "Uploads",
        "filehost.enabled",
        "Enable file uploads",
        "true"
    );
    b!(
        transfers,
        "Uploads",
        "filehost.button",
        "Show upload button",
        "true"
    );
    b!(
        transfers,
        "Uploads",
        "filehost.paste",
        "Upload pasted files",
        "true"
    );
    b!(
        transfers,
        "Uploads",
        "filehost.file_drop",
        "Upload dropped files",
        "false"
    );
    specs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaArea {
    Previews,
    Transfers,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Text,
    List,
    Patterns,
    /// Boolean or URL pattern list: mode is edited separately from patterns.
    UrlMode,
    Scope,
    CacheSize,
    TrimInterval,
    ServerEnabled,
    Address,
    Port,
}
#[derive(Debug, Clone)]
pub struct Field {
    pub area: MediaArea,
    pub group: &'static str,
    pub path: String,
    pub label: String,
    pub help: &'static str,
    pub control: Control,
    pub value: String,
    original: String,
}
impl Field {
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
    pub fn id(&self) -> String {
        format!("settings-media-{}", self.path.replace('.', "-"))
    }
}
#[derive(Debug, Clone)]
pub struct Media {
    pub fields: Vec<Field>,
    original_auto_accept: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub field: usize,
    pub message: String,
}

fn lookup<'a>(root: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    path.split('.').try_fold(root, |v, key| v.get(key))
}
fn strings(value: Option<&toml::Value>) -> Vec<String> {
    match value {
        Some(toml::Value::String(s)) => vec![s.clone()],
        Some(toml::Value::Array(a)) => a
            .iter()
            .filter_map(toml::Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => vec![],
    }
}
fn array(items: Vec<String>) -> Item {
    let mut array = Array::new();
    for item in items {
        array.push(item);
    }
    value(array)
}
fn inclusivity(values: Option<&Inclusivity>) -> String {
    match values {
        Some(Inclusivity::All) => "*".into(),
        Some(Inclusivity::Any(items)) => display_list(items),
        None => String::new(),
    }
}
impl Media {
    pub fn parse(document: &DocumentMut) -> Result<Self, String> {
        let root: toml::Value =
            toml::from_str(&document.to_string()).map_err(|e| e.to_string())?;
        let mut fields = vec![];
        let mut add = |area,
                       group,
                       path: &str,
                       label: &str,
                       control,
                       value: String,
                       help| {
            fields.push(Field {
                area,
                group,
                path: path.into(),
                label: label.into(),
                help,
                control,
                original: value.clone(),
                value,
            });
        };
        for (path, group, label, default) in [
            ("preview.enabled", "Links", "Show previews", true),
            (
                "preview.card.description_decode_html",
                "Link cards",
                "Decode HTML entities",
                false,
            ),
        ] {
            let source = lookup(&root, path);
            let mode = match source.and_then(toml::Value::as_bool) {
                Some(true) => "Enabled",
                Some(false) => "Disabled",
                None if source.is_some() => "Matching URLs",
                None if default => "Enabled",
                None => "Disabled",
            };
            add(
                MediaArea::Previews,
                group,
                path,
                label,
                Control::UrlMode,
                mode.into(),
                "Matching URLs uses regular expressions; exclusions take priority.",
            );
            add(
                MediaArea::Previews,
                group,
                &format!("{path}.patterns"),
                "URL patterns",
                Control::Patterns,
                display_list(&strings(source.filter(|v| !v.is_bool()))),
                "Separate patterns with semicolons; quote an entry containing a semicolon.",
            );
        }
        add(
            MediaArea::Previews,
            "Links",
            "preview.exclude",
            "Exclude URL patterns",
            Control::Patterns,
            display_list(&strings(lookup(&root, "preview.exclude"))),
            "Matching URLs never show a preview. Separate patterns with semicolons.",
        );
        for (path, group) in
            [("preview.card", "Link cards"), ("preview.image", "Images")]
        {
            for (direction, prefix) in
                [("exclude", "Exclude"), ("include", "Include")]
            {
                let path = format!("{path}.{direction}");
                let conditions = lookup(&root, &path)
                    .cloned()
                    .map(toml::Value::try_into::<Inclusivities>)
                    .transpose()
                    .map_err(|e| e.to_string())?;
                for (leaf, label) in [
                    ("channels", "channels"),
                    ("users", "users"),
                    ("servers", "servers"),
                    ("server_messages", "server messages"),
                ] {
                    let values = conditions.as_ref().and_then(|c| match leaf {
                        "channels" => c.channels.as_ref(),
                        "users" => c.users.as_ref(),
                        "servers" => c.servers.as_ref(),
                        _ => c.server_messages.as_ref(),
                    });
                    add(
                        MediaArea::Previews,
                        group,
                        &format!("{path}.{leaf}"),
                        &format!("{prefix} {label}"),
                        Control::Scope,
                        inclusivity(values),
                        "Separate names with semicolons, or use * for all. Includes take priority. Existing compound criteria are retained.",
                    );
                }
            }
        }
        for (area, group, path, label, control, default, help) in [
            (
                MediaArea::Previews,
                "Requests and cache",
                "preview.request.user_agent",
                "User agent",
                Control::Text,
                "",
                "Empty restores Halloy’s name and version.",
            ),
            (
                MediaArea::Previews,
                "Requests and cache",
                "preview.request.image_cache.max_size",
                "Cache limit (MB)",
                Control::CacheSize,
                "500",
                "Enter a positive number, or unlimited.",
            ),
            (
                MediaArea::Previews,
                "Requests and cache",
                "preview.request.image_cache.trim_interval",
                "Trim every image saves",
                Control::TrimInterval,
                "32",
                "Enter a positive number, or first-save-only.",
            ),
            (
                MediaArea::Transfers,
                "DCC transfers",
                "file_transfer.save_directory",
                "Save folder",
                Control::Text,
                "",
                "Empty asks for a folder for each transfer. Required for automatic acceptance.",
            ),
            (
                MediaArea::Transfers,
                "Automatic acceptance",
                "file_transfer.auto_accept.nicks",
                "Accept from nicknames",
                Control::List,
                "",
                "Separate nicknames with semicolons. Empty leaves nicknames unrestricted.",
            ),
            (
                MediaArea::Transfers,
                "Automatic acceptance",
                "file_transfer.auto_accept.masks",
                "Accept from masks",
                Control::Patterns,
                "",
                "Regular expressions matching nickname!username@hostname. Empty leaves masks unrestricted.",
            ),
            (
                MediaArea::Transfers,
                "DCC connection",
                "file_transfer.server.public_address",
                "Public IP address",
                Control::Address,
                "",
                "IP address advertised to the other user.",
            ),
            (
                MediaArea::Transfers,
                "DCC connection",
                "file_transfer.server.bind_address",
                "Bind IP address",
                Control::Address,
                "",
                "Local IP address for accepting connections.",
            ),
            (
                MediaArea::Transfers,
                "DCC connection",
                "file_transfer.server.bind_port_first",
                "First port",
                Control::Port,
                "",
                "First port in the inclusive range (1–65535).",
            ),
            (
                MediaArea::Transfers,
                "DCC connection",
                "file_transfer.server.bind_port_last",
                "Last port",
                Control::Port,
                "",
                "Last port must be at least the first port.",
            ),
        ] {
            let v = lookup(&root, path).map_or_else(
                || default.into(),
                |v| match control {
                    Control::List | Control::Patterns => {
                        display_list(&strings(Some(v)))
                    }
                    _ => {
                        v.as_str().map_or_else(|| v.to_string(), str::to_owned)
                    }
                },
            );
            add(area, group, path, label, control, v, help);
        }
        // Insert before the server fields so keyboard order matches the form.
        let position = fields
            .iter()
            .position(|f| f.control == Control::Address)
            .unwrap();
        let enabled =
            lookup(&root, "file_transfer.server").is_some().to_string();
        fields.insert(position,Field { area: MediaArea::Transfers,group:"DCC connection",path:"file_transfer.server".into(),label:"Configure DCC server".into(),help:"One side must accept connections. Fill both IP addresses and the port range before applying.",control:Control::ServerEnabled,original:enabled.clone(),value:enabled });
        Ok(Self {
            fields,
            original_auto_accept: lookup(
                &root,
                "file_transfer.auto_accept.enabled",
            )
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        })
    }
    pub fn is_dirty(&self) -> bool {
        self.fields.iter().any(Field::changed)
    }
    pub fn visible(&self, index: usize) -> bool {
        let field = &self.fields[index];
        if field.path.ends_with(".patterns") {
            return self.find(field.path.trim_end_matches(".patterns")).value
                == "Matching URLs";
        }
        if matches!(field.control, Control::Address | Control::Port) {
            return self.find("file_transfer.server").value == "true";
        }
        true
    }
    fn find(&self, path: &str) -> &Field {
        self.fields
            .iter()
            .find(|f| f.path == path)
            .expect("registered media field")
    }
    fn error(&self, path: &str, message: impl Into<String>) -> Error {
        Error {
            field: self
                .fields
                .iter()
                .position(|f| f.path == path)
                .expect("registered media field"),
            message: message.into(),
        }
    }

    /// Writes only changed leaves into a clone and commits it on success. Run
    /// this after Preferences::render_into so cross-field checks see all edits.
    pub fn render_into(&self, document: &mut DocumentMut) -> Result<(), Error> {
        let mut next = document.clone();
        for (index, field) in self.fields.iter().enumerate() {
            if !field.changed() || !self.visible(index) {
                continue;
            }
            let result = self.write_field(&mut next, field);
            result.map_err(|message| Error {
                field: index,
                message,
            })?;
        }
        // Complete the union even if only its pattern list changed. A mode
        // switch to Matching URLs must write its retained patterns as one value.
        for path in ["preview.enabled", "preview.card.description_decode_html"]
        {
            let mode = self.find(path);
            let patterns = self.find(&format!("{path}.patterns"));
            if mode.changed()
                || (mode.value == "Matching URLs" && patterns.changed())
            {
                let item = match mode.value.as_str() {
                    "Enabled" => value(true),
                    "Disabled" => value(false),
                    "Matching URLs" => array(
                        valid_patterns(&patterns.value)
                            .map_err(|e| self.error(&patterns.path, e))?,
                    ),
                    _ => {
                        return Err(self.error(
                            path,
                            "Choose enabled, disabled, or matching URLs.",
                        ));
                    }
                };
                write(&mut next, path, Some(item))
                    .map_err(|e| self.error(path, e))?;
            }
        }
        let server_changed = self
            .fields
            .iter()
            .any(|f| f.path.starts_with("file_transfer.server") && f.changed());
        if server_changed {
            if self.find("file_transfer.server").value == "false" {
                write(&mut next, "file_transfer.server", None)
                    .map_err(|e| self.error("file_transfer.server", e))?;
            } else {
                for leaf in [
                    "public_address",
                    "bind_address",
                    "bind_port_first",
                    "bind_port_last",
                ] {
                    let path = format!("file_transfer.server.{leaf}");
                    let field = self.find(&path);
                    let item = if field.control == Control::Address {
                        field.value.parse::<IpAddr>().map_err(|_| {
                            self.error(
                                &path,
                                "Enter a valid IPv4 or IPv6 address.",
                            )
                        })?;
                        value(&field.value)
                    } else {
                        value(i64::from(
                            field
                                .value
                                .parse::<u16>()
                                .ok()
                                .filter(|p| *p > 0)
                                .ok_or_else(|| {
                                    self.error(
                                        &path,
                                        "Enter a port from 1 to 65535.",
                                    )
                                })?,
                        ))
                    };
                    // Keep existing unchanged leaves (and their comments) intact.
                    if field.changed()
                        || self.find("file_transfer.server").changed()
                    {
                        write(&mut next, &path, Some(item))
                            .map_err(|e| self.error(&path, e))?;
                    }
                }
                let first = self
                    .find("file_transfer.server.bind_port_first")
                    .value
                    .parse::<u16>()
                    .unwrap();
                let last = self
                    .find("file_transfer.server.bind_port_last")
                    .value
                    .parse::<u16>()
                    .unwrap();
                if last < first {
                    return Err(self.error(
                        "file_transfer.server.bind_port_last",
                        "Last port must be at least the first port.",
                    ));
                }
            }
        }
        // Validate the requirement only when automatic acceptance or its folder
        // was explicitly edited; retain advanced existing configurations.
        let root: toml::Value =
            toml::from_str(&next.to_string()).map_err(|e| {
                self.error("file_transfer.save_directory", e.to_string())
            })?;
        let auto_path = "file_transfer.auto_accept.enabled";
        if lookup(&root, auto_path).and_then(toml::Value::as_bool) == Some(true)
            && (self.find("file_transfer.save_directory").changed()
                || !self.original_auto_accept)
            && lookup(&root, "file_transfer.save_directory")
                .and_then(toml::Value::as_str)
                .is_none_or(str::is_empty)
        {
            return Err(self.error(
                "file_transfer.save_directory",
                "Set a save folder before enabling automatic acceptance.",
            ));
        }
        *document = next;
        Ok(())
    }
    fn write_field(
        &self,
        document: &mut DocumentMut,
        field: &Field,
    ) -> Result<(), String> {
        let item = match field.control {
            Control::UrlMode
            | Control::ServerEnabled
            | Control::Address
            | Control::Port => return Ok(()),
            Control::Patterns if field.path.ends_with(".patterns") => {
                return Ok(());
            }
            Control::Text => {
                if field.value.is_empty() {
                    None
                } else {
                    Some(value(&field.value))
                }
            }
            Control::List | Control::Patterns | Control::Scope => {
                let items = if field.control == Control::Patterns {
                    valid_patterns(&field.value)?
                } else {
                    parse_list(&field.value)?
                };
                if field.control == Control::Scope
                    && matches!(field.value.trim(), "*" | "all")
                {
                    Some(value("*"))
                } else if items.is_empty() {
                    None
                } else {
                    Some(array(items))
                }
            }
            Control::CacheSize | Control::TrimInterval => {
                let special = if field.control == Control::CacheSize {
                    "unlimited"
                } else {
                    "first-save-only"
                };
                if field.value.is_empty() {
                    None
                } else if field.value.trim() == special {
                    Some(value(special))
                } else {
                    let number = field
                        .value
                        .parse::<i64>()
                        .ok()
                        .filter(|v| *v > 0)
                        .ok_or_else(|| {
                            format!(
                                "Enter a positive whole number, or {special}."
                            )
                        })?;
                    Some(value(number))
                }
            }
        };
        if field.control == Control::Scope {
            normalize_scope(document, field.path.rsplit_once('.').unwrap().0)?;
        }
        write(document, &field.path, item)
    }
}
fn valid_patterns(input: &str) -> Result<Vec<String>, String> {
    let items = parse_list(input)?;
    for pattern in &items {
        fancy_regex::Regex::new(pattern).map_err(|e| {
            format!("Invalid regular expression ‘{pattern}’: {e}")
        })?;
    }
    Ok(items)
}
fn write(
    document: &mut DocumentMut,
    path: &str,
    item: Option<Item>,
) -> Result<(), String> {
    let mut parts = path.split('.').collect::<Vec<_>>();
    let key = parts.pop().ok_or("Missing field name")?;
    let mut parent: &mut dyn TableLike = document.as_table_mut();
    // Clearing absent values must not create empty parent tables.
    if item.is_none() {
        for part in &parts {
            let Some(child) =
                parent.get_mut(part).and_then(Item::as_table_like_mut)
            else {
                return Ok(());
            };
            parent = child;
        }
    } else {
        for part in parts {
            parent = child_table(parent, part)?;
        }
    }
    if let Some(item) = item {
        set(parent, key, item);
    } else {
        parent.remove(key);
    }
    Ok(())
}
fn normalize_scope(
    document: &mut DocumentMut,
    path: &str,
) -> Result<(), String> {
    let root: toml::Value =
        toml::from_str(&document.to_string()).map_err(|e| e.to_string())?;
    let Some(source) = lookup(&root, path).filter(|v| !v.is_table()) else {
        return Ok(());
    };
    let source: Inclusivities = source
        .clone()
        .try_into()
        .map_err(|e: toml::de::Error| e.to_string())?;
    let mut table = InlineTable::new();
    for (name, scope) in [
        ("channels", source.channels),
        ("users", source.users),
        ("servers", source.servers),
        ("server_messages", source.server_messages),
    ] {
        if let Some(scope) = scope {
            table.insert(
                name,
                match scope {
                    Inclusivity::All => Value::from("*"),
                    Inclusivity::Any(items) => {
                        array(items).into_value().unwrap()
                    }
                },
            );
        }
    }
    write(document, path, Some(value(table)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edit(media: &mut Media, path: &str, input: &str) {
        media
            .fields
            .iter_mut()
            .find(|f| f.path == path)
            .unwrap()
            .value = input.into();
    }
    fn apply(source: &str, edits: &[(&str, &str)]) -> Result<String, Error> {
        let mut doc = source.parse::<DocumentMut>().unwrap();
        let mut media = Media::parse(&doc).unwrap();
        for (path, input) in edits {
            edit(&mut media, path, input);
        }
        media.render_into(&mut doc)?;
        crate::Config::validate(&doc.to_string()).unwrap();
        Ok(doc.to_string())
    }
    #[test]
    fn unchanged_media_is_byte_identical() {
        let source = "# keep\n[preview]\nenabled = 'example\\.org' # single regex\n[preview.card]\nexclude = { criteria = [{ user = 'bot', server = 'Libera' }], channels = ['#quiet'] }\n[preview.request.image_cache]\nmax_size = 'unlimited'\ntrim_interval = 'first-save-only'\n";
        assert_eq!(apply(source, &[]).unwrap(), source);
    }
    #[test]
    fn union_modes_and_quoted_pattern_lists_round_trip() {
        let output = apply(
            "[preview]\nenabled = 'old' # keep suffix\n",
            &[(
                "preview.enabled.patterns",
                r#""https://example.org/a;b"; example\.net"#,
            )],
        )
        .unwrap();
        assert!(output.contains("# keep suffix"));
        let parsed: toml::Value = toml::from_str(&output).unwrap();
        assert_eq!(
            parsed["preview"]["enabled"][0].as_str(),
            Some("https://example.org/a;b")
        );
        let output =
            apply(&output, &[("preview.enabled", "Disabled")]).unwrap();
        assert_eq!(toml::from_str::<toml::Value>(&output).unwrap()["preview"]["enabled"].as_bool(),Some(false));
        assert!(
            apply(
                "",
                &[
                    ("preview.enabled", "Matching URLs"),
                    ("preview.enabled.patterns", "[")
                ]
            )
            .is_err()
        );
    }
    #[test]
    fn scopes_preserve_compound_rules_and_convert_legacy_only_when_edited() {
        let source = "[preview.image]\nexclude = { criteria = [{ user = 'bot', server = 'Libera' }], channels = ['#old'] } # scope\n";
        let output =
            apply(source, &[("preview.image.exclude.channels", "#new; #a.b")])
                .unwrap();
        assert!(
            output.contains("criteria = [{ user = 'bot', server = 'Libera' }]")
        );
        assert!(output.contains("# scope"));
        let output = apply(
            "[preview.card]\nexclude = '*'\n",
            &[("preview.card.exclude.channels", "")],
        )
        .unwrap();
        let parsed: toml::Value = toml::from_str(&output).unwrap();
        assert_eq!(
            parsed["preview"]["card"]["exclude"]["users"].as_str(),
            Some("*")
        );
        assert!(
            parsed["preview"]["card"]["exclude"]
                .get("channels")
                .is_none()
        );
    }
    #[test]
    fn server_updates_are_atomic_and_validate_ports_and_addresses() {
        let edits = [
            ("file_transfer.server", "true"),
            ("file_transfer.server.public_address", "203.0.113.1"),
            ("file_transfer.server.bind_address", "0.0.0.0"),
            ("file_transfer.server.bind_port_first", "5000"),
            ("file_transfer.server.bind_port_last", "5010"),
        ];
        let output = apply("", &edits).unwrap();
        assert!(
            apply(&output, &[("file_transfer.server.bind_port_last", "4999")])
                .is_err()
        );
        assert!(
            apply(
                &output,
                &[("file_transfer.server.bind_address", "hostname")]
            )
            .is_err()
        );
        assert!(
            apply(&output, &[("file_transfer.server.bind_port_first", "0")])
                .is_err()
        );
        assert!(
            !apply(&output, &[("file_transfer.server", "false")])
                .unwrap()
                .contains("[file_transfer.server]")
        );
        let mut doc = "".parse::<DocumentMut>().unwrap();
        let mut media = Media::parse(&doc).unwrap();
        edit(&mut media, "file_transfer.server", "true");
        assert!(media.render_into(&mut doc).is_err());
        assert_eq!(doc.to_string(), "");
    }
    #[test]
    fn cache_special_values_and_folder_removal_are_supported() {
        let output=apply("[file_transfer]\nsave_directory = '/tmp' # folder\npassive = true\n",&[("file_transfer.save_directory",""),("preview.request.image_cache.max_size","unlimited"),("preview.request.image_cache.trim_interval","first-save-only")]).unwrap();
        assert!(!output.contains("save_directory"));
        assert!(output.contains("passive = true"));
        for path in [
            "preview.request.image_cache.max_size",
            "preview.request.image_cache.trim_interval",
        ] {
            assert!(apply("", &[(path, "0")]).is_err());
        }
    }
    #[test]
    fn legacy_scopes_and_unrelated_upload_credentials_survive_edits() {
        let source = "[preview.card]\nexclude = ['NickServ', '#quiet'] # legacy\n[servers.test]\nserver = 'irc.example.org'\nnickname = 'test'\n[servers.test.filehost]\noverride_url = 'https://example.org/upload'\n";
        // The media writer edits only its own leaves and retains per-network
        // upload overrides owned by the network form.
        let mut doc = source.parse::<DocumentMut>().unwrap();
        let mut media = Media::parse(&doc).unwrap();
        edit(&mut media, "preview.card.exclude.servers", "Libera");
        media.render_into(&mut doc).unwrap();
        let output = doc.to_string();
        assert!(output.contains("override_url = 'https://example.org/upload'"));
        crate::Config::validate(&output).unwrap();
        assert!(output.contains("# legacy"));
        let parsed: toml::Value = toml::from_str(&output).unwrap();
        let scope = &parsed["preview"]["card"]["exclude"];
        assert_eq!(scope["users"][0].as_str(), Some("NickServ"));
        assert_eq!(scope["channels"][0].as_str(), Some("#quiet"));
        assert_eq!(scope["servers"][0].as_str(), Some("Libera"));
    }
    #[test]
    fn automatic_acceptance_requires_folder_when_newly_enabled() {
        let mut doc = "".parse::<DocumentMut>().unwrap();
        let media = Media::parse(&doc).unwrap();
        // Simulate Preferences writing the switch before the media transaction.
        write(
            &mut doc,
            "file_transfer.auto_accept.enabled",
            Some(value(true)),
        )
        .unwrap();
        let error = media.render_into(&mut doc).unwrap_err();
        assert_eq!(
            media.fields[error.field].path,
            "file_transfer.save_directory"
        );
        let mut media = media;
        edit(&mut media, "file_transfer.save_directory", "/tmp");
        media.render_into(&mut doc).unwrap();
        crate::Config::validate(&doc.to_string()).unwrap();
    }
    #[test]
    fn masks_are_validated_and_empty_optional_lists_restore_unrestricted_matching()
     {
        assert!(
            apply("", &[("file_transfer.auto_accept.masks", "[")]).is_err()
        );
        let output = apply(
            "[file_transfer.auto_accept]\nnicks = ['nick']\nmasks = ['.*']\n",
            &[
                ("file_transfer.auto_accept.nicks", ""),
                ("file_transfer.auto_accept.masks", ""),
            ],
        )
        .unwrap();
        let parsed: toml::Value = toml::from_str(&output).unwrap();
        assert!(
            parsed["file_transfer"]["auto_accept"]
                .get("nicks")
                .is_none()
        );
        assert!(
            parsed["file_transfer"]["auto_accept"]
                .get("masks")
                .is_none()
        );
    }
    #[test]
    fn schema_defaults_and_choices_are_runtime_fields() {
        // Areas are labels only; existing variants let this test run before
        // the integrator adds Previews/Transfers to the shared enum.
        for spec in schema(Area::Chat, Area::Navigation) {
            let values = match spec.kind {
                Kind::Choice(c) => {
                    c.iter().map(|c| c.value).collect::<Vec<_>>()
                }
                Kind::Boolean => vec!["true", "false"],
                _ => vec![spec.default],
            };
            for input in values {
                let mut doc = DocumentMut::new();
                write(
                    &mut doc,
                    spec.path,
                    Some(Item::Value(input.parse::<Value>().unwrap())),
                )
                .unwrap();
                let mut ignored = vec![];
                let _: crate::config::Configuration =
                    serde_ignored::deserialize(
                        toml::Deserializer::parse(&doc.to_string()).unwrap(),
                        |p| ignored.push(p.to_string()),
                    )
                    .unwrap();
                assert!(ignored.is_empty(), "{}: {ignored:?}", spec.path);
            }
        }
    }
}
