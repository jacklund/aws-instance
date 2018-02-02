#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use docopt::Docopt;
use rusoto_core::{default_tls_client, Region};
use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::{ProfileProvider, ProvideAwsCredentials};
use rusoto_ec2::{Ec2, Ec2Client};
use std::str::FromStr;

const USAGE: &'static str = "
Manage AWS instances

Usage:
    aws-instance [options] list
    aws-instance [options] create <name>
    aws-instance [options] start <name>
    aws-instance [options] ssh <name> [-- <sshopt>...]
    aws-instance [options] stop <name>
    aws-instance [options] get-state <name>

Options:
  -h --help                    Show this screen.
  -r --region <region>         Set the region (Default: us-east-2)
  -p --profile <profile_name>  Set the profile name to use
";


#[derive(Debug, Deserialize)]
struct Args {
    cmd_list: bool,
    cmd_create: bool,
    cmd_start: bool,
    cmd_ssh: bool,
    cmd_stop: bool,
    cmd_get_state: bool,
    arg_name: Vec<String>,
    flag_profile: String,
    flag_region: String,
}

fn get_name(instance: &rusoto_ec2::Instance) -> String {
    match instance.tags {
        Some(ref tags) => {
            for tag in tags {
                if let Some("Name") = tag.key.as_ref().map(String::as_str) {
                    match tag.value {
                        Some(ref value) => return value.to_string(),
                        None => return String::new(),
                    }
                }
            }
        },
        None => return String::new(),
    }

    String::new()
}

fn get_state(instance: &rusoto_ec2::Instance) -> String {
    match instance.state {
        Some(ref state) => {
            match state.name {
                Some(ref state_name) => state_name.to_string(),
                None => String::new(),
            }
        },
        None => String::new(),
    }
}

fn list<P, D>(ec2_client: &Ec2Client<P, D>)
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    println!("Name\tInstance ID\t\tState\tAMI ID\t\tPublic IP");
    let request = rusoto_ec2::DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    let result = ec2_client.describe_instances(&request).expect("Error in describe instances");
    if result.reservations.is_some() {
        for reservation in result.reservations.unwrap() {
            if reservation.instances.is_some() {
                for instance in reservation.instances.unwrap() {
                    let name = get_name(&instance);
                    let state = get_state(&instance);
                    let instance_id = instance.instance_id.unwrap();
                    let image_id = instance.image_id.unwrap();
                    let public_ip = instance.public_ip_address.unwrap_or_else(|| "N/A".to_string());
                    println!("{}\t{}\t{}\t{}\t{}", name, instance_id, state, image_id, public_ip);
                }
            }
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
    let mut profile_provider = ProfileProvider::new().expect("Error creating profile provider");
    if ! args.flag_profile.is_empty() {
        profile_provider.set_profile(args.flag_profile);
    }
    let region = match args.flag_region.as_str() {
        "" => Region::from_str("us-east-2"),
        _ => Region::from_str(args.flag_region.as_str()),
    }.expect("Error parsing region name");
    let ec2_client = Ec2Client::new(
        default_tls_client().unwrap(),
        profile_provider,
        region,
    );

    if args.cmd_create {
        eprintln!("Unimplemented");
    }
    else if args.cmd_get_state {
        eprintln!("Unimplemented");
    }
    else if args.cmd_list {
        list(&ec2_client);
    }
    else if args.cmd_ssh {
        eprintln!("Unimplemented");
    }
    else if args.cmd_start {
        eprintln!("Unimplemented");
    }
    else if args.cmd_stop {
        eprintln!("Unimplemented");
    }
}
