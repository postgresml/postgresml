#!/bin/bash
#
# Start local dev
#
echo "Starting PostgresML"
service postgresql start
echo "Starting dashboard"
sudo -u postgres bash /app/dashboard.sh &

exec "$@"
