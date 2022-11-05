# aws-instance
Command-line program to manage AWS instances.

You can use this to create new AWS instances, start them, stop them, list the instances you have, and SSH into them.

## Usage

Type `aws-instance -h` or `aws-instance <command> -h`

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
