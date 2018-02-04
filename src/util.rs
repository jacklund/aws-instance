extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_ec2::Ec2;

pub fn get_name(instance: &rusoto_ec2::Instance) -> String {
    match instance.tags {
        Some(ref tags) => {
            for tag in tags {
                if let Some("Name") = tag.key.as_ref().map(String::as_str) {
                    match tag.value {
                        Some(ref value) => return value.to_string(),
                        None => return String::new(),
                    }
                }
            }
        },
        None => return String::new(),
    }

    String::new()
}

pub fn get_state(instance: &rusoto_ec2::Instance) -> String {
    match instance.state {
        Some(ref state) => {
            match state.name {
                Some(ref state_name) => state_name.to_string(),
                None => String::new(),
            }
        },
        None => String::new(),
    }
}

pub fn get_public_ip_address(instance: &rusoto_ec2::Instance) -> Option<String> {
    instance.public_ip_address.clone()
}

pub fn get_instance_by_name<P, D>(ec2_client: &rusoto_ec2::Ec2Client<P, D>, name: &String) -> Result<Option<rusoto_ec2::Instance>, rusoto_ec2::DescribeInstancesError>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    let mut request = rusoto_ec2::DescribeInstancesRequest::default();
    let filter = rusoto_ec2::Filter {
        name: Some("tag:Name".to_string()),
        values: Some(vec![name.to_string()]),
    };
    request.filters = Some(vec![filter]);

    let result = ec2_client.describe_instances(&request)?;
    let reservations = result.reservations.unwrap();
    let mut instance = None;
    if ! reservations.is_empty() {
        instance = Some(reservations[0].clone().instances.unwrap()[0].clone());
    }
    Ok(instance)
}

pub fn get_all_instances<P, D>(ec2_client: &rusoto_ec2::Ec2Client<P, D>) -> Result<Vec<rusoto_ec2::Instance>, rusoto_ec2::DescribeInstancesError>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    let request = rusoto_ec2::DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    let mut instances = Vec::new();

    let result = ec2_client.describe_instances(&request)?;
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