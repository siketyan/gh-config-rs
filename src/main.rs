use anyhow::anyhow;
use clap::Parser;
use gh_config::{Config, Hosts};
use std::path::PathBuf;

#[derive(Debug, clap::Subcommand)]
enum ConfigAction {
    /// Shows the entire configuration.
    Show,
}

#[derive(Debug, clap::Subcommand)]
enum AuthnAction {
    /// Lists all authentications for each hosts.
    List,
    /// Gets an authentication for the host.
    Get {
        /// The GitHub host to get an authentication for.
        host: String,

        /// Outputs only the OAuth token.
        #[clap(long)]
        token_only: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
enum Location {
    /// Gets gh CLI configuration.
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    /// Gets authentications for gh CLI.
    Authn {
        #[clap(subcommand)]
        action: AuthnAction,
    },
}

/// Retrieves your configuration of gh CLI.
#[derive(Debug, clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Where to get data from.
    #[clap(subcommand)]
    location: Location,

    /// Uses JSON instead of YAML for outputs.
    #[clap(short, long)]
    json: bool,

    /// File path to load configuration or authentications from.
    path: Option<PathBuf>,
}

macro_rules! output {
    ($args: expr, $value: expr) => {
        Ok::<(), anyhow::Error>(print!(
            "{}",
            match $args.json {
                true => serde_json::to_string($value)?,
                false => serde_yaml::to_string($value)?,
            }
        ))
    };
}

fn run() -> Result<(), anyhow::Error> {
    let args: Args = Parser::parse();

    Ok(match args.location {
        Location::Config { action } => {
            let config = match args.path {
                Some(p) => Config::load_from(p),
                _ => Config::load(),
            }?;

            match action {
                ConfigAction::Show => output!(args, &config)?,
            }
        }
        Location::Authn { action } => {
            let hosts = match args.path {
                Some(p) => Hosts::load_from(p),
                _ => Hosts::load(),
            }?;

            match action {
                AuthnAction::List => output!(args, &hosts)?,
                AuthnAction::Get { host, token_only } => match hosts.get(&host) {
                    Some(h) => match token_only {
                        true => Ok(print!("{}", h.oauth_token)),
                        _ => output!(args, &h),
                    },
                    _ => Err(anyhow!(
                        "The specified host not found in the configuration."
                    )),
                }?,
            }
        }
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }
}
