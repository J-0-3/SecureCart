#!/usr/bin/env bash

AMBER='\033[1;33m'
RED='\033[1;31m'
GREEN='\033[1;32m'
RESET='\033[0m'
BOLD='\033[1m'

if [ -f ".env" ]; then
    set -a
    source .env
    set +a
fi

if [[ "$1" == "down" ]]; then
    echo -e "[${AMBER}*${RESET}] ${AMBER} Stopping all services... ${RESET}"
    COMPOSE_BAKE=true docker --log-level ERROR compose -f ./compose.yml down || { 
        echo -e "[${RED}!${RESET}] Failed to stop running services."
        exit 1
    }
    echo -e "[${GREEN}✔${RESET}] ${GREEN}All services stopped successfully!${RESET}"
    exit 0
fi

COMPOSE_FLAGS=""

if [[ "${BUILD}" == "true" ]]; then
    COMPOSE_FLAGS="$COMPOSE_FLAGS --build"
fi

if [[ "${RECREATE}" == "true" ]]; then
    COMPOSE_FLAGS="$COMPOSE_FLAGS --force-recreate"
fi

ECHO_SECRETS_AT_END=false

if [[ -z "${DB_DATABASE}" ]]; then
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}DB_DATABASE${RESET}${AMBER} is not set. Will use the default ${RESET}${BOLD}securecart${RESET}${AMBER}.${RESET}"
    export DB_DATABASE=securecart
fi
if [[ -z "${DB_USERNAME}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}DB_USERNAME${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export DB_USERNAME="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${DB_PASSWORD}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}DB_PASSWORD${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export DB_PASSWORD="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${DB_ENCRYPTION_KEY}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}DB_ENCRYPTION_KEY${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export DB_ENCRYPTION_KEY="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${ADMIN_EMAIL}" ]]; then
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}ADMIN_EMAIL${RESET}${AMBER} is not set. Will use the default ${RESET}${BOLD}admin@securecart.dev${RESET}${AMBER}.${RESET}"
    export ADMIN_EMAIL="admin@securecart.dev"
fi
if [[ -z "${ADMIN_PASSWORD}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}ADMIN_PASSWORD${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export ADMIN_PASSWORD="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${MINIO_ROOT_USER}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}MINIO_ROOT_USER${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export MINIO_ROOT_USER="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${MINIO_ROOT_PASSWORD}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}MINIO_ROOT_PASSWORD${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export MINIO_ROOT_PASSWORD="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${MINIO_ACCESS_KEY}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}MINIO_ACCESS_KEY${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export MINIO_ACCESS_KEY="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi
if [[ -z "${MINIO_SECRET_KEY}" ]]; then
    ECHO_SECRETS_AT_END=true
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}MINIO_SECRET_KEY${RESET}${AMBER} is not set. Will randomly generate a value.${RESET}"
    export MINIO_SECRET_KEY="$(cat /dev/urandom | LC_ALL=C tr -dc 'a-zA-Z0-9' | fold -w 50 | head -n 1)"
fi


if [[ "${ENABLE_STRIPE}" == "true" ]]; then
    if [[ -z "${STRIPE_SECRET_KEY}" || -z "${STRIPE_PUBLISHABLE_KEY}" ]]; then
        echo -e "[${RED}!${RESET}] ${RED}ENABLE_STRIPE is set, but either STRIPE_SECRET_KEY or STRIPE_PUBLISHABLE_KEY is missing${RESET}"
        exit 1
    fi
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}ENABLE_STRIPE${RESET}${AMBER}, ${BOLD}STRIPE_SECRET_KEY${RESET}${AMBER}, and ${BOLD}STRIPE_PUBLISHABLE_KEY${RESET}${AMBER} are all set. Building with Stripe support.${RESET}"
    if [[ -z "${STRIPE_WEBHOOK_SECRET}" ]]; then
        echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}STRIPE_WEBHOOK_SECRET${RESET}${AMBER} is missing. Will start a Stripe CLI event listener for testing.${RESET}"
        echo -e "[.] Starting Stripe webhook event listener."
        
        COMPOSE_BAKE=true docker compose --profile stripe-webhook -f ./compose.yml up $COMPOSE_FLAGS --no-deps -d stripe-webhook-forward-local || { 
            echo -e "[${RED}!${RESET}] ${RED}Failed to start Stripe webhook listener.${RESET}"
            exit 1
        }
        echo -e "[${GREEN}✔${RESET}] Stripe webhook event listener started successfully."
        echo -e "[.] Waiting for Stripe webhook secret..."
        STRIPE_WEBHOOK_SECRET=""
        for i in {1..60}; do
            STRIPE_WEBHOOK_SECRET=$(docker compose --profile stripe-webhook -f ./compose.yml logs --no-log-prefix --no-color stripe-webhook-forward-local | grep -oE -m1 "whsec_[A-Za-z0-9]+")
            if [[ -n "${STRIPE_WEBHOOK_SECRET}" ]]; then
                break
            fi
            sleep 1
        done
        echo -e "[${GREEN}✔${RESET}] Stripe webhook secret is: ${STRIPE_WEBHOOK_SECRET}."
    fi
    COMPOSE_BAKE=true STRIPE_WEBHOOK_SECRET="$STRIPE_WEBHOOK_SECRET" docker compose -f ./compose.yml up $COMPOSE_FLAGS -d || {
        echo -e "[${RED}!${RESET}] Failed to start services."
        exit 1
    }
else 
    echo -e "[${AMBER}*${RESET}] ${AMBER}${BOLD}ENABLE_STRIPE${RESET}${AMBER} is not set, building without Stripe support.${RESET}"
    echo -e "[.] Starting all services."
    
    STRIPE_SECRET_KEY='' STRIPE_PUBLISHABLE_KEY='' STRIPE_WEBHOOK_SECRET='' COMPOSE_BAKE=true docker compose -f ./compose.yml up $COMPOSE_FLAGS -d || { 
        echo -e "[${RED}!${RESET}] Failed to start services."
        exit 1
    }
fi
echo -e "[${GREEN}✔${RESET}] ${GREEN}All services started successfully.${RESET}"
if [[ -n "{ECHO_SECRETS_AT_END}" ]]; then
    echo -e "[.] Some secrets were not set in environment variables, and have been randomly generated."
    echo -e -n "[.] You will likely need these to access and restart services in the future. Would you like me to list them for you? (Y/n) "
    read choice
    if [[ "${choice}" != [Nn]* ]]; then
        echo -e "[${AMBER}*${RESET}] ${AMBER}Make sure to keep these values secret!${RESET}"
        echo "DB_USERNAME=${DB_USERNAME}"
        echo "DB_PASSWORD=${DB_PASSWORD}"
        echo "DB_ENCRYPTION_KEY=${DB_ENCRYPTION_KEY}"
        echo "ADMIN_EMAIL=${ADMIN_EMAIL}"
        echo "ADMIN_PASSWORD=${ADMIN_PASSWORD}"
        echo "MINIO_ROOT_USER=${MINIO_ROOT_USER}"
        echo "MINIO_ROOT_PASSWORD=${MINIO_ROOT_PASSWORD}"
        echo "MINIO_ACCESS_KEY=${MINIO_ACCESS_KEY}"
        echo "MINIO_SECRET_KEY=${MINIO_SECRET_KEY}"
    fi
    if [ -f ".env" ]; then
        echo -e "[.] Would you like me to append them to your .env file so that they can be automatically loaded in the future? (Y/n) "
    else
        echo -e "[.] Would you like me to create a .env file containing them so that they can be automatically loaded in the future? (Y/n) "
    fi
    read choice
    if [[ "${choice}" != [Nn]* ]]; then
        echo "DB_USERNAME=${DB_USERNAME}" >> .env
        echo "DB_PASSWORD=${DB_PASSWORD}" >> .env
        echo "DB_ENCRYPTION_KEY=${DB_ENCRYPTION_KEY}" >> .env
        echo "ADMIN_EMAIL=${ADMIN_EMAIL}" >> .env
        echo "ADMIN_PASSWORD=${ADMIN_PASSWORD}" >> .env
        echo "MINIO_ROOT_USER=${MINIO_ROOT_USER}" >> .env
        echo "MINIO_ROOT_PASSWORD=${MINIO_ROOT_PASSWORD}" >> .env
        echo "MINIO_ACCESS_KEY=${MINIO_ACCESS_KEY}" >> .env
        echo "MINIO_SECRET_KEY=${MINIO_SECRET_KEY}" >> .env
        echo -e "[.] Secrets have been added to your local .env!"
    fi
fi
