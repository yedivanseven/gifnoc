use gifnoc::{Configurable, config};
use serde_json::json;

config! {
    DbConfig {
        host: String = "localhost",
        port: u32 = 5432u32,
    }
}

config! {
    AppConfig {
        name: String = "app",
        debug: bool = false,
        db: DbConfig = DbConfig::default(),
    }
}

#[test]
fn defaults_are_correct() {
    let config = AppConfig::default();
    assert_eq!(config.name, "app");
    assert!(!config.debug);
    assert_eq!(config.db.host, "localhost");
    assert_eq!(config.db.port, 5432);
}

#[test]
fn empty_update_is_noop() {
    let config = AppConfig::default().update(json!({}));
    assert_eq!(config.name, "app");
    assert_eq!(config.db.host, "localhost");
    assert_eq!(config.db.port, 5432);
}

#[test]
fn top_level_field_override() {
    let config = AppConfig::default().update(json!({"name": "myapp"}));
    assert_eq!(config.name, "myapp");
    assert!(!config.debug); // unchanged
    assert_eq!(config.db.host, "localhost"); // unchanged
}

#[test]
fn nested_override_preserves_siblings() {
    let config = AppConfig::default().update(json!({"db": {"host": "remotehost"}}));
    assert_eq!(config.db.host, "remotehost");
    assert_eq!(config.db.port, 5432); // sibling within db preserved
    assert_eq!(config.name, "app"); // sibling at top level preserved
}

#[test]
fn layered_updates_later_wins() {
    let config = AppConfig::default()
        .update(json!({"name": "first"}))
        .update(json!({"name": "second"}));
    assert_eq!(config.name, "second");
}

#[test]
fn layered_updates_merge_different_keys() {
    let config = AppConfig::default()
        .update(json!({"name": "myapp"}))
        .update(json!({"debug": true}));
    assert_eq!(config.name, "myapp"); // survives second update
    assert!(config.debug);
}

#[test]
fn nested_override_across_layers() {
    let config = AppConfig::default()
        .update(json!({"db": {"host": "host1", "port": 6000}}))
        .update(json!({"db": {"host": "host2"}}));
    assert_eq!(config.db.host, "host2"); // overridden
    assert_eq!(config.db.port, 6000); // survives second update
}

#[test]
fn dotted_key_override_preserves_siblings() {
    let config = AppConfig::default().update(json!({"db.host": "remotehost"}));
    assert_eq!(config.db.host, "remotehost");
    assert_eq!(config.db.port, 5432); // sibling within db preserved
    assert_eq!(config.name, "app"); // sibling at top level preserved
}

#[test]
fn dotted_and_nested_styles_compose() {
    let config = AppConfig::default().update(json!({
        "db.host": "remotehost",
        "db": {"port": 9999},
    }));
    assert_eq!(config.db.host, "remotehost");
    assert_eq!(config.db.port, 9999);
}

#[test]
#[should_panic(expected = "unknown config key: 'prot'")]
fn unknown_top_level_key_panics() {
    AppConfig::default().update(json!({"prot": 9000}));
}

#[test]
#[should_panic(expected = "unknown config key: 'db.prot'")]
fn unknown_nested_key_panics() {
    AppConfig::default().update(json!({"db": {"prot": 9000}}));
}

#[test]
#[should_panic(expected = "unknown config key: 'db.prot'")]
fn unknown_dotted_key_panics() {
    AppConfig::default().update(json!({"db.prot": 9000}));
}
