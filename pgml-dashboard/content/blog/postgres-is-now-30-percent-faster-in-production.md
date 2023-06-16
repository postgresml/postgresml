# Postgres is Now 30 Percent Faster in Production

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/lev.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
  	<p class="m-0">Lev Kokotov</p>
  	<p class="m-0">June 16, 2022</p>
  </div>
</div>

Anyone who runs Postgres at scale knows that performance comes with trade offs. The typical playbook is to place a pooler like PgBouncer in front of your database and turn on transaction mode. This makes multiple clients reuse the same server connection, which allows thousands of clients to connect to your database without causing a fork bomb.

Unfortunately, this comes with a trade off. Since multiple clients used the same server, they couldn't take advantage of prepared statements. Prepared statements are a way for Postgres to cache a query plan and execute it multiple times with different parameters. If you have never tried this before, you can run `pgbench` against your local DB and you'll see that `--protocol prepared` outpeforms all others by at least 30 percent. Giving up this feature has been a given for production deployments for as long as I can remember, but not anymore.

## PgCat Prepared Statements

Since [#474](https://github.com/postgresml/pgcat/pull/474), PgCat supports prepared statements in session and transaction mode. Our initial benchmarks show 30% increase over extended protocol (`--protocol extended`) and 15% against simple protocol (`--simple`). Most (all?) web frameworks use at least the extended protocol, so we are looking at a **30% performance increase across the board for everyone** who writes web apps and use Postgres in production.

This is not only a performance benefit, but also a usability improvement for client libraries that don't, or can't, use prepared statements, like the popular Rust crate [SQLx](https://github.com/launchbadge/sqlx). Until now, the typical recommendation was either to disable prepared statements or just not use a pooler.


## Benchmarks

Prepared statements are a known optimization, but how does PgCat's implementation compare to not using them at all?
