# gh-config-rs

[![Rust](https://github.com/siketyan/gh-config-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/siketyan/gh-config-rs/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/gh-config.svg)](https://crates.io/crates/gh-config)
[![docs](https://docs.rs/gh-config/badge.svg)](https://docs.rs/gh-config/)

Loads config and hosts for gh CLI in Rust.

## Getting started

```toml
[dependencies]
gh-config = "0.5.0"
```

## Usage

```rust
use std::error::Error;
use gh_config::*;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;
    let hosts = Hosts::load()?;

    match hosts.get(GITHUB_COM) {
        Some(host) => println!("Token for github.com: {}", host.oauth_token),
        _ => eprintln!("Token not found."),
    }

    Ok(())
}
```

## CLI

gh-config-rs is a hybrid crate that can be used as a library or a CLI.
To use as a CLI, can be installed using the command line below:

```shell
cargo install gh-config --features=cli
```

### Usages

Lists all configuration in YAML:

```shell
gh-config config show
```

Uses JSON instead:

```shell
gh-config --json config show
```

Uses custom path of config.yaml instead of default:

```shell
gh-config --path /path/to/config.yaml config show
```

Gets an authentication for github.com:

```shell
gh-config authn get github.com
```

Outputs only the OAuth token instead:

```shell
gh-config authn get --token-only github.com
```
