#!/bin/bash
#
#
#
set -e

deb_dir="/tmp/postgresml/deb-build"
major=${1:-"14"}

export PGVERSION=${major}
export PACKAGE_VERSION=${1:-"2.7.4"}
export PYTHON_VERSION=${2:-"3.10"}

if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
  export PGVECTOR_MARCH=skylake
else
  export ARCH=arm64
  export PGVECTOR_MARCH=native
fi

rm -rf "$deb_dir"
mkdir -p "$deb_dir"

cp -R packages/postgresml/* "$deb_dir"
rm "$deb_dir/build.sh"

(cat packages/postgresml/DEBIAN/control | envsubst) > "$deb_dir/DEBIAN/control"
(cat packages/postgresml/DEBIAN/postinst | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/postinst"
(cat packages/postgresml/DEBIAN/prerm | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/prerm"
(cat packages/postgresml/DEBIAN/postrm | envsubst '${PGVERSION}') > "$deb_dir/DEBIAN/postrm"

cp pgml-extension/requirements.txt "$deb_dir/etc/postgresml/requirements.txt"
cp pgml-extension/requirements-xformers.txt "$deb_dir/etc/postgresml/requirements-xformers.txt"

virtualenv --python="python$PYTHON_VERSION" "$deb_dir/var/lib/postgresml/pgml-venv"
source "$deb_dir/var/lib/postgresml/pgml-venv/bin/activate"

python -m pip install -r "${deb_dir}/etc/postgresml/requirements.txt"
python -m pip install -r "${deb_dir}/etc/postgresml/requirements-xformers.txt" --no-dependencies

deactivate

chmod 755 ${deb_dir}/DEBIAN/post*
chmod 755 ${deb_dir}/DEBIAN/pre*

dpkg-deb \
  --root-owner-group \
  -z1 \
  --build "$deb_dir" \
  postgresml-${PGVERSION}-${PACKAGE_VERSION}-ubuntu22.04-${ARCH}.deb

rm -rf "$deb_dir"
