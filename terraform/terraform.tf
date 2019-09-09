resource "aws_s3_bucket" "tfstate" {
  bucket = "jack-terraform-tfstate"
  acl    = "private"

  versioning {
    enabled = true
  }
}

terraform {
  backend "s3" {
    bucket = "jack-terraform-tfstate"
    key    = "aws-instance.tfstate"
    region = "us-east-2"
  }
}
