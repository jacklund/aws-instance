extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use std::process::{Command, exit};
use super::util;

pub fn ssh<P, D>(ec2_client: &rusoto_ec2::Ec2Client<P, D>, name: &String, ssh_opts: &Vec<String>) -> Result<(), rusoto_ec2::DescribeInstancesError>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
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