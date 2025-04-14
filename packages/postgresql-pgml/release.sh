#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [[ -z "${1}" ]]; then
  echo "Usage: $0 <package version, e.g. 2.10.0> [ubuntu version, e.g. 22.04]"
  exit 1
fi

export PACKAGE_VERSION=${1}
export TARGET_UBUNTU_VERSION=${2}

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

extension_dir="${SCRIPT_DIR}/../../pgml-extension"

function package_name() {
  local pg_version=$1
  local ubuntu_version=$2
  local arch=$3
  echo "postgresql-pgml-${pg_version}_${PACKAGE_VERSION}-ubuntu${ubuntu_version}-${arch}.deb"
}

build_packages() {
  local ubuntu_version=$1
  local codename=$2
  
  echo "Building packages for Ubuntu ${ubuntu_version} (${codename})"

  # Loop through PostgreSQL versions
  for pg in {11..17}; do
    echo "Building PostgreSQL ${pg} package..."

    release_dir="$extension_dir/target/release/pgml-pg${pg}"
    mkdir -p "$release_dir/DEBIAN"

    export PGVERSION=${pg}
    # Update control file with Ubuntu version
    (cat ${SCRIPT_DIR}/DEBIAN/control |
     envsubst '${PGVERSION} ${PACKAGE_VERSION} ${ARCH}') > "$release_dir/DEBIAN/control"

    # Build the package
    dpkg-deb \
      --root-owner-group \
      -z1 \
      --build "$release_dir" \
      $(package_name ${pg} ${ubuntu_version} ${ARCH})

    # Upload to S3 with a unique ID to avoid lock contention
    deb-s3 upload \
      --lock \
      --visibility=public \
      --bucket apt.postgresml.org \
      $(package_name ${pg} ${ubuntu_version} ${ARCH}) \
      --codename ${codename} \
      --lock-name="${ARCH}-${ubuntu_version}-$(date +%s)"

    # Clean up the package file
    rm $(package_name ${pg} ${ubuntu_version} ${ARCH})
  done
}

# If a specific Ubuntu version is provided, only build for that version
if [[ ! -z "$TARGET_UBUNTU_VERSION" ]]; then
  if [[ -z "${ubuntu_versions[$TARGET_UBUNTU_VERSION]}" ]]; then
    echo "Error: Ubuntu version $TARGET_UBUNTU_VERSION is not supported."
    echo "Supported versions: ${!ubuntu_versions[@]}"
    exit 1
  fi
  
  build_packages "$TARGET_UBUNTU_VERSION" "${ubuntu_versions[$TARGET_UBUNTU_VERSION]}"
else
  # If no version specified, loop through all supported Ubuntu versions
  for ubuntu_version in "${!ubuntu_versions[@]}"; do
    build_packages "$ubuntu_version" "${ubuntu_versions[$ubuntu_version]}"
  done
fi