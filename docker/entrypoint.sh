#!/bin/bash
#
# Start local dev
#
echo "Starting PostgresML"
service postgresql start

# Setup users
useradd postgresml -m 2> /dev/null 1>&2
sudo -u postgresml touch /home/postgresml/.psql_history
sudo -u postgres createuser root --superuser --login 2> /dev/null 1>&2
sudo -u postgres psql -c "CREATE ROLE postgresml PASSWORD 'postgresml' SUPERUSER LOGIN" 2> /dev/null 1>&2
sudo -u postgres createdb postgresml --owner postgresml 2> /dev/null 1>&2
sudo -u postgres psql -c 'ALTER ROLE postgresml SET search_path TO public,pgml' 2> /dev/null 1>&2

# Create the vector extension
sudo -u postgres psql -c 'CREATE EXTENSION vector' 2> /dev/null 1>&2

echo "Starting dashboard"
PGPASSWORD=postgresml psql -c 'CREATE EXTENSION IF NOT EXISTS pgml' \
	-d postgresml \
	-U postgresml \
	-h 127.0.0.1 \
	-p 5432 2> /dev/null 1>&2

bash /app/dashboard.sh &

exec "$@"
