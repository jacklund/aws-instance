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

#[cfg(test)]
mod test {
    use ec2_wrapper::Ec2Wrapper;
    use rusoto_ec2::{DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult, StartInstancesError, StartInstancesRequest, StartInstancesResult};

    type DescribeInstancesLambda = Fn(&DescribeInstancesRequest) -> Result<DescribeInstancesResult, DescribeInstancesError>;
    type StartInstancesLambda = Fn(&StartInstancesRequest) -> Result<StartInstancesResult, StartInstancesError>;

    pub struct MockEc2Wrapper<'a> {
        describe_instances_lambda: &'a mut DescribeInstancesLambda,
        start_instances_lambda: &'a mut StartInstancesLambda,
    }

    impl <'a> MockEc2Wrapper<'a> {
        pub fn mock_describe_instances(&mut self, closure: &'a mut DescribeInstancesLambda) {
            self.describe_instances_lambda = closure;
        }

        pub fn mock_start_instances(&mut self, closure: &'a mut StartInstancesLambda) {
            self.start_instances_lambda = closure;
        }
    }

    impl <'a> Ec2Wrapper for MockEc2Wrapper<'a> {
        fn describe_instances(&self, input: &DescribeInstancesRequest)
                -> Result<DescribeInstancesResult, DescribeInstancesError>
        {
            (self.describe_instances_lambda)(input)
        }

        fn start_instances(&self, input: &StartInstancesRequest)
            -> Result<StartInstancesResult, StartInstancesError>
        {
            (self.start_instances_lambda)(input)
        }
    }
}