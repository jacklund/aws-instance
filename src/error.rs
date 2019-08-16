use rusoto_core::request::HttpDispatchError;
use rusoto_core::{CredentialsError, RusotoError};
use serde_xml_rs;
use snafu::Snafu;
use std::convert::From;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, AwsInstanceError>;

#[derive(Debug, Snafu)]
pub enum AwsInstanceError {
    #[snafu(display("Error parsing region string: {}", source))]
    ParseRegionError {
        source: rusoto_core::region::ParseRegionError,
    },

    #[snafu(display("HTTP Dispatch error: {}", source))]
    HttpDispatch { source: HttpDispatchError },

    #[snafu(display("Credentials error: {}", source))]
    Credentials { source: CredentialsError },

    #[snafu(display("Error in AWS service: {}", message))]
    Service { message: String },

    #[snafu(display("Parse error: {}", message))]
    ParseError { message: String },

    #[snafu(display("Validation error: {}", message))]
    Validation { message: String },

    #[snafu(display("Error with request ID {}: {}", request_id, errors[0]))]
    Unknown {
        errors: Vec<AwsXmlError>,
        request_id: String,
    },

    #[snafu(display("Error describing instances: {}", source))]
    DescribeInstancesError {
        source: rusoto_ec2::DescribeInstancesError,
    },

    #[snafu(display("Error describing images: {}", source))]
    DescribeImagesError {
        source: rusoto_ec2::DescribeImagesError,
    },

    #[snafu(display("Error starting instance {}: {}", instance_name, message))]
    StartInstanceError {
        instance_name: String,
        message: String,
    },

    #[snafu(display("Error stopping instance {}: {}", instance_name, message))]
    StopInstanceError {
        instance_name: String,
        message: String,
    },

    #[snafu(display("Error destroying instance {}: {}", instance_name, message))]
    DestroyInstanceError {
        instance_name: String,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
struct AwsXmlResponse {
    #[serde(rename = "Errors")]
    errors: Vec<AwsXmlErrors>,

    #[serde(rename = "RequestID")]
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct AwsXmlErrors {
    #[serde(rename = "Error")]
    error: AwsXmlError,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AwsXmlError {
    #[serde(rename = "Code")]
    code: Option<String>,

    #[serde(rename = "Message")]
    message: Option<String>,
}

impl Display for AwsXmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nError code: {}",
            self.message.clone().unwrap_or("None".into()),
            self.code.clone().unwrap_or("None".into())
        )
    }
}

impl From<rusoto_core::region::ParseRegionError> for AwsInstanceError {
    fn from(e: rusoto_core::region::ParseRegionError) -> Self {
        AwsInstanceError::ParseRegionError { source: e }
    }
}

impl<T> From<RusotoError<T>> for AwsInstanceError
where
    T: std::error::Error,
{
    fn from(e: RusotoError<T>) -> Self {
        match e {
            RusotoError::Service(inner) => AwsInstanceError::Service {
                message: inner.description().into(),
            },
            RusotoError::HttpDispatch(error) => AwsInstanceError::HttpDispatch { source: error },
            RusotoError::Credentials(error) => AwsInstanceError::Credentials { source: error },
            RusotoError::ParseError(msg) => AwsInstanceError::ParseError { message: msg },
            RusotoError::Validation(msg) => AwsInstanceError::Validation { message: msg },
            RusotoError::Unknown(response) => {
                let xml_response: AwsXmlResponse =
                    serde_xml_rs::de::from_str(response.body_as_str()).unwrap();
                AwsInstanceError::Unknown {
                    errors: xml_response
                        .errors
                        .iter()
                        .map(|e| e.error.clone())
                        .collect(),
                    request_id: xml_response.request_id,
                }
            }
        }
    }
}

impl From<rusoto_ec2::DescribeInstancesError> for AwsInstanceError {
    fn from(e: rusoto_ec2::DescribeInstancesError) -> Self {
        AwsInstanceError::DescribeInstancesError { source: e }
    }
}

impl From<rusoto_ec2::DescribeImagesError> for AwsInstanceError {
    fn from(e: rusoto_ec2::DescribeImagesError) -> Self {
        AwsInstanceError::DescribeImagesError { source: e }
    }
}
