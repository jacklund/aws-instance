use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "aws-instance", about = "Manage AWS instances")]
pub struct CmdLineOptions {
    #[structopt(short, long)]
    /// Set the AWS profile to use
    pub profile: Option<String>,

    #[structopt(short, long)]
    /// Set the AWS region to use
    pub region: Option<String>,

    #[structopt(subcommand)]
    pub subcommand: SubCommands,
}

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    #[structopt(name = "create", about = "Create a named AWS instance")]
    Create {
        #[structopt(name = "NAME")]
        name: String,

        #[structopt(name = "AMI-ID")]
        ami_id: String,

        #[structopt(short, long = "ebs-optimized")]
        ebs_optimized: bool,

        #[structopt(short, long = "iam-profile")]
        iam_profile: Option<String>,

        #[structopt(short = "t", long = "instance-type")]
        instance_type: Option<String>,

        #[structopt(short, long = "keypair")]
        keypair_name: Option<String>,

        #[structopt(short, long = "security-groups")]
        security_group_ids: Vec<String>,
    },

    #[structopt(name = "destroy", about = "Destroy an AWS instance by name")]
    Destroy { name: String },

    #[structopt(name = "list", about = "List AWS instances")]
    List,

    #[structopt(name = "list-amis", about = "List AMIs")]
    ListAmis {
        #[structopt(long, default_value = "x86_64")]
        architecture: String,

        #[structopt(long, name = "image-id")]
        image_id: Option<String>,

        #[structopt(long)]
        search: Option<String>,
    },

    #[structopt(name = "ssh", about = "SSH into an instance")]
    Ssh { name: String, sshopts: Vec<String> },

    #[structopt(name = "start", about = "Start a stopped instance")]
    Start {
        #[structopt(name = "NAME")]
        name: String,
    },

    #[structopt(name = "stop", about = "Stop a running instance")]
    Stop {
        #[structopt(name = "NAME")]
        name: String,
    },
}

pub fn parse_command_line() -> CmdLineOptions {
    CmdLineOptions::from_args()
}
