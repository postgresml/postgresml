# Import your data

Machine learning always depends on input data, whether it's generating text with pretrained LLMs, training a retention model on customer data, or predicting session abandonment in real time. Just like any PostgreSQL database, PostgresML can be configured as the authoritative application data store, a streaming replica from some other primary, or use foreign data wrappers to query another data host on demand. Depending on how frequently your data changes and where your authoritative data resides, different methodologies imply different tradeoffs.

PostgresML can easily ingest data from your existing data stores.&#x20;

## Static data

Data that changes infrequently can be easily imported into PostgresML using `COPY`. All you have to do is export your data as a CSV file, create a table in Postgres to store it, and import it using the command line.

{% content-ref url="csv.md" %}
[csv.md](csv.md)
{% endcontent-ref %}

## Live data

Importing data from online databases can be done with foreign data wrappers. Hosted PostgresML databases come with both `postgres_fdw` and `dblink` extensions pre-installed, so you can import data from any of your existing Postgres databases, and export machine learning artifacts from PostgresML using just a few lines of SQL.

{% content-ref url="foreign-data-wrapper.md" %}
[foreign-data-wrapper.md](foreign-data-wrapper.md)
{% endcontent-ref %}

####
