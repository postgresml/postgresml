#!/bin/bash
#
# Start local dev
#
echo "Starting PostgresML"
service postgresql start
echo "Starting dashboard"
bash /app/dashboard.sh &

exec "$@"
