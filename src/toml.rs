use serde_json::Value;
use std::path::Path;

/// Reads a TOML file and returns its contents as a [`serde_json::Value`].
///
/// The returned value is intended to be passed to [`Configurable::update`][crate::Configurable::update].
/// Panics if the file cannot be read or contains invalid TOML.
///
/// Requires the `toml` feature:
/// ```toml
/// [dependencies]
/// gifnoc = { version = "...", features = ["toml"] }
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
/// let config = AppConfig::default().update(gifnoc::toml::from_file("config.toml"));
/// ```
pub fn from_file(path: impl AsRef<Path>) -> Value {
    let content = std::fs::read_to_string(path).unwrap();
    ::toml::from_str(&content).unwrap()
}
