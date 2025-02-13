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
if [ -z "${APP_USER}" ]; then
    APP_USER=$(cat /run/secrets/minio_app_user)
fi
if [ -z "${APP_PASSWORD}" ]; then
    APP_PASSWORD=$(cat /run/secrets/minio_app_password)
fi

mc alias set securecart "http://$MINIO_HOST:9000" "$ROOT_USER" "$ROOT_PASSWORD"
mc mb securecart/$BUCKET_NAME
sed -i "s/<BUCKET_NAME>/$BUCKET_NAME/g" /user-policy.json
mc admin user add securecart "$APP_USER" "$APP_PASSWORD"
mc admin policy create securecart appreadwrite /user-policy.json
mc admin policy attach securecart appreadwrite --user $APP_USER
mc anonymous set download securecart/$BUCKET_NAME
