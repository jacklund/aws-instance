extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use crate::{ec2_wrapper, util, AwsInstanceError, Result};
use std::process::{exit, Command};

pub fn ssh(
    ec2_client: &dyn ec2_wrapper::Ec2Wrapper,
    name: &str,
    username: &str,
    ssh_opts: &[String],
) -> Result<()> {
    debug!("Calling util::get_instance_by_name({:?})", name);
    let instance = match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => instance,
        None => {
            return Err(AwsInstanceError::InstanceNotFoundError {
                instance_name: name.into(),
            });
        }
    };
    debug!("Calling util::get_public_ip_address");
    let ip_address = match util::get_public_ip_address(&instance) {
        Some(ip_address) => ip_address,
        None => {
            return Err(AwsInstanceError::IPAddressNotFoundError {
                instance_name: name.into(),
            });
        }
    };

    debug!(
        "Calling command 'ssh {} -l {} {:?}",
        ip_address, username, ssh_opts
    );
    let mut child = Command::new("ssh")
        .arg(ip_address)
        .args(vec!["-l", username])
        .args(ssh_opts)
        .spawn()
        .expect("SSH Error");

    let status = child.wait().expect("failed to wait on child");

    match status.code() {
        Some(code) => exit(code),
        None => exit(1),
    }
}
