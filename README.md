# aws-instance
Command-line program to manage AWS instances.

You can use this to create new AWS instances, start them, stop them, list the instances you have, and SSH into them.

## Usage
```
% aws-instance -h
aws-instance 0.1.0
Jack Lund <jackl@geekheads.net>
Manage AWS instances

USAGE:
    aws-instance [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --profile <profile>    Set the AWS profile to use
    -r, --region <region>      Set the AWS region to use

SUBCOMMANDS:
    create       Create a named AWS instance
    destroy      Destroy an AWS instance by name
    help         Prints this message or the help of the given subcommand(s)
    list         List AWS instances
    list-amis    List AMIs
    ssh          SSH into an instance
    start        Start a stopped instance
    stop         Stop a running instance
```

The subcommands will be detailed below. For commonly-used options, you can specify them in a config file,
`~/.aws-instance/config`, which is a TOML-formatted file (details below).

### Create

Create a new instance. Usage:

```
% aws-instance create -h
aws-instance-create 0.1.0
Jack Lund <jackl@geekheads.net>
Create a named AWS instance

USAGE:
    aws-instance create [OPTIONS] <NAME> <AMI-ID>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --ebs-optimized <ebs_optimized>              Is it EBS optimized? [default: false]
    -i, --iam-profile <iam_profile>                  IAM profile to use
    -t, --instance-type <instance_type>              Instance type [default: m1.small]
    -k, --keypair <keypair_name>                     Key pair to use to connect
    -s, --security-groups <security_group_ids>...    Security groups for the instance

ARGS:
    <NAME>      Instance name
    <AMI-ID>    AMI Image ID to use
```

You can specify a variety of options for your new instance; for more information, see
https://docs.aws.amazon.com/cli/latest/reference/ec2/run-instances.html.

The `NAME` parameter is used to identify your instance for use with the other subcommands.
You cannot have more than one instance with a given name in any given region.

### Destroy

Destroy your instance. Usage:

```
% aws-instance destroy -h
aws-instance-destroy 0.1.0
Jack Lund <jackl@geekheads.net>
Destroy an AWS instance by name

USAGE:
    aws-instance destroy <name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <name>    Instance name
```

### List

List your available instances. Usage:

```
% aws-instance list -h
aws-instance-list 0.1.0
Jack Lund <jackl@geekheads.net>
List AWS instances

USAGE:
    aws-instance list

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

### List-Amis

List the AMIs available to you for your instances. Usage:

```
% aws-instance list-amis -h
aws-instance-list-amis 0.1.0
Jack Lund <jackl@geekheads.net>
List AMIs

USAGE:
    aws-instance list-amis [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --architecture <architecture>    Instance architecture [default: x86_64]
        --image_id <image-id>            AMI Image ID
    -n, --name <name>                    Image name. You may use '?' and '*' to return multiple values
        --search <search>                Filter images by image name using regular expression
```

Note: The `--search` option can take a long time, because it has to filter the instances on the client side.

### SSH

SSH into your instances. Usage:

```
% aws-instance ssh -h
aws-instance-ssh 0.1.0
Jack Lund <jackl@geekheads.net>
SSH into an instance

USAGE:
    aws-instance ssh <name> [sshopts]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <name>          Instance name
    <sshopts>...    SSH options
```

### Start

Start a stopped instance. Usage:

```
% aws-instance start -h
aws-instance-start 0.1.0
Jack Lund <jackl@geekheads.net>
Start a stopped instance

USAGE:
    aws-instance start <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <NAME>    Instance name
```

### Stop

Stop a running instance. Usage:

```
% aws-instance stop -h
aws-instance-stop 0.1.0
Jack Lund <jackl@geekheads.net>
Stop a running instance

USAGE:
    aws-instance stop <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <NAME>    Instance name
```

## Profiles

Similarly to AWS profiles, `aws-instance` has a config file (`~/.aws-instance/config`) which contains defaults you can specify
by profile name. If you don't specify a profile name on the command line, it will look for a profile named
`default`; barring that, it will use the application defaults.

Example of a config file:

```
[default]
keypair = default_keypair
security-groups = sg-2ac23f43
key = /home/jack/.ssh/default_keypair.pem

[work]
keypair = work_keypair
security-groups = sg-2ac23f43
key = /home/jack/.ssh/work_keypair.pem
```
