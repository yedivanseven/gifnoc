use gifnoc::{Configurable, config};

config! {
    Database {
        host: String = "localhost",
        port: u32 = 5432u32,
    }
}

config! {
    MyConfig {
        name: String = "Georg",
        age: u32 = 32u32,
        database: Database = Database::default(),
        tags: Vec<String> = vec![],
        email: Option<String> = Some("not given".into()),
    }
}

fn main() {
    let config = MyConfig::default();
    println!("{} is {}", config.name, config.age);

    let json = serde_json::json!({
        "name": "Franz",
        "database.host": "remotehost",
        "email": null,
        "tags": ["foo", "bar"]
    });

    let config = config.update(json);
    println!("{} is {}", config.name, config.age);
    println!("Email: {}", config.email.unwrap_or("not given".to_string()));
    println!("uri is {}:{}", config.database.host, config.database.port);
    println!("tags:{:?}", config.tags);
}
