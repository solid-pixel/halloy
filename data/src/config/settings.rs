//! Editable settings backed by the original document, rather than the runtime
//! config (which contains defaults and resolved credentials).
pub mod preferences;

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use toml_edit::{DocumentMut, Item, Table, TableLike, Value, value};

use super::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authentication {
    None,
    Sasl,
    NickServ,
    Advanced,
}

impl std::fmt::Display for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "None",
            Self::Sasl => "SASL PLAIN",
            Self::NickServ => "NickServ",
            Self::Advanced => "Configured in file",
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Network {
    pub original_name: Option<String>,
    pub name: String,
    pub server: String,
    pub port: String,
    pub nickname: String,
    pub channels: String,
    pub use_tls: bool,
    pub autoconnect: bool,
    pub authentication: Authentication,
    pub username: String,
    pub password: String,
}

impl Network {
    fn from_table(name: &str, table: &dyn TableLike) -> Self {
        let string = |key| string_at(table.get(key));
        let sasl = table.get("sasl").and_then(Item::as_table_like);
        let plain = sasl
            .and_then(|sasl| sasl.get("plain"))
            .and_then(Item::as_table_like);
        let external_secret = |table: &dyn TableLike, prefix: &str| {
            ["file", "command", "keyring"]
                .iter()
                .any(|suffix| table.contains_key(&format!("{prefix}_{suffix}")))
        };
        let advanced = table.contains_key("sasl") && plain.is_none()
            || external_secret(table, "nick_password")
            || plain.is_some_and(|plain| external_secret(plain, "password"))
            || table.contains_key("sasl")
                && table.contains_key("nick_password");
        let authentication = if advanced {
            Authentication::Advanced
        } else if plain.is_some() {
            Authentication::Sasl
        } else if table.contains_key("nick_password") {
            Authentication::NickServ
        } else {
            Authentication::None
        };
        Self {
            original_name: Some(name.to_owned()),
            name: name.to_owned(),
            server: string("server"),
            port: table
                .get("port")
                .and_then(Item::as_integer)
                .map(|port| port.to_string())
                .unwrap_or_default(),
            nickname: table
                .get("nickname")
                .or_else(|| table.get("nick"))
                .and_then(Item::as_str)
                .unwrap_or_default()
                .to_owned(),
            channels: table
                .get("channels")
                .and_then(Item::as_array)
                .map(|channels| {
                    channels
                        .iter()
                        .filter_map(channel_name)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default(),
            use_tls: table
                .get("use_tls")
                .and_then(Item::as_bool)
                .unwrap_or(true),
            autoconnect: table
                .get("autoconnect")
                .and_then(Item::as_bool)
                .unwrap_or(true),
            authentication,
            username: plain
                .map(|plain| string_at(plain.get("username")))
                .unwrap_or_default(),
            password: if authentication == Authentication::Sasl {
                plain
                    .map(|plain| string_at(plain.get("password")))
                    .unwrap_or_default()
            } else if authentication == Authentication::NickServ {
                string("nick_password")
            } else {
                String::new()
            },
        }
    }

    pub fn new(name: String) -> Self {
        Self {
            original_name: None,
            name,
            server: String::new(),
            port: String::new(),
            nickname: String::new(),
            channels: String::new(),
            use_tls: true,
            autoconnect: true,
            authentication: Authentication::None,
            username: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Single,
    System,
    Configured,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Single => "One theme",
            Self::System => "Follow system appearance",
            Self::Configured => "Configured in file",
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Appearance {
    pub mode: ThemeMode,
    pub theme: String,
    pub light: String,
    pub dark: String,
    pub font_family: String,
    pub font_size: String,
    pub line_height: String,
    pub scale_factor: String,
    pub sidebar_position: String,
}

#[derive(Clone)]
pub struct Settings {
    source: String,
    original_networks: Vec<Network>,
    original_appearance: Appearance,
    pub networks: Vec<Network>,
    pub appearance: Appearance,
    pub preferences: preferences::Preferences,
}

impl Settings {
    pub fn load() -> Result<Self, String> {
        let source = std::fs::read_to_string(Config::path())
            .map_err(|error| error.to_string())?;
        Self::parse(source)
    }

    pub fn parse(source: String) -> Result<Self, String> {
        Config::validate(&source).map_err(|error| error.to_string())?;
        let document = source
            .parse::<DocumentMut>()
            .map_err(|error| error.to_string())?;
        let networks = document
            .get("servers")
            .and_then(Item::as_table_like)
            .map(|servers| {
                servers
                    .iter()
                    .filter_map(|(name, item)| {
                        item.as_table_like()
                            .map(|table| Network::from_table(name, table))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let font = document.get("font").and_then(Item::as_table_like);
        let theme = document.get("theme");
        let dynamic = theme.and_then(Item::as_table_like);
        let mode = match theme {
            None => ThemeMode::Single,
            Some(theme) if theme.is_str() => ThemeMode::Single,
            _ if dynamic.is_some_and(|table| {
                table.get("light").is_some_and(Item::is_str)
                    && table.get("dark").is_some_and(Item::is_str)
            }) =>
            {
                ThemeMode::System
            }
            _ => ThemeMode::Configured,
        };
        let appearance = Appearance {
            mode,
            theme: theme.and_then(Item::as_str).unwrap_or("ferra").to_owned(),
            light: dynamic
                .map(|table| string_at(table.get("light")))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "ferra".into()),
            dark: dynamic
                .map(|table| string_at(table.get("dark")))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "ferra".into()),
            font_family: font
                .map(|font| string_at(font.get("family")))
                .unwrap_or_default(),
            font_size: font
                .and_then(|font| font.get("size"))
                .and_then(Item::as_integer)
                .map(|size| size.to_string())
                .unwrap_or_default(),
            line_height: font
                .and_then(|font| font.get("line_height"))
                .map(number_text)
                .unwrap_or_default(),
            scale_factor: document
                .get("scale_factor")
                .map_or_else(|| "1".into(), number_text),
            sidebar_position: document
                .get("sidebar")
                .and_then(Item::as_table_like)
                .and_then(|sidebar| sidebar.get("position"))
                .and_then(Item::as_str)
                .unwrap_or("left")
                .to_owned(),
        };
        let preferences = preferences::Preferences::parse(&document)?;
        Ok(Self {
            source,
            original_networks: networks.clone(),
            original_appearance: appearance.clone(),
            networks,
            appearance,
            preferences,
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.networks != self.original_networks
            || self.appearance != self.original_appearance
            || self.preferences.is_dirty()
    }

    pub fn font_changed(&self) -> bool {
        let a = &self.appearance;
        let b = &self.original_appearance;
        a.font_family != b.font_family
            || a.font_size != b.font_size
            || a.line_height != b.line_height
    }

    pub fn networks_changed(&self) -> bool {
        self.networks != self.original_networks
    }

    pub fn has_configured_theme(&self) -> bool {
        self.original_appearance.mode == ThemeMode::Configured
    }

    pub fn has_advanced_authentication(&self, network: &Network) -> bool {
        self.original_networks.iter().any(|original| {
            Some(&original.name) == network.original_name.as_ref()
                && original.authentication == Authentication::Advanced
        })
    }

    pub fn render(&self) -> Result<String, String> {
        let mut names = HashSet::new();
        for network in &self.networks {
            if network.name.trim().is_empty()
                || network.name.contains(['\n', '\r', '\0'])
            {
                return Err("Give every network a name.".into());
            }
            if !names.insert(&network.name) {
                return Err(format!(
                    "The network name ‘{}’ is already in use.",
                    network.name
                ));
            }
            // Existing unusual configuration can still be managed in the text editor.
            if self.original_networks.iter().any(|old| old == network) {
                continue;
            }
            if network.server.trim().is_empty()
                || network.server.chars().any(char::is_whitespace)
                || network.server.contains("://")
            {
                return Err(format!(
                    "{}: enter a server hostname, such as irc.libera.chat.",
                    network.name
                ));
            }
            if network.nickname.is_empty()
                || network.nickname.chars().any(char::is_whitespace)
                || network.nickname.contains([',', ':', '\0'])
            {
                return Err(format!(
                    "{}: enter a nickname without spaces, commas or colons.",
                    network.name
                ));
            }
            if !network.port.is_empty()
                && network
                    .port
                    .parse::<u16>()
                    .ok()
                    .is_none_or(|port| port == 0)
            {
                return Err(format!(
                    "{}: the port must be between 1 and 65535, or empty for automatic.",
                    network.name
                ));
            }
            for channel in channels(&network.channels) {
                if !channel.starts_with(['#', '&', '+', '!'])
                    || channel.contains(['\0', '\u{7}', ':'])
                {
                    return Err(format!(
                        "{}: channels must start with #, &, + or !.",
                        network.name
                    ));
                }
            }
            if matches!(
                network.authentication,
                Authentication::Sasl | Authentication::NickServ
            ) && network.password.is_empty()
            {
                return Err(format!(
                    "{}: enter an authentication password or choose None.",
                    network.name
                ));
            }
        }
        let mut document = self
            .source
            .parse::<DocumentMut>()
            .map_err(|error| error.to_string())?;
        if self.networks_changed() {
            let original_document = document.clone();
            let servers = table(&mut document, "servers")?;
            for old in &self.original_networks {
                servers.remove(&old.name);
            }
            for network in &self.networks {
                let old = self.original_networks.iter().find(|old| {
                    Some(&old.name) == network.original_name.as_ref()
                });
                let mut item = network
                    .original_name
                    .as_ref()
                    .and_then(|name| {
                        original_document
                            .get("servers")?
                            .as_table_like()?
                            .get(name)
                    })
                    .cloned()
                    .unwrap_or_else(|| Item::Table(Table::new()));
                let target = item
                    .as_table_like_mut()
                    .ok_or("Network configuration must be a table.")?;
                macro_rules! update {
                    ($field:ident, $value:expr) => {
                        if old.is_none_or(|old| old.$field != network.$field) {
                            set(target, stringify!($field), $value);
                        }
                    };
                }
                update!(server, value(&network.server));
                if old.is_none_or(|old| old.nickname != network.nickname) {
                    target.remove("nick");
                    set(target, "nickname", value(&network.nickname));
                }
                update!(use_tls, value(network.use_tls));
                update!(autoconnect, value(network.autoconnect));
                if old.is_none_or(|old| old.port != network.port) {
                    if network.port.is_empty() {
                        target.remove("port");
                    } else {
                        set(
                            target,
                            "port",
                            value(i64::from(
                                network
                                    .port
                                    .parse::<u16>()
                                    .map_err(|_| "Invalid port")?,
                            )),
                        );
                    }
                }
                if old.is_none_or(|old| old.channels != network.channels) {
                    let previous = target
                        .get("channels")
                        .and_then(Item::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut array = toml_edit::Array::new();
                    for name in channels(&network.channels) {
                        array.push(
                            previous
                                .iter()
                                .find(|value| channel_name(value) == Some(name))
                                .cloned()
                                .unwrap_or_else(|| Value::from(name)),
                        );
                    }
                    set(target, "channels", value(array));
                }
                let auth_changed = old.is_none_or(|old| {
                    old.authentication != network.authentication
                });
                if auth_changed {
                    for key in [
                        "sasl",
                        "nick_password",
                        "nick_password_file",
                        "nick_password_command",
                        "nick_password_keyring",
                    ] {
                        target.remove(key);
                    }
                }
                match network.authentication {
                    Authentication::Sasl => {
                        if auth_changed
                            || old.is_none_or(|old| {
                                old.username != network.username
                                    || old.password != network.password
                            })
                        {
                            let sasl = child_table(target, "sasl")?;
                            let plain = child_table(sasl, "plain")?;
                            if network.username.is_empty() {
                                plain.remove("username");
                            } else {
                                set(
                                    plain,
                                    "username",
                                    value(&network.username),
                                );
                            }
                            set(plain, "password", value(&network.password));
                        }
                    }
                    Authentication::NickServ => {
                        if auth_changed
                            || old.is_none_or(|old| {
                                old.password != network.password
                            })
                        {
                            set(
                                target,
                                "nick_password",
                                value(&network.password),
                            );
                        }
                    }
                    Authentication::None | Authentication::Advanced => {}
                }
                servers.insert(&network.name, item);
            }
        }
        let a = &self.appearance;
        let b = &self.original_appearance;
        if a.mode != b.mode
            || a.theme != b.theme
            || a.light != b.light
            || a.dark != b.dark
        {
            match a.mode {
                ThemeMode::Single => {
                    set(document.as_table_mut(), "theme", value(&a.theme));
                }
                ThemeMode::System => {
                    let mut dynamic = toml_edit::InlineTable::new();
                    dynamic.insert("light", Value::from(a.light.as_str()));
                    dynamic.insert("dark", Value::from(a.dark.as_str()));
                    set(document.as_table_mut(), "theme", value(dynamic));
                }
                ThemeMode::Configured => {}
            }
        }
        if a.font_family != b.font_family {
            optional_string(
                table(&mut document, "font")?,
                "family",
                &a.font_family,
            );
        }
        if a.font_size != b.font_size {
            let font = table(&mut document, "font")?;
            if a.font_size.is_empty() {
                font.remove("size");
            } else {
                let size = a.font_size.parse::<u8>().ok().filter(|size| *size > 0).ok_or("Font size must be between 1 and 255, or empty for the default.")?;
                set(font, "size", value(i64::from(size)));
            }
        }
        if a.line_height != b.line_height {
            let font = table(&mut document, "font")?;
            if a.line_height.is_empty() {
                font.remove("line_height");
            } else {
                set(
                    font,
                    "line_height",
                    value(positive_float(&a.line_height, "Line height")?),
                );
            }
        }
        if a.scale_factor != b.scale_factor {
            let scale = positive_float(&a.scale_factor, "Interface scale")?;
            if !(0.1..=3.0).contains(&scale) {
                return Err("Interface scale must be between 0.1 and 3.".into());
            }
            set(document.as_table_mut(), "scale_factor", value(scale));
        }
        if a.sidebar_position != b.sidebar_position {
            set(
                table(&mut document, "sidebar")?,
                "position",
                value(&a.sidebar_position),
            );
        }
        self.preferences.render_into(&mut document)?;
        let contents = document.to_string();
        Config::validate(&contents).map_err(|error| error.to_string())?;
        Ok(contents)
    }

    /// Compare before replacing, use a same-directory atomic rename, and keep
    /// the previous configuration available for recovery. No credentials resolve.
    pub fn save_to(&self, path: &Path) -> Result<String, String> {
        let contents = self.render()?;
        let resolved_path =
            path.canonicalize().map_err(|error| error.to_string())?;
        let path = resolved_path.as_path();
        let current =
            std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        if current != self.source {
            return Err("The config file changed outside Settings. Reopen Settings to load the latest version before saving.".into());
        }
        let parent = path
            .parent()
            .ok_or("The config file has no parent directory.")?;
        let write_temp =
            |text: &str| -> Result<tempfile::NamedTempFile, String> {
                let mut temporary = tempfile::NamedTempFile::new_in(parent)
                    .map_err(|error| error.to_string())?;
                temporary
                    .write_all(text.as_bytes())
                    .and_then(|()| temporary.as_file().sync_all())
                    .map_err(|error| error.to_string())?;
                Ok(temporary)
            };
        let temporary = write_temp(&contents)?;
        let backup = write_temp(&current)?;
        backup
            .persist(path.with_extension("toml.settings-backup"))
            .map_err(|error| error.to_string())?;
        if std::fs::read_to_string(path).map_err(|error| error.to_string())?
            != self.source
        {
            return Err("The config file changed while saving. Reopen Settings and try again.".into());
        }
        temporary.persist(path).map_err(|error| error.to_string())?;
        Ok(contents)
    }
}

fn number_text(item: &Item) -> String {
    item.as_float()
        .map(|value| value.to_string())
        .or_else(|| item.as_integer().map(|value| value.to_string()))
        .unwrap_or_default()
}

fn positive_float(input: &str, label: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| format!("{label} must be a positive number."))
}

fn channels(input: &str) -> impl Iterator<Item = &str> {
    input
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
}

fn channel_name(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.as_inline_table()?.get("name")?.as_str())
}

fn string_at(item: Option<&Item>) -> String {
    item.and_then(Item::as_str).unwrap_or_default().to_owned()
}

fn table<'a>(
    document: &'a mut DocumentMut,
    key: &str,
) -> Result<&'a mut dyn TableLike, String> {
    child_table(document.as_table_mut(), key)
}

fn child_table<'a>(
    parent: &'a mut dyn TableLike,
    key: &str,
) -> Result<&'a mut dyn TableLike, String> {
    if !parent.contains_key(key) {
        parent.insert(key, Item::Table(Table::new()));
    }
    parent
        .get_mut(key)
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| {
            format!("{key} must be a table; edit it in the config file first.")
        })
}

fn optional_string(table: &mut dyn TableLike, key: &str, input: &str) {
    if input.is_empty() {
        table.remove(key);
    } else {
        set(table, key, value(input));
    }
}

fn set(table: &mut dyn TableLike, key: &str, mut item: Item) {
    if let Some(previous) = table.get(key).and_then(Item::as_value)
        && let Some(next) = item.as_value_mut()
    {
        *next.decor_mut() = previous.decor().clone();
    }
    table.insert(key, item);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "# My settings\ntheme = [\"ferra\", \"other\"] # randomized\n[font]\nsize = 13 # keep this comment\n[servers.libera]\nserver = \"irc.libera.chat\"\nnickname = \"hello\"\nchannels = [{ name = \"#halloy\", mute = true }, \"#rust\"]\npassword_command = \"get-secret\"\n[servers.libera.sasl.external]\ncert = \"/tmp/cert.pem\"\n[servers.libera.filters]\nignore = []\n";

    #[test]
    fn unchanged_is_byte_for_byte_identical() {
        let settings = Settings::parse(SOURCE.into()).unwrap();
        assert!(!settings.is_dirty());
        assert_eq!(settings.render().unwrap(), SOURCE);
    }

    #[test]
    fn appearance_change_preserves_advanced_settings_and_comments() {
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.appearance.font_size = "16".into();
        let result = settings.render().unwrap();
        assert!(result.contains("size = 16 # keep this comment"));
        assert!(result.contains("theme = [\"ferra\", \"other\"] # randomized"));
        assert!(result.contains("password_command = \"get-secret\""));
        assert!(result.contains("[servers.libera.sasl.external]"));
    }

    #[test]
    fn rename_and_channel_edit_preserve_nested_options_and_mute() {
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.networks[0].name = "work.network".into();
        settings.networks[0].channels = "#halloy, #new".into();
        let result = settings.render().unwrap();
        let doc = result.parse::<DocumentMut>().unwrap();
        assert!(doc["servers"].get("libera").is_none());
        assert_eq!(
            doc["servers"]["work.network"]["sasl"]["external"]["cert"].as_str(),
            Some("/tmp/cert.pem")
        );
        assert!(result.contains("mute = true"));
        assert!(!result.contains("#rust"));
    }

    #[test]
    fn validates_duplicates_ports_and_missing_authentication() {
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.networks.push(settings.networks[0].clone());
        assert!(settings.render().unwrap_err().contains("already in use"));
        settings.networks.pop();
        settings.networks[0].port = "65536".into();
        assert!(settings.render().unwrap_err().contains("port"));
        settings.networks[0].port.clear();
        settings.networks[0].authentication = Authentication::Sasl;
        assert!(settings.render().unwrap_err().contains("password"));
    }

    #[test]
    fn inline_tables_and_nick_alias_are_editable() {
        let mut settings = Settings::parse("font = { size = 13 }\nservers = { test = { server = 'localhost', nick = 'user' } }\n".into()).unwrap();
        settings.appearance.font_size = "15".into();
        settings.networks[0].nickname = "other".into();
        let result = settings.render().unwrap();
        Config::validate(&result).unwrap();
        let doc = result.parse::<DocumentMut>().unwrap();
        assert_eq!(doc["servers"]["test"]["nickname"].as_str(), Some("other"));
        assert!(doc["servers"]["test"].get("nick").is_none());
    }

    #[test]
    fn add_and_remove_networks_in_an_inline_document() {
        let mut settings = Settings::parse(
            "servers = { old = { server = 'localhost', nickname = 'user' } }\n"
                .into(),
        )
        .unwrap();
        let mut network = Network::new("new.network".into());
        network.server = "irc.example.com".into();
        network.nickname = "new_user".into();
        network.autoconnect = false;
        network.authentication = Authentication::Sasl;
        network.password = "password with \"quotes\"".into();
        settings.networks = vec![network];
        let result = settings.render().unwrap();
        let doc = result.parse::<DocumentMut>().unwrap();
        assert!(doc["servers"].get("old").is_none());
        assert_eq!(
            doc["servers"]["new.network"]["sasl"]["plain"]["password"].as_str(),
            Some("password with \"quotes\"")
        );
        assert_eq!(
            doc["servers"]["new.network"]["autoconnect"].as_bool(),
            Some(false)
        );
        Config::validate(&result).unwrap();
    }

    #[test]
    fn numeric_values_and_dynamic_themes_are_preserved() {
        let source = "scale_factor = 2\ntheme = { light = 'light', dark = 'dark' }\nfont.line_height = 2\n";
        let mut settings = Settings::parse(source.into()).unwrap();
        assert_eq!(settings.appearance.scale_factor, "2");
        assert_eq!(settings.appearance.line_height, "2");
        assert_eq!(settings.appearance.mode, ThemeMode::System);
        assert_eq!(settings.render().unwrap(), source);
        settings.appearance.dark = "ferra".into();
        assert!(settings.render().unwrap().contains("ferra"));
    }

    #[cfg(unix)]
    #[test]
    fn saving_a_symlink_updates_its_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("dotfiles.toml");
        let link = directory.path().join("config.toml");
        std::fs::write(&target, SOURCE).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.appearance.font_size = "15".into();
        let written = settings.save_to(&link).unwrap();
        assert!(link.is_symlink());
        assert_eq!(std::fs::read_to_string(target).unwrap(), written);
    }

    #[test]
    fn switching_authentication_clears_conflicting_credentials() {
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.networks[0].authentication = Authentication::Sasl;
        settings.networks[0].password = "example".into();
        let result = settings.render().unwrap();
        assert!(!result.contains("sasl.external"));
        assert!(result.contains("[servers.libera.sasl.plain]"));
        assert!(result.contains("password_command = \"get-secret\""));
    }

    #[test]
    fn save_keeps_backup_and_refuses_external_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        std::fs::write(&path, SOURCE).unwrap();
        let mut settings = Settings::parse(SOURCE.into()).unwrap();
        settings.appearance.font_size = "15".into();
        let written = settings.save_to(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), written);
        assert_eq!(
            std::fs::read_to_string(
                path.with_extension("toml.settings-backup")
            )
            .unwrap(),
            SOURCE
        );
        assert!(
            settings
                .save_to(&path)
                .unwrap_err()
                .contains("changed outside")
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), written);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
