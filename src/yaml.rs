use std::path::Path;
use serde_json::Value;

/// Reads a YAML file and returns its contents as a [`serde_json::Value`].
///
/// The returned value is intended to be passed to [`Configurable::update`][crate::Configurable::update].
/// Panics if the file cannot be read or contains invalid YAML.
///
/// Requires the `yaml` feature:
/// ```toml
/// [dependencies]
/// gifnoc = { version = "...", features = ["yaml"] }
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use gifnoc::{config, Configurable};
///
/// config! {
///     AppConfig {
///         host: String = "localhost",
///         port: u32 = 8080u32,
///     }
/// }
///
/// let config = AppConfig::default().update(gifnoc::yaml::from_file("config.yaml"));
/// ```
pub fn from_file(path: impl AsRef<Path>) -> Value {
    let content = std::fs::read_to_string(path).unwrap();
    serde_yml::from_str(&content).unwrap()
}
