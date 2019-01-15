extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use super::{ec2_wrapper, util};
use std::error::Error;
use std::process::{exit, Command};

pub fn ssh(
    ec2_client: &ec2_wrapper::Ec2Wrapper,
    name: &str,
    ssh_opts: &[&str],
) -> Result<(), Box<Error>> {
    debug!("Calling util::get_instance_by_name({:?})", name);
    let instance = match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => instance,
        None => {
            eprintln!("Instance named '{}' not found", name);
            exit(1);
        }
    };
    debug!("Calling util::get_public_ip_address");
    let ip_address = match util::get_public_ip_address(&instance) {
        Some(ip_address) => ip_address,
        None => {
            eprintln!("No public IP address found for '{}' - is it stopped?", name);
            exit(1);
        }
    };

    debug!("Calling ssh");
    let mut child = Command::new("ssh")
        .arg(ip_address)
        .args(vec!["-l", "admin"])
        .args(ssh_opts)
        .spawn()
        .expect("SSH Error");

    let status = child.wait().expect("failed to wait on child");

    match status.code() {
        Some(code) => exit(code),
        None => exit(1),
    }
}
