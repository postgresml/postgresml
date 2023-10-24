#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"

if [[ -z "$package_version" ]]; then
	echo "postgresml package build and release script"
	echo "usage: $0 <package version, e.g. 2.7.12>"
	exit 1
fi

if ! which deb-s3; then
	curl -sLO https://github.com/deb-s3/deb-s3/releases/download/0.11.4/deb-s3-0.11.4.gem
	sudo gem install deb-s3-0.11.4.gem
	deb-s3
fi

function package_name() {
	echo "postgresml-$1-$package_version-ubuntu22.04-all.deb"
}

for pg in {12..16}; do
  bash ${SCRIPT_DIR}/build.sh ${package_version} ${pg}

  if [[ ! -f $(package_name ${pg}) ]]; then
  	echo "File $(package_name ${pg}) doesn't exist"
  	exit 1
  fi
  
  deb-s3 upload \
    --lock \
    --bucket apt.postgresml.org \
    $(package_name ${pg}) \
    --codename $(lsb_release -cs)

  rm $(package_name ${pg}) 
done
