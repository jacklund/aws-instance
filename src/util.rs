use crate::{cmdline::OsNames, Result};
use rusoto_ec2::{Ec2, Ec2Client};
use std::str::FromStr;

pub fn get_name(instance: &rusoto_ec2::Instance) -> String {
    match instance.tags {
        Some(ref tags) => {
            for tag in tags {
                if let Some("Name") = tag.key.as_deref() {
                    match tag.value {
                        Some(ref value) => return value.to_string(),
                        None => return String::new(),
                    }
                }
            }
        }
        None => return String::new(),
    }

    String::new()
}

pub fn get_state(instance: &rusoto_ec2::Instance) -> String {
    match instance.state {
        Some(ref state) => match state.name {
            Some(ref state_name) => state_name.to_string(),
            None => String::new(),
        },
        None => String::new(),
    }
}

pub async fn get_instance_by_name(
    ec2_client: &Ec2Client,
    name: &str,
) -> Result<Option<rusoto_ec2::Instance>> {
    let mut request = rusoto_ec2::DescribeInstancesRequest::default();
    let filter = rusoto_ec2::Filter {
        name: Some("tag:Name".to_string()),
        values: Some(vec![name.to_string()]),
    };
    request.filters = Some(vec![filter]);

    let result = ec2_client.describe_instances(request).await?;
    let reservations = result.reservations.unwrap();
    let instance = if !reservations.is_empty() {
        Some(reservations[0].clone().instances.unwrap()[0].clone())
    } else {
        None
    };

    Ok(instance)
}

pub async fn get_os_for_instance(instance: &rusoto_ec2::Instance) -> Option<OsNames> {
    match instance.tags.clone().map(|v| {
        v.iter()
            .find(|t| t.key.is_some() && t.key.clone().unwrap() == "OS")
            .map(|t| t.value.clone().unwrap())
    }) {
        None => None,
        Some(None) => None,
        Some(Some(value)) => match OsNames::from_str(&value) {
            Ok(value) => Some(value),
            Err(_) => None,
        },
    }
}

pub async fn get_all_instances(ec2_client: &Ec2Client) -> Result<Vec<rusoto_ec2::Instance>> {
    let request = rusoto_ec2::DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    let mut instances = Vec::new();

    let result = ec2_client.describe_instances(request).await?;
    if result.reservations.is_some() {
        for reservation in result.reservations.unwrap() {
            if reservation.instances.is_some() {
                for instance in reservation.instances.unwrap() {
                    instances.push(instance);
                }
            }
        }
    }

    Ok(instances)
}

pub fn print_state_changes(state_changes: Vec<rusoto_ec2::InstanceStateChange>) {
    for state_change in state_changes {
        println!(
            "{}: {} => {}",
            state_change.instance_id.unwrap(),
            state_change.previous_state.unwrap().name.unwrap(),
            state_change.current_state.unwrap().name.unwrap()
        );
    }
}
