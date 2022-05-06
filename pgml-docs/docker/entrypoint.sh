#!/bin/bash

# Exit on error, real CI
set -e

if [[ ! -z $@ ]]; then
	echo
	echo "To visit docs: "
	echo "  http://127.0.0.1:8001/"
	echo
	$@
fi

