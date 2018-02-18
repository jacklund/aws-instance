use rusoto_ec2::{DescribeImagesRequest, Filter};
use std::collections::HashMap;
use std::process::exit;
use super::ec2_wrapper;

fn print_option(option: Option<String>) -> String {
    match option {
        Some(string) => {
            string.to_string()
        },
        None => {
            "".to_string()
        }
    }
}

pub fn list_amis(ec2_client: &ec2_wrapper::Ec2Wrapper, filter_values: HashMap<String, Vec<String>>) {
    let mut request = DescribeImagesRequest::default();
    if !filter_values.is_empty() {
        let mut filters = vec![];
        for (key, values) in filter_values.iter() {
            let mut filter = Filter::default();
            filter.name = Some(key.to_string());
            filter.values = Some(values.to_vec());
            filters.push(filter);
        }
        request.filters = Some(filters);
    }

    match ec2_client.describe_images(&request) {
        Ok(result) => {
            match result.images {
                Some(images) => {
                    println!("{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
                        "AMI ID", "State", "Creation Date", "Name", "Description");
                    for image in images {
                        println!("{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
                            print_option(image.image_id),
                            print_option(image.state),
                            print_option(image.creation_date),
                            print_option(image.name),
                            print_option(image.description));
                    }
                },
                None => {
                    println!("No images found");
                }
            }
        },
        Err(error) => {
            eprintln!("{:?}", error);
            exit(1);
        },
    }
}