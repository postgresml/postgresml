#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Parse arguments with environment variable fallbacks
export PACKAGE_VERSION=${1:-${PACKAGE_VERSION:-"2.10.0"}}
export PGVERSION=${2:-${PGVERSION:-"17"}}
export UBUNTU_VERSION=${3:-${ubuntu_version:-$(lsb_release -rs)}}

echo "Building package:"
echo "- Package Version: ${PACKAGE_VERSION}"
echo "- PostgreSQL Version: ${PGVERSION}"
echo "- Ubuntu Version: ${UBUNTU_VERSION}"

deb_dir="/tmp/postgresml/deb-build"

rm -rf "$deb_dir"
mkdir -p "$deb_dir"

cp -R ${SCRIPT_DIR}/* "$deb_dir"
rm "$deb_dir/build.sh"
rm "$deb_dir/release.sh"

(cat ${SCRIPT_DIR}/DEBIAN/control | envsubst) > "$deb_dir/DEBIAN/control"
(cat ${SCRIPT_DIR}/DEBIAN/postinst | envsubst) > "$deb_dir/DEBIAN/postinst"
(cat ${SCRIPT_DIR}/DEBIAN/prerm | envsubst) > "$deb_dir/DEBIAN/prerm"

chmod 755 ${deb_dir}/DEBIAN/post*
chmod 755 ${deb_dir}/DEBIAN/pre*

dpkg-deb \
  --root-owner-group \
  -z1 \
  --build "$deb_dir" \
  "postgresml-${PGVERSION}-${PACKAGE_VERSION}-ubuntu${UBUNTU_VERSION}-all.deb"
