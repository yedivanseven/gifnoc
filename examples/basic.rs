use gifnoc::{config, Configurable};


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
        email: Option<String> = None,
    }
}

fn main() {
    let config = MyConfig::default();
    println!("{} is {}", config.name, config.age);

    let json = serde_json::json!({
        "name": "Franz",
        "database.host": "remotehost",
    });

    let config = config.update(json);
    println!("{} is {}", config.name, config.age);
    println!("uri is {}:{}", config.database.host, config.database.port);
}
