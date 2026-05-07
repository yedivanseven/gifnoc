use gifnoc::{config, Configurable};

config! {
    RouteConfig {
        api_prefix: String = "/api",
    }
}

config! {
    ServerConfig {
        host: String = "0.0.0.0",
        port: u32 = 8080u32,
        routes: RouteConfig = RouteConfig::default(),
    }
}

config! {
    WorkerConfig {
        interval_seconds: u32 = 60u32,
        queue_name: String = "default",
    }
}

config! {
    AppConfig {
        server: ServerConfig = ServerConfig::default(),
        worker: WorkerConfig = WorkerConfig::default(),
    }
}

struct Router {
    api_prefix: String,
}

impl Router {
    fn new(config: &RouteConfig) -> Self {
        Router { api_prefix: config.api_prefix.clone() }
    }

    fn describe(&self) {
        println!("  router mounting routes under {}", self.api_prefix);
    }
}

struct Server {
    host: String,
    port: u32,
    router: Router,
}

impl Server {
    fn new(config: &ServerConfig) -> Self {
        Server {
            host: config.host.clone(),
            port: config.port,
            router: Router::new(&config.routes),
        }
    }

    fn run(&self) {
        println!("server listening on {}:{}", self.host, self.port);
        self.router.describe();
    }
}

struct Worker {
    interval_seconds: u32,
    queue_name: String,
}

impl Worker {
    fn new(config: &WorkerConfig) -> Self {
        Worker {
            interval_seconds: config.interval_seconds,
            queue_name: config.queue_name.clone(),
        }
    }

    fn run(&self) {
        println!(
            "worker draining queue '{}' every {}s",
            self.queue_name, self.interval_seconds
        );
    }
}

struct App {
    server: Server,
    worker: Worker,
}

impl App {
    fn new(config: AppConfig) -> Self {
        App {
            server: Server::new(&config.server),
            worker: Worker::new(&config.worker),
        }
    }
}

fn main() {
    let (actions, flags) = gifnoc::args::parse();
    let config = AppConfig::default()
        .update(gifnoc::env::with_prefix("APP"))
        .update(flags);

    let app = App::new(config);

    for action in &actions {
        match action.as_str() {
            "serve" => app.server.run(),
            "work"  => app.worker.run(),
            other   => {
                eprintln!("error: unknown action '{other}'");
                std::process::exit(1);
            }
        }
    }
}
