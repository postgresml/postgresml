#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [[ -z "${1}" ]]; then
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

export PACKAGE_VERSION=${1}

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

extension_dir="${SCRIPT_DIR}/../../pgml-extension"

function package_name() {
  local pg_version=$1
  echo "postgresql-pgml-${pg_version}_${PACKAGE_VERSION}-ubuntu${ubuntu_version}-${ARCH}.deb"
}

echo "Building packages for Ubuntu ${ubuntu_version} (${CODENAME}) ${ARCH}"

# Loop through PostgreSQL versions
for pg in {12..17}; do
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
    $(package_name ${pg})

  # Upload to S3
  deb-s3 upload \
    --lock \
    --bucket apt.postgresml.org \
    $(package_name ${pg}) \
    --codename ${CODENAME}

  # Clean up the package file
  rm $(package_name ${pg})
done
