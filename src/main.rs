#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

#[macro_use]
mod util;
mod create;
mod destroy;
mod ec2_wrapper;
mod list;
mod list_amis;
mod ssh;
mod start;
mod stop;

use clap::ArgMatches;
use rusoto_core::Region;
use rusoto_ec2::IamInstanceProfileSpecification;
use std::collections::HashMap;
use std::process::exit;
use std::str::FromStr;

use create::{create_instance, CreateOptions};
use destroy::destroy_instance;
use list::list;
use list_amis::list_amis;
use ssh::ssh;
use start::start;
use stop::stop;

pub static mut DEBUG: bool = false;

fn get_create_options(matches: &ArgMatches) -> CreateOptions {
    let mut ret = CreateOptions::default();

    ret.ebs_optimized = Some(matches.is_present("ebs_optimized"));
    if let Some(profile) = matches.value_of("iam_instance_profile") {
        let mut iam_profile_spec = IamInstanceProfileSpecification::default();
        iam_profile_spec.name = Some(profile.to_string());
        ret.iam_instance_profile = Some(iam_profile_spec);
    }
    if let Some(ami_id) = matches.value_of("AMI_ID") {
        ret.image_id = ami_id.to_string();
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

fn main() {
    env_logger::init();

    let matches = clap_app!(myapp =>
        (about: "Manage AWS instances")
        (@arg profile: -p --profile +takes_value "Set the AWS profile to use")
        (@arg region: -r --region +takes_value +required "Set the AWS region to use")
        (@arg debug: -d "Turns on debugging")
        (@subcommand create =>
            (about: "Create an instance from an AMI")
            (@arg NAME: +required "Name of the instance")
            (@arg AMI_ID: +required "The AMI ID")
            (@arg ebs_optimized: -e --ebs_optimized "Indicates whether the instance is optimized for Amazon EBS I/O")
            (@arg iam_instance_profile: -i --iam_profile +takes_value "The IAM instance profile")
            (@arg instance_type: -t --instance_type +takes_value "The EC2 instance type")
            (@arg keypair_name: -k --keypair +required +takes_value "The keypair name")
            (@arg security_group_ids: -s... --security_group_id +required +takes_value "Security group ids")
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
    ).name(crate_name!()).get_matches();

    let region =
        Region::from_str(matches.value_of("region").unwrap()).expect("Error parsing region name");
    let profile = matches.value_of("profile").unwrap_or("");

    let ec2_wrapper = ec2_wrapper::AwsEc2Client::new(region, profile);

    if matches.subcommand_matches("list").is_some() {
        if let Err(error) = list(&ec2_wrapper) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("list_amis") {
        let mut filter_values: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(filters) = matches.values_of("filters") {
            for key_value in filters {
                let split: Vec<&str> = key_value.split('=').collect();
                match split.len() {
                    1 => {
                        eprintln!("Filter value {} doesn't contain an '='", key_value);
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
                        eprintln!("Filter value {} contains too many '='s", key_value);
                        exit(1);
                    }
                }
            }
        }
        if let Err(error) = list_amis(&ec2_wrapper, &filter_values) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("ssh") {
        let name = matches.value_of("NAME").unwrap();
        let sshopts: Vec<&str> = matches.values_of("sshopts").unwrap_or_default().collect();
        if let Err(error) = ssh(&ec2_wrapper, name, &sshopts) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("start") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = start(&ec2_wrapper, name) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("stop") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = stop(&ec2_wrapper, name) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("create") {
        let name = matches.value_of("NAME").unwrap();
        let create_options = get_create_options(matches);
        if let Err(error) = create_instance(&ec2_wrapper, name, create_options) {
            eprintln!("{:?}", error);
        }
    } else if let Some(matches) = matches.subcommand_matches("destroy") {
        let name = matches.value_of("NAME").unwrap();
        if let Err(error) = destroy_instance(&ec2_wrapper, name) {
            eprintln!("{:?}", error);
        }
    }
}
