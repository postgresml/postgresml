#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [[ -z "${1}" ]]; then
  echo "Usage: $0 <package version, e.g. 2.9.4>"
  exit 1
fi

export PACKAGE_VERSION=${1}
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

extension_dir="${SCRIPT_DIR}/../../pgml-extension"

function package_name() {
  echo "postgresql-pgml-${1}_${PACKAGE_VERSION}-ubuntu22.04-${ARCH}.deb"
}

for pg in {12..16}; do
  release_dir="$extension_dir/target/release/pgml-pg${pg}"

  mkdir -p "$release_dir/DEBIAN"

  export PGVERSION=${pg}
  (cat ${SCRIPT_DIR}/DEBIAN/control | envsubst '${PGVERSION} ${PACKAGE_VERSION} ${ARCH}') > "$release_dir/DEBIAN/control"

  dpkg-deb \
    --root-owner-group \
    -z1 \
    --build "$release_dir" \
    $(package_name ${pg})

  deb-s3 upload \
    --bucket apt.postgresml.org \
    $(package_name ${pg}) \
    --codename $(lsb_release -cs)
done
