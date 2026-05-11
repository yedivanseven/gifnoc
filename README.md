[![Build](https://github.com/yedivanseven/gifnoc/actions/workflows/build.yml/badge.svg)](https://github.com/yedivanseven/gifnoc/actions/workflows/build.yml)
[![Publish](https://github.com/yedivanseven/gifnoc/actions/workflows/publish.yml/badge.svg)](https://github.com/yedivanseven/gifnoc/actions/workflows/publish.yml)

# gifnoc
_Type-safe and layered project configuration from multiple sources_

## Motivation
Using various third-party tools, I often dispair when trying to configure them.
Some options can only be set via a command-line flag, some are read from a
config file, while still others can only be set via environment variables.
These ambiguities are exacerbated when using third-party tools in docker or
docker compose.

At the same time, I know from personal experience how tedious it can be to
maintain a well-documented set of configuration options, especially in a fast
moving world, where the code base changes on a daily basis and quick
experiments need to be run on short notice.

I have implemented a solution in `python` that I use in all my projects. It
can be found in the `cli` and `jsonobject` subpackages of
[swak](https://github.com/yedivanseven/swak), available from the python
package index [PyPI](https://pypi.org/project/swak/). Working mostly in
data-science and machine learning and, therefore, using
[pola.rs](https://docs.pola.rs/) for data pipelines, the `gifnoc` crate
(_config_ in reverse), is my attempt to replicate a pragmatic project
configuration solution in `rust`.

## Design
- There is one and only one global project configuration.
- The project configuration has the form of a nested `struct`.
- _All_ configuration options can be set by _all_ mechanisms:
  1. command-line arguments
  2. environment arguments
  3. configuration file (TOML or YAML)
- The precedence of these update is a matter of choice and taste.
- Adding fields to the config struct is the only code change required to make
  a new config option available for all mechanisms.

## Usage Example
```rust
use gifnoc::{Configurable, config};

// Define defaults
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

fn main() {
    let cfg_file = gifnoc::toml::from_file("config.toml");  // Read config file
    let env_vars = gifnoc::env::with_prefix("APP");         // Parse environment
    let (positional, flags) = gifnoc::args::parse();        // Read command line

    // Chose order of precedence
    let config = AppConfig::default()
        .update(cfg_file)
        .update(env_vars)
        .update(args);

    // positional arguments can be flexibly used to steer app behaviour
    println!("positional arguments: {:?}", positional);

}
```
