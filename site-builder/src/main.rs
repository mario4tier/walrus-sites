mod calls;
mod network;
mod publish;
mod site;
mod suins;
mod util;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use network::NetworkConfig;
use publish::publish;
use serde::Deserialize;
use site::content::ContentEncoding;
use sui_types::{base_types::ObjectID, Identifier};
use suins::set_suins_name;
use util::handle_pagination;

use crate::util::id_to_base36;

#[derive(Parser, Debug)]
#[clap(rename_all = "kebab-case")]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    /// Publish a new site on sui
    Publish {
        /// The directory containing the site sources
        directory: PathBuf,
        /// The encoding for the contents of the BlockPages
        #[clap(short = 'e', long, value_enum, default_value_t = ContentEncoding::PlainText)]
        content_encoding: ContentEncoding,
        /// The name of the BlockSite
        #[arg(short, long, default_value = "test site")]
        site_name: String,
        /// The object ID of a partially published site to be completed
        #[arg(short, long, default_value=None)]
        object_id: Option<ObjectID>,
    },
    /// Convert an object ID in hex format to the equivalent base36 format.
    /// Useful to browse sites at particular object IDs.
    Convert {
        /// The object id (in hex format) to convert
        object_id: ObjectID,
    },
    /// Set the SuiNs record to an ObjectID.
    SetNs {
        /// The SuiNs packages
        #[clap(short, long)]
        package: ObjectID,
        /// The SuiNs object to be updated
        #[clap(short, long)]
        sui_ns: ObjectID,
        /// The SuiNsRegistration NFT with the SuiNs name
        #[clap(short, long)]
        registration: ObjectID,
        /// The address to be added to the record
        #[clap(short, long)]
        target: ObjectID,
    },
    /// Show the pages composing the blocksite at the given id
    Sitemap { object: ObjectID },
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "blocksite_module")]
    pub module: Identifier,
    #[serde(default = "testnet_package")]
    pub package: ObjectID,

    pub gas_coin: ObjectID,
    pub gas_budget: u64,

    #[serde(default)]
    pub network: NetworkConfig,
}

fn blocksite_module() -> Identifier {
    Identifier::new("blocksite").expect("valid literal identifier")
}

fn testnet_package() -> ObjectID {
    ObjectID::from_hex_literal("0x66b0b2d46dcd2e56952f1bd9e90218deaab0885e0f60ca29163f5e53c72ef810")
        .expect("valid hex literal")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut config: Config = std::fs::read_to_string(&args.config)
        .context(format!("unable to read config file {:?}", args.config))
        .and_then(|s| {
            toml::from_str(&s).context(format!("unable to parse toml in file {:?}", args.config))
        })?;
    config.network.load()?;

    match &args.command {
        Commands::Publish {
            directory,
            content_encoding,
            site_name,
            object_id,
        } => publish(directory, content_encoding, site_name, object_id, &config).await?,

        Commands::SetNs {
            package,
            sui_ns,
            registration,
            target,
        } => {
            set_suins_name(config, package, sui_ns, registration, target).await?;
        }

        Commands::Sitemap { object } => {
            let client = config.network.get_sui_client().await?;
            let all_dynamic_fields = handle_pagination(|cursor| {
                client.read_api().get_dynamic_fields(*object, cursor, None)
            })
            .await?;
            println!("Pages in site at object id: {}", object);
            for d in all_dynamic_fields {
                println!(
                    "  - {:<40} {:?}",
                    d.name.value.as_str().unwrap(),
                    d.object_id
                );
            }
        }

        Commands::Convert { object_id } => println!("{}", id_to_base36(object_id)?),
    };

    Ok(())
}