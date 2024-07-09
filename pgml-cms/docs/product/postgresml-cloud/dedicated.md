# Dedicated

Dedicated databases are created on dedicated hardware in our hosting provider (currently AWS EC2) and have guaranteed capacity and basically limitless horizontal scalability. PostgresML supports up to 16 Postgres replicas, 16 PgCat poolers and petabytes of disk storage, allowing teams that use it to scale to millions of requests per second at a click of a button.

Dedicated databases support for CPU and GPU hardware configurations, allowing to switch between the two at any time. Additionally, any Dedicated database can be scaled vertically by upgrading the number of CPUs, GPUs, RAM and Disk storage to accommodate growing utilization.

Dedicated databases provide access to PostgreSQL settings and knobs allowing the user to tune Postgres for their desired use case, higher degree of visibility with detailed metrics and logs, and custom backup schedules and branching.

Last but not least, Dedicated databases have a high availability configuration that allows to automatically faliover to standby instances for 4 9's of uptime required in enterprise-level production deployments.

### Creating a Dedicated database

To create a Dedicated database, make sure you have an account on postgresml.org. If you don't, you can create one now.

Once logged in, select "New Database" from the left menu and choose the Dedicated Plan.

<figure><img src="../../.gitbook/assets/spaces_B7HH1yMjCs0skMpuwNIR_uploads_S9xbhlwvqnnFUYSJLJug_image.webp" alt=""><figcaption><p>Create new database</p></figcaption></figure>

<figure><img src="../../.gitbook/assets/image (4).png" alt=""><figcaption><p>Choose the Dedicated plan</p></figcaption></figure>

### Configuring the database
