extern crate rusoto_ec2;

use super::{ec2_wrapper, util};

pub fn start(ec2_client: &ec2_wrapper::Ec2Wrapper, name: &String) -> Result<(), rusoto_ec2::DescribeInstancesError>
{
    debug!("Calling get_instance_by_name({:?})", name);
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::StartInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            debug!("Calling start_instances");
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