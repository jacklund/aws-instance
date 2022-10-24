use crate::{print_state_changes, util, AwsInstanceError, Result};
use rusoto_ec2::{Ec2, Ec2Client};

pub async fn start(ec2_client: &Ec2Client, name: &str) -> Result<()> {
    match util::get_instance_by_name(ec2_client, name).await? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();
            let mut request = rusoto_ec2::StartInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            let result = ec2_client.start_instances(request).await?;
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
