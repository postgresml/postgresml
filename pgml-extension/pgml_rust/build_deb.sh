#!/bin/bash

VERSION=${1:-0.0.1}
ARCH=${2:-amd64}

mkdir -p target/release/pgml_rust-pg14/DEBIAN
cp control target/release/pgml_rust-pg14/DEBIAN/control
sed -i "s/VERSION/${VERSION}/g" target/release/pgml_rust-pg14/DEBIAN/control
sed -i "s/ARCH/${ARCH}/g" target/release/pgml_rust-pg14/DEBIAN/control

cat target/release/pgml_rust-pg14/DEBIAN/control

cd target/release
dpkg-deb --build pgml_rust-pg14

mv pgml_rust-pg14.deb postgresql-pgml-14_${VERSION}-ubuntu1.0-${ARCH}.deb
