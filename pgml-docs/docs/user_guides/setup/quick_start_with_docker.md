# Quick Start w/ Docker

We've prebuilt docker images for common operating systems that will allow you to quickly spin up a new PostgreSQL server with PL/Python installed, along with the PostgresML extension. This database is seeded with a few toy datasets so you can experiment with a PostgresML workflow and quickly see results in the dashboard without needing to bring your own data. You can skip to [Native Installation](/user_guides/setup/native_installation/) if you're ready to start using your own data in a native PostgreSQL installation.

=== ":material-apple: OS X"

    [Install Docker for OS X](https://docs.docker.com/desktop/mac/install/).

=== ":material-linux: Linux"

    [Install Docker for Linux](https://docs.docker.com/engine/install/ubuntu/). Some package managers (e.g. Ubuntu/Debian) additionally require the `docker-compose` package to be installed separately.

=== ":material-microsoft-windows: Windows"

    [Install Docker for Windows](https://docs.docker.com/desktop/windows/install/). Use the Linux instructions if you're installing in Windows Subsystem for Linux.

1. Clone the repo:
```bash
$ git clone git@github.com:postgresml/postgresml.git
```

2. Start dockerized services. PostgresML will run on port 5433, just in case you already have Postgres running:
```bash
$ cd postgresml && docker-compose up
```

3. Connect to Postgres in the container with PostgresML installed:
```bash
$ psql postgres://postgres@localhost:5433/pgml_development
```

4. Validate your installation:
```sql
pgml_development=# SELECT pgml.version();
 version
---------
 0.8.1
(1 row)
```

Docker Compose will also start the dashboard app running locally [http://localhost:8000/](http://localhost:8000/)
