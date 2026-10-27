//! Read-only access to the public Halloy gallery. The gallery's SvelteKit data
//! response is a reference table; only names and color objects are decoded.
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use super::theme::{Styles, Theme};

const GALLERY_DATA: &str = "https://themes.halloy.chat/__data.json";
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: u64,
    pub name: String,
    pub styles: Styles,
}

impl Entry {
    pub fn is_light(&self) -> bool {
        self.styles.general.background.relative_luminance() > 0.5
    }

    pub fn file_stem(&self) -> Result<String, String> {
        let name = sanitize_filename::sanitize(self.name.trim());
        if name.is_empty() || name == "." || name == ".." {
            return Err("This theme has an invalid file name.".into());
        }
        Ok(name)
    }

    pub fn install(&self, directory: &Path) -> Result<Theme, String> {
        let name = self.file_stem()?;
        let path = directory.join(format!("{name}.toml"));
        let contents = toml::to_string(&self.styles)
            .map_err(|_| "This theme could not be converted to TOML.")?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)
            .map_err(|error| {
                format!("Could not create the theme file: {error}")
            })?;
        temporary
            .write_all(contents.as_bytes())
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| format!("Could not save the theme: {error}"))?;
        temporary.persist_noclobber(&path).map_err(|error| {
            if error.error.kind() == std::io::ErrorKind::AlreadyExists {
                "A theme file with this name already exists. Your file was kept.".into()
            } else { format!("Could not save the theme: {}", error.error) }
        })?;
        Ok(Theme::new(name, self.styles))
    }
}

pub async fn fetch() -> Result<Vec<Entry>, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(25))
        .user_agent(format!("Halloy/{}", crate::environment::VERSION.trim()))
        .build()
        .map_err(|error| {
            format!("Could not connect to the theme gallery: {error}")
        })?;
    let mut response = client.get(GALLERY_DATA).send().await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|_| "Could not load the theme gallery. Check your connection and try again.")?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "The theme gallery download was interrupted. Try again.")?
    {
        if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err("The theme gallery response is too large. Open the gallery website to browse themes.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    parse(&bytes)
}

pub fn installed(directory: &Path) -> Vec<Theme> {
    let mut themes = vec![Theme::default()];
    let Ok(files) = std::fs::read_dir(directory) else {
        return themes;
    };
    for entry in files.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "toml") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        // V1 themes contain a top-level name and use a different color schema.
        if toml::from_str::<toml::Value>(&source)
            .is_ok_and(|value| value.get("name").is_some())
        {
            continue;
        }
        if let Ok(styles) = toml::from_str::<Styles>(&source) {
            themes.retain(|theme| !theme.name.eq_ignore_ascii_case(name));
            themes.push(Theme::new(name.to_owned(), styles));
        }
    }
    themes.sort_by_key(|theme| theme.name.to_lowercase());
    themes
}

pub fn install_path(
    entry: &Entry,
    directory: &Path,
) -> Result<PathBuf, String> {
    Ok(directory.join(format!("{}.toml", entry.file_stem()?)))
}

fn parse(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let response: Value = serde_json::from_slice(bytes)
        .map_err(|_| "The theme gallery returned an unreadable response. Try again later.")?;
    let nodes = response.get("nodes").and_then(Value::as_array)
        .ok_or("The theme gallery format changed. Open the gallery website to browse themes.")?;
    let mut entries = vec![];
    let mut names = HashSet::new();
    for node in nodes {
        let Some(pool) = node.get("data").and_then(Value::as_array) else {
            continue;
        };
        let Some(themes) = pool
            .first()
            .and_then(|root| reference(pool, root.get("themes")?))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for theme in themes.iter().take(2000) {
            let Some(theme) = reference(pool, theme) else {
                continue;
            };
            let Some(id) = theme
                .get("id")
                .and_then(|id| reference(pool, id))
                .and_then(Value::as_u64)
            else {
                continue;
            };
            let Some(name) = theme
                .get("name")
                .and_then(|name| reference(pool, name))
                .and_then(Value::as_str)
            else {
                continue;
            };
            if name.trim().is_empty()
                || name.len() > 160
                || name.chars().any(char::is_control)
            {
                continue;
            }
            let Some(colors) = theme.get("colors") else {
                continue;
            };
            let mut budget = 2048;
            let Some(colors) = expand(pool, colors, 0, &mut budget) else {
                continue;
            };
            if !colors.is_object() {
                continue;
            }
            let Ok(styles) = serde_json::from_value::<Styles>(colors) else {
                continue;
            };
            if names.insert(name.to_lowercase()) {
                entries.push(Entry {
                    id,
                    name: name.to_owned(),
                    styles,
                });
            }
        }
    }
    if entries.is_empty() {
        return Err("No compatible themes were found. Try again later or open the gallery website.".into());
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(entries)
}

fn reference<'a>(pool: &'a [Value], index: &Value) -> Option<&'a Value> {
    pool.get(usize::try_from(index.as_u64()?).ok()?)
}

fn expand(
    pool: &[Value],
    index: &Value,
    depth: usize,
    budget: &mut usize,
) -> Option<Value> {
    if depth > 12 || *budget == 0 {
        return None;
    }
    *budget -= 1;
    match reference(pool, index)? {
        Value::Object(object) => {
            let mut result = serde_json::Map::new();
            for (key, value) in object {
                result.insert(
                    key.clone(),
                    expand(pool, value, depth + 1, budget)?,
                );
            }
            Some(Value::Object(result))
        }
        // Theme color/style data consists of objects and ASCII primitives.
        // Reject non-ASCII color strings before the legacy color parser slices them.
        Value::String(string) if string.is_ascii() && string.len() <= 128 => {
            Some(Value::String(string.clone()))
        }
        Value::Bool(boolean) => Some(Value::Bool(*boolean)),
        Value::Number(number) => Some(Value::Number(number.clone())),
        Value::Null => Some(Value::Null),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response() -> Value {
        serde_json::json!({"nodes":[{"data":[{"themes":1},[2],{"id":3,"name":4,"colors":5},42,"Test theme",{"general":6},{"background":7},"#ffffff"]}]})
    }

    #[test]
    fn reads_gallery_reference_tables() {
        let themes = parse(response().to_string().as_bytes()).unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "Test theme");
        assert!(themes[0].is_light());
    }

    #[test]
    fn rejects_cycles_invalid_references_and_unicode_color_strings() {
        let mut response = response();
        response["nodes"][0]["data"][7] = Value::String("☃abcd".into());
        assert!(parse(response.to_string().as_bytes()).is_err());
        response["nodes"][0]["data"][6]["background"] = Value::from(6);
        assert!(parse(response.to_string().as_bytes()).is_err());
        response["nodes"][0]["data"][6]["background"] = Value::from(9999);
        assert!(parse(response.to_string().as_bytes()).is_err());
    }

    #[test]
    fn installs_valid_toml_without_overwriting_existing_files() {
        let directory = tempfile::tempdir().unwrap();
        let entry = Entry {
            id: 1,
            name: "../Test/theme".into(),
            styles: Styles::default(),
        };
        let theme = entry.install(directory.path()).unwrap();
        let path = install_path(&entry, directory.path()).unwrap();
        assert_eq!(path.parent(), Some(directory.path()));
        assert_eq!(theme.name, entry.file_stem().unwrap());
        assert!(
            toml::from_str::<Styles>(&std::fs::read_to_string(&path).unwrap())
                .is_ok()
        );
        std::fs::write(&path, "# My custom theme").unwrap();
        assert!(
            entry
                .install(directory.path())
                .unwrap_err()
                .contains("already exists")
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), "# My custom theme");
    }
}
