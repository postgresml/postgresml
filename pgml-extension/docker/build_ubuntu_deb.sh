#!/bin/bash
#
# Build a .deb for the Postgres and Ubuntu version.
#

PACKAGE_VERSION=${PACKAGE_VERSION:-0.0.0}
OUTPUT_DIR=${OUTPUT_DIR:-"."}

if [[ $(uname -a) == *"aarch64"* ]]; then
    ARCH="arm64"
else
    ARCH="amd64"
fi

PGVERSION=$(pg_config | grep "VERSION")

if [[ $PGVERSION == *"12."* ]]; then
    PGVERSION="12"
elif [[ $PGVERSION == *"13."* ]]; then
    PGVERSION="13"
elif [[ $PGVERSION == *"14."* ]]; then
    PGVERSION="14"
elif [[ $PGVERSION == *"11."* ]]; then
    PGVERSION="11"
elif [[ $PGVERSION == *"10."* ]]; then
    PGVERSION="10"
else
    echo "Unknown PostgreSQL version detected: ${PGVERSION}"
    exit 1
fi

TARGET="target/release/pgml-pg${PGVERSION}"
UBUNTU_VERSION=$(lsb_release -a | grep Release | awk '{ print $2 }')

ls -R ${TARGET}

mkdir -p ${TARGET}/DEBIAN
cp control ${TARGET}/DEBIAN/control

# Save version and arch.
sed -i "s/PGVERSION/${PGVERSION}/g" ${TARGET}/DEBIAN/control
sed -i "s/PACKAGE_VERSION/${PACKAGE_VERSION}/g" ${TARGET}/DEBIAN/control
sed -i "s/ARCH/${ARCH}/g" ${TARGET}/DEBIAN/control

# Show me what we got.
cat ${TARGET}/DEBIAN/control

PACKAGE=postgresql-pgml-${PGVERSION}_${PACKAGE_VERSION}-ubuntu${UBUNTU_VERSION}-${ARCH}.deb

# Build the debian package
dpkg-deb --build ${TARGET} $OUTPUT_DIR/${PACKAGE}
