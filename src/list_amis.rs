use crate::{ec2_wrapper, Result};
use rusoto_ec2::{DescribeImagesRequest, Filter, Image};
use std::collections::HashMap;

fn print_option(option: Option<String>) -> String {
    match option {
        Some(string) => string.to_string(),
        None => "".to_string(),
    }
}

pub fn list_amis(
    ec2_client: &ec2_wrapper::Ec2Wrapper,
    filter_values: &HashMap<String, Vec<String>>,
    search_string: Option<String>,
) -> Result<()> {
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

    info!("Request: {:?}", request);

    match ec2_client.describe_images(request)?.images {
        Some(images) => {
            info!("Call returned {} images", images.len());
            println!(
                "{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
                "AMI ID", "State", "Creation Date", "Name", "Description"
            );
            for image in images {
                match search_string {
                    None => print_image(image),
                    Some(ref search) => {
                        if let Some(name) = image.clone().name {
                            if name.contains(search) {
                                print_image(image);
                            }
                        }
                    }
                }
            }
        }
        None => {
            info!("Call returned");
            println!("No images found");
        }
    };

    Ok(())
}
fn print_image(image: Image) {
    println!(
        "{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
        print_option(image.image_id),
        print_option(image.state),
        print_option(image.creation_date),
        print_option(image.name),
        print_option(image.description)
    );
}
