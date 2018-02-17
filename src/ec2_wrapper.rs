// Wrapper class for ec2_client. I wrap it so that I can mock it (below), which allows me to test
// the util functions
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::{default_tls_client, Region};
use rusoto_credential::{DefaultCredentialsProvider, ProfileProvider};
use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client,
    StartInstancesError, StartInstancesRequest, StartInstancesResult, StopInstancesError, StopInstancesRequest, StopInstancesResult};

pub trait Ec2Wrapper {
    fn describe_instances(&self, input: &DescribeInstancesRequest)
        -> Result<DescribeInstancesResult, DescribeInstancesError>;
    fn start_instances(&self, input: &StartInstancesRequest)
        -> Result<StartInstancesResult, StartInstancesError>;
    fn stop_instances(&self, input: &StopInstancesRequest)
        -> Result<StopInstancesResult, StopInstancesError>;
}

pub struct AwsEc2Client {
    ec2: Box<Ec2>,
}

impl AwsEc2Client {
    pub fn new(region: Region, profile: &str) -> AwsEc2Client {
        debug!("Creating profile provider");
        let mut profile_provider = ProfileProvider::new().expect("Error creating profile provider");
        if ! profile.is_empty() {
            profile_provider.set_profile(profile);
            AwsEc2Client {
                ec2: Box::new(Ec2Client::new(
                    default_tls_client().unwrap(),
                    profile_provider,
                    region,
                )),
            }
        } else {
            AwsEc2Client {
                ec2: Box::new(Ec2Client::new(
                    default_tls_client().unwrap(),
                    DefaultCredentialsProvider::new().unwrap(),
                    region,
                )),
            }
        }
    }
}

impl Ec2Wrapper for AwsEc2Client {
   fn describe_instances(&self, input: &DescribeInstancesRequest)
        -> Result<DescribeInstancesResult, DescribeInstancesError>
    {
        self.ec2.describe_instances(input)
    }

    fn start_instances(&self, input: &StartInstancesRequest)
        -> Result<StartInstancesResult, StartInstancesError>
    {
        self.ec2.start_instances(input)
    }

    fn stop_instances(&self, input: &StopInstancesRequest)
        -> Result<StopInstancesResult, StopInstancesError>
    {
        self.ec2.stop_instances(input)
    }
}


// Ec2Client mock wrapper. I use closures here rather than just passing in the results because of a Rust issue
// (https://github.com/rust-lang/rust/issues/26925) which gives an error if you try to use .clone() on
// Result.
#[cfg(test)]
pub mod test {
    use ec2_wrapper::Ec2Wrapper;
    use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult,
        StartInstancesError, StartInstancesRequest, StartInstancesResult, StopInstancesError, StopInstancesRequest, StopInstancesResult};

    type DescribeInstancesLambda = Fn(&DescribeInstancesRequest) -> Result<DescribeInstancesResult, DescribeInstancesError>;
    type StartInstancesLambda = Fn(&StartInstancesRequest) -> Result<StartInstancesResult, StartInstancesError>;
    type StopInstancesLambda = Fn(&StopInstancesRequest) -> Result<StopInstancesResult, StopInstancesError>;
    
    #[derive(Default)]
    pub struct MockEc2Wrapper<'a> {
        describe_instances_lambda: Option<&'a DescribeInstancesLambda>,
        start_instances_lambda: Option<&'a StartInstancesLambda>,
        stop_instances_lambda: Option<&'a StopInstancesLambda>,
    }

    impl <'a> MockEc2Wrapper<'a> {
        pub fn mock_describe_instances(&mut self, closure: &'a DescribeInstancesLambda) {
            self.describe_instances_lambda = Some(closure);
        }

        pub fn mock_start_instances(&mut self, closure: &'a StartInstancesLambda) {
            self.start_instances_lambda = Some(closure);
        }

        pub fn mock_stop_instances(&mut self, closure: &'a StopInstancesLambda) {
            self.stop_instances_lambda = Some(closure);
        }
    }

    impl <'a> Ec2Wrapper for MockEc2Wrapper<'a> {
        fn describe_instances(&self, input: &DescribeInstancesRequest)
                -> Result<DescribeInstancesResult, DescribeInstancesError>
        {
            (self.describe_instances_lambda.unwrap())(input)
        }

        fn start_instances(&self, input: &StartInstancesRequest) -> Result<StartInstancesResult, StartInstancesError>
        {
            (self.start_instances_lambda.unwrap())(input)
        }

        fn stop_instances(&self, input: &StopInstancesRequest) -> Result<StopInstancesResult, StopInstancesError>
        {
            (self.stop_instances_lambda.unwrap())(input)
        }
    }
}