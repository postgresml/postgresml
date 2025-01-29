#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"

if [[ -z "$package_version" ]]; then
  echo "postgresml package build and release script"
  echo "usage: $0 <package version, e.g. 2.10.0>"
  exit 1
fi

# Get Ubuntu version from environment or try to detect it
if [[ -z "${ubuntu_version}" ]]; then
  ubuntu_version=$(lsb_release -rs)
  echo "No ubuntu_version specified, detected: ${ubuntu_version}"
fi

# Map version number to codename
case "${ubuntu_version}" in
  "20.04")
    export CODENAME="focal"
    ;;
  "22.04")
    export CODENAME="jammy"
    ;;
  "24.04")
    export CODENAME="noble"
    ;;
  *)
    echo "Error: Unsupported Ubuntu version: ${ubuntu_version}"
    exit 1
    ;;
esac

if ! which deb-s3; then
  curl -sLO https://github.com/deb-s3/deb-s3/releases/download/0.11.4/deb-s3-0.11.4.gem
  sudo gem install deb-s3-0.11.4.gem
  deb-s3
fi

function package_name() {
  local pg_version=$1
  echo "postgresml-${pg_version}-${package_version}-ubuntu${ubuntu_version}-all.deb"
}

echo "Building packages for Ubuntu ${ubuntu_version} (${CODENAME})"

# Loop through PostgreSQL versions
for pg in {11..17}; do
  echo "Building PostgreSQL ${pg} package..."
  bash ${SCRIPT_DIR}/build.sh ${package_version} ${pg} ${ubuntu_version}

  if [[ ! -f $(package_name ${pg}) ]]; then
    echo "File $(package_name ${pg}) doesn't exist"
    exit 1
  fi

  deb-s3 upload \
    --lock \
    --bucket apt.postgresml.org \
    $(package_name ${pg}) \
    --codename ${CODENAME}

  rm $(package_name ${pg})
done
