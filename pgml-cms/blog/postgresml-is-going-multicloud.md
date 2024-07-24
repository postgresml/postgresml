---
image: ".gitbook/assets/Blog-Image_Multicloud.jpg"
---
# PostgresML is going multicloud

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Lev Kokotov

Jan 18, 2024

We started PostgresML two years ago with the goal of making machine learning and AI accessible and easy for everyone.  To make this a reality, we needed to deploy PostgresML as closely as possible to our end users. With that goal mind, today we're proud to announce support for a new cloud provider: Azure.

### How we got here

When we first launched PostgresML Cloud, we knew that we needed to deploy our AI application database in many different environments. Since we used AWS at Instacart for over a decade, we started with AWS EC2.  However, to ensure that we didn't have much trouble going multicloud in the future, we made some important architectural decisions.

Our operating system of choice, Ubuntu 22.04, is widely available and supported in all major (and small) infrastructure hosting vendors. It's secure, regularly updated and has support for NVIDIA GPUs, CUDA, and latest and most performant hardware we needed to make machine learning performant at scale.

So to get PostgresML working on multiple clouds, we first needed to make it work on Ubuntu.

### apt-get install postgresml

The best part about using a Linux distribution is its package manager. You can install any number of useful packages and tools with just a single command. PostgresML needn't be any different. To make it easy to install PostgresML on Ubuntu, we built a set of .deb packages, containing the PostgreSQL extension, Python dependencies, and configuration files, which we regularly publish to our own Aptitude repository.

Our cloud includes additional packages that install CPU-optimized pgvector, our custom configs, and various utilities we use to configure and monitor the hardware. We install and update those packages with just one command:

```
apt-get update && \
apt-get upgrade
```

Aptitude proved to be a great utility for distributing binaries and configuration files, and we use the same packages and repository as our community to power our Cloud.

### Separating storage and compute

Both Azure and AWS EC2 have the same philosophy when it comes to deploying virtual machines: separate the storage (disks & operating system) from the compute (CPUs, GPUs, memory). This allowed us to transplant our AWS deployment strategy into Azure without any modifications to our deployment strategy.

Instead of creating EBS volumes, we create Azure volumes. Instead of launching EC2 compute instances, we launch Azure VMs. When creating backups, we create EBS snapshots on EC2 and Azure volume snapshots on Azure, all at the cost of single if/else statement:

```rust
match cloud {
    Cloud::Aws => launch_ec2_instance().await,
    Cloud::Azure => launch_azure_vm().await,
}
```

Azure is our first foray into multicloud, but certainly not our last. Stay tuned for more, and thanks for your continued support of PostgresML.
