extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use crate::{ec2_wrapper, Result};

pub fn get_name(instance: &rusoto_ec2::Instance) -> String {
    match instance.tags {
        Some(ref tags) => {
            for tag in tags {
                if let Some("Name") = tag.key.as_ref().map(String::as_str) {
                    match tag.value {
                        Some(ref value) => return value.to_string(),
                        None => return String::new(),
                    }
                }
            }
        }
        None => return String::new(),
    }

    String::new()
}

pub fn get_state(instance: &rusoto_ec2::Instance) -> String {
    match instance.state {
        Some(ref state) => match state.name {
            Some(ref state_name) => state_name.to_string(),
            None => String::new(),
        },
        None => String::new(),
    }
}

pub fn get_public_ip_address(instance: &rusoto_ec2::Instance) -> Option<String> {
    instance.public_ip_address.clone()
}

pub fn get_instance_by_name(
    ec2_client: &dyn ec2_wrapper::Ec2Wrapper,
    name: &str,
) -> Result<Option<rusoto_ec2::Instance>> {
    let mut request = rusoto_ec2::DescribeInstancesRequest::default();
    let filter = rusoto_ec2::Filter {
        name: Some("tag:Name".to_string()),
        values: Some(vec![name.to_string()]),
    };
    request.filters = Some(vec![filter]);

    let result = ec2_client.describe_instances(request)?;
    let reservations = result.reservations.unwrap();
    let instance = if !reservations.is_empty() {
        Some(reservations[0].clone().instances.unwrap()[0].clone())
    } else {
        None
    };

    Ok(instance)
}

pub fn get_all_instances(
    ec2_client: &dyn ec2_wrapper::Ec2Wrapper,
) -> Result<Vec<rusoto_ec2::Instance>> {
    let request = rusoto_ec2::DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    let mut instances = Vec::new();

    let result = ec2_client.describe_instances(request)?;
    if result.reservations.is_some() {
        for reservation in result.reservations.unwrap() {
            if reservation.instances.is_some() {
                for instance in reservation.instances.unwrap() {
                    instances.push(instance);
                }
            }
        }
    }

    Ok(instances)
}

pub fn print_state_changes(state_changes: Vec<rusoto_ec2::InstanceStateChange>) {
    for state_change in state_changes {
        println!(
            "{}: {} => {}",
            state_change.instance_id.unwrap(),
            state_change.previous_state.unwrap().name.unwrap(),
            state_change.current_state.unwrap().name.unwrap()
        );
    }
}

#[cfg(test)]
mod test {
    use super::{get_all_instances, get_instance_by_name};
    use crate::ec2_wrapper::test::MockEc2Wrapper;
    use rusoto_ec2::{DescribeInstancesRequest, DescribeInstancesResult, Instance, Reservation};

    #[test]
    fn get_all_no_instances_found() {
        let mut good_result = DescribeInstancesResult::default();
        good_result.reservations = Some(vec![]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_all_instances(&ec2_client).unwrap().is_empty());
    }

    #[test]
    fn get_all_one_instance_found() {
        let mut good_result = DescribeInstancesResult::default();
        let mut reservation = Reservation::default();
        reservation.instances = Some(vec![Instance::default()]);
        good_result.reservations = Some(vec![reservation]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_all_instances(&ec2_client).unwrap().len() == 1);
    }

    #[test]
    fn get_all_two_instances_one_reservation() {
        let mut good_result = DescribeInstancesResult::default();
        let mut reservation = Reservation::default();
        reservation.instances = Some(vec![Instance::default(), Instance::default()]);
        good_result.reservations = Some(vec![reservation]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_all_instances(&ec2_client).unwrap().len() == 2);
    }

    #[test]
    fn get_all_two_instances_two_reservations() {
        let mut good_result = DescribeInstancesResult::default();
        let mut reservation1 = Reservation::default();
        reservation1.instances = Some(vec![Instance::default()]);
        let mut reservation2 = Reservation::default();
        reservation2.instances = Some(vec![Instance::default()]);
        good_result.reservations = Some(vec![reservation1, reservation2]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_all_instances(&ec2_client).unwrap().len() == 2);
    }

    #[test]
    fn get_by_name_no_instances_found() {
        let mut good_result = DescribeInstancesResult::default();
        good_result.reservations = Some(vec![]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_instance_by_name(&ec2_client, "Foo").unwrap().is_none());
    }

    #[test]
    fn get_by_name_instance_found() {
        let mut good_result = DescribeInstancesResult::default();
        let mut reservation = Reservation::default();
        reservation.instances = Some(vec![Instance::default()]);
        good_result.reservations = Some(vec![reservation]);
        let good_lambda = move |_: DescribeInstancesRequest| Ok(good_result.clone());
        let mut ec2_client = MockEc2Wrapper::default();
        ec2_client.mock_describe_instances(&good_lambda);
        assert!(get_instance_by_name(&ec2_client, "Foo").unwrap().is_some());
    }
}
