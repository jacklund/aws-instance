use crate::{cmdline::OsNames, util, AwsInstanceError, Result};
use lazy_static::lazy_static;
use rusoto_ec2::Ec2Client;
use std::collections::HashMap;
use std::process::{exit, Command};

lazy_static! {
    static ref USERNAME_MAP: HashMap<OsNames, &'static str> = {
        let mut m = HashMap::new();
        m.insert(OsNames::CentOS, "centos");
        m.insert(OsNames::Debian, "admin");
        m.insert(OsNames::Fedora, "fedora");
        m.insert(OsNames::Ubuntu, "ubuntu");
        m
    };
}

const DEFAULT_USERNAME: &str = "ec2-user";

pub async fn ssh(
    ec2_client: &Ec2Client,
    name: &str,
    username: &Option<String>,
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
    let ip_address = match instance.clone().public_ip_address {
        Some(ip_address) => ip_address,
        None => {
            return Err(AwsInstanceError::IPAddressNotFoundError {
                instance_name: name.into(),
            });
        }
    };

    let username = match username {
        Some(username) => username,
        None => {
            let username = match util::get_os_for_instance(&instance).await {
                Some(os) => match USERNAME_MAP.get(&os) {
                    Some(username) => username,
                    None => DEFAULT_USERNAME,
                },
                None => DEFAULT_USERNAME,
            };
            println!("Attempting to log in using username '{}'", username);
            username
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
