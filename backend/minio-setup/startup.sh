#!/bin/sh

if [ -z "${BUCKET_NAME}" ]; then 
    BUCKET_NAME=$(cat /run/secrets/minio_bucket)
fi
if [ -z "${ROOT_USER}" ]; then
    ROOT_USER=$(cat /run/secrets/minio_root_user)
fi
if [ -z "${ROOT_PASSWORD}" ]; then
    ROOT_PASSWORD=$(cat /run/secrets/minio_root_password)
fi
if [ -z "${ACCESS_KEY}" ]; then
    ACCESS_KEY=$(cat /run/secrets/minio_access_key)
fi
if [ -z "${SECRET_KEY}" ]; then
    SECRET_KEY=$(cat /run/secrets/minio_secret_key)
fi

mc alias set securecart "http://$MINIO_HOST:9000" "$ROOT_USER" "$ROOT_PASSWORD"
mc mb securecart/$BUCKET_NAME
sed -i "s/<BUCKET_NAME>/$BUCKET_NAME/g" /user-policy.json
mc admin user add securecart "$ACCESS_KEY" "$SECRET_KEY"
mc admin policy create securecart appreadwrite /user-policy.json
mc admin policy attach securecart appreadwrite --user $ACCESS_KEY
mc anonymous set download securecart/$BUCKET_NAME
