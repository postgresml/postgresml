# Replication

PostgresML is fully integrated into the Postgres replication system and requires no special considerations. Setting up a PostgreSQL replica may seem to be a daunting task, but it's actually a quite straight forward step-by-step process.

### Architecture

PostgreSQL replication is composed of three (3) parts: a primary, a replica, and a Write-Ahead Log archive. Each is independently configured and operated, providing a high degree of reliability in the architecture.

#### Primary

The primary serves all queries, including writes and reads. In a replicated configuration, every single write made to the primary is replicated to the replicas and to the Write-Ahead Log archive.

#### Replica

A replica serves only read queries. Setting up additional replicas helps to horizontally scale the read capacity of a database cluster. Adding more replicas to the system can be done dynamically as demand on the system increases, and removed, as the number of clients and queries decreases.

Postgres supports three (3) kinds of replication: streaming, logical, and log-shipping. Streaming replication sends data changes as they are written to the database files, ensuring that replicas are almost byte-for-byte identical to the primary. Logical replication sends the queries as they are interpreted by the primary, e.g. `SELECT`/ `UPDATE` / `DELETE`, which are then replayed on the replica. Log-shipping replicas download the Write-Ahead Log from the archive and replay it at their own pace.

Each replication type has its own pros and cons. In this guide, we'll focus on setting up the more commonly used streaming replication.

#### Write-Ahead Log archive

The Write-Ahead Log archive, or WAL for short, is a safe place where the primary can upload every single data change that occurs in order for the replicas to download and apply them on their own system. Typically, the WAL archive is stored on a separate machine, network-attached storage or more commonly these days, in an object storage system like S3 or CloudFlare's R2.

### Dependencies

PostgreSQL replication requires third-party software to operate smoothly. At PostgresML, we're big fans of the [pgBackRest](https://pgbackrest.org) project, and we'll be using it in this guide. In order to install it and some other dependencies on your system, add the PostgreSQL APT repository to your sources:

```bash
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
sudo apt update
```

Finally, install pgBackRest:

```bash
sudo apt install -y pgbackrest
```

### **Configure the primary**

The primary needs to be configured to allow replication. By default, replication is disabled in PostgreSQL. First, to enable replication, change the following settings in `/etc/postgresql/14/main/postgresql.conf`:

```
archive_mode = on
wal_level = replica
archive_command = 'pgbackrest --stanza=main archive-push %p'
```

Second, Postgres requires that a user with replication permissions is used for replicas to connect to the primary. To create this user, login as a superuser and run:

```sql
CREATE ROLE replication_user PASSWORD '<secure password>' LOGIN REPLICATION;
```

Once the user is created, it has to be allowed to connect to the database from another machine. Postgres configures this type of access in `/etc/postgresql/14/main/pg_hba.conf`

Open that file and append this to the end:

```
host replication replication_user 0.0.0.0/0 scram-sha-256
```

This configures Postgres to allow the `replication_user` to connect from anywhere (`0.0.0.0/0`) and authenticate using the now default SCRAM-SHA-256 algorithm.

Finally, restart PostreSQL for all these settings changes to take effect:

```bash
sudo service postgresql restart
```

### Create a WAL archive

In this guide, we'll be using an S3 bucket for the WAL archive. S3 is a very reliable and affordable place to store WAL. We've used it in the past to transfer, store and replicate petabytes of data.

#### **Create an S3 bucket**

You can create an S3 bucket in the AWS Console or by using the AWS CLI:

```bash
aws s3api create-bucket \
    --bucket postgresml-tutorial-wal-archive \
    --create-bucket-configuration="LocationConstraint=us-west-2"
```

By default, S3 buckets are protected against public access, so it's a safe place to store your WAL.

#### **Configure pgBackRest**

pgBackRest can be configured by editing the `/etc/pgbackrest.conf` file. This file should be readable by the `postgres` user since it'll contain some important information.&#x20;

Using the S3 bucket we created above, we can configure pgBackRest to use it for the WAL archive:

```
[main]
pg1-path=/var/lib/postgresql/14/main/

[global]
process-max=4
repo1-path=/wal-archive/main
repo1-s3-bucket=postgresml-tutorial-wal-archive
repo1-s3-endpoint=s3.us-west-2.amazonaws.com
repo1-s3-region=us-west-2
repo1-s3-key=<YOUR AWS ACCESS KEY ID>
repo1-s3-key-<YOUR AWS SECRET ACCESS KEY>
repo1-type=s3
start-fast=y
compress-type=lz4
archive-mode-check=n
archive-check=n

[global:archive-push]
compress-level=3
```

Once configured, we can create the archive:

```bash
sudo -u postgres pgbackrest stanza-create --stanza main
```

You can validate the archive created successfully by listing the files using the AWS CLI:

```bash
aws s3 ls s3://postgresml-tutorial-wal-archive/wal-archive/main/
                           PRE archive/
                           PRE backup/
```

### Create a replica

A PostgreSQL replica should run on a different system than the primary. The two machines have to be able to communicate via the network in order for Postgres to send changes made to the primary over to the replica.

#### Install dependencies

Before configuring the replica, we need to make sure it's running the same software the primary is. Before proceeding, follow the [Self-hosting](./) guide to install PostgresML on the system. Once done, install pgBackRest and configure it the same way we did above for the primary. The replica has to be able to access the WAL files stored in the WAL archive.

#### Replicating data

A streaming replica is byte-for-byte identical to the primary, so in order to create one, we first need to copy all the database files stored on the primary over to the replica. Postgres provides a very handy command line tool for this called `pg_basebackup`.&#x20;

On Ubuntu 22.04, PostgreSQL 14 Debian package automatically creates a new Postgres data directory. Since the replica has to have the same data as the primary, first thing we need to do is to delete that automatically created data directory and replace it with the one stored on the primary.

To do so, first, stop the PostgreSQL server:

```
sudo service postgresql stop
```

Once stopped, delete the data directory:

```
sudo rm -r /var/lib/postgresql/14/main
```

Finally, copy the data directory from the primary onto the replica:

```
PGPASSWORD=<secure password> pg_basebackup \
    -h <the host or IP address of the primary> \
    -p 5432 \
    -U replication_user \
    -D /var/lib/postgresql/14/main
```

Depending on how big your database is, this will take a few seconds to a few hours. Once complete, don't start Postgres just yet. We need to set a few configuration options first.

#### Configuring the replica

In order to start replicating from the primary, the replica needs to be able to connect to it. To do so, edit the configuration file `/etc/postgresql/14/main/postgresql.conf` and add the following settings:

```
primary_conninfo = 'host=<the host or IP of the primary> port=5432 user=replication_user password=<secure password>'
restore_command = 'pgbackrest --stanza=demo archive-get %f "%p"'
```

#### Enable standby mode

By default, if Postgres is started as a replica, it will download all the WAL it can find from the archive, apply the data changes and promote itself to the primary role. To avoid this and keep the Postgres replica running as a read replica, we need to configure it to run in standby mode. To do so, place a file called `standby.signal` into the data directory, like so:

```bash
sudo -u postgres touch /var/lib/postgresql/14/main/standby.signal
```

#### Start the replica

Finally, the replica is ready to start:

```bash
sudo service postgresql start
```

If you connect to it with `psql`, you can validate it's running in read-only mode:

```sql
SELECT pg_is_in_recovery();
```

which will return `true`.

### Adding more replicas

Adding more replicas to the system is done the same way. A Postgres primary can support up to 16 replicas, which is more than enough to serve millions of queries per second and provide high availability for enterprise-grade deployments of PostgresML.
