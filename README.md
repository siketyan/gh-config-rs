# gh-config-rs
[![Rust](https://github.com/siketyan/gh-config-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/siketyan/gh-config-rs/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/gh-config.svg)](https://crates.io/crates/apdu)
[![docs](https://docs.rs/gh-config/badge.svg)](https://docs.rs/apdu/)

Loads config and hosts for gh CLI in Rust.

## Getting started
```toml
[dependencies]
gh-config = "0.1"
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
