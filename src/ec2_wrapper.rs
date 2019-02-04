// Wrapper class for ec2_client. I wrap it so that I can mock it (below), which allows me to test
// the util functions
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::{HttpClient, Region};
use rusoto_credential::{DefaultCredentialsProvider, ProfileProvider};
use rusoto_ec2::{
    DescribeImagesError, DescribeImagesRequest, DescribeImagesResult, DescribeInstancesError,
    DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client, Reservation,
    RunInstancesError, RunInstancesRequest, StartInstancesError, StartInstancesRequest,
    StartInstancesResult, StopInstancesError, StopInstancesRequest, StopInstancesResult,
    TerminateInstancesError, TerminateInstancesRequest, TerminateInstancesResult,
};

pub trait Ec2Wrapper {
    fn describe_images(
        &self,
        input: DescribeImagesRequest,
    ) -> Result<DescribeImagesResult, DescribeImagesError>;
    fn describe_instances(
        &self,
        input: DescribeInstancesRequest,
    ) -> Result<DescribeInstancesResult, DescribeInstancesError>;
    fn run_instances(&self, input: RunInstancesRequest) -> Result<Reservation, RunInstancesError>;
    fn start_instances(
        &self,
        input: StartInstancesRequest,
    ) -> Result<StartInstancesResult, StartInstancesError>;
    fn stop_instances(
        &self,
        input: StopInstancesRequest,
    ) -> Result<StopInstancesResult, StopInstancesError>;
    fn terminate_instances(
        &self,
        input: TerminateInstancesRequest,
    ) -> Result<TerminateInstancesResult, TerminateInstancesError>;
}

pub struct AwsEc2Client {
    ec2: Box<Ec2>,
}

impl AwsEc2Client {
    pub fn new(region: Region, profile: &str) -> AwsEc2Client {
        debug!("Creating profile provider");
        let mut profile_provider = ProfileProvider::new().expect("Error creating profile provider");
        if !profile.is_empty() {
            profile_provider.set_profile(profile);
            AwsEc2Client {
                ec2: Box::new(Ec2Client::new_with(
                    HttpClient::new().unwrap(),
                    profile_provider,
                    region,
                )),
            }
        } else {
            AwsEc2Client {
                ec2: Box::new(Ec2Client::new_with(
                    HttpClient::new().unwrap(),
                    DefaultCredentialsProvider::new().unwrap(),
                    region,
                )),
            }
        }
    }
}

impl Ec2Wrapper for AwsEc2Client {
    fn describe_images(
        &self,
        input: DescribeImagesRequest,
    ) -> Result<DescribeImagesResult, DescribeImagesError> {
        self.ec2.describe_images(input).sync()
    }

    fn describe_instances(
        &self,
        input: DescribeInstancesRequest,
    ) -> Result<DescribeInstancesResult, DescribeInstancesError> {
        self.ec2.describe_instances(input).sync()
    }

    fn run_instances(&self, input: RunInstancesRequest) -> Result<Reservation, RunInstancesError> {
        self.ec2.run_instances(input).sync()
    }

    fn start_instances(
        &self,
        input: StartInstancesRequest,
    ) -> Result<StartInstancesResult, StartInstancesError> {
        self.ec2.start_instances(input).sync()
    }

    fn stop_instances(
        &self,
        input: StopInstancesRequest,
    ) -> Result<StopInstancesResult, StopInstancesError> {
        self.ec2.stop_instances(input).sync()
    }

    fn terminate_instances(
        &self,
        input: TerminateInstancesRequest,
    ) -> Result<TerminateInstancesResult, TerminateInstancesError> {
        self.ec2.terminate_instances(input).sync()
    }
}

// Ec2Client mock wrapper. I use closures here rather than just passing in the results because of a Rust issue
// (https://github.com/rust-lang/rust/issues/26925) which gives an error if you try to use .clone() on
// Result.
#[cfg(test)]
pub mod test {
    use crate::ec2_wrapper::Ec2Wrapper;
    use rusoto_ec2::{
        DescribeImagesError, DescribeImagesRequest, DescribeImagesResult, DescribeInstancesError,
        DescribeInstancesRequest, DescribeInstancesResult, Reservation, RunInstancesError,
        RunInstancesRequest, StartInstancesError, StartInstancesRequest, StartInstancesResult,
        StopInstancesError, StopInstancesRequest, StopInstancesResult, TerminateInstancesError,
        TerminateInstancesRequest, TerminateInstancesResult,
    };

    type DescribeImagesLambda =
        Fn(DescribeImagesRequest) -> Result<DescribeImagesResult, DescribeImagesError>;
    type DescribeInstancesLambda =
        Fn(DescribeInstancesRequest) -> Result<DescribeInstancesResult, DescribeInstancesError>;
    type RunInstancesLambda = Fn(RunInstancesRequest) -> Result<Reservation, RunInstancesError>;
    type StartInstancesLambda =
        Fn(StartInstancesRequest) -> Result<StartInstancesResult, StartInstancesError>;
    type StopInstancesLambda =
        Fn(StopInstancesRequest) -> Result<StopInstancesResult, StopInstancesError>;
    type TerminateInstancesLambda =
        Fn(TerminateInstancesRequest) -> Result<TerminateInstancesResult, TerminateInstancesError>;

    #[derive(Default)]
    pub struct MockEc2Wrapper<'a> {
        describe_images_lambda: Option<&'a DescribeImagesLambda>,
        describe_instances_lambda: Option<&'a DescribeInstancesLambda>,
        run_instances_lambda: Option<&'a RunInstancesLambda>,
        start_instances_lambda: Option<&'a StartInstancesLambda>,
        stop_instances_lambda: Option<&'a StopInstancesLambda>,
        terminate_instances_lambda: Option<&'a TerminateInstancesLambda>,
    }

    impl<'a> MockEc2Wrapper<'a> {
        pub fn mock_describe_images(&mut self, closure: &'a DescribeImagesLambda) {
            self.describe_images_lambda = Some(closure);
        }

        pub fn mock_describe_instances(&mut self, closure: &'a DescribeInstancesLambda) {
            self.describe_instances_lambda = Some(closure);
        }

        pub fn mock_run_instances(&mut self, closure: &'a RunInstancesLambda) {
            self.run_instances_lambda = Some(closure);
        }

        pub fn mock_start_instances(&mut self, closure: &'a StartInstancesLambda) {
            self.start_instances_lambda = Some(closure);
        }

        pub fn mock_stop_instances(&mut self, closure: &'a StopInstancesLambda) {
            self.stop_instances_lambda = Some(closure);
        }

        pub fn mock_terminate_instances(&mut self, closure: &'a TerminateInstancesLambda) {
            self.terminate_instances_lambda = Some(closure);
        }
    }

    impl<'a> Ec2Wrapper for MockEc2Wrapper<'a> {
        fn describe_images(
            &self,
            input: DescribeImagesRequest,
        ) -> Result<DescribeImagesResult, DescribeImagesError> {
            (self.describe_images_lambda.unwrap())(input)
        }

        fn describe_instances(
            &self,
            input: DescribeInstancesRequest,
        ) -> Result<DescribeInstancesResult, DescribeInstancesError> {
            (self.describe_instances_lambda.unwrap())(input)
        }

        fn run_instances(
            &self,
            input: RunInstancesRequest,
        ) -> Result<Reservation, RunInstancesError> {
            (self.run_instances_lambda.unwrap())(input)
        }

        fn start_instances(
            &self,
            input: StartInstancesRequest,
        ) -> Result<StartInstancesResult, StartInstancesError> {
            (self.start_instances_lambda.unwrap())(input)
        }

        fn stop_instances(
            &self,
            input: StopInstancesRequest,
        ) -> Result<StopInstancesResult, StopInstancesError> {
            (self.stop_instances_lambda.unwrap())(input)
        }

        fn terminate_instances(
            &self,
            input: TerminateInstancesRequest,
        ) -> Result<TerminateInstancesResult, TerminateInstancesError> {
            (self.terminate_instances_lambda.unwrap())(input)
        }
    }
}
