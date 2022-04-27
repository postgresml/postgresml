#!/bin/bash

# Exit on error, real CI
set -e

echo "Starting Postgres..."
service postgresql start

echo "Installing pgml extension..."
pip3 install .

echo "Connecting to Postgres..."
while ! psql -c 'SELECT 1' -U postgres -h 127.0.0.1 > /dev/null; do
	sleep 1
done

echo "Creating user and database..."
(createdb -U postgres -h 127.0.0.1 pgml_development 2> /dev/null) || true
psql -d pgml_development -f sql/install.sql -U postgres -h 127.0.0.1 > /dev/null

echo "Ready!"
if [[ ! -z $@ ]]; then
	echo
	echo "To connect to the database: "
	echo "  psql postgres://postgres@127.0.0.1:5433/pgml_development"
	echo
	$@
fi
