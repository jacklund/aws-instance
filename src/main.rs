mod cmdline;
mod commands;
mod error;
mod profile;
mod util;

use rusoto_core::{HttpClient, Region};
use rusoto_credential::{DefaultCredentialsProvider, ProfileProvider};
use rusoto_ec2::Ec2Client;
use std::str::FromStr;

use crate::cmdline::parse_command_line;
use crate::profile::{get_profile, ConfigFileReader, Profile};

pub use crate::error::{AwsInstanceError, Result};
pub use crate::util::print_state_changes;

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(error) = run_commands().await {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn get_ec2_client(region: Region, profile: &str) -> Ec2Client {
    let mut profile_provider = ProfileProvider::new().expect("Error creating profile provider");
    if !profile.is_empty() {
        profile_provider.set_profile(profile);
        Ec2Client::new_with(HttpClient::new().unwrap(), profile_provider, region)
    } else {
        Ec2Client::new_with(
            HttpClient::new().unwrap(),
            DefaultCredentialsProvider::new().unwrap(),
            region,
        )
    }
}

async fn run_commands() -> Result<()> {
    let options = parse_command_line();

    let config_file = ConfigFileReader::new(options.config_file);
    let profile_name = options.profile.or_else(|| Some("default".into())).unwrap();
    let profile = get_profile(&profile_name, &config_file)?;
    let region = match options.region {
        Some(region_name) => Region::from_str(&region_name)?,
        None => profile.region.clone(),
    };

    let ec2_client = get_ec2_client(region, &profile_name);
    options.subcommand.run(&ec2_client, profile).await?;

    Ok(())
}
