#!/bin/bash

service postgresql start
while ! sudo -u postgres psql -c 'SELECT 1' > /dev/null; do
	sleep 1
done
echo "Creating user and database..."
sudo -u postgres createuser root --superuser 2> /dev/null
sudo -u postgres createdb root 2> /dev/null

echo "Installing pgml extension..."
cd pgml/ > /dev/null
pip install . > /dev/null
cd ../ > /dev/null
psql -q -f sql/install.sql > /dev/null

echo "Ready"

if [[ ! -z $@ ]]; then
	echo
	echo "To connect to the database: "
	echo "  psql -p 5433 -h 127.0.0.1 -U root"
	echo
	$@
fi
