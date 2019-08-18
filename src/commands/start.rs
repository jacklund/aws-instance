extern crate rusoto_ec2;

use crate::{ec2_wrapper, print_state_changes, util, AwsInstanceError, Result};

pub fn start(ec2_client: &dyn ec2_wrapper::Ec2Wrapper, name: &str) -> Result<()> {
    debug!("Calling get_instance_by_name({:?})", name);
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::StartInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            debug!("Calling start_instances");
            let result = ec2_client.start_instances(request)?;
            if let Some(state_changes) = result.starting_instances {
                print_state_changes(state_changes);
            } else {
                return Err(AwsInstanceError::StartInstanceError {
                    instance_name: name.into(),
                    message: "No state change returned".into(),
                });
            }
        }
        None => {
            return Err(AwsInstanceError::StartInstanceError {
                instance_name: name.into(),
                message: "Instance not found".into(),
            })
        }
    }
    Ok(())
}
