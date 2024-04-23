# Terraform configuration for pgml-rds-proxy on EC2

This is a sample Terraform deployment for running pgml-rds-proxy on EC2. This will spin up an EC2 instance
with a public IP and a working security group & install the community Docker runtime.

Once the instance is running, you can connect to it using the root key and run the pgml-rds-proxy Docker container
with the correct PostgresML `DATABASE_URL`.
