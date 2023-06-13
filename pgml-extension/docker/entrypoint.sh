#!/bin/bash

# Exit on error, real CI
echo "Starting Postgres..."
service postgresql start

echo "Connecting to Postgres..."
while ! psql -c 'SELECT 1' -U postgres -h 127.0.0.1 > /dev/null; do
	sleep 1
done

echo "Creating user and database..."
(createdb -U postgres -h 127.0.0.1 pgml_development 2> /dev/null) || true

echo "Installing pgml extension..."
psql -U postgres -h 127.0.0.1 pgml_development -f sql/setup_examples.sql -P pager


if [ -d "/docker-entrypoint-initdb.d" ]; then
echo "Running custom scripts..."
for f in /docker-entrypoint-initdb.d/*.sql; do
	echo "Running custom script ${f}"
	psql -U postgres -h 127.0.0.1 pgml_development -f "${f}"
done
fi

echo "Installing pgvector.. "
psql -U postgres -h 127.0.0.1 pgml_development -c 'CREATE EXTENSION IF NOT EXISTS vector'

echo "Ready!"
if [[ ! -z $@ ]]; then
	echo
	echo "To connect to the database: "
	echo "  psql postgres://postgres@127.0.0.1:5433/pgml_development"
	echo
	$@
fi
