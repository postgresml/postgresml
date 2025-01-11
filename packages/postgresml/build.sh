#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

export PACKAGE_VERSION=${1:-"2.7.12"}
export PGVERSION=${2:-"14"}
export UBUNTU_VERSION=${3:-"24.04"}

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
  postgresml-${PGVERSION}-${PACKAGE_VERSION}-ubuntu${UBUNTU_VERSION}-all.deb