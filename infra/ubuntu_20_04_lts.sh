#1/bin/bash
#
# Deploy PostgresML on Ubuntu 20.04 LTS.
#

# Exit on error.
set -e

# Generate a random password to use for Postgres.
PGPASSWORD=$(cat /dev/urandom | tr -dc '[:alpha:]' | fold -w ${1:-20} | head -n 1)

if [[ $(whoami) != "root" ]]; then
    echo "$0 must run as root"
    exit 1
fi

while fuser /var/lib/dpkg/lock; do
    echo "Waiting for automatic upgrade to finish"
    sleep 5
done

# We're in /root.
cd /root

# Upgrade the system, always nice to run the latest and greatest
# LTS software.
apt-get update -y && apt-get upgrade -y

# Install Postgres with PL/Python and git and curl (most likely already present).
apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12 git curl libpq-dev nginx

sudo -u postgres dropdb --if-exists pgml_production
sudo -u postgres createdb pgml_production
sudo -u postgres psql -c "DROP ROLE IF EXISTS pgml_production"
sudo -u postgres psql -c "CREATE ROLE pgml_production ENCRYPTED PASSWORD '${PGPASSWORD}' SUPERUSER LOGIN"

# Clone our repo
rm -rf postgresml/
git clone https://github.com/postgresml/postgresml.git postgresml

cd "/root/postgresml/pgml-extension"

# Install the extension
pip3 install .
PGPASSWORD=${PGPASSWORD} psql -U pgml_production -d pgml_production -h 127.0.0.1 -f sql/install.sql

cd "/root/postgresml/pgml-admin"

# Install the admin UI
pip3 install -r requirements.txt

# Generate a secret key for django to use.
DJANGO_KEY=$(python3 -c 'from django.core.management.utils import get_random_secret_key; \
            print(get_random_secret_key())')


# Generate the .env file.
echo "DJANGO_SECRET_KEY='${DJANGO_KEY}'
DJANGO_DEBUG='False'
DJANGO_ALLOWED_HOSTS='*'
PGML_DATABASE_URL='postgres://pgml_production:${PGPASSWORD}@127.0.0.1:5432/pgml_production'
" > .env

# Install under www-data
mkdir -p /var/lib/www/pgml-admin
chown www-data:www-data /var/lib/www/pgml-admin
cp -R . /var/lib/www/pgml-admin
chown -R www-data:www-data /var/lib/www/pgml-admin

cp /root/postgresml/infra/pgml-admin.service /etc/systemd/system/pgml-admin.service
chmod 755 /etc/systemd/system/pgml-admin.service
systemctl daemon-reload

# Start gunicorn
service pgml-admin start
