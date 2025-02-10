#!/bin/bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
deb_dir="/tmp/postgresml-python/deb-build"

# Parse arguments with environment variable fallbacks
export PACKAGE_VERSION=${1:-${PACKAGE_VERSION:-"2.10.0"}}
export UBUNTU_VERSION=${2:-${ubuntu_version:-$(lsb_release -rs)}}
export PYTHON_VERSION=${3:-${PYTHON_VERSION:-"3.10"}}

# Set architecture from environment or detect it
if [[ -z "${ARCH}" ]]; then
  if [[ $(arch) == "x86_64" ]]; then
    export ARCH=amd64
  else
    export ARCH=arm64
  fi
fi

echo "Building package:"
echo "- Package Version: ${PACKAGE_VERSION}"
echo "- Ubuntu Version: ${UBUNTU_VERSION}"
echo "- Python Version: ${PYTHON_VERSION}"
echo "- Architecture: ${ARCH}"

rm -rf "$deb_dir"
mkdir -p "$deb_dir"

cp -R ${SCRIPT_DIR}/* "$deb_dir"
rm "$deb_dir/build.sh"
rm "$deb_dir/release.sh"

# Process template files
(cat ${SCRIPT_DIR}/DEBIAN/control | envsubst '${PACKAGE_VERSION} ${UBUNTU_VERSION} ${ARCH} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/control"
(cat ${SCRIPT_DIR}/DEBIAN/postinst | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/postinst"
(cat ${SCRIPT_DIR}/DEBIAN/prerm | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/prerm"
(cat ${SCRIPT_DIR}/DEBIAN/postrm | envsubst '${PGVERSION} ${PYTHON_VERSION}') > "$deb_dir/DEBIAN/postrm"

# Select requirements file based on Ubuntu version and architecture
if [[ "${UBUNTU_VERSION}" == "20.04" ]]; then
  # Frozen requirements are not longer available on Ubuntu 20.04
  cp ${SCRIPT_DIR}/../../pgml-extension/requirements.txt "$deb_dir/etc/postgresml-python/requirements.txt"
  echo "Recomputing requirements.txt for Ubuntu 20.04"
else
  # Use frozen requirements for newer Ubuntu versions
  if [[ "$ARCH" == "amd64" ]]; then
    cp ${SCRIPT_DIR}/../../pgml-extension/requirements.linux.txt "$deb_dir/etc/postgresml-python/requirements.txt"
    echo "Using frozen Linux requirements for Ubuntu ${UBUNTU_VERSION}"
  else
    cp ${SCRIPT_DIR}/../../pgml-extension/requirements.macos.txt "$deb_dir/etc/postgresml-python/requirements.txt"
    echo "Using frozen macOS requirements for Ubuntu ${UBUNTU_VERSION}"
  fi
fi

# Create and populate virtualenv
echo "Creating Python virtual environment with Python ${PYTHON_VERSION}"
virtualenv --python="python${PYTHON_VERSION}" "$deb_dir/var/lib/postgresml-python/pgml-venv"
source "$deb_dir/var/lib/postgresml-python/pgml-venv/bin/activate"

pip install --upgrade setuptools

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
