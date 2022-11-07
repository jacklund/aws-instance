use crate::Result;
use rusoto_ec2::{DescribeSecurityGroupsRequest, Ec2, Ec2Client, Filter};

pub async fn list_security_groups(ec2_client: &Ec2Client, name: &Option<String>) -> Result<()> {
    let mut request = DescribeSecurityGroupsRequest::default();
    if name.is_some() {
        request.filters = Some(vec![Filter {
            name: Some("group-name".to_string()),
            values: Some(vec![name.clone().unwrap()]),
        }]);
    }
    let result = ec2_client.describe_security_groups(request).await?;
    println!(
        "{0: <10} {1: <25} {2: <35}",
        "Name", "Group ID", "Description"
    );
    if name.is_some() {
        if let Some(security_groups) = result.security_groups {
            for group in security_groups {
                println!(
                    "{0: <10} {1: <25} {2: <35}",
                    group.group_name.unwrap_or_else(|| "N/A".to_string()),
                    group.group_id.unwrap_or_else(|| "N/A".to_string()),
                    group.description.unwrap_or_else(|| "N/A".to_string()),
                );
                println!("  ingress:");
                for ingress in group.ip_permissions.unwrap_or_default() {
                    let from_port_string = ingress
                        .from_port
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let to_port_string = ingress
                        .to_port
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let port_string = if from_port_string == to_port_string {
                        from_port_string
                    } else {
                        format!("{}:{}", from_port_string, to_port_string)
                    };
                    println!(
                        "    {0: <10} {1: <15}",
                        port_string,
                        ingress
                            .ip_ranges
                            .map(|v| v
                                .iter()
                                .map(|ip| ip.cidr_ip.clone().unwrap_or_else(|| "N/A".to_string()))
                                .collect::<Vec<String>>()
                                .join(", "))
                            .unwrap_or_else(|| "N/A".to_string()),
                    );
                }
                println!("  egress:");
                for egress in group.ip_permissions_egress.unwrap_or_default() {
                    let from_port_string = egress
                        .from_port
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let to_port_string = egress
                        .to_port
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let port_string = if from_port_string == to_port_string {
                        from_port_string
                    } else {
                        format!("{}:{}", from_port_string, to_port_string)
                    };
                    println!(
                        "    {0: <10} {1: <15}",
                        port_string,
                        egress
                            .ip_ranges
                            .map(|v| v
                                .iter()
                                .map(|ip| ip.cidr_ip.clone().unwrap_or_else(|| "N/A".to_string()))
                                .collect::<Vec<String>>()
                                .join(", "))
                            .unwrap_or_else(|| "N/A".to_string()),
                    );
                }
            }
        }
    } else if let Some(security_groups) = result.security_groups {
        for group in security_groups {
            println!(
                "{0: <10} {1: <25} {2: <35}",
                group.group_name.unwrap_or_else(|| "N/A".to_string()),
                group.group_id.unwrap_or_else(|| "N/A".to_string()),
                group.description.unwrap_or_else(|| "N/A".to_string()),
            );
        }
    }
    Ok(())
}
