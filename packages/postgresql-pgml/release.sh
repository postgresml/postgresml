#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [[ -z "${1}" ]]; then
  echo "Usage: $0 <package version, e.g. 2.10.0>"
  exit 1
fi

export PACKAGE_VERSION=${1}

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

extension_dir="${SCRIPT_DIR}/../../pgml-extension"

function package_name() {
  local pg_version=$1
  local ubuntu_version=$2
  local arch=$3
  echo "postgresql-pgml-${pg_version}_${PACKAGE_VERSION}-ubuntu${ubuntu_version}-${arch}.deb"
}

# Loop through Ubuntu versions
for ubuntu_version in "${!ubuntu_versions[@]}"; do
  codename=${ubuntu_versions[$ubuntu_version]}
  echo "Building packages for Ubuntu ${ubuntu_version} (${codename})"

  # Loop through architectures
  for arch in "${architectures[@]}"; do
    echo "Building for architecture: ${arch}"
    export ARCH=${arch}

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
        $(package_name ${pg} ${ubuntu_version} ${arch})

      # Upload to S3
      deb-s3 upload \
        --lock \
        --bucket apt.postgresml.org \
        $(package_name ${pg} ${ubuntu_version} ${arch}) \
        --codename ${codename}

      # Clean up the package file
      rm $(package_name ${pg} ${ubuntu_version} ${arch})
    done
  done
done
