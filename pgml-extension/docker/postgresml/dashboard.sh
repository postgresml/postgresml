#!/bin/bash
set -e

export DATABASE_URL=postgres:///postgres
export DASHBOARD_STATIC_DIRECTORY=/usr/share/pgml-dashboard/dashboard-static
export DASHBOARD_CONTENT_DIRECTORY=/usr/share/pgml-dashboard/content
export ROCKET_SECRET_KEY=$(openssl rand -hex 32)
export ROCKET_ADDRESS=0.0.0.0
export RUST_LOG=info

exec /usr/bin/pgml-dashboard
