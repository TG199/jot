#!/usr/bin/bash
set -x
set -eo pipefail

REDIS_PORT="${REDIS_PORT:=6379}"

if [[ -z "${SKIP_DOCKER}" ]]
then
  docker run \
    -p "${REDIS_PORT}":6379 \
    -d redis:7 \
    redis-server
fi

>&2 echo "Redis is ready to go on port ${REDIS_PORT}!"