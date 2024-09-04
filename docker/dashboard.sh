#!/bin/bash
set -e

export DATABASE_URL=postgres://postgresml:postgresml@127.0.0.1:5432/postgresml
export SITE_SEARCH_DATABASE_URL=postgres://postgresml:postgresml@127.0.0.1:5432/postgresml
export DASHBOARD_STATIC_DIRECTORY=/usr/share/pgml-dashboard/dashboard-static
export DASHBOARD_CMS_DIRECTORY=/usr/share/pgml-cms
export SEARCH_INDEX_DIRECTORY=/var/lib/pgml-dashboard/search-index
export ROCKET_SECRET_KEY=$(openssl rand -hex 32)
export ROCKET_ADDRESS=0.0.0.0
export RUST_LOG=info

exec /usr/bin/pgml-dashboard > /dev/null 2>&1
