use gifnoc::{config, Configurable};

config! {
    App {
        name: String = "default",
        port: u32 = 8000u32,
    }
}

fn main() {
    let (actions, flags) = gifnoc::args::parse();
    let config = App::default()
        .update(gifnoc::env::with_prefix("APP"))
        .update(flags);
    println!("actions: {:?}", actions);
    println!("{} on {}", config.name, config.port);
}
