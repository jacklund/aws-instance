use failure::Fail;
use rusoto_core::region::ParseRegionError;
use rusoto_core::request::HttpDispatchError;
use rusoto_core::{CredentialsError, RusotoError};
use rusoto_ec2::{DescribeImagesError, DescribeInstancesError};
use std::convert::From;

pub type Result<T> = std::result::Result<T, AwsInstanceError>;

#[derive(Debug, Fail)]
pub enum AwsInstanceError {
    #[fail(display = "{}", _0)]
    ParseRegionError(#[cause] ParseRegionError),

    #[fail(display = "{}", _0)]
    HttpDispatch(#[cause] HttpDispatchError),

    #[fail(display = "{}", _0)]
    Credentials(#[cause] CredentialsError),

    #[fail(display = "ServiceError: {}", _0)]
    Service(String),

    #[fail(display = "ParseError: {}", _0)]
    ParseError(String),

    #[fail(display = "Validation: {}", _0)]
    Validation(String),

    #[fail(display = "Unknown Error: {}", _0)]
    Unknown(String),

    #[fail(display = "{}", _0)]
    DescribeInstancesError(#[cause] DescribeInstancesError),

    #[fail(display = "{}", _0)]
    DescribeImagesError(#[cause] DescribeImagesError),
}

impl From<ParseRegionError> for AwsInstanceError {
    fn from(e: ParseRegionError) -> Self {
        AwsInstanceError::ParseRegionError(e)
    }
}

impl<T> From<RusotoError<T>> for AwsInstanceError
where
    T: std::error::Error,
{
    fn from(e: RusotoError<T>) -> Self {
        match e {
            RusotoError::Service(inner) => AwsInstanceError::Service(inner.description().into()),
            RusotoError::HttpDispatch(error) => AwsInstanceError::HttpDispatch(error),
            RusotoError::Credentials(error) => AwsInstanceError::Credentials(error),
            RusotoError::ParseError(msg) => AwsInstanceError::ParseError(msg),
            RusotoError::Validation(msg) => AwsInstanceError::Validation(msg),
            RusotoError::Unknown(response) => {
                AwsInstanceError::Unknown(response.body_as_str().into())
            }
        }
    }
}

impl From<DescribeInstancesError> for AwsInstanceError {
    fn from(e: DescribeInstancesError) -> Self {
        AwsInstanceError::DescribeInstancesError(e)
    }
}

impl From<DescribeImagesError> for AwsInstanceError {
    fn from(e: DescribeImagesError) -> Self {
        AwsInstanceError::DescribeImagesError(e)
    }
}
