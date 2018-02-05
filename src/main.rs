#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

#[macro_use]
mod util;
mod ec2_wrapper;
mod list;
mod ssh;
mod start;

use docopt::Docopt;
use rusoto_core::{default_tls_client, Region};
use rusoto_credential::ProfileProvider;
use rusoto_ec2::Ec2Client;
use std::str::FromStr;

const USAGE: &'static str = "
Manage AWS instances

Usage:
    aws-instance [options] list
    aws-instance [options] create <name>
    aws-instance [options] start <name>
    aws-instance [options] ssh <name> [-- <sshopt>...]
    aws-instance [options] stop <name>
    aws-instance --help

Options:
  -h --help                    Show this screen.
  -d --debug                   Print information about what it's doing
  -r --region <region>         Set the region [default: us-east-2]
  -p --profile <profile_name>  Set the profile name to use
";

pub static mut DEBUG: bool = false;

#[derive(Debug, Deserialize)]
struct Args {
    cmd_list: bool,
    cmd_create: bool,
    cmd_start: bool,
    cmd_ssh: bool,
    cmd_stop: bool,
    arg_name: Vec<String>,
    flag_debug: bool,
    flag_profile: String,
    flag_region: String,
    arg_sshopt: Vec<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    unsafe {
        DEBUG = args.flag_debug;
    }

    debug!("Creating profile provider");
    let mut profile_provider = ProfileProvider::new().expect("Error creating profile provider");
    if ! args.flag_profile.is_empty() {
        profile_provider.set_profile(args.flag_profile);
    }

    debug!("Parsing region string '{}'", args.flag_region);
    let region = Region::from_str(args.flag_region.as_str()).expect("Error parsing region name");

    debug!("Instantiating Ec2Client");
    let ec2_client = Ec2Client::new(
        default_tls_client().unwrap(),
        profile_provider,
        region,
    );
    let ec2_wrapper = ec2_wrapper::AwsEc2Client::new(ec2_client);

    if args.cmd_create {
        eprintln!("Unimplemented");
    }
    else if args.cmd_list {
        debug!("Calling list::list");
        if let Err(error) = list::list(&ec2_wrapper) {
            eprintln!("{:?}", error);
        }
    }
    else if args.cmd_ssh {
        debug!("Calling ssh::ssh");
        if let Err(error) = ssh::ssh(&ec2_wrapper, &args.arg_name[0], &args.arg_sshopt) {
            eprintln!("{:?}", error);
        }
    }
    else if args.cmd_start {
        debug!("Calling start::start");
        if let Err(error) = start::start(&ec2_wrapper, &args.arg_name[0]) {
            eprintln!("{:?}", error);
        }
    }
    else if args.cmd_stop {
        eprintln!("Unimplemented");
    }
}
