use crate::{print_state_changes, util, AwsInstanceError, Result};
use rusoto_ec2::{Ec2, Ec2Client};

pub async fn destroy_instance(ec2_client: &Ec2Client, name: &str) -> Result<()> {
    match util::get_instance_by_name(ec2_client, name).await? {
        Some(instance) => {
            let instance_id = instance.instance_id.unwrap();

            // Change the instance name tag
            let mut tag_request = rusoto_ec2::CreateTagsRequest::default();
            tag_request.resources = vec![instance_id.clone()];
            tag_request.tags = vec![rusoto_ec2::Tag {
                key: Some("Name".into()),
                value: Some(format!("{}-terminated", name)),
            }];
            ec2_client.create_tags(tag_request).await?;

            // Terminate the instance
            let mut request = rusoto_ec2::TerminateInstancesRequest::default();
            request.instance_ids = vec![instance_id];

            let result = ec2_client.terminate_instances(request).await?;

            // Print the state change
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
