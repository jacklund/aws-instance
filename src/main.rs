#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

mod list;
mod util;

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
        list::list(&ec2_client);
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
