use super::{ec2_wrapper, util};
use rusoto_ec2;
use std::error::Error;

pub fn destroy_instance(
    ec2_client: &ec2_wrapper::Ec2Wrapper,
    name: &str,
) -> Result<(), Box<Error>> {
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::TerminateInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            let result = ec2_client.terminate_instances(request)?;
            if let Some(state_changes) = result.terminating_instances {
                for state_change in state_changes {
                    println!(
                        "{}: {} => {}",
                        state_change.instance_id.unwrap(),
                        state_change.previous_state.unwrap().name.unwrap(),
                        state_change.current_state.unwrap().name.unwrap()
                    );
                }
            } else {
                println!("No state change returned");
            }
        }
        None => eprintln!("No instance named '{}' found", name),
    }
    Ok(())
}
