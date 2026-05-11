use crate::nesting;
use serde_json::Value;
use std::collections::HashMap;

/// Parses command-line arguments into positional actions and config flag overrides.
///
/// Returns `(Vec<String>, serde_json::Value)`:
///
/// - **Positionals**: bare words before the first `--` flag, returned as-is.
///   These are typically the names of steps or commands for the app to run.
/// - **Flags**: long-form only. Both `--key value` and `--key=value` are
///   accepted. Dot-separated keys map to nested config fields (`--db.port 9999`).
///   Values are parsed as JSON where possible; otherwise kept as strings.
///   Boolean flags require an explicit value (`--debug true`).
///
/// Short-form flags (`-x`) and positional arguments appearing after flags are
/// rejected with an informative error message and a non-zero exit code.
///
/// The `Value` half is intended to be passed to [`Configurable::update`][crate::Configurable::update].
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
/// // Called as: ./myapp serve ingest --host myserver --port=9000
/// let (actions, flags) = gifnoc::args::parse();
/// let config = AppConfig::default().update(flags);
///
/// // actions == ["serve", "ingest"]
/// // config.host == "myserver", config.port == 9000
///
/// for action in &actions {
///     match action.as_str() {
///         "serve"  => { /* ... */ }
///         "ingest" => { /* ... */ }
///         other    => { eprintln!("unknown action: {other}"); std::process::exit(1); }
///     }
/// }
/// ```
pub fn parse() -> (Vec<String>, Value) {
    parse_from(std::env::args().skip(1)).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    })
}

fn parse_from(args: impl Iterator<Item = String>) -> Result<(Vec<String>, Value), String> {
    let args: Vec<String> = args.collect();
    let mut positionals = Vec::new();
    let mut flat: HashMap<String, Value> = HashMap::new();

    let mut i = 0;

    // collect positionals — everything before the first '--' arg
    while i < args.len() && !args[i].starts_with("--") {
        if args[i].starts_with('-') {
            return Err(format!(
                "error: short-form flags are not supported: '{}'; \
                 use --key value or --key=value",
                &args[i]
            ));
        }
        positionals.push(args[i].clone());
        i += 1;
    }

    // parse flags
    while i < args.len() {
        if let Some(rest) = args[i].strip_prefix("--") {
            if let Some((key, value)) = rest.split_once('=') {
                let parsed =
                    serde_json::from_str(value).unwrap_or(Value::String(value.to_string()));
                flat.insert(key.to_string(), parsed);
                i += 1;
            } else {
                let key = rest.to_string();
                i += 1;
                if i >= args.len() || args[i].starts_with('-') {
                    return Err(format!(
                        "error: flag '--{key}' requires a value \
                         (use --{key}=<value> or --{key} <value>)"
                    ));
                }
                let parsed =
                    serde_json::from_str(&args[i]).unwrap_or(Value::String(args[i].clone()));
                flat.insert(key, parsed);
                i += 1;
            }
        } else {
            return Err(format!(
                "error: positional argument '{}' must come before all flags",
                &args[i]
            ));
        }
    }

    Ok((positionals, nesting::nest(flat)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(a: &[&str]) -> impl Iterator<Item = String> {
        a.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn empty_args() {
        let (positionals, flags) = parse_from(args(&[])).unwrap();
        assert!(positionals.is_empty());
        assert_eq!(flags, json!({}));
    }

    #[test]
    fn only_positionals() {
        let (positionals, flags) = parse_from(args(&["step1", "step2"])).unwrap();
        assert_eq!(positionals, vec!["step1", "step2"]);
        assert_eq!(flags, json!({}));
    }

    #[test]
    fn flag_equals_form() {
        let (_, flags) = parse_from(args(&["--host=localhost"])).unwrap();
        assert_eq!(flags["host"], json!("localhost"));
    }

    #[test]
    fn flag_space_form() {
        let (_, flags) = parse_from(args(&["--host", "localhost"])).unwrap();
        assert_eq!(flags["host"], json!("localhost"));
    }

    #[test]
    fn positionals_and_flags() {
        let (positionals, flags) = parse_from(args(&["run", "--port", "8080"])).unwrap();
        assert_eq!(positionals, vec!["run"]);
        assert_eq!(flags["port"], json!(8080));
    }

    #[test]
    fn nested_key() {
        let (_, flags) = parse_from(args(&["--server.port", "9000"])).unwrap();
        assert_eq!(flags["server"]["port"], json!(9000));
    }

    #[test]
    fn json_bool_parsed() {
        let (_, flags) = parse_from(args(&["--debug", "true"])).unwrap();
        assert_eq!(flags["debug"], json!(true));
    }

    #[test]
    fn non_json_kept_as_string() {
        let (_, flags) = parse_from(args(&["--name", "my app"])).unwrap();
        assert_eq!(flags["name"], json!("my app"));
    }

    #[test]
    fn multiple_flags() {
        let (_, flags) = parse_from(args(&["--host", "localhost", "--port=9000"])).unwrap();
        assert_eq!(flags["host"], json!("localhost"));
        assert_eq!(flags["port"], json!(9000));
    }

    #[test]
    fn short_flag_rejected() {
        let err = parse_from(args(&["-x"])).unwrap_err();
        assert!(err.contains("-x"));
    }

    #[test]
    fn flag_missing_value_last_arg() {
        let err = parse_from(args(&["--host"])).unwrap_err();
        assert!(err.contains("host"));
    }

    #[test]
    fn flag_missing_value_next_is_flag() {
        let err = parse_from(args(&["--host", "--port"])).unwrap_err();
        assert!(err.contains("host"));
    }

    #[test]
    fn positional_after_flag_rejected() {
        let err = parse_from(args(&["--host", "localhost", "extra"])).unwrap_err();
        assert!(err.contains("extra"));
    }
}
