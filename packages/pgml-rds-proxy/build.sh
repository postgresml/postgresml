#!/bin/bash
#
# pgml-rds-proxy docker entrypoint
#
# Author: PostgresML <team@postgresml.org>
# License: MIT
#
architecture=$(arch)
name=$(uname)
url="https://static.postgresml.org/packages/pgcat"

if [[ "$architecture" == "aarch64" && "$name" == "Linux" ]]; then
	url="${url}/arm64/pgcat"
elif [[ "$architecture" == "x86_64" && "$name" == "Linux" ]]; then
	url="${url}/amd64/pgcat"
else
	echo "Unsupported platform: ${name} ${architecture}"
	exit 1
fi

echo "Downloading pgcat from $url"
curl -L -o /usr/local/bin/pgcat ${url}
chmod +x /usr/local/bin/pgcat
