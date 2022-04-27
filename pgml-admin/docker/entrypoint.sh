#!/bin/bash

# Exit on error, real CI
set -e

cp /app/docker/.env.docker .env
source .env

while ! psql $PGML_DATABASE_URL 2> /dev/null; do
	sleep 1
done

python3 manage.py migrate

if [[ ! -z $@ ]]; then
	echo
	echo "To visit admin: "
	echo "  http://127.0.0.1:8000/"
	echo
	$@
fi

