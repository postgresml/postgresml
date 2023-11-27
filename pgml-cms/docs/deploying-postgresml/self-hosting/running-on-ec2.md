# Running on EC2

AWS EC2 has been around for quite a while and requires no introduction. Running PostgresML on EC2 is very similar as any other cloud provider or on-prem deployment, but it does provide a few additional features that allow PostgresML to pretty easily scale into terabytes and beyond.

### Operating  system

We're big fans of Ubuntu and use it in our Cloud. AWS provides its own Ubuntu images (called AMIs, or Amazon Machine Images) which work very well and come with all the standard tools needed to run a PostgreSQL server.

### Storage

The choice of storage is critical to scalable and performant AI database operations. PostgresML deals with large datasets and even larger models, so performant and durable storage is important.

EC2 provides two kinds of storage that can be used for running databases: EBS (Elastic Block Storage) and ephemeral NVMes. NVMe storage is typically faster than EBS and provides much lower latency, but it does lack some of the durability guarantees that one may want from a database deployment. We've ran databases on both, but currently prefer to use EBS because it allows us to take instant backups of our databases and to scale the storage of a database cluster independently from compute.

#### Choosing storage type

EBS has many different kinds of volumes, such as `gp2`, `gp3`, `io1`, `io2`, etc. The type of volume to use really depends on the cost/benefit analysis for the deployment in question. For example, if money is no object, running on `io2` would provide pretty great performance and durability guarantees. That being said, most deployments would be quite happy running on `gp3`.

#### Choosing the filesystem

The choice of the filesystem is a bit like getting married. You should really know what you're getting yourself into and more often than not, you're choice will stay with you for years to come. We've benchmarked and used many different types of filesystems in production, including ext4, ZFS, btrfs and NTFS. Our current filesystem of choice is ZFS because of its high durability, consistency and reasonable performance guarantees.

### Backups

If you choose to use EBS for your database storage, special consideration should be taken around backups. If you decide to use pgBackRest to backup your database, you needn't read any further, however if you'd like to use EBS snapshots, there is a quick tip that could save you from problematic outcomes down the line.

EBS snapshots are an atomic point-in-time copy of the EBS volume. That means that if you take a snapshot of an EBS volume and restore it, whatever you have on that volume at the time of the snapshot will be exactly the way you left it. However, if you take a snapshot while you're writing to the volume, that write may only be partially saved in the snapshot. This is because EBS snapshots are controlled by the EBS server and the filesystem is not aware of its internal operations or that it's taking a snapshot at all. This is very similar to how snapshots work on hardware RAID volume managers.

If you don't pause writes to your filesystem before you take an EBS snapshot, you will run the risk of possibly losing some of your data, or in the worst case, corrupting your filesystem. That means,  if you're using a filesystem like ext4, consider running `fsfreeze(8)` before taking an EBS backup.

If you're like us and prefer ZFS, you don't need to do anything. ZFS is a copy-on-write filesystem that guarantees that all writes made to it are atomic. So even if the EBS volume cuts it off mid write, the filesystem will always be in a consistent state, although you may lose that last write that never fully made it into the snapshot.

#### Taking an EBS backup

You can use EBS backups for creating replicas and for disaster recovery. An EBS backup works exactly like `pg_basebackup` except it's instantaneous. To ensure that your backup is easily restorable, make sure to first create the `/var/lib/postgresql/14/main/standby.signal` file and only then taking a snapshot.

This ensures that when you restore from that backup, Postgres does not automatically promote itself and start accepting writes. If that happens, you won't be able to use it as a replica without getting into `pg_rewind`.

Alternatively, you can disable the `posgresql` service by default ensuring that Postgres does not start on system boot automatically.

#### pgBackRest

If you're using pgBackRest for backups and archiving, you can take advantage of EC2 IAM integration. Instead of saving AWS IAM keys and secrets in `/etc/pgbackrest.conf`, you can instead configure it to fetch temporary credentials from the EC2 API:

```
[global]
repo1-s3-key-type=auto
```

Make sure that your EC2 IAM role has sufficient permissions to access your WAL archive S3 bucket.

### Performance

A typical single volume storage configuration is fine for low traffic databases. However, if you need additional performance, you have a few options. One option is to simply allocate more IOPS to your volume. That works, but that may be a bit costly when used at scale. Another option is to RAID multiple EBS volumes into either a RAID0 for maximum throughput or a RAIDZ1 for good throughput and reasonable durability guarantees.

ZFS supports both RAID0 and RAIDZ1 configurations. If you have say 4 volumes, you can setup a RAID0 with just a couple commands:

```
zfs create tank /dev/nvme1n1 /dev/nvme2n1 /dev/nvme3n1 /dev/nvme4n1
zfs create -o mountpoint=/var/lib/postgresql tank/pgdata
```

or a RAIDZ1 with 5 volumes:

<pre><code><strong>zfs create tank raidz /dev/nvme1n1 /dev/nvme2n1 /dev/nvme3n1 /dev/nvme4n1 /dev/nvme5n1
</strong></code></pre>

RAIDZ1 protects against single volume failure, allowing you to replace an EBS volume without taking your database offline or restoring from backup. Considering EBS guarantees and additional redundancy provided by RAIDZ, this is a reasonable configuration to use for systems that require good durability and performance guarantees.

A RAID configuration with 4 volumes allows up to 4x read throughput of a single volume which, in EBS terms, can produce up to 600MBps, without having to pay for additional IOPS.

