#!/bin/sh

curl -f http://localhost || exit 1
curl -f http://localhost/auth || exit 1
curl -f http://localhost/onboard || exit 1
