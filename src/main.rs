#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate dirs;
extern crate env_logger;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;
#[macro_use]
extern crate lazy_static;

mod create;
mod destroy;
mod ec2_wrapper;
mod list;
mod list_amis;
mod profile;
mod ssh;
mod start;
mod stop;
mod util;

use clap::ArgMatches;
use rusoto_core::Region;
use rusoto_ec2::{DescribeImagesError, IamInstanceProfileSpecification, RunInstancesError};
use std::collections::HashMap;
use std::process::exit;
use std::str;
use std::str::FromStr;

use create::{create_instance, CreateOptions};
use destroy::destroy_instance;
use list::list;
use list_amis::list_amis;
use profile::{ConfigFileReader, Profile};
use ssh::ssh;
use start::start;
use stop::stop;

pub static mut DEBUG: bool = false;

fn get_create_options(matches: &ArgMatches, profile: Profile) -> CreateOptions {
    let mut ret = CreateOptions::default();
    ret.instance_type = profile.default_instance_type;
    ret.key_name = profile.keypair;
    ret.security_group_ids = profile.security_groups;

    ret.ebs_optimized = Some(matches.is_present("ebs_optimized"));
    if let Some(profile) = matches.value_of("iam_instance_profile") {
        let mut iam_profile_spec = IamInstanceProfileSpecification::default();
        iam_profile_spec.name = Some(profile.to_string());
        ret.iam_instance_profile = Some(iam_profile_spec);
    }
    if let Some(ami_id) = matches.value_of("AMI_ID") {
        ret.image_id = Some(ami_id.to_string());
    }
    if let Some(instance_type) = matches.value_of("instance_type") {
        ret.instance_type = Some(instance_type.to_string());
    }
    if let Some(keypair_name) = matches.value_of("keypair_name") {
        ret.key_name = Some(keypair_name.to_string());
    }
    if let Some(security_group_ids) = matches.values_of("security_group_ids") {
        ret.security_group_ids = Some(security_group_ids.map(|s| s.to_string()).collect());
    }

    ret
}

fn parse_filters(matches: &ArgMatches) -> HashMap<String, Vec<String>> {
    let mut filter_values: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(filters) = matches.values_of("filters") {
        for key_value in filters {
            let split: Vec<&str> = key_value.split('=').collect();
            match split.len() {
                1 => {
                    error!("Filter value {} doesn't contain an '='", key_value);
                    exit(1);
                }
                2 => {
                    if filter_values.contains_key(split[0]) {
                        filter_values
                            .get_mut(split[0])
                            .unwrap()
                            .push(split[1].to_string());
                    } else {
                        filter_values.insert(split[0].to_string(), vec![split[1].to_string()]);
                    }
                }
                _ => {
                    error!("Filter value {} contains too many '='s", key_value);
                    exit(1);
                }
            }
        }
    }

    filter_values
}

fn parse_command_line<'a>() -> ArgMatches<'a> {
    clap_app!(myapp =>
        (about: "Manage AWS instances")
        (@arg profile: -p --profile +takes_value "Set the AWS profile to use")
        (@arg region: -r --region +takes_value "Set the AWS region to use")
        (@arg debug: -d "Turns on debugging")
        (@subcommand create =>
            (about: "Create an instance from an AMI")
            (@arg NAME: +required "Name of the instance")
            (@arg AMI_ID: +required "The AMI ID")
            (@arg ebs_optimized: -e --ebs_optimized "Indicates whether the instance is optimized for Amazon EBS I/O")
            (@arg iam_instance_profile: -i --iam_profile +takes_value "The IAM instance profile")
            (@arg instance_type: -t --instance_type +takes_value "The EC2 instance type")
            (@arg keypair_name: -k --keypair +takes_value "The keypair name")
            (@arg security_group_ids: -s... --security_group_id +takes_value "Security group ids")
        )
        (@subcommand destroy =>
            (about: "Destroy an instance")
            (@arg NAME: +required "Name of the instance")
        )
        (@subcommand list =>
            (about: "List the instances you currently have")
        )
        (@subcommand list_amis =>
            (about: "List AMIs based on a filter")
            (@arg filters: -f... --filter +takes_value "Filter for AMIs")
        )
        (@subcommand ssh =>
            (about: "SSH into an instance")
            (@arg NAME: +required "Name of the instance")
            (@arg sshopts: +multiple "SSH options")
        )
        (@subcommand start =>
            (about: "Start an instance")
            (@arg NAME: +required "Name of the instance")
        )
        (@subcommand stop =>
            (about: "Stop an instance")
            (@arg NAME: +required "Name of the instance")
        )
    ).name(crate_name!()).get_matches()
}

fn get_profile<'a>(matches: ArgMatches<'a>, config_file: &'a ConfigFileReader) -> &'a Profile {
    let profile_name = matches.value_of("profile").or(Some("default")).unwrap();
    debug!("Calling ConfigFileReader::new()");
    let profile = config_file
        .get_profile(profile_name)
        .unwrap_or_else(|| panic!("No profile named {} found", profile_name));
    debug!("Using profile {:?}", profile);

    profile
}

fn main() {
    env_logger::init();

    let matches = parse_command_line();

    debug!("Calling ConfigFileReader::new()");
    let config_file = ConfigFileReader::new();
    let profile_name = matches.value_of("profile").or(Some("default")).unwrap();
    let profile = get_profile(matches.clone(), &config_file);
    let region = match matches.value_of("region") {
        None => profile.clone().region,
        Some(region_name) => Region::from_str(region_name).expect("Error parsing region name"),
    };

    let ec2_wrapper = ec2_wrapper::AwsEc2Client::new(region, profile_name);

    if matches.subcommand_matches("list").is_some() {
        if let Err(error) = list(&ec2_wrapper) {
            error!("{}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("list_amis") {
        let filter_values = parse_filters(matches);
        if let Err(error) = list_amis(&ec2_wrapper, &filter_values) {
            match error {
                DescribeImagesError::Unknown(http_response) => error!(
                    "Unknown error: status: {}, body: {}",
                    http_response.status,
                    str::from_utf8(&http_response.body).unwrap()
                ),
                _ => error!("List AMIs error: {}", error),
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("ssh") {
        let name = matches.value_of("NAME").unwrap();
        let mut sshopts: Vec<&str> = matches.values_of("sshopts").unwrap_or_default().collect();
        if profile.ssh_key.exists() && !sshopts.contains(&"-i") {
            debug!(
                "Adding -i {} to ssh opts",
                profile.ssh_key.to_str().unwrap()
            );
            sshopts.push("-i");
            sshopts.push(profile.ssh_key.to_str().unwrap());
        }
        if let Err(error) = ssh(&ec2_wrapper, name, &sshopts) {
            error!("{}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("start") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = start(&ec2_wrapper, name) {
            error!("{}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("stop") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = stop(&ec2_wrapper, name) {
            error!("{}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("create") {
        let name = matches.value_of("NAME").unwrap();
        let create_options = get_create_options(matches, profile.clone());
        if let Err(error) = create_instance(&ec2_wrapper, name, create_options) {
            match error {
                RunInstancesError::Unknown(http_response) => error!(
                    "Unknown error: status: {}, body: {}",
                    http_response.status,
                    str::from_utf8(&http_response.body).unwrap()
                ),
                _ => error!("Create error: {}", error),
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("destroy") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = destroy_instance(&ec2_wrapper, name) {
            error!("{:?}", error);
        }
    }
}
