#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"

if [[ -z "$package_version" ]]; then
  echo "Usage: $0 <package version, e.g. 2.10.0>"
  exit 1
fi

if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
else
  export ARCH=arm64
fi

if ! which deb-s3; then
  curl -sLO https://github.com/deb-s3/deb-s3/releases/download/0.11.4/deb-s3-0.11.4.gem
  sudo gem install deb-s3-0.11.4.gem
  deb-s3
fi

function package_name() {
  echo "postgresml-dashboard-${package_version}-ubuntu22.04-${ARCH}.deb"
}

bash ${SCRIPT_DIR}/build.sh "$package_version"

if [[ ! -f $(package_name) ]]; then
  echo "File $(package_name) doesn't exist"
  exit 1
fi

deb-s3 upload \
  --lock \
  --bucket apt.postgresml.org \
  $(package_name) \
  --codename $(lsb_release -cs)
