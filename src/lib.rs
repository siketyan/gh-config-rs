//! # gh-config
//! Loads config and hosts for gh CLI.
//!
//! ## Getting started
//! ```toml
//! [dependencies]
//! gh-config = "0.3"
//! ```
//!
//! ## Usage
//! ```rust
//! use std::error::Error;
//! use gh_config::*;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let config = Config::load()?;
//!     let hosts = Hosts::load()?;
//!     
//!     match hosts.get(GITHUB_COM) {
//!         Some(host) => println!("Token for github.com: {}", hosts.retrieve_token(GITHUB_COM)?.unwrap()),
//!         _ => eprintln!("Token not found."),
//!     }
//!
//!     Ok(())
//! }
//! ```

mod keyring;

use std::collections::HashMap;
use std::env::var;
use std::path::{Path, PathBuf};

use dirs::home_dir;
use serde::{Deserialize, Serialize};

use crate::keyring::{GhKeyring, Keyring};

#[cfg(target_os = "windows")]
const APP_DATA: &str = "AppData";
const GH_CONFIG_DIR: &str = "GH_CONFIG_DIR";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

const CONFIG_FILE_NAME: &str = "config.yml";
const HOSTS_FILE_NAME: &str = "hosts.yml";

/// Hostname of github.com.
pub const GITHUB_COM: &str = "github.com";
pub const GHE_COM: &str = "ghe.com";
pub const LOCALHOST: &str = "github.localhost";

/// An error occurred in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialize config from YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Secure storage error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Config file not found.")]
    ConfigNotFound,
}

/// What protocol to use when performing git operations.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitProtocol {
    Https,
    Ssh,
}

/// When to interactively prompt.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Prompt {
    Enabled,
    Disabled,
}

impl From<Prompt> for bool {
    fn from(p: Prompt) -> Self {
        matches!(p, Prompt::Enabled)
    }
}

/// Config representation for gh CLI.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// What protocol to use when performing git operations.
    pub git_protocol: GitProtocol,

    /// What editor gh should run when creating issues, pull requests, etc.
    /// If blank, will refer to environment.
    pub editor: Option<String>,

    /// When to interactively prompt.
    /// This is a global config that cannot be overridden by hostname.
    pub prompt: Prompt,

    /// A pager program to send command output to, e.g. "less".
    /// Set the value to "cat" to disable the pager.
    pub pager: Option<String>,

    /// Aliases allow you to create nicknames for gh commands.
    #[serde(default)]
    pub aliases: HashMap<String, String>,

    /// The path to a unix socket through which send HTTP connections.
    /// If blank, HTTP traffic will be handled by default transport.
    pub http_unix_socket: Option<String>,

    /// What web browser gh should use when opening URLs.
    /// If blank, will refer to environment.
    pub browser: Option<String>,
}

impl Config {
    /// Loads a config from the default path.
    pub fn load() -> Result<Self, Error> {
        Self::load_from(CONFIG_FILE_NAME)
    }

    /// Loads all host configs from the specified path.
    pub fn load_from<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        load(path)
    }
}

/// Host config representation for gh CLI.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Host {
    pub user: Option<String>,
    #[serde(default)]
    oauth_token: String,
    pub git_protocol: Option<GitProtocol>,
}

/// Mapped host configs by their hostname.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hosts(HashMap<String, Host>);

impl Hosts {
    /// Loads all host configs from the default path.
    pub fn load() -> Result<Self, Error> {
        Self::load_from(HOSTS_FILE_NAME)
    }

    /// Loads all host configs from the specified path.
    pub fn load_from<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        load(path).map(Self)
    }

    /// Gets a host config by the hostname.
    pub fn get(&self, hostname: &str) -> Option<&Host> {
        self.0.get(hostname)
    }

    /// Sets a host config and returns the current value.
    /// If no values present currently, returns `None` .
    pub fn set(&mut self, hostname: impl Into<String>, host: Host) -> Option<Host> {
        self.0.insert(hostname.into(), host)
    }

    /// Retrieves a token from the environment variables, the hosts file, or the secure storage.
    /// User interaction may be required to unlock the keychain, depending on the OS.
    /// If any token found for the hostname, returns None.
    pub fn retrieve_token(&self, hostname: &str) -> Result<Option<String>, Error> {
        if let Some(token) = retrieve_token_from_env(is_enterprise(hostname)) {
            return Ok(Some(token));
        }

        if let Some(token) = self
            .get(hostname)
            .and_then(|h| match h.oauth_token.is_empty() {
                true => None,
                _ => Some(h.oauth_token.to_owned()),
            })
        {
            return Ok(Some(token));
        }

        retrieve_token_secure(hostname)
    }

    /// Retrieves a token from the secure storage only.
    /// User interaction may be required to unlock the keychain, depending on the OS.
    /// If any token found for the hostname, returns None.
    #[deprecated(
        since = "0.4.0",
        note = "Use `retrieve_token_secure` without `Hosts` struct instead."
    )]
    pub fn retrieve_token_secure(&self, hostname: &str) -> Result<Option<String>, Error> {
        retrieve_token_secure(hostname)
    }
}

/// Determines the provided hostname is a GitHub Enterprise Server instance or not.
pub fn is_enterprise(host: &str) -> bool {
    host != GITHUB_COM && host != LOCALHOST && !host.ends_with(&format!(".{}", GHE_COM))
}

/// Retrieves a token from the environment variables `GH_TOKEN` or `GITHUB_TOKEN`.
/// Also tries to retrieve from `GH_ENTERPRISE_TOKEN` or `GITHUB_ENTERPRISE_TOKEN`, if the
/// enterprise flag enabled.
pub fn retrieve_token_from_env(enterprise: bool) -> Option<String> {
    if enterprise {
        if let Ok(token) = var("GH_ENTERPRISE_TOKEN").or_else(|_| var("GITHUB_ENTERPRISE_TOKEN")) {
            return Some(token);
        }
    }

    var("GH_TOKEN").or_else(|_| var("GITHUB_TOKEN")).ok()
}

/// Retrieves a token from the secure storage.
/// User interaction may be required to unlock the keychain, depending on the OS.
/// If any token found for the hostname, returns None.
pub fn retrieve_token_secure(hostname: &str) -> Result<Option<String>, Error> {
    Ok(Keyring
        .get(hostname)?
        .map(|t| String::from_utf8(t).unwrap()))
}

/// Finds the default config directory effected by the environment.
pub fn find_config_directory() -> Option<PathBuf> {
    let gh_config_dir = var(GH_CONFIG_DIR).unwrap_or_default();
    if !gh_config_dir.is_empty() {
        return Some(PathBuf::from(gh_config_dir));
    }

    let xdg_config_home = var(XDG_CONFIG_HOME).unwrap_or_default();
    if !xdg_config_home.is_empty() {
        return Some(PathBuf::from(xdg_config_home).join("gh"));
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = var(APP_DATA).unwrap_or_default();
        if !app_data.is_empty() {
            return Some(PathBuf::from(app_data).join("GitHub CLI"));
        }
    }

    home_dir().map(|p| p.join(".config").join("gh"))
}

/// Loads a file in the config directory as `T` type.
pub fn load<T, P>(path: P) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    serde_yaml::from_slice(
        std::fs::read(
            find_config_directory()
                .ok_or(Error::ConfigNotFound)?
                .join(path),
        )
        .map_err(Error::Io)?
        .as_ref(),
    )
    .map_err(Error::Yaml)
}
