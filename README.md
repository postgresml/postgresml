## Postgres ML demo

Quick demo with Postgres, PL/Python, and Scikit.

### Installation in WSL or Ubuntu

Install Python3, pip, and Pl/Python3:

```bash
sudo apt install -y postgresql-plpython3-12 python3 python3-pip
```

Restart the Postgres server:

```bash
sudo service postgresql restart
```

Create the extension:

```sql
CREATE EXTENSION plpython3u;
```

Install Scikit globally (I didn't bother setup Postgres with a virtualenv, but it's possible):

```
sudo pip3 install sklearn
```

### Run the demo

```bash
psql -f scikit_train_and_predict.sql
```

Example output:

```
psql:scikit_train_and_predict.sql:4: NOTICE:  drop cascades to view scikit_train_view
DROP TABLE
CREATE TABLE
psql:scikit_train_and_predict.sql:14: NOTICE:  view "scikit_train_view" does not exist, skipping
DROP VIEW
CREATE VIEW
INSERT 0 500
CREATE FUNCTION
 scikit_learn_train_example
----------------------------
 OK
(1 row)

CREATE FUNCTION
 value | weight | prediction
-------+--------+------------
     1 |      5 |          5
     2 |      5 |          5
     3 |      5 |          5
     4 |      5 |          5
     5 |      5 |          5
(5 rows)
```
