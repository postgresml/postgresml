#!/bin/bash
#
# Configure pgcat from a DATABASE_URL environment variable and run it as PID 1.
# This will regenerate the configuration file every time so modifications to it won't be saved.
#
# If you want to modify the configuration file, generate it first and then run pgcat with `--config <path to file>` instead.
#
# Author: PostgresML <team@postgresml.org>
# License: MIT
#
exec /usr/local/bin/pgcat --database-url ${DATABASE_URL}
