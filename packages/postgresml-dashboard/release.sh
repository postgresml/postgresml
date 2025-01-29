#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"

if [[ -z "$package_version" ]]; then
  echo "postgresml dashboard package build and release script"
  echo "Usage: $0 <package version, e.g. 2.10.0>"
  exit 1
fi

# Set architecture based on system unless overridden by environment
if [[ -z "${ARCH}" ]]; then
  if [[ $(arch) == "x86_64" ]]; then
    export ARCH=amd64
  else
    export ARCH=arm64
  fi
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
  echo "postgresml-dashboard-${package_version}-ubuntu${ubuntu_version}-${ARCH}.deb"
}

echo "Building package for Ubuntu ${ubuntu_version} (${CODENAME}) ${ARCH}"

# Build the package
bash ${SCRIPT_DIR}/build.sh "$package_version" "$ubuntu_version" "$ARCH"

if [[ ! -f $(package_name) ]]; then
  echo "File $(package_name) doesn't exist"
  exit 1
fi

# Upload to S3
deb-s3 upload \
  --lock \
  --bucket apt.postgresml.org \
  $(package_name) \
  --codename ${CODENAME}

# Clean up the package file
rm $(package_name)
