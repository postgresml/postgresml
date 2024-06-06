---
description: >-
  The story of how we moved terabytes of real time data between cloud providers with minimal downtime.
featured: false
tags: [engineering]
---

# Our migration from AWS to GCP

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Lev Kokotov

June 8, 2024

- intro (aws as first choice, because we know it well)
- customers using other cloud providers
- how our codebase works
- cloud-neutral primitives, e.g. ubuntu, zfs
- migration
  - replicating data
  - switching over
-

From the beginning, our plan for PostgresML was to be cloud-agnostic. Since we're an infrastructure provider, we have to deploy our code where our customers are. Like most startups out there, we started on AWS, because that's what we knew best. After over 10 years of AWS experience, and its general dominance in the cloud market, it seemed right to build something we've done before, this time in Rust of course.

After talking to several customers, we've noticed a pattern: most of them were using either Azure or GCP. So our original plan had to be put back into motion. Our platform managers all infrastructure internally, representing common concepts like hosts, networking rules, open ports, and domains as first class entities in our codebase. To add additional cloud vendors, we just had to write integrations with their APIs.

## Cloud-agnostic from the start

PostgresML, much like Postgres itself, can run on a variety of platforms. Our operating system of choice, **Ubuntu**, is available on all clouds, and comes with great support for GPUs. We therefore had no trouble spinning up machines on Azure and GCP with identical software to match our AWS deployments.

Since we're first and foremost a database company, data integrity and security are extremely important. To achieve that goal, and to be cloud-neutral from the start, we are using **ZFS** as our file system to store Postgres data and LLMs.

## The migration

Our primary serverless deployment is in Oregon, AWS *us-west-2* region. We were migrating to GCP in Iowa (*us-central1*).

### Moving data

Moving data is hard. Moving terabytes of data between machines in the same cloud can be achieved with volume snapshots, and the hard part of ensuring data integrity is delegated to the cloud vendor. But to move data between clouds, one has to rely on your own tooling.

Since we use ZFS, our original plan was to just send a ZFS snapshot across the country and synchronize later with Postgres replication. To make sure the data isn't intercepted by nefarious entities while in transit, the typical recommendation is to pipe it through SSH:

```bash
zfs send tank/pgdata@snapshot | ssh ubuntu@my-other-host \
zfs recv tank/pgdata@snapshot
```

#### First attempt

Our filesystem was multiple terabytes, but both machines had 10GBit NICs, so we expected this to take just a few hours. To our surprise, the transfer speed wouldn't go higher than 30MB/second. At that rate, the migration would take days. Since we had to setup Postgres replication afterwards, we had to keep a replication slot open to prevent WAL cleanup on the primary.

A dangling replication slot left unattended for days would accumulate terabytes of write-ahead log and eventually run our filesystem out of space and shut down the database. To make things harder, _zfs send_ is an all or nothing operation; if interrupted for any reason, e.g. network errors, one would have to start from scratch.

So realistically, a multi-day operation was out of the question. At this point, we were stuck and a realization loomed: there is a good reason why most organizations don't attempt a cloud migration.

#### Trial and error

The cause for the slow transfer was not immediately clear. At first we suspected some kind of artificial bandwidth limit for instances uploading to the public Internet. After all, cloud vendors charge quite a bit for this feature, so it would make sense to throttle it to avoid massive surprise bills.

AWS encourages object storage like S3 to serve large files over the Internet, where transfer speeds are advertised as virtually unlimited and storage costs are a franction of what they are on EBS.

So we had a thought: why not upload our ZFS filesystem to S3 first, transfer it to its GCP counterpart (Cloud Storage) using the [Storage Transfer Service](https://cloud.google.com/storage-transfer/docs/cloud-storage-to-cloud-storage), and then finally download it to our new machine. Bandwidth between internal cloud resources is free and as fast as it can be, at least in theory.

