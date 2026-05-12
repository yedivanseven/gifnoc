use crate::nesting;
use serde_json::Value;
use std::path::Path;

/// Reads a JSON file and returns its contents as a [`serde_json::Value`].
///
/// The returned value is intended to be passed to [`Configurable::update`][crate::Configurable::update].
/// Panics if the file cannot be read or contains invalid JSON. Dot-separated keys are parsed into
/// hierarchically nested JSONs, so a file written by [`Configurable::to_json`][crate::Configurable::to_json]
/// round-trips losslessly.
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
/// let config = AppConfig::default().update(gifnoc::json::from_file("config.json"));
/// ```
pub fn from_file(path: impl AsRef<Path>) -> Value {
    let content = std::fs::read_to_string(path).unwrap();
    let parsed: Value = ::serde_json::from_str(&content).unwrap();
    let flat = nesting::flatten(parsed);
    nesting::nest(flat)
}
