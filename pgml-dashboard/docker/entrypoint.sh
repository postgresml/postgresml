#!/bin/bash

# Exit on error, real CI
set -e

cp /app/docker/.env.docker .env
source .env

while ! psql -U postgres -h 172.17.0.1 -p 5433 -d pgml_development 2> /dev/null; do
	echo "waiting on postgres"
	sleep 1
done

python3 manage.py migrate
python3 manage.py loaddata notebooks

if [[ ! -z $@ ]]; then
	echo
	echo "To visit the dashboard: "
	echo "  http://127.0.0.1:8000/"
	echo
	$@
fi
