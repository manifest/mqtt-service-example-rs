#!/bin/bash

PROJECT="mqtt-svc-ex"
PROJECT_DIR="/build"
DOCKER_CONTAINER_NAME="sandbox/${PROJECT}.develop"
DOCKER_CONTAINER_COMMAND=${DOCKER_CONTAINER_COMMAND:-'/bin/bash'}
DOCKER_RUN_OPTIONS=${DOCKER_RUN_OPTIONS:-'-ti --rm'}
DOCKER_MQTT_PORT=${DOCKER_MQTT_PORT:-'1883'}

read -r DOCKER_RUN_COMMAND <<-EOF
    vernemq start
EOF

set -ex

docker build -t ${DOCKER_CONTAINER_NAME} -f docker/develop.dockerfile .
docker run ${DOCKER_RUN_OPTIONS} \
    -v $(pwd):${PROJECT_DIR} \
    -p ${DOCKER_MQTT_PORT}:1883 \
    ${DOCKER_CONTAINER_NAME} \
    /bin/bash -c "set -x && cd ${PROJECT_DIR} && ${DOCKER_RUN_COMMAND} && set +x && ${DOCKER_CONTAINER_COMMAND}"
