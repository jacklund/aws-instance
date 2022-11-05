use crate::{util, Result};
use rusoto_ec2::Ec2Client;
use serde_json::{Map, Value};

pub async fn list(ec2_client: &Ec2Client, ansible: bool) -> Result<()> {
    let instances = util::get_all_instances(ec2_client).await?;
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
            "{0: <15} {1: <25} {2: <15} {3: <25} {4: <15} {5: <15} {6: <15}",
            "Name", "Instance ID", "State", "AMI ID", "OS", "Public IP", "Security Groups"
        );
        for instance in instances {
            let name = util::get_name(&instance);
            let state = util::get_state(&instance);
            let instance_id = instance.instance_id.clone().unwrap();
            let image_id = instance.image_id.clone().unwrap();
            let os: String = instance
                .tags
                .map(|v| {
                    v.iter()
                        .find(|t| t.key.is_some() && t.key.clone().unwrap() == "OS")
                        .map(|t| t.value.clone().unwrap())
                        .unwrap_or_else(|| "N/A".to_string())
                })
                .unwrap_or_else(|| "N/A".to_string());
            let public_ip = instance
                .public_ip_address
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let security_groups: String = instance
                .security_groups
                .map(|v| {
                    v.iter()
                        .map(|gid| gid.clone().group_name)
                        .map(|gname| gname.unwrap_or_else(|| "N/A".to_string()))
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "None".to_string());
            println!(
                "{0: <15} {1: <25} {2: <15} {3: <25} {4: <15} {5: <15} {6: <15}",
                name, instance_id, state, image_id, os, public_ip, security_groups,
            );
        }
    }

    Ok(())
}
