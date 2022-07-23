# gh-config-rs
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
