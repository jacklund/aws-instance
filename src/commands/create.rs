use crate::{ec2_wrapper, util, AwsInstanceError, Result};
use rusoto_ec2::{
    IamInstanceProfileSpecification, Reservation, RunInstancesRequest, Tag, TagSpecification,
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
}

pub fn create_instance(
    ec2_client: &dyn ec2_wrapper::Ec2Wrapper,
    options: CreateOptions,
) -> Result<Reservation> {
    match util::get_instance_by_name(ec2_client, &options.name)? {
        Some(_) => Err(AwsInstanceError::CreateInstanceError {
            instance_name: options.name,
            message: "Instance with that name already exists".into(),
        }),
        None => {
            let mut request = RunInstancesRequest::default();
            request.min_count = 1;
            request.max_count = 1;
            request.image_id = Some(options.ami_id);
            request.ebs_optimized = Some(options.ebs_optimized);
            let mut iam_instance_profile = IamInstanceProfileSpecification::default();
            iam_instance_profile.name = options.iam_profile;
            request.iam_instance_profile = Some(iam_instance_profile);
            request.instance_type = options.instance_type;
            request.key_name = options.keypair_name;
            request.security_group_ids = Some(options.security_group_ids);
            let name_tag_spec = TagSpecification {
                resource_type: Some("instance".to_string()),
                tags: Some(vec![Tag {
                    key: Some("Name".to_string()),
                    value: Some(options.name.to_string()),
                }]),
            };
            match request.tag_specifications {
                Some(tag_spec) => {
                    let mut my_tag_spec = tag_spec.clone();
                    my_tag_spec.push(name_tag_spec);
                    request.tag_specifications = Some(my_tag_spec);
                }
                None => {
                    request.tag_specifications = Some(vec![name_tag_spec]);
                }
            }
            debug!("Calling run_instances with request: {:?}", request);
            Ok(ec2_client.run_instances(request)?)
        }
    }
}
