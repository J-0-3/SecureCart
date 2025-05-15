#!/bin/sh

curl -fI http://localhost:9000/minio/health/live || exit 1
