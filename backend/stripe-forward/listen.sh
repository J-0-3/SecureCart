#!/bin/sh

stripe listen --forward-to http://api/webhook/stripe --api-key $(cat /run/secrets/stripe_secret_key)
