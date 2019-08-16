use crate::{ec2_wrapper, print_state_changes, util, AwsInstanceError, Result};
use rusoto_ec2;

pub fn destroy_instance(ec2_client: &ec2_wrapper::Ec2Wrapper, name: &str) -> Result<()> {
    match util::get_instance_by_name(ec2_client, name)? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::TerminateInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            let result = ec2_client.terminate_instances(request)?;
            if let Some(state_changes) = result.terminating_instances {
                print_state_changes(state_changes);
            } else {
                return Err(AwsInstanceError::DestroyInstanceError {
                    instance_name: name.into(),
                    message: "No state change returned".into(),
                });
            }
        }
        None => {
            return Err(AwsInstanceError::DestroyInstanceError {
                instance_name: name.into(),
                message: "Instance not found".into(),
            })
        }
    }
    Ok(())
}
