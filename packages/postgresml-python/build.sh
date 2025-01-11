#!/bin/bash
#
#
#
set -e
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
deb_dir="/tmp/postgresml-python/deb-build"
major=${1:-"14"}

export PACKAGE_VERSION=${1:-"2.9.4"}
export PYTHON_VERSION=${2:-"3.10"}

if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
else
  export ARCH=arm64
fi

rm -rf "$deb_dir"
mkdir -p "$deb_dir"

cp -R ${SCRIPT_DIR}/* "$deb_dir"
rm "$deb_dir/build.sh"
rm "$deb_dir/release.sh"

(cat ${SCRIPT_DIR}/DEBIAN/control | envsubst) > "$deb_dir/DEBIAN/control"
(cat ${SCRIPT_DIR}/DEBIAN/postinst | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/postinst"
(cat ${SCRIPT_DIR}/DEBIAN/prerm | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/prerm"
(cat ${SCRIPT_DIR}/DEBIAN/postrm | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/postrm"

if [[ "$ARCH" == "amd64" ]]; then
  cp ${SCRIPT_DIR}/../../pgml-extension/requirements.linux.txt "$deb_dir/etc/postgresml-python/requirements.txt"
else
  cp ${SCRIPT_DIR}/../../pgml-extension/requirements.macos.txt "$deb_dir/etc/postgresml-python/requirements.txt"
fi

virtualenv --python="python$PYTHON_VERSION" "$deb_dir/var/lib/postgresml-python/pgml-venv"
source "$deb_dir/var/lib/postgresml-python/pgml-venv/bin/activate"

python -m pip install -r "${deb_dir}/etc/postgresml-python/requirements.txt"

deactivate

chmod 755 ${deb_dir}/DEBIAN/post*
chmod 755 ${deb_dir}/DEBIAN/pre*

dpkg-deb \
  --root-owner-group \
  -z1 \
  --build "$deb_dir" \
  postgresml-python-${PACKAGE_VERSION}-ubuntu22.04-${ARCH}.deb

rm -rf "$deb_dir"
