#!/bin/bash
service postgresql start
while ! sudo -u postgres psql -c 'SELECT 1' > /dev/null; do
	sleep 1
done
echo "Creating user and database..."
sudo -u postgres createuser root --superuser
sudo -u postgres createdb root

mkdir /app/models
chown postgres:postgres /app/models
echo "Ready"

if [[ ! -z $@ ]]; then
	$@
fi
