#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
package_version="$1"
target_ubuntu_version="$2"

if [[ -z "$package_version" ]]; then
  echo "postgresml-python package build and release script"
  echo "Usage: $0 <package version, e.g. 2.10.0> [ubuntu version, e.g. 22.04]"
  exit 1
fi

# Active LTS Ubuntu versions and their codenames
declare -A ubuntu_versions=(
  ["20.04"]="focal"
  ["22.04"]="jammy"
  ["24.04"]="noble"
)

# Detect current architecture
if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
elif [[ $(arch) == "aarch64" ]]; then
  export ARCH=arm64
else
  echo "Unsupported architecture: $(arch)"
  exit 1
fi

echo "Building for architecture: ${ARCH}"

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

build_package() {
  local ubuntu_version=$1
  local codename=$2
  
  echo "Building packages for Ubuntu ${ubuntu_version} (${codename})"

  # Build the Python package
  bash ${SCRIPT_DIR}/build.sh "$package_version" "$ubuntu_version"

  if [[ ! -f $(package_name ${ubuntu_version} ${ARCH}) ]]; then
    echo "File $(package_name ${ubuntu_version} ${ARCH}) doesn't exist"
    exit 1
  fi

  # Upload to S3 with a unique ID to avoid lock contention
  deb-s3 upload \
    --lock \
    --visibility=public \
    --bucket apt.postgresml.org \
    $(package_name ${ubuntu_version} ${ARCH}) \
    --codename ${codename} \
    --lock-name="${ARCH}-${ubuntu_version}-$(date +%s)"

  # Clean up the package file
  rm $(package_name ${ubuntu_version} ${ARCH})
}

# If a specific Ubuntu version is provided, only build for that version
if [[ ! -z "$target_ubuntu_version" ]]; then
  if [[ -z "${ubuntu_versions[$target_ubuntu_version]}" ]]; then
    echo "Error: Ubuntu version $target_ubuntu_version is not supported."
    echo "Supported versions: ${!ubuntu_versions[@]}"
    exit 1
  fi
  
  build_package "$target_ubuntu_version" "${ubuntu_versions[$target_ubuntu_version]}"
else
  # If no version specified, loop through all supported Ubuntu versions
  for ubuntu_version in "${!ubuntu_versions[@]}"; do
    build_package "$ubuntu_version" "${ubuntu_versions[$ubuntu_version]}"
  done
fi