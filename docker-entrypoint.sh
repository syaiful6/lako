#!/bin/sh

set -e

until diesel migration run --locked-schema; do
  echo "Migration failed, retrying in 5 seconds..."
  sleep 5
done

exec "$@"
