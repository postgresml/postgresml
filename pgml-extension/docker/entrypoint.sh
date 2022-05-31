#!/bin/bash

# Exit on error, real CI
set -e

echo "Starting Postgres..."
service postgresql start

echo "Connecting to Postgres..."
while ! psql -c 'SELECT 1' -U postgres -h 127.0.0.1 > /dev/null; do
	sleep 1
done

echo "Creating user and database..."
(createdb -U postgres -h 127.0.0.1 pgml_development 2> /dev/null) || true

echo "Installing pgml extension..."
sudo pip3 install .
sudo python3 -m pgml_extension --database-url=postgres://postgres@127.0.0.1:5432/pgml_development
psql -U postgres -h 127.0.0.1 pgml_development -f tests/test.sql -P pager

echo "Ready!"
if [[ ! -z $@ ]]; then
	echo
	echo "To connect to the database: "
	echo "  psql postgres://postgres@127.0.0.1:5433/pgml_development"
	echo
	$@
fi
