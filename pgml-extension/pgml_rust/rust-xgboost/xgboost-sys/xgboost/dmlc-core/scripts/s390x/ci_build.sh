#!/usr/bin/env bash
#
# Execute command within a docker container
#
# Usage: ci_build.sh <DOCKER_IMG_NAME> [-e ENV_VAR] [-it] <COMMAND>
#
# DOCKER_IMG_NAME: Docker image name
# COMMAND: Command to be executed in the docker container
#
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Validate command line arguments.
if [ "$#" -lt 1 ]
then
    echo "Usage: $(basename $0) DOCKER_IMG_NAME COMMAND"
    exit 1
fi

DOCKER_IMG_NAME="$1"
shift 1

while [[ "$1" == "-e" ]]; do
    ENV_VAR="$2"
    CI_DOCKER_EXTRA_PARAMS+=('-e' "${ENV_VAR}")
    shift 2
done

if [[ "$1" == "-it" ]]; then
    CI_DOCKER_EXTRA_PARAMS+=('-it')
    shift 1
fi

COMMAND=("$@")

DOCKER_BINARY="docker"
DOCKER_CONTEXT_PATH="${SCRIPT_DIR}"
WORKSPACE="${WORKSPACE:-${SCRIPT_DIR}/../../}"

# Bash on Ubuntu on Windows
UBUNTU_ON_WINDOWS=$([ -e /proc/version ] && grep -l Microsoft /proc/version || echo "")
# MSYS, Git Bash, etc.
MSYS=$([ -e /proc/version ] && grep -l MINGW /proc/version || echo "")

if [[ -z "$UBUNTU_ON_WINDOWS" ]] && [[ -z "$MSYS" ]]; then
    USER_IDS="-e CI_BUILD_UID=$( id -u ) -e CI_BUILD_GID=$( id -g ) -e CI_BUILD_USER=$( id -un ) -e CI_BUILD_GROUP=$( id -gn ) -e CI_BUILD_HOME=${WORKSPACE}"
fi

# Print arguments.
cat <<EOF
   WORKSPACE: ${WORKSPACE}
   CI_DOCKER_EXTRA_PARAMS: ${CI_DOCKER_EXTRA_PARAMS[*]}
   COMMAND: ${COMMAND[*]}
   DOCKER CONTAINER NAME: ${DOCKER_IMG_NAME}
   USER_IDS: ${USER_IDS}
EOF


# Build the docker container.
echo "Building container (${DOCKER_IMG_NAME})..."
# --pull should be default
docker build \
    -t "${DOCKER_IMG_NAME}" \
    "${DOCKER_CONTEXT_PATH}"

# Check docker build status
if [[ $? != "0" ]]; then
    echo "ERROR: docker build failed."
    exit 1
fi


# Run the command inside the container.
echo "Running '${COMMAND[*]}' inside ${DOCKER_IMG_NAME}..."

# By default we cleanup - remove the container once it finish running (--rm)
# and share the PID namespace (--pid=host) so the process inside does not have
# pid 1 and SIGKILL is propagated to the process inside (jenkins can kill it).

${DOCKER_BINARY} run --rm --pid=host \
    -v "${WORKSPACE}":/workspace \
    -w /workspace \
    ${USER_IDS} \
    "${CI_DOCKER_EXTRA_PARAMS[@]}" \
    "${DOCKER_IMG_NAME}" \
    "${COMMAND[@]}"

