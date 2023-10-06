# Backups

Regular backups are necessary for pretty much any kind of PostgreSQL deployment. Even in development accidents happen, and instead of losing data one can always restore from a backup and get back to a working state.

PostgresML backups work the same way as regular PostgreSQL database backups. PostgresML stores its data in regular Postgres tables, which will be backed up together with your other tables and schemas.

### Architecture

Postgres backups are composed of two (2) components: a Write-Ahead Log archive and the copies of the data files. The WAL archive will store every single write made to the database. The data file copies will contain point-in-time snapshots of what your databases had, going back up to the retention period of the backup repository.

Using the WAL and backups together, Postgres can be restored to any point-in-time version of the database. This is a very powerful tool used for development and disaster recovery.

### Configure the archive

If you have followed the [Replication](replication.md) guide, you should have a working WAL archive. If not, take a look to get your archive configured. You can come back to this guide once you have working WAL archive.

### Take your first backup

Since we are using pgBackRest already for archiving WAL, we can continue to use it to take backups. pgBackRest can easily take full and incremental backups of pretty large database clusters. We've used in previously in production to backup terabytes of Postgres data on a weekly basis.

To take a backup using pgBackRest, you can simply run this command:

```bash
pgbackrest backup --stanza=main
```

Once the command completes, you'll have a full backup of your database cluster safely stored in your S3 bucket. If you'd like to see what it takes to take a backup of a PostgreSQL database, you can add this to the command above:

```
--log-level-console=debug
```

pgBackRest will log every single step it does to take a working backup.

### Restoring from backup

When a disaster happens or you just would like to travel back in time, you can restore your database from your latest backup with just a couple commands.

#### Stop the PostgreSQL server

Restoring from backup will completely overwrite your existing database files. Therefore, don't do this unless you actually need to restore from backup.

To do so, first, stop the PostgreSQL database server, if it's running:

```
sudo service postgresql stop
```

#### Restore the latest backup

Now that PostgreSQL is no longer running, you can restore the latest backup using pgBackRest:

```
pgbackrest restore --stanza=main --delta
```

The `--delta` option will make pgBackRest check every single file in the Postgres data directory and, if it's different, overwrite it with the one saved in the backup repository. This is a quick way to restore a backup when most of the database files have not been corrupted or modified.

#### Start the PostgreSQL server

Once complete, your PostgreSQL server is ready to start again. You can do so with:

```
sudo service postgresql start
```

This will start PostgreSQL and make it check its local data files for consistency. This will be done pretty quickly and when complete, Postgres will start downloading and re-applying Write-Ahead Log files from the archive. When that operation completes, your PostgreSQL database will start and you'll be able to connect and use it again.

Depending on how much data has been written to the archive since the last backup, the restore operation could take a bit of time. To minimize the time it takes for Postgres to start again, you can take more frequent backups, e.g. every 6 hours or every 2 hours. While costing more in storage and compute, this will ensure that your database recovers from a disaster much quicker than would of otherwise happened with just a daily backup.

### Managing backups

Backups can take a lot of space over time and some of them may no longer be needed. You can view what backups and WAL files are stored in your S3 bucket with:

```
pgbackrest info
```

#### Retention policy

For most production deployments, you don't need or should retain more than a few backups. We would usually recommend keeping two (2) weeks of backups and WAL files, which should be enough time to notice that some data may be missing and needs to be restored.

If you run full backups once a day (which should be plenty), you can set your pgBackRest backup retention policy to 14 days, by adding a couple settings to your `/etc/pgbackrest.conf` file:

```
[global]
repo1-retention-full=14
repo1-retention-archive=14
```

This configuration will ensure that you have at least 14 backups and 14 backups worth of WAL files. Because Postgres allows point-in-time recovery, you'll be able to restore your database to any version (up to millisecond precision) going back two weeks.

#### Automating backups

Backups can be automated by running `pgbackrest backup --stanza=main` with a cron. You can edit your cron with `crontab -e` and add a daily midnight run, ensuring that you have fresh backups every day. Make sure you're editing the crontab of the `postgres` user since no other user will be allowed to backup Postgres or read the pgBackRest configuration file.

#### Backup overruns

If backups are taken frequently and take a long time to complete, it is possible for one backup to overrun the other. pgBackRest uses lock files located in `/tmp/pgbackrest` to ensure that no two backups are taken concurrently. If a backup attempts to start when another one is running, pgBackRest will abort the later backup.

This is a good safety measure, but if it happens, the backup schedule will break and you could end up with missing backups. There are a couple options to avoid this problem: take less frequent backups as not to overrun them, or implement a lock and wait protection outside of pgBackRest.

#### Lock and wait

To implement a lock and wait protection using only Bash, you can use `flock(1)`. Flock will open and hold a filesystem lock on a file until a command it's running is complete. When the lock is released, any other waiting flock will take the lock and run its own command.

To implement backups that don't overrun, it's usually sufficient to just protect the pgBackRest command with flock, like so:

```bash
touch /tmp/pgbackrest-flock-lock
flock /tmp/pgbackrest-flock-lock pgbackrest backup --stanza=main
```

If you find yourself in a situation with too many overrunning backups, you end up with a system that's constantly backing up. As comforting as that sounds, that's not a great backup policy since you can't be sure that your backup schedule is being followed. If that's your situation, it may be time to consider alternative backup solutions like filesystem snapshots (e.g. ZFS snapshots) or volume level snapshots (e.g. EBS snapshots).

### PostgresML considerations

Since PostgresML stores most of its data in regular Postgres tables, a PostgreSQL backup is a valid PostgresML backup. The only thing stored outside of Postgres is the Hugging Face LLM cache, which is stored directly on disk in `/var/lib/postgresql/.cache`. In case of a disaster, the cache will be lost, but that's fine; since it's only a cache, next time PostgresML `pgml.embed()` or `pgml.transform()` functions are used, PostgresML will automatically repopulate all the necessary files in the cache from Hugging Face and resume normal operations.

#### HuggingFace cold starts

In order to avoid cold starts, it's reasonable to backup the entire contents of the cache in a separate S3 location. When restoring from backup, one can just use `aws s3 sync` to download everything that should be in the cache folder back onto the machine. Make sure to do so before you start PostgreSQL in order to avoid a race condition with the Hugging Face library.
