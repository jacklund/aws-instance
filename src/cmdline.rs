use std::collections::HashMap;
use structopt::StructOpt;

use crate::commands::create::{create_instance, CreateOptions};
use crate::commands::destroy::destroy_instance;
use crate::commands::list::list;
use crate::commands::list_amis::list_amis;
use crate::commands::ssh::ssh;
use crate::commands::start::start;
use crate::commands::stop::stop;
use crate::Profile;
use crate::Result;
use rusoto_ec2::Ec2Client;

const DEFAULT_INSTANCE_TYPE: &str = "m1.small";

#[derive(Debug, StructOpt)]
#[structopt(name = "aws-instance", about = "Manage AWS instances")]
pub struct CmdLineOptions {
    #[structopt(long = "config-file", short = "C")]
    /// Path to config file
    pub config_file: Option<String>,

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
        /// Instance name
        name: String,

        #[structopt(short, long = "ami-id")]
        /// AMI Image ID to use
        ami_id: String,

        #[structopt(
            short,
            long = "ebs-optimized",
            parse(try_from_str),
            default_value = "false"
        )]
        /// Is it EBS optimized?
        ebs_optimized: bool,

        #[structopt(short, long = "iam-profile")]
        /// IAM profile to use
        iam_profile: Option<String>,

        #[structopt(short = "t", long = "instance-type")]
        /// Instance type [default: m1.small]
        instance_type: Option<String>,

        #[structopt(short, long = "keypair")]
        /// Key pair to use to connect
        keypair_name: Option<String>,

        #[structopt(short, long = "security-groups")]
        /// Security groups for the instance
        security_group_ids: Vec<String>,
    },

    #[structopt(name = "destroy", about = "Destroy an AWS instance by name")]
    Destroy {
        /// Instance name
        name: String,
    },

    #[structopt(name = "list", about = "List AWS instances")]
    List {
        #[structopt(long)]
        /// Whether to output as required by ansible inventory
        ansible: bool,
    },

    #[structopt(name = "list-amis", about = "List AMIs")]
    ListAmis {
        #[structopt(long, short)]
        /// Image name. You may use '?' and '*' to return multiple values
        name: Option<String>,

        #[structopt(long, default_value = "x86_64")]
        /// Instance architecture
        architecture: String,

        #[structopt(long, name = "image-id")]
        /// AMI Image ID
        image_id: Option<String>,

        #[structopt(long)]
        /// Filter images by image name using regular expression
        search: Option<String>,
    },

    #[structopt(name = "ssh", about = "SSH into an instance")]
    Ssh {
        /// Instance name
        name: String,

        #[structopt(long, short)]
        /// User name to log in as
        username: String,

        #[structopt(long, short)]
        /// Path to SSH key to use
        key: Option<String>,

        /// SSH options
        sshopts: Vec<String>,
    },

    #[structopt(name = "start", about = "Start a stopped instance")]
    Start {
        #[structopt(name = "NAME")]
        /// Instance name
        name: String,
    },

    #[structopt(name = "stop", about = "Stop a running instance")]
    Stop {
        #[structopt(name = "NAME")]
        /// Instance name
        name: String,
    },
}

pub fn parse_command_line() -> CmdLineOptions {
    CmdLineOptions::from_args()
}

impl SubCommands {
    pub async fn run(&self, client: &Ec2Client, profile: Profile) -> Result<()> {
        match self {
            SubCommands::List { .. } => {
                self.list(client).await?;
            }

            SubCommands::ListAmis { .. } => {
                self.list_amis(client).await?;
            }

            SubCommands::Create { .. } => {
                self.create(client, profile).await?;
            }

            SubCommands::Destroy { name } => {
                destroy_instance(client, name).await?;
            }

            SubCommands::Ssh { .. } => {
                self.ssh(client, profile).await?;
            }

            SubCommands::Start { name } => {
                start(client, name).await?;
            }

            SubCommands::Stop { name } => {
                stop(client, name).await?;
            }
        }
        Ok(())
    }

    pub async fn list(&self, client: &Ec2Client) -> Result<()> {
        if let SubCommands::List { ansible } = self {
            list(client, *ansible).await?;
        } else {
            panic!("Unexpected value in list: {:?}", self);
        }

        Ok(())
    }

    async fn list_amis(&self, client: &Ec2Client) -> Result<()> {
        if let SubCommands::ListAmis {
            name,
            architecture,
            image_id,
            search,
        } = self
        {
            let mut filters: HashMap<String, Vec<String>> = HashMap::new();
            if let Some(name) = name {
                filters.insert("name".into(), vec![name.into()]);
            }

            filters.insert(
                "architecture".into(),
                architecture.split(',').map(|s| s.into()).collect(),
            );
            if let Some(image_id) = image_id {
                filters.insert(
                    "image_id".into(),
                    image_id.split(',').map(|s| s.into()).collect(),
                );
            }
            list_amis(client, &filters, search.clone()).await?;
        } else {
            panic!("Unexpected value in list_amis: {:?}", self);
        }

        Ok(())
    }

    async fn create(&self, client: &Ec2Client, profile: Profile) -> Result<()> {
        if let SubCommands::Create {
            name,
            ami_id,
            ebs_optimized,
            iam_profile,
            instance_type,
            keypair_name,
            security_group_ids,
        } = self
        {
            let my_security_groups =
                if security_group_ids.is_empty() && profile.security_groups.is_some() {
                    profile.security_groups.unwrap()
                } else {
                    security_group_ids.clone()
                };
            create_instance(
                client,
                CreateOptions {
                    name: name.clone(),
                    ami_id: ami_id.clone(),
                    ebs_optimized: *ebs_optimized,
                    iam_profile: iam_profile.clone(),
                    instance_type: instance_type
                        .clone()
                        .or(profile.default_instance_type)
                        .or(Some(DEFAULT_INSTANCE_TYPE.into())),
                    keypair_name: keypair_name.clone().or(profile.keypair),
                    security_group_ids: my_security_groups,
                },
            )
            .await?;
        } else {
            panic!("Unexpected value in create: {:?}", self);
        }

        Ok(())
    }

    async fn ssh(&self, client: &Ec2Client, profile: Profile) -> Result<()> {
        if let SubCommands::Ssh {
            name,
            username,
            key,
            sshopts,
        } = self
        {
            let mut mysshopts = sshopts.clone();
            if let Some(keypath) = key.clone().or(profile.ssh_key) {
                if !sshopts.contains(&("-i".into())) {
                    mysshopts.push("-i".into());
                    mysshopts.push(keypath);
                }
            }
            ssh(client, name, username, &mysshopts).await?;
        } else {
            panic!("Unexpected value in ssh: {:?}", self);
        }

        Ok(())
    }
}
