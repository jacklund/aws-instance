extern crate rusoto_ec2;

use super::{ec2_wrapper, util};

pub fn stop(ec2_client: &ec2_wrapper::Ec2Wrapper, name: &String) -> Result<(), rusoto_ec2::DescribeInstancesError>
{
    debug!("Calling get_instance_by_name({:?})", name);
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::StopInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            debug!("Calling stop_instances");
            if let Ok(result) = ec2_client.stop_instances(&request) {
                if let Some(state_changes) = result.stopping_instances {
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