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
  local pg_version=$1
  local ubuntu_version=$2
  echo "postgresml-${pg_version}-${package_version}-ubuntu${ubuntu_version}-all.deb"
}

# Active LTS Ubuntu versions
ubuntu_versions=("20.04" "22.04" "24.04")

# Map Ubuntu versions to codenames
declare -A ubuntu_codenames=(
  ["20.04"]="focal"
  ["22.04"]="jammy"
  ["24.04"]="noble"
)

for ubuntu_version in "${ubuntu_versions[@]}"; do
  codename=${ubuntu_codenames[$ubuntu_version]}
  echo "Building packages for Ubuntu ${ubuntu_version} (${codename})"

  for pg in {11..17}; do
    echo "Building PostgreSQL ${pg} package..."
    bash ${SCRIPT_DIR}/build.sh ${package_version} ${pg} ${ubuntu_version}

    if [[ ! -f $(package_name ${pg} ${ubuntu_version}) ]]; then
      echo "File $(package_name ${pg} ${ubuntu_version}) doesn't exist"
      exit 1
    fi

    deb-s3 upload \
      --lock \
      --bucket apt.postgresml.org \
      $(package_name ${pg} ${ubuntu_version}) \
      --codename ${codename}

    rm $(package_name ${pg} ${ubuntu_version})
  done
done
