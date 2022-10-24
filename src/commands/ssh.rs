use crate::{util, AwsInstanceError, Result};
use rusoto_ec2::Ec2Client;
use std::process::{exit, Command};

pub async fn ssh(
    ec2_client: &Ec2Client,
    name: &str,
    username: &str,
    ssh_opts: &[String],
) -> Result<()> {
    let instance = match util::get_instance_by_name(ec2_client, name).await? {
        Some(instance) => instance,
        None => {
            return Err(AwsInstanceError::InstanceNotFoundError {
                instance_name: name.into(),
            });
        }
    };
    let ip_address = match instance.public_ip_address {
        Some(ip_address) => ip_address,
        None => {
            return Err(AwsInstanceError::IPAddressNotFoundError {
                instance_name: name.into(),
            });
        }
    };

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
