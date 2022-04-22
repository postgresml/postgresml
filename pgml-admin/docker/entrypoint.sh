#!/bin/bash

# Exit on error, real CI
set -e

echo "Installing requirements.txt..."
pip install -r requirements.txt > /dev/null

echo "Ready"

if [[ ! -z $@ ]]; then
	echo
	echo "To visit admin: "
	echo "  http://127.0.0.1:8000/"
	echo
	$@
fi

