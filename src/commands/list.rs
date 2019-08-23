extern crate rusoto_credential;
extern crate rusoto_ec2;

use crate::{ec2_wrapper, util, Result};
use serde_json::{Map, Value};

pub fn list(ec2_client: &dyn ec2_wrapper::Ec2Wrapper, ansible: bool) -> Result<()> {
    debug!("Calling get_all_instances");
    let instances = util::get_all_instances(ec2_client)?;
    if ansible {
        let mut inventory = Map::new();
        for instance in instances {
            if let Some(ref dns_name) = instance.public_dns_name {
                if !dns_name.is_empty() {
                    let name = util::get_name(&instance);
                    let mut hosts = Map::new();
                    hosts.insert(
                        "hosts".into(),
                        Value::Array(vec![Value::String(instance.public_dns_name.unwrap())]),
                    );
                    inventory.insert(name, Value::Object(hosts));
                }
            }
        }
        println!("{}", serde_json::to_string(&inventory)?);
    } else {
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
