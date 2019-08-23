extern crate rusoto_credential;
extern crate rusoto_ec2;

use crate::{ec2_wrapper, util, Result};

pub fn list(ec2_client: &dyn ec2_wrapper::Ec2Wrapper) -> Result<()> {
    debug!("Calling get_all_instances");
    let instances = util::get_all_instances(ec2_client)?;
    println!(
        "{0: <15} {1: <25} {2: <15} {3: <15} {4: <15}",
        "Name", "Instance ID", "State", "AMI ID", "Public IP"
    );
    for instance in instances {
        let name = util::get_name(&instance);
        let state = util::get_state(&instance);
        let instance_id = instance.instance_id.clone().unwrap();
        let image_id = instance.image_id.clone().unwrap();
        let public_ip = util::get_public_ip_address(&instance).unwrap_or_else(|| "N/A".to_string());
        println!(
            "{0: <15} {1: <25} {2: <15} {3: <25} {4: <15} {5: <15}",
            "Name", "Instance ID", "State", "AMI ID", "Public IP", "Public DNS"
        );
        for instance in instances {
            let name = util::get_name(&instance);
            let state = util::get_state(&instance);
            let instance_id = instance.instance_id.clone().unwrap();
            let image_id = instance.image_id.clone().unwrap();
            let public_ip = instance
                .public_ip_address
                .clone()
                .unwrap_or("N/A".to_string());
            let public_dns = instance
                .public_dns_name
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            println!(
                "{0: <15} {1: <25} {2: <15} {3: <25} {4: <15} {5: <15}",
                name, instance_id, state, image_id, public_ip, public_dns,
            );
        }
    }

    Ok(())
}
