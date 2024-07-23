---
description: Connect your application to PostgresML using our SDK or any standard PostgreSQL client.
---

# Connect your app

You can connect to your PostgresML database from any PostgreSQL-compatible client. PostgresML can serve in the traditional role of an application database, along with it's extended role as an MLOps platform, to make it easy to build and maintain AI applications together with your application data.

## Client SDK

We provide a client SDK for JavaScript, Python and Rust. The SDK manages connections to the database, and makes it easy to construct efficient queries for AI use cases, like managing RAG document collections, or building chatbots. All of the ML & AI still happens inside the database, with centralized operations, hardware and dependency management.

### Installation

The SDK is available from npm and PyPI:

{% tabs %}
{% tab title="JavaScript" %}
```bash
npm i pgml
```
{% endtab %}

{% tab title="Python " %}
```bash
pip install pgml
```
{% endtab %}
{% endtabs %}

Our SDK comes with zero additional dependencies. The core of the SDK is written in Rust, and we provide language bindings and native packaging & distribution.

### Test the connection

Once you have installed our SDK into your environment, you can test connectivity to our cloud with just a few lines of code:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");

const main = () => {
    const client = pgml.newOpenSourceAI();
    const results = client.chat_completions_create(
          "meta-llama/Meta-Llama-3.1-8B-Instruct",
          [
              {
                  role: "system",
                  content: "You are a friendly chatbot who always responds in the style of a pirate",
              },
              {
                  role: "user",
                  content: "How many helicopters can a human eat in one sitting?",
              },
          ],
    );
    console.log(results);
}
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml

async def main():
    client = pgml.OpenSourceAI()
    results = client.chat_completions_create(
        "meta-llama/Meta-Llama-3.1-8B-Instruct",
        [
            {
                "role": "system",
                "content": "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                "role": "user",
                "content": "How many helicopters can a human eat in one sitting?",
            },
        ],
        temperature=0.85,
    )
    print(results)
```
{% endtab %}
{% endtabs %}

## Native PostgreSQL libraries

Using the SDK is completely optional. If you're comfortable with writing SQL, you can connect directly to the database using your favorite PostgreSQL client library or ORM:

* C++: [libpqxx](https://www.tutorialspoint.com/postgresql/postgresql\_c\_cpp.htm)
* C#: [Npgsql](https://github.com/npgsql/npgsql),[Dapper](https://github.com/DapperLib/Dapper), or [Entity Framework Core](https://github.com/dotnet/efcore)
* Elixir: [ecto](https://github.com/elixir-ecto/ecto) or [Postgrex](https://github.com/elixir-ecto/postgrex)
* Go: [pgx](https://github.com/jackc/pgx), [pg](https://github.com/go-pg/pg) or [Bun](https://github.com/uptrace/bun)
* Haskell: [postgresql-simple](https://hackage.haskell.org/package/postgresql-simple)
* Java & Scala: [JDBC](https://jdbc.postgresql.org/) or [Slick](https://github.com/slick/slick)
* Julia: [LibPQ.jl](https://github.com/iamed2/LibPQ.jl)
* Lua: [pgmoon](https://github.com/leafo/pgmoon)
* Node: [node-postgres](https://github.com/brianc/node-postgres), [pg-promise](https://github.com/vitaly-t/pg-promise), or [Sequelize](https://sequelize.org/)
* Perl: [DBD::Pg](https://github.com/bucardo/dbdpg)
* PHP: [Laravel](https://laravel.com/) or [PHP](https://www.php.net/manual/en/book.pgsql.php)
* Python: [psycopg2](https://github.com/psycopg/psycopg2/), [SQLAlchemy](https://www.sqlalchemy.org/), or [Django](https://www.djangoproject.com/)
* R: [DBI](https://github.com/r-dbi/DBI) or [dbx](https://github.com/ankane/dbx)
* Ruby: [pg](https://github.com/ged/ruby-pg) or [Rails](https://rubyonrails.org/)
* Rust: [postgres](https://crates.io/crates/postgres), [SQLx](https://github.com/launchbadge/sqlx) or [Diesel](https://github.com/diesel-rs/diesel)
* Swift: [PostgresNIO](https://github.com/vapor/postgres-nio) or [PostgresClientKit](https://github.com/codewinsdotcom/PostgresClientKit)

## SQL editors

If you need to write ad-hoc queries, you can use any of these popular tools to execute SQL queries directly on your database:

* [Apache Superset](https://superset.apache.org/)
* [DBeaver](https://dbeaver.io/)
* [Data Grip](https://www.jetbrains.com/datagrip/)
* [Postico 2](https://eggerapps.at/postico2/)
* [Popsql](https://popsql.com/)
* [Tableau](https://www.tableau.com/)
* [PowerBI](https://powerbi.microsoft.com/en-us/)
* [Jupyter](https://jupyter.org/)
* [VSCode](https://code.visualstudio.com/)
