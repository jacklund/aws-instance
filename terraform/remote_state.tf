data "terraform_remote_state" "s3_package_repo_state" {
  backend = "s3"
  config = {
    bucket = "jack-terraform-tfstate"
    region = "us-east-2"
    key    = "s3-package-repo.tfstate"
  }
}
