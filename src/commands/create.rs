use crate::{cmdline::OsNames, util, AwsInstanceError, Result};
use rusoto_ec2::{
    Ec2, Ec2Client, IamInstanceProfileSpecification, Reservation, RunInstancesRequest, Tag,
    TagSpecification,
};

#[derive(Debug)]
pub struct CreateOptions {
    pub name: String,
    pub ami_id: String,
    pub ebs_optimized: bool,
    pub iam_profile: Option<String>,
    pub instance_type: Option<String>,
    pub keypair_name: Option<String>,
    pub security_group_ids: Vec<String>,
    pub os_name: Option<OsNames>,
}

pub async fn create_instance(
    ec2_client: &Ec2Client,
    options: CreateOptions,
) -> Result<Reservation> {
    match util::get_instance_by_name(ec2_client, &options.name).await? {
        Some(_) => Err(AwsInstanceError::CreateInstanceError {
            instance_name: options.name,
            message: "Instance with that name already exists".into(),
        }),
        None => {
            let iam_instance_profile = IamInstanceProfileSpecification {
                name: options.iam_profile,
                ..Default::default()
            };
            let mut tags = vec![Tag {
                key: Some("Name".to_string()),
                value: Some(options.name.to_string()),
            }];
            if options.os_name.is_some() {
                tags.push(Tag {
                    key: Some("OS".to_string()),
                    value: Some(options.os_name.unwrap().to_string()),
                });
            }
            let name_tag_spec = TagSpecification {
                resource_type: Some("instance".to_string()),
                tags: Some(tags),
            };
            let request = RunInstancesRequest {
                min_count: 1,
                max_count: 1,
                image_id: Some(options.ami_id),
                ebs_optimized: Some(options.ebs_optimized),
                iam_instance_profile: Some(iam_instance_profile),
                instance_type: options.instance_type,
                key_name: options.keypair_name,
                security_group_ids: Some(options.security_group_ids),
                tag_specifications: Some(vec![name_tag_spec]),
                ..Default::default()
            };
            Ok(ec2_client.run_instances(request).await?)
        }
    }
}
