use super::ec2_wrapper;
use rusoto_ec2::{Reservation, RunInstancesError, RunInstancesRequest, Tag, TagSpecification};

pub type CreateOptions = RunInstancesRequest;

pub fn create_instance(
    ec2_client: &ec2_wrapper::Ec2Wrapper,
    name: &str,
    mut request: CreateOptions,
) -> Result<Reservation, RunInstancesError> {
    request.min_count = 1;
    request.max_count = 1;
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
    if request.instance_type.is_none() {
        request.instance_type = Some("t2.micro".to_string());
    }
    debug!("Calling run_instances with request: {:?}", request);
    Ok(ec2_client.run_instances(request)?)
}
