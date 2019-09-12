resource "aws_security_group" "tor" {
  name        = "tor"
  description = "Allow tor"

  ingress {
    # ORPort and DirPort
    from_port   = 9000
    to_port     = 9001
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    # ORPort and DirPort
    from_port   = 9100
    to_port     = 9101
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
