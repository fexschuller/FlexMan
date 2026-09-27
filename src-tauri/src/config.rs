use crate::error::AppError;
use crate::fsutil;
use crate::paths;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut, Item, Table, TableLike};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, PartialEq)]
pub struct Config {
    pub backend: Option<String>,
    #[serde(rename = "bufferSizeSamples")]
    pub buffer_size_samples: Option<u32>,
    #[serde(default)]
    pub input: DeviceConfig,
    #[serde(default)]
    pub output: DeviceConfig,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    pub device: Option<String>,
    pub device_regex: Option<String>,
    pub channels: Option<u32>,
    pub sample_type: Option<String>,
    pub suggested_latency_seconds: Option<f64>,
    pub wasapi_exclusive_mode: Option<bool>,
    pub wasapi_auto_convert: Option<bool>,
    pub wasapi_explicit_sample_format: Option<bool>,
}

/// `None` means "leave/remove this key" when saving - a single convention
/// for every field instead of empty-string-for-strings, None-for-the-rest.
#[derive(serde::Deserialize, Debug, Default)]
pub struct ConfigUpdate {
    pub backend: Option<String>,
    #[serde(rename = "bufferSizeSamples")]
    pub buffer_size_samples: Option<u32>,
    pub input: DeviceUpdate,
    pub output: DeviceUpdate,
}

#[derive(serde::Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdate {
    pub device: Option<String>,
    pub channels: Option<u32>,
    pub sample_type: Option<String>,
    pub suggested_latency_seconds: Option<f64>,
    pub wasapi_exclusive_mode: Option<bool>,
    pub wasapi_auto_convert: Option<bool>,
    pub wasapi_explicit_sample_format: Option<bool>,
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct SyntaxError {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

pub fn path() -> Result<PathBuf, AppError> {
    Ok(paths::userprofile()?.join("FlexASIO.toml"))
}

pub fn load() -> Result<Config, AppError> {
    parse(&read_raw()?)
}

pub fn read_raw() -> Result<String, AppError> {
    read_raw_from(&path()?)
}

/// FlexASIO treats a missing file as "all defaults", so this does too -
/// callers never need to special-case "not installed yet".
fn read_raw_from(path: &Path) -> Result<String, AppError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(AppError::io(path, e)),
    }
}

/// Leaves every other key in the file untouched (round-trips via toml_edit)
/// and only rewrites keys whose value actually changed, so untouched keys
/// keep their original formatting/inline comments.
pub fn save(update: &ConfigUpdate) -> Result<(), AppError> {
    fsutil::write_atomic(&path()?, &merged_text(update)?)
}

/// Like `save`, but returns the merged text instead of writing it.
pub fn merged_text(update: &ConfigUpdate) -> Result<String, AppError> {
    merge(&read_raw()?, update)
}

pub fn write_raw(raw: &str) -> Result<(), AppError> {
    validate_raw(raw)?;
    fsutil::write_atomic(&path()?, raw)
}

pub fn find_syntax_error(raw: &str) -> Option<SyntaxError> {
    let err = raw.parse::<DocumentMut>().err()?;
    let span = err.span().unwrap_or(0..0);
    Some(SyntaxError {
        message: err.message().to_string(),
        start: span.start,
        end: span.end,
    })
}

fn parse(raw: &str) -> Result<Config, AppError> {
    toml_edit::de::from_str(raw).map_err(|e| AppError::InvalidConfig(e.to_string()))
}

fn validate_raw(raw: &str) -> Result<(), AppError> {
    raw.parse::<DocumentMut>()
        .map(|_| ())
        .map_err(|e| AppError::InvalidConfig(e.to_string()))
}

fn merge(raw: &str, update: &ConfigUpdate) -> Result<String, AppError> {
    let mut doc = raw
        .parse::<DocumentMut>()
        .map_err(|e| AppError::InvalidConfig(e.to_string()))?;

    set_or_remove_str(doc.as_table_mut(), "backend", update.backend.as_deref());
    set_or_remove_i64(
        doc.as_table_mut(),
        "bufferSizeSamples",
        update.buffer_size_samples.map(i64::from),
    );
    merge_device(&mut doc, "input", &update.input)?;
    merge_device(&mut doc, "output", &update.output)?;

    Ok(doc.to_string())
}

fn merge_device(doc: &mut DocumentMut, section: &str, update: &DeviceUpdate) -> Result<(), AppError> {
    let has_any = update.device.is_some()
        || update.channels.is_some()
        || update.sample_type.is_some()
        || update.suggested_latency_seconds.is_some()
        || update.wasapi_exclusive_mode.is_some()
        || update.wasapi_auto_convert.is_some()
        || update.wasapi_explicit_sample_format.is_some();
    if !has_any && !doc.contains_key(section) {
        // Nothing to set and no section to remove keys from - leave the
        // document untouched rather than creating an empty `[section]`.
        return Ok(());
    }

    let entry = doc.entry(section).or_insert_with(|| Item::Table(Table::new()));
    let table = entry
        .as_table_like_mut()
        .ok_or_else(|| AppError::InvalidConfig(format!("[{section}] is not a table in the config file")))?;

    set_or_remove_str(table, "device", update.device.as_deref());
    // The form always sends a concrete device, never a regex; a leftover
    // deviceRegex would otherwise conflict with it (FlexASIO refuses to
    // start if both are set).
    if update.device.is_some() {
        table.remove("deviceRegex");
    }
    set_or_remove_str(table, "sampleType", update.sample_type.as_deref());
    set_or_remove_i64(table, "channels", update.channels.map(i64::from));
    set_or_remove_f64(
        table,
        "suggestedLatencySeconds",
        update.suggested_latency_seconds,
    );
    set_or_remove_bool(table, "wasapiExclusiveMode", update.wasapi_exclusive_mode);
    set_or_remove_bool(table, "wasapiAutoConvert", update.wasapi_auto_convert);
    set_or_remove_bool(
        table,
        "wasapiExplicitSampleFormat",
        update.wasapi_explicit_sample_format,
    );

    Ok(())
}

fn set_or_remove_str(table: &mut dyn TableLike, key: &str, val: Option<&str>) {
    match val {
        Some(v) if table.get(key).and_then(Item::as_str) != Some(v) => {
            table.insert(key, value(v));
        }
        Some(_) => {}
        None => {
            table.remove(key);
        }
    }
}

fn set_or_remove_i64(table: &mut dyn TableLike, key: &str, val: Option<i64>) {
    match val {
        Some(v) if table.get(key).and_then(Item::as_integer) != Some(v) => {
            table.insert(key, value(v));
        }
        Some(_) => {}
        None => {
            table.remove(key);
        }
    }
}

fn set_or_remove_f64(table: &mut dyn TableLike, key: &str, val: Option<f64>) {
    match val {
        Some(v) if table.get(key).and_then(Item::as_float) != Some(v) => {
            table.insert(key, value(v));
        }
        Some(_) => {}
        None => {
            table.remove(key);
        }
    }
}

fn set_or_remove_bool(table: &mut dyn TableLike, key: &str, val: Option<bool>) {
    match val {
        Some(v) if table.get(key).and_then(Item::as_bool) != Some(v) => {
            table.insert(key, value(v));
        }
        Some(_) => {}
        None => {
            table.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_raw_from_missing_file_returns_empty_string() {
        let path = std::env::temp_dir().join("flexasio-config-test-does-not-exist.toml");

        assert_eq!(read_raw_from(&path).unwrap(), "");
    }

    #[test]
    fn read_raw_from_existing_file_returns_its_contents() {
        let dir = std::env::temp_dir().join(format!("flexasio-config-test-existing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("FlexASIO.toml");
        std::fs::write(&path, "backend = \"MME\"\n").unwrap();

        assert_eq!(read_raw_from(&path).unwrap(), "backend = \"MME\"\n");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parses_minimal_toml() {
        let raw = "backend = \"Windows WASAPI\"\n[input]\ndevice = \"Generic Input\"\n[output]\ndevice = \"Generic Output\"\n";

        let config = parse(raw).unwrap();

        assert_eq!(config.backend.as_deref(), Some("Windows WASAPI"));
        assert_eq!(config.input.device.as_deref(), Some("Generic Input"));
        assert_eq!(config.output.device.as_deref(), Some("Generic Output"));
    }

    #[test]
    fn parses_empty_toml_with_all_defaults() {
        let config = parse("").unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn returns_error_for_invalid_toml() {
        let result = parse("this is not valid toml { }");

        assert!(result.is_err());
    }

    fn sample_update() -> ConfigUpdate {
        ConfigUpdate {
            backend: Some("Windows WASAPI".to_string()),
            buffer_size_samples: Some(256),
            input: DeviceUpdate {
                device: Some("New Input".to_string()),
                channels: Some(4),
                sample_type: Some("Float32".to_string()),
                suggested_latency_seconds: Some(0.02),
                wasapi_exclusive_mode: Some(true),
                ..Default::default()
            },
            output: DeviceUpdate {
                device: Some("New Output".to_string()),
                ..Default::default()
            },
        }
    }

    #[test]
    fn merge_updates_device_fields_and_preserves_unrelated_keys() {
        let raw = "backend = \"MME\"\nsomeFutureOption = true\n[input]\ndevice = \"Old Input\"\nchannels = 2\n[output]\ndevice = \"Old Output\"\n";

        let updated = merge(raw, &sample_update()).unwrap();
        let reparsed = parse(&updated).unwrap();

        assert_eq!(reparsed.backend.as_deref(), Some("Windows WASAPI"));
        assert_eq!(reparsed.buffer_size_samples, Some(256));
        assert_eq!(reparsed.input.device.as_deref(), Some("New Input"));
        assert_eq!(reparsed.input.channels, Some(4));
        assert_eq!(reparsed.input.sample_type.as_deref(), Some("Float32"));
        assert_eq!(reparsed.input.suggested_latency_seconds, Some(0.02));
        assert_eq!(reparsed.input.wasapi_exclusive_mode, Some(true));
        assert_eq!(reparsed.output.device.as_deref(), Some("New Output"));
        assert!(updated.contains("someFutureOption = true"));
    }

    #[test]
    fn merge_removes_optional_fields_when_none() {
        let raw = "bufferSizeSamples = 512\n[input]\ndevice = \"Some Input\"\nchannels = 2\nsuggestedLatencySeconds = 0.01\nwasapiExclusiveMode = true\n";
        let update = ConfigUpdate {
            backend: Some("MME".to_string()),
            input: DeviceUpdate {
                device: Some("Some Input".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let updated = merge(raw, &update).unwrap();
        let reparsed = parse(&updated).unwrap();

        assert_eq!(reparsed.buffer_size_samples, None);
        assert_eq!(reparsed.input.channels, None);
        assert_eq!(reparsed.input.suggested_latency_seconds, None);
        assert_eq!(reparsed.input.wasapi_exclusive_mode, None);
    }

    #[test]
    fn merge_creates_missing_device_tables() {
        let raw = "backend = \"MME\"\n";
        let update = ConfigUpdate {
            backend: Some("MME".to_string()),
            input: DeviceUpdate {
                device: Some("New Input".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let updated = merge(raw, &update).unwrap();
        let reparsed = parse(&updated).unwrap();

        assert_eq!(reparsed.input.device.as_deref(), Some("New Input"));
    }

    #[test]
    fn merge_with_unchanged_values_leaves_text_byte_for_byte_identical() {
        let raw = "backend = \"MME\" # my favourite\nsomeFutureOption = true\n[input]\ndevice = \"Some Input\" # built-in mic\nchannels = 2\n";
        let update = ConfigUpdate {
            backend: Some("MME".to_string()),
            input: DeviceUpdate {
                device: Some("Some Input".to_string()),
                channels: Some(2),
                ..Default::default()
            },
            ..Default::default()
        };

        let updated = merge(raw, &update).unwrap();

        assert_eq!(updated, raw);
    }

    #[test]
    fn merge_setting_device_removes_conflicting_device_regex() {
        let raw = "[input]\ndeviceRegex = \"Foo.*\"\n";
        let update = ConfigUpdate {
            input: DeviceUpdate {
                device: Some("Concrete Device".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let updated = merge(raw, &update).unwrap();
        let reparsed = parse(&updated).unwrap();

        assert_eq!(reparsed.input.device.as_deref(), Some("Concrete Device"));
        assert_eq!(reparsed.input.device_regex, None);
    }

    #[test]
    fn merge_supports_inline_device_tables() {
        let raw = "input = { device = \"Old\" }\n";
        let update = ConfigUpdate {
            input: DeviceUpdate {
                device: Some("New".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let updated = merge(raw, &update).unwrap();
        let reparsed = parse(&updated).unwrap();

        assert_eq!(reparsed.input.device.as_deref(), Some("New"));
    }

    #[test]
    fn validate_raw_accepts_syntactically_valid_toml_with_unknown_keys() {
        assert!(validate_raw("backend = \"MME\"\nsomeFutureOption = true\n").is_ok());
    }

    #[test]
    fn validate_raw_rejects_broken_toml() {
        assert!(validate_raw("this is not valid toml { }").is_err());
    }

    #[test]
    fn find_syntax_error_returns_none_for_valid_toml() {
        assert!(find_syntax_error("backend = \"MME\"\n").is_none());
    }

    #[test]
    fn find_syntax_error_locates_broken_toml() {
        let raw = "backend = \"MME\"\nchannels = ]\n";

        let error = find_syntax_error(raw).unwrap();

        assert!(!error.message.is_empty());
        assert!(error.start <= error.end);
        assert!(error.end <= raw.len());
    }
}
