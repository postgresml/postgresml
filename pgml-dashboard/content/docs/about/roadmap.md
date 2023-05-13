# Road map
This project is currently a proof of concept. Some important features, which we are currently thinking about or working on, are listed below.

## Production deployment
Many companies that use PostgreSQL in production do so using managed services like AWS RDS, Digital Ocean, Azure, etc. Those services do not allow running custom extensions, so we have to run PostgresML directly on VMs, e.g. EC2, droplets, etc. The idea here is to replicate production data directly from Postgres and make it available in real-time to PostgresML. We're considering solutions like logical replication for small to mid-size databases, and Debezium for multi-TB deployments.

## Model management dashboard
A good looking and useful UI goes a long way. A dashboard similar to existing solutions like MLFlow or AWS SageMaker will make the experience of working with PostgresML as pleasant as possible.

## Data explorer
A data explorer allows anyone to browse the dataset in production and to find useful tables and features to build effective machine learning models.

## More algorithms
Scikit-Learn is a good start, but we're also thinking about including Tensorflow, Pytorch, and many more useful models.

## Scheduled training
In applications where data changes often, it's useful to retrain the models automatically on a schedule, e.g. every day, every week, etc.

