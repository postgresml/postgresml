#!/bin/bash

# Exit on error, real CI
set -e

echo "Starting Postgres..."
service postgresql start

echo "Installing pgml extension..."
/bin/python3.8 -m pip install .

echo "Connecting to Postgres..."
while ! psql -p 5432 -h 127.0.0.1 -U postgres -c 'SELECT 1' > /dev/null; do
	sleep 1
done

echo "Creating user and database..."
echo "SELECT 'CREATE DATABASE pgml_development' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'pgml_development')\gexec" | psql postgres://postgres@127.0.0.1:5432/postgres > /dev/null
psql postgres://postgres@127.0.0.1:5432/pgml_development -f sql/install.sql > /dev/null

echo "Ready!"
if [[ ! -z $@ ]]; then
	echo
	echo "To connect to the database: "
	echo "  psql postgres://postgres@127.0.0.1:5433/pgml_development"
	echo
	$@
fi
