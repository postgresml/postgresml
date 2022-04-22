#!/bin/bash

# Local .env
if [ -f .env ]; then
    # Load Environment Variables
    export $(cat .env | grep -v '#' | sed 's/\r$//' | awk '/=/ {print $1}' )
fi

echo "Installing requirements in venv..."
/usr/bin/env pip3 install -r requirements.txt > /dev/null

echo "Installing pgml extension globally..."
cd pgml/ > /dev/null
/bin/pip3 install . > /dev/null
cd ../ > /dev/null

echo "Creating databases..."
sudo -u postgres createdb pgml_development
./scripts/manage.py migrate 
