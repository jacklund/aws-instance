// Wrapper class for ec2_client. I wrap it so that I can mock it (below), which allows me to test
// the util functions
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

use rusoto_core::request::DispatchSignedRequest;
use rusoto_credential::ProvideAwsCredentials;
use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client, StartInstancesError, StartInstancesRequest, StartInstancesResult};

pub trait Ec2Wrapper {
    fn describe_instances(&self, input: &DescribeInstancesRequest)
        -> Result<DescribeInstancesResult, DescribeInstancesError>;
    fn start_instances(&self, input: &StartInstancesRequest)
        -> Result<StartInstancesResult, StartInstancesError>;
}

pub struct AwsEc2Client<P, D>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    ec2_client: Ec2Client<P, D>,
}

impl <P, D> AwsEc2Client<P, D>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
    pub fn new(ec2_client: Ec2Client<P, D>) -> AwsEc2Client<P, D> {
        AwsEc2Client {
            ec2_client: ec2_client,
        }
    }
}

impl <P, D> Ec2Wrapper for AwsEc2Client<P, D>
    where
        P: ProvideAwsCredentials,
        D: DispatchSignedRequest
{
   fn describe_instances(&self, input: &DescribeInstancesRequest)
        -> Result<DescribeInstancesResult, DescribeInstancesError>
    {
        self.ec2_client.describe_instances(input)
    }

    fn start_instances(&self, input: &StartInstancesRequest)
        -> Result<StartInstancesResult, StartInstancesError>
    {
        self.ec2_client.start_instances(input)
    }
}


// Ec2Client mock wrapper. I use closures here rather than just passing in the results because of a Rust issue
// (https://github.com/rust-lang/rust/issues/26925) which gives an error if you try to use .clone() on
// Result.
#[cfg(test)]
pub mod test {
    use ec2_wrapper::Ec2Wrapper;
    use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult,
        StartInstancesError, StartInstancesRequest, StartInstancesResult};

    type DescribeInstancesLambda = Fn(&DescribeInstancesRequest) -> Result<DescribeInstancesResult, DescribeInstancesError>;
    type StartInstancesLambda = Fn(&StartInstancesRequest) -> Result<StartInstancesResult, StartInstancesError>;
    
    #[derive(Default)]
    pub struct MockEc2Wrapper<'a> {
        describe_instances_lambda: Option<&'a DescribeInstancesLambda>,
        start_instances_lambda: Option<&'a StartInstancesLambda>,
    }

    impl <'a> MockEc2Wrapper<'a> {
        pub fn mock_describe_instances(&mut self, closure: &'a DescribeInstancesLambda) {
            self.describe_instances_lambda = Some(closure);
        }

        pub fn mock_start_instances(&mut self, closure: &'a StartInstancesLambda) {
            self.start_instances_lambda = Some(closure);
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
    }
}