use std::path::PathBuf;

use anyhow::Context;
use clap::Args;

use nao::{Network, SystemctlAction};
use repository::Repository;

use crate::{
    aliveness::{aliveness, Arguments as AlivenessArguments},
    hulk::{hulk, Arguments as HulkArguments},
    logs::{logs, Arguments as LogsArguments},
    parsers::{parse_network, NaoAddress, NETWORK_POSSIBLE_VALUES},
    wireless::{wireless, Arguments as WirelessArguments},
};

#[derive(Args)]
pub struct Arguments {
    /// Disable aliveness (may restart HULA if needed)
    #[clap(long)]
    pub no_aliveness: bool,
    /// The network to connect the wireless device to e.g. SPL_A or None (None disconnects from anything)
    #[clap(long, default_value = "None", possible_values = NETWORK_POSSIBLE_VALUES, parse(try_from_str = parse_network))]
    pub network: Network,
    /// Directory where to store the downloaded logs (will be created if not existing)
    pub log_directory: PathBuf,
    /// The NAOs to execute that command on e.g. 20w or 10.1.24.22
    #[clap(required = true)]
    pub naos: Vec<NaoAddress>,
}

pub async fn post_game(arguments: Arguments, repository: &Repository) -> anyhow::Result<()> {
    hulk(
        HulkArguments {
            action: SystemctlAction::Stop,
            naos: arguments.naos.clone(),
        },
        repository,
    )
    .await
    .context("Failed to start HULK service")?;

    logs(
        LogsArguments::Download {
            log_directory: arguments.log_directory,
            naos: arguments.naos.clone(),
        },
        repository,
    )
    .await
    .context("Failed to download logs")?;

    aliveness(
        if arguments.no_aliveness {
            AlivenessArguments::Disable {
                naos: arguments.naos.clone(),
            }
        } else {
            AlivenessArguments::Enable {
                naos: arguments.naos.clone(),
            }
        },
        repository,
    )
    .await
    .context("Failed to enable/disable aliveness")?;

    wireless(
        WirelessArguments::Set {
            network: arguments.network,
            naos: arguments.naos,
        },
        repository,
    )
    .await
    .context("Failed to set wireless network")?;

    Ok(())
}
