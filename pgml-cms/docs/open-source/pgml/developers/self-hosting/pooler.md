# Pooler

A pooler is a piece of software that is placed in front of a PostgreSQL cluster in order to load balance client connections and minimize the load placed on the database servers. Clients connect to the pooler, which pretends to be a Postgres database, and the pooler in turn connects to Postgres servers and forward clients' requests in an efficient manner.

### Why use a pooler

Postgres is a process-based database server (as opposed to threads), and each client connection forks the primary process to operate in its own memory space. A fork is generally more expensive than a thread because of extra memory allocation and OS scheduling overhead, but with a properly configured pooler, Postgres achieves a high degree of concurrency at massive scale in production.

#### PostgresML considerations

PostgresML caches machine learning models in the connection process memory space. For XGBoost/LightGBM/Scikit-learn models, which are typically only a few MBs in size, this is not a major concern, but for LLMs like Llama2 and Mistral, which are tens of gigabytes, the system memory and GPU memory usage is considerable. In order to be able to run these models effectively in production, the usage of a pooler running in transaction mode is essential. A pooler will route thousands of clients to the same Postgres server connection, reusing the same cached model, allowing for high concurrency and efficient use of resources.

### Choosing a pooler

The PostgreSQL open source community has developed many poolers over the years: PgBouncer, Odyssey, and PgPool. Each one has its pros and cons, but most of them can scale a PostgresML server effectively. At PostgresML, we developed our own pooler called PgCat, which supports many enterprise-grade features not available elsewhere that we needed to provide a seamless experience using Postgres in production, like load balancing, failover and sharding.

This guide will use PgCat as the pooler of choice.

### Installation

If you have followed our [Self-hosting](./) guide, you can just install PgCat for Ubuntu 22.04 from our APT repository:

```bash
sudo apt install -y pgcat
```

If not, you can easily install it from source.

#### Compiling from source

Download the source code from GitHub:

```bash
git clone https://github.com/postgresml/pgcat
```

If you don't have it already, install the Rust compiler from rust-lang.org:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Finally, compile PgCat in release mode and install it into your system folders:

<pre class="language-bash"><code class="lang-bash">cd pgcat &#x26;&#x26; \
cargo build --release &#x26;&#x26; \
<strong>sudo cp target/release/pgcat /usr/local/bin/pgcat &#x26;&#x26; \
</strong>sudo cp pgcat.toml /etc/pgcat.toml.example
</code></pre>

### Configuration

PgCat uses the TOML configuration language and, if installed from APT, will use the configuration file stored in `/etc/pgcat.toml`. If installed from source, you'll have to pass the configuration file path as an argument when launching.

This example will assume that you have a database called `postgresml` with a user `postgresml_user` already configured. You can create and use as many databases and users as you need. That being said, each database/user combination will be a separate connection pool in PgCat and will create its own PostgreSQL server connections.

For a primary-only setup used to serve Large Language Models, the pooler configuration is pretty basic:

```toml
[general]
host = "0.0.0.0"
port = 6432
admin_username = "pgcat"
admin_password = "<secure password>"
server_lifetime = 86400000
idle_timeout = 86400000

[pools.postgresml]
pool_mode = "transaction"

[pools.postgresml.shards.0]
servers = [
  ["<primary hostname or IP address>", 5432, "primary"].
]
database = "postgresml"

[pools.postgresml.users.0]
username = "postgresml_user"
password = "<secure password>"
pool_size = 1
```

An important consideration here is the `pool_size` of only `1` which will create and maintain only one PostgreSQL connection loaded with the LLM. Both `idle_timeout` and `server_lifetime` settings are set to 24 hours, so every 24 hours a new PostgreSQL connection will be created and the old one closed. This may not be desirable since loading a LLM into the GPU can take several seconds. To avoid this, this value can be set to be arbitrarily large, e.g. 100 years. In that case, the connection will basically never be closed.

Having only one server connection is not mandatory. If your hardware allows to load more than one LLM into your GPUs, you can increase the `pool_size` to a larger value. Our Dedicated databases currently support up to 256GB GPU-powered LLMs, so we allow considerably more connections than would be otherwise supported by say just a GeForce RTX 4080.

### Running the pooler

Once configured, the pooler is ready to go. If you installed it from our APT repository, you can just run:

```bash
sudo service pgcat start
```

If you compiled it from source, you can run it directly:

```
pgcat /etc/pgcat.toml
```

To validate that the pooler is running correctly, you can connect to it with `psql`:

```bash
PGPASSWORD="<secure password>" psql \
    -h "127.0.0.1" \
    -p 6432 \
    -U postgresml_user \
    -d postgresml
```

```
psql (14.5 (Ubuntu 14.5-0ubuntu0.22.04.1))
Type "help" for help.

postgresml=> SELECT pgml.version();
 version
---------
 2.10.0
(1 row)
```
