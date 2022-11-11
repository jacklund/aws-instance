use clap::{Command, CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use std::collections::HashMap;

use crate::commands::create::{create_instance, CreateOptions};
use crate::commands::destroy::destroy_instance;
use crate::commands::list::list;
use crate::commands::list_amis::list_amis;
use crate::commands::list_security_groups::list_security_groups;
use crate::commands::ssh::ssh;
use crate::commands::start::start;
use crate::commands::stop::stop;
use crate::Profile;
use crate::Result;
use rusoto_ec2::Ec2Client;

const DEFAULT_INSTANCE_TYPE: &str = "m1.small";

#[derive(Clone, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum OsNames {
    AmazonLinux,
    CentOS,
    Debian,
    Fedora,
    RHEL,
    Suse,
    Ubuntu,
}

impl std::fmt::Display for OsNames {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OsNames::AmazonLinux => "AmazonLinux",
                OsNames::CentOS => "CentOS",
                OsNames::Debian => "Debian",
                OsNames::Fedora => "Fedora",
                OsNames::RHEL => "RHEL",
                OsNames::Suse => "Suse",
                OsNames::Ubuntu => "Ubuntu",
            }
        )
    }
}

#[derive(Debug)]
pub enum OsNamesError {
    ParseError(String),
}

impl std::fmt::Display for OsNamesError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OsNamesError::ParseError(value) => write!(f, "Error parsing {}", value),
        }
    }
}

impl std::str::FromStr for OsNames {
    type Err = OsNamesError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "amazonlinux" => Ok(OsNames::AmazonLinux),
            "centos" => Ok(OsNames::CentOS),
            "debian" => Ok(OsNames::Debian),
            "fedora" => Ok(OsNames::Fedora),
            "rhel" => Ok(OsNames::RHEL),
            "suse" => Ok(OsNames::Suse),
            "ubuntu" => Ok(OsNames::Ubuntu),
            _ => Err(OsNamesError::ParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "aws-instance", about = "Manage AWS instances")]
pub struct CmdLineOptions {
    #[arg(long = "config-file", short = 'C')]
    /// Path to config file
    pub config_file: Option<String>,

    #[arg(short, long)]
    /// Set the AWS profile to use
    pub profile: Option<String>,

    #[arg(short, long)]
    /// Set the AWS region to use
    pub region: Option<String>,

    #[command(subcommand)]
    pub subcommand: SubCommands,
}

#[derive(Debug, Parser)]
pub enum SubCommands {
    #[command(name = "create", about = "Create a named AWS instance")]
    Create {
        #[arg(name = "NAME")]
        /// Instance name
        name: String,

        #[arg(short, long = "ami-id")]
        /// AMI Image ID to use
        ami_id: String,

        #[arg(short, long = "ebs-optimized", default_value = "false")]
        /// Is it EBS optimized?
        ebs_optimized: bool,

        #[arg(short, long = "iam-profile")]
        /// IAM profile to use
        iam_profile: Option<String>,

        #[arg(short = 't', long = "instance-type")]
        /// Instance type [default: m1.small]
        instance_type: Option<String>,

        #[arg(short, long = "keypair")]
        /// Key pair to use to connect
        keypair_name: Option<String>,

        #[arg(short, long = "security-groups")]
        /// Security groups for the instance
        security_group_ids: Vec<String>,

        #[arg(short, long = "os-name")]
        /// Name of the OS
        os_name: Option<OsNames>,
    },

    #[command(name = "destroy", about = "Destroy an AWS instance by name")]
    Destroy {
        /// Instance name
        name: String,
    },

    #[command(name = "list", about = "List AWS instances")]
    List {
        #[arg(long)]
        /// Whether to output as required by ansible inventory
        ansible: bool,
    },

    #[command(name = "list-amis", about = "List AMIs")]
    ListAmis {
        #[arg(long, short)]
        /// Image name. You may use '?' and '*' to return multiple values
        name: Option<String>,

        #[arg(long, default_value = "x86_64")]
        /// Instance architecture
        architecture: String,

        #[arg(long, name = "image-id")]
        /// AMI Image ID
        image_id: Option<String>,

        #[arg(long)]
        /// Filter images by image name using regular expression
        search: Option<String>,
    },

    #[command(name = "list-security-groups", about = "List AWS security groups")]
    ListGroups {
        #[arg(name = "NAME")]
        /// Security group name
        name: Option<String>,
    },

    #[command(name = "ssh", about = "SSH into an instance")]
    Ssh {
        /// Instance name
        name: String,

        #[arg(long, short)]
        /// User name to log in as
        username: Option<String>,

        #[arg(long, short)]
        /// Path to SSH key to use
        key: Option<String>,

        /// SSH options
        sshopts: Vec<String>,
    },

    #[command(name = "start", about = "Start a stopped instance")]
    Start {
        #[arg(name = "NAME")]
        /// Instance name
        name: String,
    },

    #[command(name = "stop", about = "Stop a running instance")]
    Stop {
        #[arg(name = "NAME")]
        /// Instance name
        name: String,
    },

    #[command(
        name = "generate-completions",
        about = "Generate command-line completions\n\nExample:\n   aws-instance generate-completions zsh > ~/.zsh_completions/_aws-instance"
    )]
    GenerateCompletions {
        #[arg(name = "SHELL")]
        /// Shell name
        shell: Shell,
    },
}

pub fn parse_command_line() -> CmdLineOptions {
    CmdLineOptions::parse()
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

            SubCommands::ListGroups { .. } => {
                self.list_security_groups(client).await?;
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

            SubCommands::GenerateCompletions { shell } => {
                self.generate_completions(*shell);
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

    pub async fn list_security_groups(&self, client: &Ec2Client) -> Result<()> {
        if let SubCommands::ListGroups { name } = self {
            list_security_groups(client, name).await?;
        } else {
            panic!("Unexpected value in list: {:?}", self);
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
            os_name,
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
                        .or_else(|| Some(DEFAULT_INSTANCE_TYPE.into())),
                    keypair_name: keypair_name.clone().or(profile.keypair),
                    security_group_ids: my_security_groups,
                    os_name: os_name.clone(),
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

    fn generate_completions(&self, shell: Shell) {
        print_completions(shell, &mut CmdLineOptions::command());
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
