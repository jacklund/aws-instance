resource "aws_iam_instance_profile" "default" {
  name = "default"
  role = "${aws_iam_role.instance_role.name}"
}

resource "aws_iam_role" "instance_role" {
  name = "instance_role"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "ec2.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "s3_rw_policy" {
  role       = "${aws_iam_role.instance_role.name}"
  policy_arn = "${data.terraform_remote_state.s3_package_repo_state.outputs.s3_repo_rw_policy_arn}"
}
