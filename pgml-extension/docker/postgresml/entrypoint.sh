#!/bin/bash
#
# Start local dev
#
echo "Starting PostgresML, one moment"
service postgresql start
echo "Ready"

exec "$@"
