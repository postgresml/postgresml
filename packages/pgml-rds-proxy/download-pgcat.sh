#!/bin/bash
#
# Download the right version of pgcat for the architecture.
#
# Author: PostgresML <team@postgresml.org>
# License: MIT
#
architecture=$(arch)
name=$(uname)
url="https://static.postgresml.org/packages/pgcat"
version="$PGCAT_VERSION"
bin_name="pgcat2-$version.bin"

if [[ -z "$version" ]]; then
	echo "PGCAT_VERSION environment variable is not set"
	exit 1
fi

if [[ "$architecture" == "aarch64" && "$name" == "Linux" ]]; then
	url="${url}/arm64/$bin_name"
elif [[ "$architecture" == "x86_64" && "$name" == "Linux" ]]; then
	url="${url}/amd64/$bin_name"
else
	echo "Unsupported platform: ${name} ${architecture}"
	exit 1
fi

echo "Downloading pgcat from $url"
curl -L -o /usr/local/bin/pgcat ${url}
chmod +x /usr/local/bin/pgcat
