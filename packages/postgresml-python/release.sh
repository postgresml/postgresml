#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"

if [[ -z "$package_version" ]]; then
  echo "postgresml-python package build and release script"
  echo "Usage: $0 <package version, e.g. 2.10.0>"
  exit 1
fi

# Active LTS Ubuntu versions and their codenames
declare -A ubuntu_versions=(
  ["20.04"]="focal"
  ["22.04"]="jammy"
  ["24.04"]="noble"
)

# Supported architectures
declare -a architectures=("amd64" "arm64")

# Install deb-s3 if not present
if ! which deb-s3; then
  curl -sLO https://github.com/deb-s3/deb-s3/releases/download/0.11.4/deb-s3-0.11.4.gem
  sudo gem install deb-s3-0.11.4.gem
  deb-s3
fi

# Install Python dependencies
sudo apt install python3-pip python3 python3-virtualenv -y

function package_name() {
  local ubuntu_version=$1
  local arch=$2
  echo "postgresml-python-${package_version}-ubuntu${ubuntu_version}-${arch}.deb"
}

# Loop through Ubuntu versions
for ubuntu_version in "${!ubuntu_versions[@]}"; do
  codename=${ubuntu_versions[$ubuntu_version]}
  echo "Building packages for Ubuntu ${ubuntu_version} (${codename})"

  # Loop through architectures
  for arch in "${architectures[@]}"; do
    echo "Building for architecture: ${arch}"
    export ARCH=${arch}

    # Build the Python package
    bash ${SCRIPT_DIR}/build.sh "$package_version" "$ubuntu_version"

    if [[ ! -f $(package_name ${ubuntu_version} ${arch}) ]]; then
      echo "File $(package_name ${ubuntu_version} ${arch}) doesn't exist"
      exit 1
    fi

    # Upload to S3
    deb-s3 upload \
      --lock \
      --bucket apt.postgresml.org \
      $(package_name ${ubuntu_version} ${arch}) \
      --codename ${codename}

    # Clean up the package file
    rm $(package_name ${ubuntu_version} ${arch})
  done
done
