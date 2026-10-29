//! General and command settings. This module owns only targeted leaves; the
//! Settings transaction remains responsible for external edits and backups.
use toml_edit::{
    Array, DocumentMut, InlineTable, Item, TableLike, Value, value,
};

use super::preferences::{Area, Choice, Kind, Spec};
use super::rules::{display_list, parse_list};
use super::{child_table, set};
use crate::config::inclusivities::{Inclusivities, Inclusivity};

/// Scalar schema, also suitable for adding to the shared preference schema.
/// Do not register it twice when using `Draft`, which loads these itself.
pub fn schema() -> Vec<Spec> {
    use Area::{Commands, General};
    let boolean = Kind::Boolean;
    let number = |max| Kind::Integer {
        min: 0,
        max,
        optional: false,
    };
    let optional_dimension = Kind::Integer {
        min: 1,
        max: u32::MAX.into(),
        optional: true,
    };
    let mut specs = Vec::new();
    macro_rules! add {
        ($area:ident, $group:literal, $path:literal, $label:literal, $kind:expr, $default:literal, $help:literal) => {
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
    add!(
        General,
        "Application",
        "tooltips",
        "Tooltips",
        Kind::Choice(&[
            Choice {
                label: "All",
                value: "\"all\""
            },
            Choice {
                label: "Autocomplete only",
                value: "\"autocomplete\""
            },
            Choice {
                label: "Hidden",
                value: "\"none\""
            },
            Choice {
                label: "All (legacy)",
                value: "true"
            },
            Choice {
                label: "Hidden (legacy)",
                value: "false"
            }
        ]),
        "\"all\"",
        ""
    );
    add!(
        General,
        "Application",
        "check_for_update_on_launch",
        "Check for updates on launch",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Window",
        "window.initial_width",
        "Initial width",
        optional_dimension,
        "",
        "Pixels; empty uses the saved window size."
    );
    add!(
        General,
        "Window",
        "window.initial_height",
        "Initial height",
        optional_dimension,
        "",
        "Pixels; empty uses the saved window size."
    );
    add!(
        General,
        "Window",
        "platform_specific.macos.decorations",
        "macOS window decorations",
        boolean,
        "true",
        "Restart Halloy to use window and rendering changes."
    );
    add!(
        General,
        "Window",
        "platform_specific.windows.decorations",
        "Windows window decorations",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Window",
        "platform_specific.linux.decorations",
        "Linux window decorations",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Window",
        "platform_specific.macos.content_padding",
        "macOS content padding",
        Kind::Choice(&[
            Choice {
                label: "Under title bar",
                value: "\"embedded-content\""
            },
            Choice {
                label: "Below title bar",
                value: "\"padded-content\""
            }
        ]),
        "\"embedded-content\"",
        ""
    );
    add!(
        General,
        "Window",
        "platform_specific.macos.sidebar_padding",
        "macOS sidebar padding",
        Kind::Choice(&[
            Choice {
                label: "Under title bar",
                value: "\"embedded-content\""
            },
            Choice {
                label: "Below title bar",
                value: "\"padded-content\""
            }
        ]),
        "\"padded-content\"",
        ""
    );
    add!(
        General,
        "Rendering",
        "runtime.backend",
        "Rendering backend",
        Kind::Choice(&[
            Choice {
                label: "Automatic",
                value: "\"best\""
            },
            Choice {
                label: "Hardware",
                value: "\"hardware\""
            },
            Choice {
                label: "Software",
                value: "\"software\""
            },
            Choice {
                label: "Vulkan",
                value: "{ hardware = \"vulkan\" }"
            },
            Choice {
                label: "Metal",
                value: "{ hardware = \"metal\" }"
            },
            Choice {
                label: "DirectX 12",
                value: "{ hardware = \"directx12\" }"
            },
            Choice {
                label: "OpenGL",
                value: "{ hardware = \"opengl\" }"
            },
            Choice {
                label: "WebGPU",
                value: "{ hardware = \"webgpu\" }"
            }
        ]),
        "\"best\"",
        "Restart Halloy to use rendering changes. Backend availability depends on your platform."
    );
    add!(
        General,
        "Rendering",
        "runtime.power_preference",
        "GPU power preference",
        Kind::Choice(&[
            Choice {
                label: "Automatic",
                value: "\"none\""
            },
            Choice {
                label: "Low power",
                value: "\"low-power\""
            },
            Choice {
                label: "High performance",
                value: "\"high-performance\""
            }
        ]),
        "\"none\"",
        ""
    );
    add!(
        General,
        "Rendering",
        "runtime.vsync",
        "Vertical sync",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Rendering",
        "runtime.antialiasing",
        "Antialiasing",
        boolean,
        "false",
        ""
    );
    add!(
        General,
        "Rendering",
        "runtime.metrics_hinting",
        "Font metrics hinting",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Rendering",
        "runtime.smooth_scrolling",
        "Smooth scrolling",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Logs",
        "logs.file_level",
        "File log level",
        Kind::Choice(&[
            Choice {
                label: "Off",
                value: "\"off\""
            },
            Choice {
                label: "Error",
                value: "\"error\""
            },
            Choice {
                label: "Warning",
                value: "\"warn\""
            },
            Choice {
                label: "Info",
                value: "\"info\""
            },
            Choice {
                label: "Debug",
                value: "\"debug\""
            },
            Choice {
                label: "Trace",
                value: "\"trace\""
            }
        ]),
        "\"debug\"",
        "Restart Halloy to use logging changes."
    );
    add!(
        General,
        "Logs",
        "logs.pane_level",
        "Log pane level",
        Kind::Choice(&[
            Choice {
                label: "Off",
                value: "\"off\""
            },
            Choice {
                label: "Error",
                value: "\"error\""
            },
            Choice {
                label: "Warning",
                value: "\"warn\""
            },
            Choice {
                label: "Info",
                value: "\"info\""
            },
            Choice {
                label: "Debug",
                value: "\"debug\""
            },
            Choice {
                label: "Trace",
                value: "\"trace\""
            }
        ]),
        "\"info\"",
        ""
    );
    add!(
        General,
        "Logs",
        "logs.file_timestamp",
        "File timestamps",
        Kind::Choice(&[
            Choice {
                label: "Local time",
                value: "\"local\""
            },
            Choice {
                label: "UTC",
                value: "\"utc\""
            }
        ]),
        "\"local\"",
        ""
    );
    add!(
        General,
        "Logs",
        "logs.max_file_count",
        "Maximum log files",
        number(i64::MAX),
        "4",
        ""
    );
    add!(
        General,
        "Config Editor",
        "config_editor.indent",
        "Indentation",
        Kind::Choice(&[
            Choice {
                label: "Tabs",
                value: "\"tab\""
            },
            Choice {
                label: "1 space",
                value: "1"
            },
            Choice {
                label: "2 spaces",
                value: "2"
            },
            Choice {
                label: "3 spaces",
                value: "3"
            },
            Choice {
                label: "4 spaces",
                value: "4"
            },
            Choice {
                label: "5 spaces",
                value: "5"
            },
            Choice {
                label: "6 spaces",
                value: "6"
            },
            Choice {
                label: "7 spaces",
                value: "7"
            },
            Choice {
                label: "8 spaces",
                value: "8"
            }
        ]),
        "\"tab\"",
        ""
    );
    add!(
        General,
        "Context menu",
        "context_menu.padding.entry",
        "Entry padding",
        Kind::Pair,
        "[5, 5]",
        "Vertical, horizontal pixels, separated by a comma."
    );
    add!(
        General,
        "Context menu",
        "context_menu.show_user_metadata",
        "Show user metadata",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Metadata",
        "metadata.avatar.enabled",
        "Load avatars",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Metadata",
        "metadata.avatar.size",
        "Requested avatar size",
        number(u16::MAX.into()),
        "64",
        "Pixels requested from avatar URLs that support a size placeholder."
    );
    add!(
        General,
        "Display",
        "display.direction_arrows.left",
        "Left direction arrow",
        Kind::Text,
        "\"←\"",
        ""
    );
    add!(
        General,
        "Display",
        "display.direction_arrows.right",
        "Right direction arrow",
        Kind::Text,
        "\"→\"",
        ""
    );
    add!(
        General,
        "Display",
        "display.decode_urls",
        "Decode URLs for display",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "Display",
        "display.truncation_character",
        "Truncation character",
        Kind::Text,
        "\"…\"",
        "Enter one character."
    );
    add!(
        General,
        "Display",
        "display.adapt_metadata_colors",
        "Adapt metadata colors",
        Kind::Choice(&[
            Choice {
                label: "All colors",
                value: "\"all\""
            },
            Choice {
                label: "Only hard to read colors",
                value: "\"illegible\""
            },
            Choice {
                label: "No adaptation",
                value: "\"none\""
            }
        ]),
        "\"all\"",
        ""
    );
    add!(
        General,
        "CTCP replies",
        "ctcp.ping",
        "Reply to PING",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "CTCP replies",
        "ctcp.source",
        "Reply to SOURCE",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "CTCP replies",
        "ctcp.time",
        "Reply to TIME",
        boolean,
        "true",
        ""
    );
    add!(
        General,
        "CTCP replies",
        "ctcp.version",
        "Reply to VERSION",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "Commands",
        "buffer.commands.show_description",
        "Show command descriptions",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "Run commands",
        "buffer.commands.exec.enabled",
        "Enable /exec",
        boolean,
        "false",
        "/exec runs local shell commands. Enable it only if you intend to use it."
    );
    add!(
        Commands,
        "Run commands",
        "buffer.commands.exec.timeout",
        "Command timeout",
        number(i64::MAX),
        "5",
        "Seconds."
    );
    add!(
        Commands,
        "Run commands",
        "buffer.commands.exec.max_output_bytes",
        "Maximum output",
        number(i64::MAX),
        "4096",
        "Bytes."
    );
    add!(
        Commands,
        "System information",
        "buffer.commands.sysinfo.cpu",
        "Include CPU",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "System information",
        "buffer.commands.sysinfo.memory",
        "Include Memory",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "System information",
        "buffer.commands.sysinfo.gpu",
        "Include GPU",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "System information",
        "buffer.commands.sysinfo.os",
        "Include Operating system",
        boolean,
        "true",
        ""
    );
    add!(
        Commands,
        "System information",
        "buffer.commands.sysinfo.uptime",
        "Include Uptime",
        boolean,
        "true",
        ""
    );
    specs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Default,
    Set,
}
#[derive(Debug, Clone, Copy)]
pub enum ListKind {
    MetadataKeys,
    Regex,
    Scope,
}
#[derive(Debug, Clone, Copy)]
pub enum Editor {
    Scalar(Kind),
    List(ListKind),
    OptionalText,
}
#[derive(Debug, Clone)]
pub struct Field {
    pub area: Area,
    pub group: &'static str,
    pub path: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub editor: Editor,
    pub value: String,
    pub mode: Mode,
    default: String,
    original: String,
    original_mode: Mode,
}
#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub command: String,
    pub removed: bool,
    original_name: Option<String>,
    original_command: String,
}
#[derive(Debug, Clone)]
pub struct Draft {
    pub fields: Vec<Field>,
    pub aliases: Vec<Alias>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Field(usize),
    AliasName(usize),
    AliasCommand(usize),
}
#[derive(Debug, Clone)]
pub struct Error {
    pub target: Target,
    pub message: String,
}

fn lookup<'a>(mut item: &'a Item, path: &str) -> Option<&'a Item> {
    for part in path.split('.') {
        item = item.as_table_like()?.get(part)?;
    }
    Some(item)
}
fn literal(source: &str) -> Option<Value> {
    source.parse().ok()
}
fn display(item: Option<&Item>, editor: Editor) -> String {
    let Some(item) = item else {
        return String::new();
    };
    match editor {
        Editor::OptionalText | Editor::Scalar(Kind::Text) => {
            item.as_str().unwrap_or("Configured in file").to_owned()
        }
        Editor::List(_) => {
            if let Some(string) = item.as_str() {
                return string.to_owned();
            }
            item.as_array()
                .map(|array| {
                    display_list(
                        &array
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect::<Vec<_>>(),
                    )
                })
                .unwrap_or_default()
        }
        Editor::Scalar(Kind::Pair) => item
            .as_array()
            .map(|array| {
                array
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default(),
        _ => item
            .as_value()
            .map_or_else(|| "Configured in file".into(), ToString::to_string),
    }
}
fn array(items: Vec<String>) -> Item {
    let mut array = Array::new();
    for item in items {
        array.push(item);
    }
    value(array)
}
fn inclusivities(item: &Item) -> Result<Inclusivities, String> {
    let root: toml::Value =
        toml::from_str(&format!("v = {item}")).map_err(|e| e.to_string())?;
    root.get("v")
        .unwrap()
        .clone()
        .try_into()
        .map_err(|e: toml::de::Error| e.to_string())
}
fn projected_scope(source: &Item, path: &str) -> Option<Item> {
    let (parent, key) = path.rsplit_once('.')?;
    let item = lookup(source, parent)?;
    if item.as_table_like().is_some() {
        return None;
    }
    let scopes = inclusivities(item).ok()?;
    let scope = match key {
        "channels" => scopes.channels,
        "users" => scopes.users,
        "servers" => scopes.servers,
        "server_messages" => scopes.server_messages,
        _ => None,
    }?;
    Some(match scope {
        Inclusivity::All => value("*"),
        Inclusivity::Any(values) => array(values),
    })
}
impl Draft {
    pub fn parse(document: &DocumentMut) -> Result<Self, String> {
        let source = document.as_item();
        let mut fields = schema()
            .into_iter()
            .map(|spec| {
                let editor = Editor::Scalar(spec.kind);
                let default_item = literal(spec.default).map(Item::Value);
                let default = display(default_item.as_ref(), editor);
                let item = lookup(source, spec.path);
                let mode = if item.is_some() {
                    Mode::Set
                } else {
                    Mode::Default
                };
                let value = item.map_or_else(
                    || default.clone(),
                    |i| display(Some(i), editor),
                );
                Field {
                    area: spec.area,
                    group: spec.group,
                    path: spec.path,
                    label: spec.label,
                    help: spec.help,
                    editor,
                    original: value.clone(),
                    value,
                    mode,
                    original_mode: mode,
                    default,
                }
            })
            .collect::<Vec<_>>();
        let mut add =
            |area, group, path, label, editor, default: &str, help| {
                let projected = projected_scope(source, path);
                let item = lookup(source, path).or(projected.as_ref());
                let mode = if item.is_some() {
                    Mode::Set
                } else {
                    Mode::Default
                };
                let value = item.map_or_else(
                    || default.to_owned(),
                    |i| display(Some(i), editor),
                );
                fields.push(Field {
                    area,
                    group,
                    path,
                    label,
                    help,
                    editor,
                    original: value.clone(),
                    value,
                    mode,
                    original_mode: mode,
                    default: default.into(),
                });
            };
        add(
            Area::General,
            "Metadata",
            "metadata.preferred_keys",
            "Subscribed keys, in order",
            Editor::List(ListKind::MetadataKeys),
            "display-name; avatar; pronouns; homepage; color; status",
            "Separate keys with semicolons. Empty explicitly disables all subscriptions. Allowed keys: display-name, avatar, pronouns, homepage, color, status.",
        );
        add(
            Area::General,
            "Metadata",
            "metadata.avatar.exclude",
            "Excluded avatar URLs",
            Editor::List(ListKind::Regex),
            "",
            "Regular expressions separated by semicolons. Quote entries containing a literal semicolon using TOML string syntax.",
        );
        add(
            Area::General,
            "CTCP replies",
            "ctcp.userinfo",
            "USERINFO reply",
            Editor::OptionalText,
            "",
            "Default disables the USERINFO reply. An explicitly empty reply still enables it.",
        );
        add(
            Area::Commands,
            "Default messages",
            "buffer.commands.quit.default_reason",
            "Default quit message",
            Editor::OptionalText,
            "",
            "Default uses Halloy’s normal quit behavior. An empty message also uses the default.",
        );
        add(
            Area::Commands,
            "Default messages",
            "buffer.commands.part.default_reason",
            "Default part message",
            Editor::OptionalText,
            "",
            "Default omits the part message. An empty message also omits it.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.exclude.channels",
            "Exclude channels",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.exclude.servers",
            "Exclude servers",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.exclude.users",
            "Exclude users",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.exclude.server_messages",
            "Exclude server messages",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.include.channels",
            "Include channels",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.include.servers",
            "Include servers",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.include.users",
            "Include users",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        add(
            Area::General,
            "Channel Monitor",
            "channel_monitor.include.server_messages",
            "Include server messages",
            Editor::List(ListKind::Scope),
            "",
            "Separate names with semicolons; * matches all. Includes take priority. Existing compound criteria are retained.",
        );
        let aliases = lookup(source, "buffer.commands.aliases")
            .and_then(Item::as_table_like)
            .map(|table| {
                table
                    .iter()
                    .filter_map(|(name, item)| {
                        item.as_str().map(|command| Alias {
                            name: name.into(),
                            command: command.into(),
                            removed: false,
                            original_name: Some(name.into()),
                            original_command: command.into(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { fields, aliases })
    }
    pub fn groups(&self, area: Area) -> Vec<&'static str> {
        let mut groups = Vec::new();
        for field in &self.fields {
            if field.area == area && !groups.contains(&field.group) {
                groups.push(field.group);
            }
        }
        if area == Area::Commands {
            groups.push("Aliases");
        }
        groups
    }
    pub fn is_dirty(&self) -> bool {
        self.fields.iter().any(Field::changed)
            || self.aliases.iter().any(Alias::changed)
    }
    pub fn add_alias(&mut self) {
        self.aliases.push(Alias {
            name: String::new(),
            command: "/".into(),
            removed: false,
            original_name: None,
            original_command: String::new(),
        });
    }
    pub fn edit(&mut self, index: usize, value: String) {
        if let Some(field) = self.fields.get_mut(index) {
            field.value = value;
            field.mode = Mode::Set;
        }
    }
    pub fn reset(&mut self, index: usize) {
        if let Some(field) = self.fields.get_mut(index) {
            field.value = field.default.clone();
            field.mode = Mode::Default;
        }
    }
    pub fn set_explicit(&mut self, index: usize) {
        if let Some(field) = self.fields.get_mut(index) {
            field.mode = Mode::Set;
        }
    }
    pub fn validate(&self, source: &DocumentMut) -> Result<(), Error> {
        let mut copy = source.clone();
        self.render_into(&mut copy)?;
        // No commands, credentials, or network access are used by validation.
        crate::Config::validate(&copy.to_string()).map_err(|e| Error {
            target: self
                .fields
                .iter()
                .position(Field::changed)
                .map_or(Target::AliasName(0), Target::Field),
            message: e.to_string(),
        })
    }
    pub fn render_into(&self, document: &mut DocumentMut) -> Result<(), Error> {
        // Rename validation is performed together, allowing swaps.
        let mut names = std::collections::HashSet::new();
        for (index, alias) in
            self.aliases.iter().enumerate().filter(|(_, a)| !a.removed)
        {
            // The runtime deliberately ignores non-command aliases. Preserve
            // those existing entries until the user explicitly edits them.
            if !alias.changed() && !alias.command.trim().starts_with('/') {
                continue;
            }
            let name = alias.normalized_name();
            if name.is_empty()
                || (alias.changed() && name.chars().any(char::is_whitespace))
                || !names.insert(name)
            {
                return Err(Error {
                    target: Target::AliasName(index),
                    message: "Use a unique command name without spaces.".into(),
                });
            }
            if alias.changed()
                && (!alias.command.trim().starts_with('/')
                    || alias.command.trim() == "/"
                    || !crate::command::alias::has_trailing_optional_args(
                        &alias.command,
                    ))
            {
                return Err(Error { target: Target::AliasCommand(index), message: "Enter a command beginning with /; optional arguments must follow required arguments.".into() });
            }
        }
        for (index, field) in
            self.fields.iter().enumerate().filter(|(_, f)| f.changed())
        {
            let item = field.render().map_err(|message| Error {
                target: Target::Field(index),
                message: format!("{}: {message}", field.label),
            })?;
            write_leaf(document, field.path, item).map_err(|message| {
                Error {
                    target: Target::Field(index),
                    message,
                }
            })?;
        }
        if self.aliases.iter().any(Alias::changed) {
            let parent = parent_table(document, "buffer.commands.aliases")
                .map_err(|message| Error {
                    target: Target::AliasName(0),
                    message,
                })?;
            // Rebuild this map in its existing order, carrying keys, literal
            // strings and comments through renames and swaps. Unknown entries
            // are copied verbatim; only changed alias values are replaced.
            let originals = parent
                .iter()
                .map(|(name, item)| {
                    (parent.key(name).unwrap().clone(), item.clone())
                })
                .collect::<Vec<_>>();
            parent.clear();
            for (key, mut item) in originals {
                let alias = self.aliases.iter().find(|alias| {
                    alias.original_name.as_deref() == Some(key.get())
                });
                let Some(alias) = alias else {
                    parent.entry_format(&key).or_insert(item);
                    continue;
                };
                if alias.removed {
                    continue;
                }
                if alias.command != alias.original_command {
                    let mut next = value(&alias.command);
                    if let Some(previous) = item.as_value() {
                        *next.as_value_mut().unwrap().decor_mut() =
                            previous.decor().clone();
                    }
                    item = next;
                }
                let key = if alias.name == key.get() {
                    key
                } else {
                    toml_edit::Key::new(alias.name.as_str())
                        .with_leaf_decor(key.leaf_decor().clone())
                        .with_dotted_decor(key.dotted_decor().clone())
                };
                parent.entry_format(&key).or_insert(item);
            }
            for alias in self
                .aliases
                .iter()
                .filter(|a| a.original_name.is_none() && !a.removed)
            {
                set(parent, &alias.name, value(&alias.command));
            }
        }
        Ok(())
    }
}
impl Alias {
    pub fn normalized_name(&self) -> String {
        self.name
            .trim()
            .trim_start_matches('/')
            .to_ascii_lowercase()
    }
    pub fn changed(&self) -> bool {
        match &self.original_name {
            Some(name) => {
                self.removed
                    || self.name != *name
                    || self.command != self.original_command
            }
            None => !self.removed,
        }
    }
}
impl Field {
    pub fn changed(&self) -> bool {
        self.mode != self.original_mode || self.value != self.original
    }
    pub fn is_custom(&self) -> bool {
        match self.editor {
            Editor::Scalar(Kind::Choice(choices)) => {
                !choices.iter().any(|c| same_literal(c.value, &self.value))
            }
            Editor::Scalar(Kind::Boolean) => {
                !matches!(self.value.as_str(), "true" | "false")
            }
            _ => false,
        }
    }
    fn render(&self) -> Result<Option<Item>, String> {
        if self.mode == Mode::Default {
            return Ok(None);
        }
        let input = self.value.trim();
        Ok(Some(match self.editor {
            Editor::OptionalText | Editor::Scalar(Kind::Text) => {
                if self.path == "display.truncation_character"
                    && self.value.chars().count() != 1
                {
                    return Err("Enter exactly one character.".into());
                }
                value(&self.value)
            }
            Editor::Scalar(Kind::Boolean) => value(
                input
                    .parse::<bool>()
                    .map_err(|_| "Choose enabled or disabled.")?,
            ),
            Editor::Scalar(Kind::Choice(choices)) => {
                let choice = choices
                    .iter()
                    .find(|c| same_literal(c.value, &self.value))
                    .ok_or("Choose an available option.")?;
                Item::Value(literal(choice.value).ok_or("Invalid choice.")?)
            }
            Editor::Scalar(Kind::Integer { min, max, optional }) => {
                if optional && input.is_empty() {
                    return Ok(None);
                }
                value(
                    input
                        .parse::<i64>()
                        .ok()
                        .filter(|n| (min..=max).contains(n))
                        .ok_or_else(|| {
                            format!("Enter a whole number from {min} to {max}.")
                        })?,
                )
            }
            Editor::Scalar(Kind::Float { min, max, optional }) => {
                if optional && input.is_empty() {
                    return Ok(None);
                }
                value(
                    input
                        .parse::<f64>()
                        .ok()
                        .filter(|n| n.is_finite() && (min..=max).contains(n))
                        .ok_or_else(|| {
                            format!("Enter a number from {min} to {max}.")
                        })?,
                )
            }
            Editor::Scalar(Kind::Pair) => {
                let values = input
                    .split(',')
                    .map(str::trim)
                    .map(str::parse::<u16>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Enter two whole numbers from 0 to 65535.")?;
                if values.len() != 2 {
                    return Err(
                        "Enter two numbers, separated by a comma.".into()
                    );
                }
                let mut array = Array::new();
                for v in values {
                    array.push(i64::from(v));
                }
                value(array)
            }
            Editor::List(kind) => {
                if matches!(kind, ListKind::Scope)
                    && matches!(input, "*" | "all")
                {
                    return Ok(Some(value("*")));
                }
                let values = parse_list(&self.value)?;
                for item in &values {
                    match kind {
                        ListKind::MetadataKeys
                            if ![
                                "display-name",
                                "avatar",
                                "pronouns",
                                "homepage",
                                "color",
                                "status",
                            ]
                            .contains(&item.as_str()) =>
                        {
                            return Err(format!(
                                "Unknown metadata key: {item}"
                            ));
                        }
                        ListKind::Regex => {
                            fancy_regex::Regex::new(item)
                                .map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                }
                array(values)
            }
        }))
    }
}
pub fn same_literal(left: &str, right: &str) -> bool {
    fn parse(value: &str) -> Option<toml::Value> {
        toml::from_str::<toml::Table>(&format!("v = {value}"))
            .ok()?
            .remove("v")
    }
    parse(left).zip(parse(right)).is_some_and(|(a, b)| a == b)
}
fn parent_table<'a>(
    document: &'a mut DocumentMut,
    path: &str,
) -> Result<&'a mut dyn TableLike, String> {
    let mut parent: &mut dyn TableLike = document.as_table_mut();
    for part in path.split('.') {
        if matches!(part, "include" | "exclude")
            && parent
                .get(part)
                .is_some_and(|i| i.as_table_like().is_none())
        {
            let old = inclusivities(parent.get(part).unwrap())?;
            let mut converted = InlineTable::new();
            for (name, scope) in [
                ("channels", old.channels),
                ("users", old.users),
                ("servers", old.servers),
                ("server_messages", old.server_messages),
            ] {
                if let Some(scope) = scope {
                    converted.insert(
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
            set(parent, part, value(converted));
        }
        parent = child_table(parent, part)?;
    }
    Ok(parent)
}
fn write_leaf(
    document: &mut DocumentMut,
    path: &str,
    item: Option<Item>,
) -> Result<(), String> {
    let (parent_path, key) = path.rsplit_once('.').unwrap_or(("", path));
    let parent: &mut dyn TableLike = if parent_path.is_empty() {
        document.as_table_mut()
    } else {
        parent_table(document, parent_path)?
    };
    if let Some(item) = item {
        set(parent, key, item);
    } else {
        parent.remove(key);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document(source: &str) -> DocumentMut {
        source.parse().unwrap()
    }
    fn edit(draft: &mut Draft, path: &str, value: &str) {
        let index = draft
            .fields
            .iter()
            .position(|field| field.path == path)
            .unwrap();
        draft.edit(index, value.into());
    }
    fn render(source: &str, edit_draft: impl FnOnce(&mut Draft)) -> String {
        let mut source = document(source);
        let mut draft = Draft::parse(&source).unwrap();
        edit_draft(&mut draft);
        draft.render_into(&mut source).unwrap();
        crate::Config::validate(&source.to_string()).unwrap();
        source.to_string()
    }
    #[test]
    fn scalar_choices_and_defaults_match_the_runtime_config() {
        for spec in schema() {
            let mut values = vec![spec.default];
            if let Kind::Choice(choices) = spec.kind {
                values.extend(choices.iter().map(|c| c.value));
            }
            for literal in values.into_iter().filter(|v| !v.is_empty()) {
                let mut source = document("");
                write_leaf(
                    &mut source,
                    spec.path,
                    Some(Item::Value(literal.parse().unwrap())),
                )
                .unwrap();
                crate::Config::validate(&source.to_string()).unwrap_or_else(
                    |e| panic!("{} = {}: {e}", spec.path, literal),
                );
            }
        }
    }
    #[test]
    fn untouched_advanced_config_is_byte_identical() {
        let source = "# header\ntooltips = false # old syntax\n[runtime.backend]\nhardware = 'metal'\n[channel_monitor.exclude]\ncriteria = [{ channel = '#private', server = 'libera' }]\n[buffer.commands.aliases]\n'UPPER' = '/me $1-' # command\n'ignored' = 'non-command' # runtime ignores this\n";
        assert_eq!(render(source, |_| {}), source);
    }
    #[test]
    fn edited_leaves_keep_order_inline_tables_and_comments() {
        let source = "# header\n[logs]\nfile_level = 'trace' # keep timestamp below\nfile_timestamp = 'utc'\nmax_file_count = 8 # retention\n[context_menu]\npadding = { entry = [9, 3], future = true } # padding\n[servers.test]\nserver = 'localhost'\nnickname = 'test'\nautoconnect = false\n";
        let next = render(source, |draft| {
            edit(draft, "logs.max_file_count", "10");
            edit(draft, "context_menu.padding.entry", "4, 6");
        });
        assert!(next.contains("max_file_count = 10 # retention"));
        assert!(next.contains("future = true"));
        assert!(next.contains("# padding"));
        assert!(next.contains("file_level = 'trace' # keep timestamp below"));
        assert!(next.contains("autoconnect = false"));
    }
    #[test]
    fn default_and_explicit_empty_userinfo_have_different_meanings() {
        let source = document("");
        let mut draft = Draft::parse(&source).unwrap();
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == "ctcp.userinfo")
            .unwrap();
        assert_eq!(draft.fields[index].mode, Mode::Default);
        draft.set_explicit(index);
        let mut next = source.clone();
        draft.render_into(&mut next).unwrap();
        assert_eq!(
            lookup(next.as_item(), "ctcp.userinfo").unwrap().as_str(),
            Some("")
        );
        draft.reset(index);
        assert!(!draft.is_dirty());
        let mut draft = Draft::parse(&next).unwrap();
        draft.reset(index);
        draft.render_into(&mut next).unwrap();
        assert!(lookup(next.as_item(), "ctcp.userinfo").is_none());
    }
    #[test]
    fn empty_metadata_list_disables_subscriptions_and_default_removes_override()
    {
        let next =
            render("", |draft| edit(draft, "metadata.preferred_keys", ""));
        let next = document(&next);
        assert!(
            lookup(next.as_item(), "metadata.preferred_keys")
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        let mut draft = Draft::parse(&next).unwrap();
        let index = draft
            .fields
            .iter()
            .position(|f| f.path == "metadata.preferred_keys")
            .unwrap();
        draft.reset(index);
        let mut next = next;
        draft.render_into(&mut next).unwrap();
        assert!(lookup(next.as_item(), "metadata.preferred_keys").is_none());
    }
    #[test]
    fn monitor_filters_project_legacy_without_losing_other_scopes() {
        let next =
            render("[channel_monitor]\nexclude = '*' # wildcard\n", |draft| {
                edit(draft, "channel_monitor.exclude.channels", "#quiet; #a.b");
            });
        let parsed: toml::Value = toml::from_str(&next).unwrap();
        assert_eq!(
            parsed["channel_monitor"]["exclude"]["users"].as_str(),
            Some("*")
        );
        assert_eq!(
            parsed["channel_monitor"]["exclude"]["servers"].as_str(),
            Some("*")
        );
        assert!(next.contains("#wildcard") || next.contains("# wildcard"));
        let source = "[channel_monitor.exclude]\nchannels = ['#old'] # scope\ncriteria = [{ channel = '#x', server = 'libera' }] # compound\n";
        let next = render(source, |draft| {
            edit(draft, "channel_monitor.exclude.channels", "#new");
        });
        assert!(next.contains(
            "criteria = [{ channel = '#x', server = 'libera' }] # compound"
        ));
        assert!(next.contains("# scope"));
    }
    #[test]
    fn alias_renames_and_swaps_preserve_comments_and_untargeted_entries() {
        let source = "[buffer.commands.aliases]\n# first alias\none = '/me first' # first\ntwo = '/me second' # second\nkeep = '/join #x' # keep\n";
        let next = render(source, |draft| {
            draft.aliases[0].name = "two".into();
            draft.aliases[1].name = "one".into();
        });
        assert!(next.contains("two = '/me first' # first"));
        assert!(next.contains("one = '/me second' # second"));
        assert!(next.contains("keep = '/join #x' # keep"));
        assert!(next.contains("# first alias\ntwo ="));
        assert!(next.find("two =").unwrap() < next.find("one =").unwrap());
        assert!(next.find("one =").unwrap() < next.find("keep =").unwrap());
    }
    #[test]
    fn alias_names_are_checked_after_runtime_normalization() {
        let mut draft = Draft::parse(&document(
            "[buffer.commands.aliases]\nhello = '/me hi'\n",
        ))
        .unwrap();
        draft.add_alias();
        draft.aliases[1].name = "/HELLO".into();
        draft.aliases[1].command = "/me again".into();
        let error = draft.render_into(&mut document("")).unwrap_err();
        assert_eq!(error.target, Target::AliasName(1));
    }
    #[test]
    fn invalid_fields_report_their_own_target() {
        for (path, input) in [
            ("display.truncation_character", "ab"),
            ("metadata.avatar.exclude", "["),
            ("metadata.preferred_keys", "unknown"),
            ("context_menu.padding.entry", "1,2,3"),
            ("runtime.vsync", "yes"),
            ("buffer.commands.exec.timeout", "-1"),
        ] {
            let mut draft = Draft::parse(&document("")).unwrap();
            edit(&mut draft, path, input);
            let error = draft.render_into(&mut document("")).unwrap_err();
            assert_eq!(
                error.target,
                Target::Field(
                    draft.fields.iter().position(|f| f.path == path).unwrap()
                ),
                "{path}"
            );
        }
    }
    #[test]
    fn explicitly_disabled_setting_is_kept_when_other_fields_change() {
        let next = render(
            "[metadata.avatar]\nenabled = false # disable\nsize = 96\nexclude = ['tracking']\n",
            |draft| edit(draft, "logs.max_file_count", "6"),
        );
        assert!(next.contains("enabled = false # disable"));
        assert!(next.contains("size = 96"));
        assert!(next.contains("exclude = ['tracking']"));
    }
    #[test]
    fn lists_allow_quoted_literal_semicolons() {
        let next = render("", |draft| {
            edit(draft, "metadata.avatar.exclude", "\"a;b\"; tracker");
        });
        let parsed: toml::Value = toml::from_str(&next).unwrap();
        assert_eq!(
            parsed["metadata"]["avatar"]["exclude"][0].as_str(),
            Some("a;b")
        );
    }
    #[test]
    fn removal_of_new_alias_restores_clean_draft() {
        let mut draft = Draft::parse(&document("")).unwrap();
        draft.add_alias();
        assert!(draft.is_dirty());
        draft.aliases[0].removed = true;
        assert!(!draft.is_dirty());
    }
}
