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
mod commands;
mod ec2_wrapper;
mod error;
mod profile;
mod util;

use rusoto_core::Region;
use std::str::FromStr;

use crate::cmdline::parse_command_line;
use crate::profile::{get_profile, ConfigFileReader, Profile};

pub use crate::error::{AwsInstanceError, Result};
pub use crate::util::print_state_changes;

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
    let profile_name = options.profile.or_else(|| Some("default".into())).unwrap();
    let profile = get_profile(&profile_name, &config_file)?;
    let region = match options.region {
        Some(region_name) => Region::from_str(&region_name)?,
        None => profile.region.clone(),
    };

    let ec2_wrapper = ec2_wrapper::AwsEc2Client::new(region, &profile_name);
    options.subcommand.run(&ec2_wrapper, profile)?;

    Ok(())
}
