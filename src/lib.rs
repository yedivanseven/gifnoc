//! Type-safe configuration with layered overrides.
//!
//! gifnoc provides a [`config!`] macro that generates a configuration struct
//! with typed fields and compile-time defaults. The struct implements the
//! [`Configurable`] trait, whose [`update`][Configurable::update] method
//! applies overrides from any source — environment variables, CLI flags, TOML
//! or YAML files — without clobbering unrelated fields.
//!
//! # Quick start
//!
//! ```rust
//! use gifnoc::{config, Configurable};
//!
//! config! {
//!     ServerConfig {
//!         host: String = "localhost",
//!         port: u32 = 8080u32,
//!     }
//! }
//!
//! let config = ServerConfig::default();
//! assert_eq!(config.host, "localhost");
//! assert_eq!(config.port, 8080);
//! ```
//!
//! # Sources
//!
//! All source functions return [`serde_json::Value`] and plug into
//! [`Configurable::update`] via the same flatten → merge → nest pipeline.
//! Sources are layered by chaining `.update()` calls — later calls win:
//!
//! ```rust,no_run
//! # use gifnoc::{config, Configurable};
//! # config! { AppConfig { port: u32 = 8080u32 } }
//! let (actions, flags) = gifnoc::args::parse();
//! let config = AppConfig::default()
//!     .update(gifnoc::env::with_prefix("APP"))   // env vars override defaults
//!     .update(flags);                             // CLI flags override env vars
//! ```
//!
//! | Source | Function | Convention |
//! |--------|----------|------------|
//! | Environment variables | [`env::with_prefix`] | `APP_KEY`, `APP_SECTION__KEY` |
//! | CLI flags | [`args::parse`] | `--key value`, `--section.key value` |
//! | TOML file | [`toml::from_file`] | *(requires feature `toml`)* |
//! | YAML file | [`yaml::from_file`] | *(requires feature `yaml`)* |
//!
//! # Recommended pattern: composition root
//!
//! Build the full config once in `main`, then distribute slices to subsystems
//! via their constructors. Each subsystem receives only the section it needs,
//! keeping dependencies narrow:
//!
//! ```rust,no_run
//! use gifnoc::{config, Configurable};
//!
//! config! { ServerConfig { port: u32 = 8080u32 } }
//! config! { AppConfig { server: ServerConfig = ServerConfig::default() } }
//!
//! struct Server { port: u32 }
//!
//! impl Server {
//!     fn new(config: &ServerConfig) -> Self {
//!         Server { port: config.port }
//!     }
//!     fn run(&self) { println!("listening on :{}", self.port); }
//! }
//!
//! fn main() {
//!     let (actions, flags) = gifnoc::args::parse();
//!     let config = AppConfig::default()
//!         .update(gifnoc::env::with_prefix("APP"))
//!         .update(flags);
//!
//!     let server = Server::new(&config.server);
//!
//!     for action in &actions {
//!         match action.as_str() {
//!             "serve" => server.run(),
//!             other => { eprintln!("unknown action: {other}"); std::process::exit(1); }
//!         }
//!     }
//! }
//! ```

extern crate self as gifnoc;

mod nesting;

pub mod args;
pub mod env;

#[cfg(feature = "toml")]
pub mod toml;

#[cfg(feature = "yaml")]
pub mod yaml;

/// Defines a configuration struct with typed fields and compile-time defaults.
///
/// Generates a `pub struct` with all fields public, a [`Default`] impl wired
/// to the supplied default expressions, and an empty [`Configurable`] impl.
/// The struct derives [`serde::Serialize`] and [`serde::Deserialize`].
///
/// # Syntax
///
/// ```text
/// config! {
///     StructName {
///         field_name: Type = default_expr,
///         ...
///     }
/// }
/// ```
///
/// Default expressions are passed through `.into()`, so string literals work
/// for `String` fields. For numeric types, include a type suffix to avoid
/// ambiguity (`8080u32`).
///
/// # Examples
///
/// A flat config with scalar fields:
///
/// ```rust
/// use gifnoc::{config, Configurable};
///
/// config! {
///     ServerConfig {
///         host: String = "localhost",
///         port: u32 = 8080u32,
///         debug: bool = false,
///     }
/// }
///
/// let config = ServerConfig::default();
/// assert_eq!(config.host, "localhost");
/// assert_eq!(config.port, 8080);
/// assert!(!config.debug);
/// ```
///
/// Nested configs by using another `config!`-generated type as a field:
///
/// ```rust
/// use gifnoc::{config, Configurable};
///
/// config! { DbConfig { port: u32 = 5432u32 } }
/// config! { AppConfig { db: DbConfig = DbConfig::default() } }
///
/// let config = AppConfig::default();
/// assert_eq!(config.db.port, 5432);
/// ```
pub use gifnoc_macros::config;

/// Trait for configuration structs that support layered overrides.
///
/// Implemented automatically by the [`config!`] macro. Any type that is
/// [`serde::Serialize`] + [`serde::de::DeserializeOwned`] can also implement
/// it manually with an empty `impl` body — the default [`update`][Self::update]
/// method handles everything.
pub trait Configurable: serde::Serialize + serde::de::DeserializeOwned {
    /// Returns a new instance with fields overridden by `json`.
    ///
    /// Performs a deep merge via flatten → merge → nest:
    /// 1. Serialize `self` and `json` to dot-separated flat maps
    ///    (`{"db": {"host": "x"}}` → `{"db.host": "x"}`).
    /// 2. Merge the maps — incoming keys win over existing ones.
    /// 3. Nest the merged map back into a JSON object and deserialize into `Self`.
    ///
    /// This preserves sibling fields in nested sections that are not mentioned
    /// in `json`, unlike a shallow merge which would clobber the whole section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gifnoc::{config, Configurable};
    ///
    /// config! {
    ///     DbConfig {
    ///         host: String = "localhost",
    ///         port: u32 = 5432u32,
    ///     }
    /// }
    ///
    /// let config = DbConfig::default().update(serde_json::json!({"port": 9999}));
    /// assert_eq!(config.host, "localhost"); // preserved
    /// assert_eq!(config.port, 9999);        // overridden
    /// ```
    fn update(&self, json: serde_json::Value) -> Self
    where
        Self: Sized,
    {
        let mut base = nesting::flatten(serde_json::to_value(self).unwrap());
        base.extend(nesting::flatten(json));
        serde_json::from_value(nesting::nest(base)).unwrap()
    }
}
