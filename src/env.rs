use crate::nesting;
use serde_json::Value;
use std::collections::HashMap;

/// Reads environment variables with the given prefix into a [`serde_json::Value`].
///
/// Only variables whose name begins with `{PREFIX}_` are collected. The prefix
/// and its separating `_` are stripped; the remainder is lowercased. A double
/// underscore `__` in the remainder signals a nesting level and is replaced
/// with `.`. Values are parsed as JSON where possible; otherwise kept as strings.
///
/// | Environment variable | Config path  | Notes                      |
/// |----------------------|--------------|----------------------------|
/// | `APP_HOST`           | `host`       | top-level field             |
/// | `APP_MAX_RETRIES`    | `max_retries`| single `_` is part of key  |
/// | `APP_DB__HOST`       | `db.host`    | `__` signals nesting       |
/// | `APP_PORT`           | `port`       | parsed as JSON number      |
/// | `APP_DEBUG`          | `debug`      | parsed as JSON bool        |
///
/// The returned value is intended to be passed to [`Configurable::update`][crate::Configurable::update].
///
/// # Example
///
/// ```rust,no_run
/// use gifnoc::{config, Configurable};
///
/// config! {
///     DbConfig {
///         host: String = "localhost",
///         port: u32 = 5432u32,
///     }
/// }
///
/// // With APP_DB__HOST=remotehost APP_DB__PORT=9999 in the environment:
/// let config = DbConfig::default().update(gifnoc::env::with_prefix("APP_DB"));
/// // config.host == "remotehost", config.port == 9999
/// ```
pub fn with_prefix(prefix: &str) -> Value {
    from_vars(prefix, std::env::vars())
}

fn from_vars(prefix: &str, vars: impl Iterator<Item = (String, String)>) -> Value {
    let needle = format!("{prefix}_");
    let mut flat: HashMap<String, Value> = HashMap::new();
    for (key, value) in vars {
        if let Some(rest) = key.strip_prefix(&needle) {
            let path = rest.replace("__", ".").to_lowercase();
            let parsed = serde_json::from_str(&value).unwrap_or_else(|_| Value::String(value));
            flat.insert(path, parsed);
        }
    }
    nesting::nest(flat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vars(pairs: &[(&str, &str)]) -> impl Iterator<Item = (String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn non_matching_prefix_ignored() {
        let result = from_vars("APP", vars(&[("OTHER_FOO", "bar")]));
        assert_eq!(result, json!({}));
    }

    #[test]
    fn matching_prefix_picked_up() {
        let result = from_vars("APP", vars(&[("APP_FOO", "bar")]));
        assert_eq!(result["foo"], json!("bar"));
    }

    #[test]
    fn key_is_lowercased() {
        let result = from_vars("APP", vars(&[("APP_FOO_BAR", "x")]));
        assert_eq!(result["foo_bar"], json!("x"));
    }

    #[test]
    fn double_underscore_becomes_nesting() {
        let result = from_vars("APP", vars(&[("APP_DATABASE__HOST", "localhost")]));
        assert_eq!(result["database"]["host"], json!("localhost"));
    }

    #[test]
    fn json_number_parsed() {
        let result = from_vars("APP", vars(&[("APP_PORT", "8080")]));
        assert_eq!(result["port"], json!(8080));
    }

    #[test]
    fn json_bool_parsed() {
        let result = from_vars("APP", vars(&[("APP_DEBUG", "true")]));
        assert_eq!(result["debug"], json!(true));
    }

    #[test]
    fn non_json_kept_as_string() {
        let result = from_vars("APP", vars(&[("APP_NAME", "my app")]));
        assert_eq!(result["name"], json!("my app"));
    }

    #[test]
    fn multiple_vars() {
        let result = from_vars(
            "APP",
            vars(&[
                ("APP_HOST", "localhost"),
                ("APP_PORT", "9000"),
                ("OTHER_X", "ignored"),
            ]),
        );
        assert_eq!(result["host"], json!("localhost"));
        assert_eq!(result["port"], json!(9000));
        assert_eq!(result.get("x"), None);
    }
}
