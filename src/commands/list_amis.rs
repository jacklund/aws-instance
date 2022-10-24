use crate::Result;
use chrono::{DateTime, SecondsFormat, Utc};
use regex::Regex;
use rusoto_ec2::{DescribeImagesRequest, Ec2, Ec2Client, Filter, Image};
use std::collections::HashMap;

struct AmiInfo {
    ami_id: Option<String>,
    state: Option<String>,
    creation_date: Option<DateTime<Utc>>,
    name: Option<String>,
    description: Option<String>,
}

impl AmiInfo {
    fn from_aws(image: Image) -> Result<Self> {
        Ok(AmiInfo {
            ami_id: image.image_id,
            state: image.state,
            creation_date: image
                .creation_date
                .map(|s| s.parse::<DateTime<Utc>>())
                .map_or(Ok(None), |d| d.map(Some))?,
            name: image.name,
            description: image.description,
        })
    }

    fn print(&self) {
        println!(
            "{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
            print_option(&self.ami_id),
            print_option(&self.state),
            print_option(
                &self
                    .creation_date
                    .map(|d| d.to_rfc3339_opts(SecondsFormat::Millis, true))
            ),
            print_option(&self.name),
            print_option(&self.description)
        );
    }
}

fn print_option(option: &Option<String>) -> String {
    match option {
        Some(string) => string.to_string(),
        None => "".to_string(),
    }
}

pub async fn list_amis(
    ec2_client: &Ec2Client,
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

    let mut image_info: Vec<AmiInfo> = vec![];
    let search_regex: Option<Regex> = match search_string {
        None => None,
        Some(ref search) => Some(Regex::new(search)?),
    };
    match ec2_client.describe_images(request).await?.images {
        Some(images) => {
            for image in images {
                match search_regex {
                    None => {
                        image_info.push(AmiInfo::from_aws(image)?);
                    }
                    Some(ref search) => {
                        if let Some(name) = image.clone().name {
                            if search.is_match(&name) {
                                image_info.push(AmiInfo::from_aws(image)?);
                            }
                        }
                    }
                }
            }
            image_info.sort_by(|a, b| b.creation_date.cmp(&a.creation_date));
            println!(
                "{0: <15} {1: <15} {2: <25} {3: <50.48} {4: <25}",
                "AMI ID", "State", "Creation Date", "Name", "Description"
            );
            for image in image_info {
                image.print();
            }
        }
        None => {
            println!("No images found");
        }
    };

    Ok(())
}
