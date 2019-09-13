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

## Terraform

The terraform directory contains [Terraform](https://terraform.io) code to provision some things, like the instance profiles,
security groups, etc. It also sets up access to the S3 bucket for the [S3 package repo](https://github.com/jacklund/s3-package-repo).
Note that it uses remote state from that repository, so you'll need to `terraform apply` in that repo first.

## Ansible

I use [Ansible](https://ansible.com) to provision my instances. Once they're up and accepting SSH (which you can check with either
`aws-instance ssh <hostname>` or by using ansible directly by doing `ansible <hostname> -m ping`), you can use ansible to provision the
instance as whatever type you want, using the playbooks in the `ansible` directory.

You can use `aws-instance` for your ansible inventory by using the `bin/ansible-inventory` script in this repo. Just add `inventory=/path/to/ansible-inventory`
to your [`.ansible.cfg`](https://docs.ansible.com/ansible/latest/reference_appendices/config.html#ansible-configuration-settings) file.
