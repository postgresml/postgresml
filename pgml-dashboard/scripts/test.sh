# Local .env
if [ -f .env ]; then
    # Load Environment Variables
    export $(cat .env | grep -v '#' | sed 's/\r$//' | awk '/=/ {print $1}' )
fi
sudo -u postgres createdb pgml_test 2> /dev/null

DROP SCHEMA IF EXISTS pgml CASCADE;
DJANGO_DATABASE_NAME=pgml_test ./scripts/manage.py migrate

sudo -u postgres psql -q -f sql/install.sql -d pgml_test > /dev/null
