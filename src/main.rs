extern crate clap;
extern crate dirs;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;
#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs;
extern crate snafu;
extern crate structopt;

mod cmdline;
mod create;
mod destroy;
mod ec2_wrapper;
mod error;
mod list;
mod list_amis;
mod profile;
mod ssh;
mod start;
mod stop;
mod util;

use rusoto_core::Region;
use std::collections::HashMap;
use std::str;
use std::str::FromStr;

use crate::cmdline::{parse_command_line, SubCommands};
use crate::create::create_instance;
use crate::destroy::destroy_instance;
use crate::list::list;
use crate::list_amis::list_amis;
use crate::profile::{ConfigFileReader, Profile};
use crate::ssh::ssh;
use crate::start::start;
use crate::stop::stop;

pub use crate::error::{AwsInstanceError, Result};
pub use crate::util::print_state_changes;

fn get_profile(profile_name: &str, config_file: &ConfigFileReader) -> Result<Profile> {
    match config_file.get_profile(profile_name) {
        Some(profile) => Ok(profile.clone()),
        None => Err(AwsInstanceError::ProfileNotFoundError {
            profile_name: profile_name.into(),
        }),
    }
}

fn main() {
    env_logger::init();

    if let Err(error) = run_commands() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run_commands() -> Result<()> {
    let options = parse_command_line();

    let config_file = ConfigFileReader::new();
    let profile_name = options.profile.or(Some("default".into())).unwrap();
    let profile = get_profile(&profile_name, &config_file);
    let region = match options.region {
        Some(region_name) => Region::from_str(&region_name)?,
        None => profile.region,
    };

    let ec2_wrapper = ec2_wrapper::AwsEc2Client::new(region, &profile_name);
    match options.subcommand {
        SubCommands::List => list(&ec2_wrapper)?,

        SubCommands::ListAmis {
            architecture,
            image_id,
            search,
        } => {
            let mut filters: HashMap<String, Vec<String>> = HashMap::new();
            filters.insert(
                "architecture".into(),
                architecture.split(',').map(|s| s.into()).collect(),
            );
            if let Some(image_id) = image_id {
                filters.insert(
                    "image_id".into(),
                    image_id.split(',').map(|s| s.into()).collect(),
                );
            }
            list_amis(&ec2_wrapper, &filters, search)?;
        }

        SubCommands::Create {
            name,
            ami_id,
            ebs_optimized,
            iam_profile,
            mut instance_type,
            mut keypair_name,
            mut security_group_ids,
        } => {
            instance_type = instance_type.or(profile.default_instance_type);
            keypair_name = keypair_name.or(profile.keypair);
            if security_group_ids.is_empty() && profile.security_groups.is_some() {
                security_group_ids = profile.security_groups.unwrap();
            }
            if let Err(error) = create_instance(
                &ec2_wrapper,
                &name,
                &ami_id,
                ebs_optimized,
                iam_profile,
                instance_type,
                keypair_name,
                security_group_ids,
            ) {
                eprintln!("{}", error);
            }
        }

        SubCommands::Destroy { name } => {
            destroy_instance(&ec2_wrapper, &name)?;
        }

        SubCommands::Ssh { name, mut sshopts } => {
            if profile.ssh_key.exists() && !sshopts.contains(&("-i".into())) {
                debug!(
                    "Adding -i {} to ssh opts",
                    profile.ssh_key.to_str().unwrap()
                );
                sshopts.push("-i".into());
                sshopts.push(profile.ssh_key.to_str().unwrap().into());
            }
            ssh(&ec2_wrapper, &name, &sshopts)?;
        }

        SubCommands::Start { name } => {
            start(&ec2_wrapper, &name)?;
        }

        SubCommands::Stop { name } => {
            stop(&ec2_wrapper, &name)?;
        }
    }

    Ok(())
}
