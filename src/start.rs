extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_ec2::Ec2;
use super::util;

pub fn start<P, D>(ec2_client: &rusoto_ec2::Ec2Client<P, D>, name: &String) -> Result<(), rusoto_ec2::DescribeInstancesError>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::StartInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            if let Ok(result) = ec2_client.start_instances(&request) {
                if let Some(state_changes) = result.starting_instances {
                    for state_change in state_changes {
                        println!("{}: {} => {}", state_change.instance_id.unwrap(), state_change.previous_state.unwrap().name.unwrap(), state_change.current_state.unwrap().name.unwrap());
                    }
                }
            }
        },
        None => eprintln!("No instance named '{}' found", name),
    }
    Ok(())
}