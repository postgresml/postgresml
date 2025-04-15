#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
deb_dir="/tmp/postgresml-python/deb-build"

# Parse arguments with defaults
export PACKAGE_VERSION=${1:-"2.10.0"}
export UBUNTU_VERSION=${2:-"22.04"}
export PYTHON_VERSION=${3:-"3.11"}

# Handle architecture
if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
else
  export ARCH=arm64
fi

# We use Python 3.11 for all Ubuntu versions for better dependency compatibility
if [[ -z "$3" ]]; then
  PYTHON_VERSION="3.11"
fi

rm -rf "$deb_dir"
mkdir -p "$deb_dir"

cp -R ${SCRIPT_DIR}/* "$deb_dir"
rm "$deb_dir/build.sh"
rm "$deb_dir/release.sh"

(cat ${SCRIPT_DIR}/DEBIAN/control | envsubst '${PACKAGE_VERSION} ${UBUNTU_VERSION} ${ARCH} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/control"
(cat ${SCRIPT_DIR}/DEBIAN/postinst | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/postinst"
(cat ${SCRIPT_DIR}/DEBIAN/prerm | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/prerm"
(cat ${SCRIPT_DIR}/DEBIAN/postrm | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/postrm"

if [[ "$ARCH" == "amd64" ]]; then
  # Use AMD64-specific requirements (x86_64)
  cp ${SCRIPT_DIR}/../../pgml-extension/requirements.amd64.txt "$deb_dir/etc/postgresml-python/requirements.txt"
else
  # Use ARM64-specific requirements (aarch64)
  cp ${SCRIPT_DIR}/../../pgml-extension/requirements.arm64.txt "$deb_dir/etc/postgresml-python/requirements.txt"
fi

virtualenv --python="python${PYTHON_VERSION}" "$deb_dir/var/lib/postgresml-python/pgml-venv"
source "$deb_dir/var/lib/postgresml-python/pgml-venv/bin/activate"

# Install PyTorch first to help with dependency resolution
python -m pip install torch

python -m pip install -r "${deb_dir}/etc/postgresml-python/requirements.txt"

deactivate

chmod 755 ${deb_dir}/DEBIAN/post*
chmod 755 ${deb_dir}/DEBIAN/pre*

dpkg-deb \
  --root-owner-group \
  -z1 \
  --build "$deb_dir" \
  "postgresml-python-${PACKAGE_VERSION}-ubuntu${UBUNTU_VERSION}-${ARCH}.deb"

rm -rf "$deb_dir"
