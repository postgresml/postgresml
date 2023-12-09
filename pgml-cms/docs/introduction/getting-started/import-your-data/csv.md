# CSV

## Static data

Data that changes infrequently can be easily imported into PostgresML using `COPY`. All you have to do is export your data as a CSV file, create a table in Postgres to store it, and import it using the command line.

Let's use a simple CSV file with 3 columns as an example:

| Column           | Data type | Example |
| ---------------- | --------- | ------- |
| name             | text      | John    |
| age              | integer   | 30      |
| is\_paying\_user | boolean   | true    |

### Export data as CSV

If you're using a Postgres database already, you can export any table as CSV with just one command:

```bash
psql -c "\copy your_table TO '~/Desktop/your_table.csv' CSV HEADER"
```

If you're using another  data store, it should almost always provide a CSV export functionality, since CSV is the most commonly used data format in machine learning.

### Create table in Postgres

Creating a table in Postgres with the correct schema is as easy as:

```
CREATE TABLE your_table (
  name TEXT,
  age INTEGER,
  is_paying_user BOOLEAN
);
```

### Import data using the command line

Once you have a table and your data exported as CSV, importing it can also be done with just one command:

```bash
psql -c "\copy your_table FROM '~/Desktop/your_table.csv' CSV HEADER"
```

We took our export command and changed `TO` to `FROM`, and that's it. Make sure you're connecting to your PostgresML database when importing data.

### Refreshing data

If your data changed, repeat this process again. To avoid duplicate entries in your table, you can truncate (or delete) all rows beforehand:

```
TRUNCATE your_table;
```
