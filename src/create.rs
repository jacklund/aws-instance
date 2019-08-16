use crate::{ec2_wrapper, Result};
use rusoto_ec2::{
    IamInstanceProfileSpecification, Reservation, RunInstancesRequest, Tag, TagSpecification,
};

pub fn create_instance(
    ec2_client: &ec2_wrapper::Ec2Wrapper,
    name: &str,
    ami_id: &str,
    ebs_optimized: bool,
    iam_profile: Option<String>,
    instance_type: Option<String>,
    keypair_name: Option<String>,
    security_group_ids: Vec<String>,
) -> Result<Reservation> {
    let mut request = RunInstancesRequest::default();
    request.min_count = 1;
    request.max_count = 1;
    request.image_id = Some(ami_id.into());
    request.ebs_optimized = Some(ebs_optimized);
    let mut iam_instance_profile = IamInstanceProfileSpecification::default();
    iam_instance_profile.name = iam_profile;
    request.iam_instance_profile = Some(iam_instance_profile);
    request.instance_type = instance_type;
    request.key_name = keypair_name;
    request.security_group_ids = Some(security_group_ids);
    let name_tag_spec = TagSpecification {
        resource_type: Some("instance".to_string()),
        tags: Some(vec![Tag {
            key: Some("Name".to_string()),
            value: Some(name.to_string()),
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
