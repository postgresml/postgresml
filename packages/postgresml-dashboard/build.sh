#!/bin/bash
set -e

dir="/tmp/postgresml-dashboard"
deb_dir="$dir/deb-build"
source_dir="$dir/source"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
export PACKAGE_VERSION=${1:-"2.9.4"}
export GITHUB_STARS=$(curl -s "https://api.github.com/repos/postgresml/postgresml" | grep stargazers_count | cut -d : -f 2 | tr -d " " | tr -d ",")
if [[ $(arch) == "x86_64" ]]; then
  export ARCH=amd64
else
  export ARCH=arm64
fi

rm -rf "$dir"
mkdir -p "$deb_dir"

cp -R ${SCRIPT_DIR}/* "$deb_dir"
rm "$deb_dir/build.sh"
rm "$deb_dir/release.sh"


( cd ${SCRIPT_DIR}/../../pgml-dashboard && \
  cargo build --release && \
  cp target/release/pgml-dashboard "$deb_dir/usr/bin/pgml-dashboard" && \
  cp -R static "$deb_dir/usr/share/pgml-dashboard/dashboard-static" && \
  cp -R ../pgml-cms "$deb_dir/usr/share/pgml-cms" )

(cat ${SCRIPT_DIR}/DEBIAN/control | envsubst) > "$deb_dir/DEBIAN/control"
(cat ${SCRIPT_DIR}/etc/systemd/system/pgml-dashboard.service | envsubst) > "$deb_dir/etc/systemd/system/pgml-dashboard.service"

chmod 755 ${deb_dir}/DEBIAN/post*
chmod 755 ${deb_dir}/DEBIAN/pre*

dpkg-deb \
  --root-owner-group \
  --build "$deb_dir" \
  postgresml-dashboard-${PACKAGE_VERSION}-ubuntu22.04-${ARCH}.deb

rm -rf "$dir"
