extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_ec2::{Ec2, Ec2Client};
use super::util;

pub fn list<P, D>(ec2_client: &Ec2Client<P, D>)
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    println!("Name\tInstance ID\t\tState\tAMI ID\t\tPublic IP");
    let request = rusoto_ec2::DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    let result = ec2_client.describe_instances(&request).expect("Error in describe instances");
    if result.reservations.is_some() {
        for reservation in result.reservations.unwrap() {
            if reservation.instances.is_some() {
                for instance in reservation.instances.unwrap() {
                    let name = util::get_name(&instance);
                    let state = util::get_state(&instance);
                    let instance_id = instance.instance_id.unwrap();
                    let image_id = instance.image_id.unwrap();
                    let public_ip = instance.public_ip_address.unwrap_or_else(|| "N/A".to_string());
                    println!("{}\t{}\t{}\t{}\t{}", name, instance_id, state, image_id, public_ip);
                }
            }
        }
    }
}