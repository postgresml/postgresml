# Self-hosting

PostgresML is a Postgres extension, so running it is very similar to running a self-hosted PostgreSQL database server. A typical architecture consists of a primary database that will serve reads and writes, optional replicas to scale reads horizontally, and a pooler to load balance connections.

### Operating system

At PostgresML, we prefer running Postgres on Ubuntu, mainly because of its extensive network of supported hardware architectures, packages, and drivers. The rest of this guide will assume that we're using Ubuntu 22.04, the current long term support release of Ubuntu, but you can run PostgresML pretty easily on any other flavor of Linux.

### Installing PostgresML

PostgresML for Ubuntu 22.04 can be downloaded directly from our APT repository. There is no need to install any additional dependencies or compiling from source.

To add our APT repository to our sources, you can run:

```bash
echo "deb [trusted=yes] https://apt.postgresml.org jammy main" | \
sudo tee -a /etc/apt/sources.list
```

We don't sign our Debian packages since we can rely on HTTPS to guarantee the authenticity of our binaries.

Once you've added the repository, make sure to update APT:

```bash
sudo apt update
```

Finally, you can install PostgresML:

```bash
sudo apt install -y postgresml-14
```

Ubuntu 22.04 ships with PostgreSQL 14, but if you have a different version installed on your system, just change `14` in the package name to your Postgres version. We currently support all  versions supported by the community: Postgres 12 through 15.

### Validate your installation

You should be able to connect to Postgres and install the extension into the database of your choice:

```bash
sudo -u postgres psql
```

```
postgres=# CREATE EXTENSION pgml;
INFO:  Python version: 3.10.6 (main, Nov  2 2022, 18:53:38) [GCC 11.3.0]
INFO:  Scikit-learn 1.1.3, XGBoost 1.7.1, LightGBM 3.3.3, NumPy 1.23.5
CREATE EXTENSION
postgres=#
```

### GPU support

If you have access to Nvidia GPUs and would like to use them for accelerating LLMs or XGBoost/LightGBM/Catboost, you'll need to install Cuda and the matching drivers.

#### Installing Cuda

Nvidia has an apt repository that can be added to your system pretty easily:

```bash
curl -LsSf \
    https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb \
    -o /tmp/cuda-keyring.deb
sudo dpkg -i /tmp/cuda-keyring.deb
sudo apt update
sudo apt install -y cuda
```

Once installed, you should check your installation by running `nvidia-smi`:

<pre><code><strong>$ nvidia-smi
</strong>
Fri Oct  6 09:38:19 2023
+---------------------------------------------------------------------------------------+
| NVIDIA-SMI 535.54.04              Driver Version: 536.23       CUDA Version: 12.2     |
|-----------------------------------------+----------------------+----------------------+
| GPU  Name                 Persistence-M | Bus-Id        Disp.A | Volatile Uncorr. ECC |
| Fan  Temp   Perf          Pwr:Usage/Cap |         Memory-Usage | GPU-Util  Compute M. |
|                                         |                      |               MIG M. |
|=========================================+======================+======================|
|   0  NVIDIA GeForce RTX 3070 Ti     On  | 00000000:08:00.0  On |                  N/A |
|  0%   41C    P8              28W / 290W |   1268MiB /  8192MiB |      5%      Default |
|                                         |                      |                  N/A |
+-----------------------------------------+----------------------+----------------------+
</code></pre>

It's important that the Cuda version and the Nvidia driver versions are compatible. When installing Cuda for the first time, it's common to have to reboot the system before both are detected successfully.

### pgvector

`pgvector` is optimized for the CPU architecture of your machine, so it's best to compile it from source directly on the machine that will be using it.

#### Dependencies

`pgvector` has very few dependencies beyond just the standard build chain. You can install all of them with this command:

```bash
sudo apt install -y \
    build-essential \
    postgresql-server-dev-14
```

Replace `14` in `postgresql-server-dev-14` with your Postgres version.

#### Install pgvector

You can install `pgvector` directly from GitHub by just running:

```
git clone https://github.com/pgvector/pgvector /tmp/pgvector
git -C /tmp/pgvector checkout v0.5.0
echo "trusted = true" >> "/tmp/pgvector/vector.control"
make -C /tmp/pgvector
sudo make install -C /tmp/pgvector
```

Once installed, you can create the extension in the database of your choice:

```
postgresml=# CREATE EXTENSION vector;
CREATE EXTENSION
```

