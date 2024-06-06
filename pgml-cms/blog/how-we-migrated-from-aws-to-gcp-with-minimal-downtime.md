---
description: >-
  Lessons learned from moving terabytes of real time data between cloud providers.
featured: false
tags: [engineering]
---

# How we migrated from AWS to GCP with minimal downtime

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Lev Kokotov

June 6, 2024

From the beginning, our plan for PostgresML was to be cloud-agnostic. Since we are an infrastructure provider, we have to deploy our code where our customers are. Like most startups, we started on AWS, because that is what we knew best. After over 10 years of AWS experience, and its general dominance in the market, it seemed right to build something we have done before, this time in Rust of course.

After talking to several customers, we have noticed a pattern: most of them were using either Azure or GCP. So we had to go back to our original plan. Our platform manages all infrastructure internally, by representing common concepts like virtual machines, networking rules, and DNS as first class entities in our codebase. To add additional cloud vendors, we just had to write integrations with their APIs.

## Cloud-agnostic from the start

PostgresML, much like Postgres itself, can run on a variety of platforms. Our operating system of choice, **Ubuntu**, is available on all clouds, and comes with good support for GPUs. We therefore had no trouble spinning up machines on Azure and GCP with identical software to match our AWS deployments.

Since we are first and foremost a database company, data integrity and security are extremely important. To achieve that goal, and to be independent from any cloud-specific storage solutions, we are using **ZFS** as our filesystem to store Postgres data.

Moving ZFS filesystems between machines is a solved problem, or so we thought.

## The migration

Our primary Serverless deployment was in Oregon, AWS *us-west-2* region. We were moving it to GCP in Iowa, *us-central1* region.

### Moving data is hard

Moving data is hard. Moving terabytes of data between machines in the same cloud can be achieved with volume snapshots, and the hard part of ensuring data integrity is delegated to the cloud vendor. Of course, that is not always guaranteed, and you can still corrupt your data if you are not careful, but that is a story for another time.

That being said, to move data between clouds, one has to rely on your own tooling. Since we use ZFS, our original plan was to just send a ZFS snapshot across the country and synchronize later with Postgres replication. To make sure the data is not intercepted by nefarious entities while in transit, the typical recommendation is to pipe it through SSH:

```bash
zfs send tank/pgdata@snapshot | ssh ubuntu@machine \
zfs recv tank/pgdata@snapshot
```

#### First attempt

Our filesystem was multiple terabytes, but both machines had 100Gbit NICs, so we expected this to take just a few hours. To our surprise, the transfer speed would not go higher than 30MB/second. At that rate, the migration would take days. Since we had to setup Postgres replication afterwards, we had to keep a replication slot open to prevent WAL cleanup on the primary.

A dangling replication slot left unattended for days would accumulate terabytes of write-ahead log and eventually run our filesystem out of space and shut down the database. To make things harder, _zfs send_ is an all or nothing operation: if interrupted for any reason, e.g. network errors, one would have to start over from scratch.

So realistically, a multi-day operation was out of the question. At this point, we were stuck and a realization loomed: there is a good reason why most organizations do not attempt a cloud migration.

#### Trial and error

The cause for the slow transfer was not immediately clear. At first we suspected some kind of artificial bandwidth limit for machines uploading to the public Internet. After all, cloud vendors charge quite a bit for this feature, so it would make sense to throttle it to avoid massive surprise bills.

AWS encourages object storage like S3 to serve large files over the Internet, where transfer speeds are advertised as virtually unlimited and storage costs are a fraction of what they are on EBS.

So we had a thought: why not upload our ZFS filesystem to S3 first, transfer it to its GCP counterpart (Cloud Storage) using the [Storage Transfer Service](https://cloud.google.com/storage-transfer/docs/cloud-storage-to-cloud-storage), and then  download it to our new machine. Bandwidth between internal cloud resources is free and as fast as it can be, at least in theory.

#### Our own S3 uploader

As of this writing, we could not find any existing tools to send a ZFS file system to S3 and download it from Cloud Storage, in real time. Most tools like [z3](https://github.com/presslabs/z3) are used for backup purposes, but we needed to transfer filesystem chunks as quickly as possible. 

So just like with everything else, we decided to write our own, in Rust. After days of digging through Tokio documentation and networking theory blog posts to understand how to move bytes as fast as possible between the filesystem and an HTTP endpoint, we had a pretty basic application that could chunk a byte stream, send it to an object storage service as separate files, download those files as they are being created in real time, re-assemble and pipe them into a ZFS snapshot.

This was an exciting moment. We created something new and were going to open source it once we made sure it worked well, increasing our contribution to the community. The moment arrived and we started our data transfer. After a few minutes, our measured transfer speed was: roughly 30MB/second.

Was there a conspiracy afoot? We thought so. We even tried using S3 Transfer Acceleration, which produced the same result. We were stuck.

### Occam's razor

Something was clearly wrong. Our migration plans were at risk and since we wanted to move our Serverless cloud to GCP, we were pretty concerned. Were we trapped on AWS forever?

Something stood out though after trying so many different approaches. Why 30MB/second? That seems like a made up number, and on two separate clouds too? Clearly, it was not an issue with the network or our tooling, but with how we used it.

#### Buffer and compress

After researching a bit about how other people migrated filesystems (it is quite common in the ZFS community, since it makes it convenient, our problems notwithstanding), the issue emerged: _zfs send_ and _zfs recv_ do not buffer data. For each chunk of data they send and receive, they issue separate `write(2)` and `read(2)` calls to the kernel, and process whatever data they get. 

In case of a network transfer, these kernel calls propagate all the way to the network stack, and like any experienced network engineer would tell you, makes things very slow.

In comes `mbuffer(1)`. If you are not familiar with it, mbuffer is a tool that _buffers_ whatever data it receives and sends it in larger chunks to its destination, in our case SSH on the sender side and ZFS on the receiver side. Combined with a multi-threaded stream compressor, `pbzip2(1)`, which cut our data size in half, we were finally in business, transferring our data at over 200 MB/second which cut our migration time from days to just a few hours, all with just one command:

```bash
zfs send tank/pgdata@snapshot | pbzip2 | mbuffer -s 12M -m 2G | ssh ubuntu@gcp \
mbuffer -s 12M -m 2G | pbzip2 -d | zfs recv tank/pgdata@snapshot
```

### Double check everything

Once the ZFS snapshot finally made it from the West Coast to the Midwest, we configured Postgres streaming replication, which went as you would expect, and we had a live hot standby in GCP, ready to go. Before cutting the AWS cord, we wanted to double check that everything was okay. We were moving customer data after all, and losing data is bad for business — especially for a database company.

#### The case of the missing bytes

ZFS is a reliable and battle tested filesystem, so we were not worried, but there is nothing wrong with a second opinion. The naive way to check that all your data is still there is to compare the size of the filesystems. Not a terrible place to start, so we ran `df -h` and immediately our jaws dropped: only half the data made it over to GCP.

After days of roadblocks, this was not a good sign, and there was no reasonable explanation for what happened. ZFS checksums every single block, mbuffer is a simple tool, pbzip definitely decompressed the stream and SSH has not lost a byte since the 1990s.

In addition, just to make things even weirder, Postgres replication did not complain and the data was, seemingly, all there. We checked by running your typical `SELECT COUNT(*) FROM a_few_tables` and everything added up: as the data was changing in AWS, it was updating in GCP.

#### (File)systems are virtual

If you ever tried to find out how much free memory your computer has, you know there is no obvious answer. Are you asking for RSS of every single process, virtual memory, and do you have swap enabled, and are you considering the kernel page cache or fragmentation? At the end, you just have to trust that the kernel knows what it is doing.

Filesystems are exactly the same, and to the uninitiated, the difference in file sizes can be scary. After a few Google searches and reading a bunch of panicked system administrator's forum posts from the mid-2000s, it was the manual page for `du(1)` that provided the answer:

```
--apparent-size
    print  apparent  sizes, rather than disk usage; although the apparent size is usually smaller, it may be
    larger due to holes in ('sparse') files, internal fragmentation, indirect blocks, and the like
```

The database files were the same on GCP and AWS, if one checked them for their "apparent" size: the size of the file as seen by applications, not what they actually used on disk. ZFS is quite clever, and during the transfer with `zfs send`, repacked the filesystem which was somewhat fragmented after years of random writes.

### The cutover

The final step was to move our customers' traffic from AWS to GCP, and do so without losing a byte of data. We picked the lowest traffic period, midnight Pacific time, and shut down our AWS primary.

As soon as the Systemd service stopped, we changed the DNS record to point to our GCP standby and ran `SELECT pg_promote()`. Traffic moved over almost immediately, thanks to our low DNS TTL and we were back in business.

## Lessons learned

Migrating between clouds is hard, but not impossible. The key is to understand how your tools work and why they work the way they do. For us, these were the takeaways:

1. Network buffering is essential
2. Data compression will save you time and money
3. Advanced filesystems are complex
3. You can solve hard problems, just take it one step at time

At PostgresML, we are excited to solve hard problems. If you are too, feel free to explore [career opportunities](/careers) with us, or check out our [open-source docs](/docs) and contribute to our project.

