use chrono;
use regex;
use rusoto_core::request::HttpDispatchError;
use rusoto_core::RusotoError;
use rusoto_credential::CredentialsError;
use serde::{self, Deserialize};
use serde_json;
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
    RusotoParseError { message: String },

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

    #[snafu(display("Error creating instance {}: {}", instance_name, message))]
    CreateInstanceError {
        instance_name: String,
        message: String,
    },

    #[snafu(display("Error destroying instance {}: {}", instance_name, message))]
    DestroyInstanceError {
        instance_name: String,
        message: String,
    },

    #[snafu(display("Profile named {} not found", profile_name))]
    ProfileNotFoundError { profile_name: String },

    #[snafu(display("Instance named {} not found", instance_name))]
    InstanceNotFoundError { instance_name: String },

    #[snafu(display("Public IP address not found for {} - is it stopped?", instance_name))]
    IPAddressNotFoundError { instance_name: String },

    #[snafu(display("Error parsing date: {}", error))]
    DateParseError { error: chrono::ParseError },

    #[snafu(display("Error parsing search string: {}", error))]
    RegexParseError { error: regex::Error },

    #[snafu(display("Error parsing JSON: {}", error))]
    JSONParseError { error: serde_json::Error },

    #[snafu(display("Blocking error"))]
    Blocking,
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
            self.message.clone().unwrap_or_else(|| "None".into()),
            self.code.clone().unwrap_or_else(|| "None".into())
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
                message: inner.to_string(),
            },
            RusotoError::HttpDispatch(error) => AwsInstanceError::HttpDispatch { source: error },
            RusotoError::Credentials(error) => AwsInstanceError::Credentials { source: error },
            RusotoError::ParseError(msg) => AwsInstanceError::RusotoParseError { message: msg },
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
            RusotoError::Blocking => AwsInstanceError::Blocking,
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

impl From<chrono::ParseError> for AwsInstanceError {
    fn from(e: chrono::ParseError) -> Self {
        AwsInstanceError::DateParseError { error: e }
    }
}

impl From<regex::Error> for AwsInstanceError {
    fn from(e: regex::Error) -> Self {
        AwsInstanceError::RegexParseError { error: e }
    }
}

impl From<serde_json::Error> for AwsInstanceError {
    fn from(e: serde_json::Error) -> Self {
        AwsInstanceError::JSONParseError { error: e }
    }
}
