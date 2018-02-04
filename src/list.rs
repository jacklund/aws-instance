extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_ec2::Ec2Client;
use super::util;

pub fn list<P, D>(ec2_client: &Ec2Client<P, D>) -> Result<(), rusoto_ec2::DescribeInstancesError>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    debug!("Calling get_all_instances");
    let instances = util::get_all_instances(ec2_client)?;
    println!("Name\tInstance ID\t\tState\tAMI ID\t\tPublic IP");
    for instance in instances {
        let name = util::get_name(&instance);
        let state = util::get_state(&instance);
        let instance_id = instance.instance_id.clone().unwrap();
        let image_id = instance.image_id.clone().unwrap();
        let public_ip = util::get_public_ip_address(&instance).unwrap_or_else(|| "N/A".to_string());
        println!("{}\t{}\t{}\t{}\t{}", name, instance_id, state, image_id, public_ip);
    }

    Ok(())
}