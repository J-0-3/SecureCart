# SecureCart

## Dependencies 
To run the application, you will need docker and docker-compose installed.

## Quickstart

In order to quickly start a testing build of the application, with Stripe disabled, 
you can run the following command.

```bash
BUILD=true ENABLE_STRIPE=false ./run-dev.sh 
```

This will take a few minutes to build, and will then be accessible at [https://localhost:8443](https://localhost:8443).
Make sure that you answer Yes (Y) when prompted whether you want to see generated secrets,
since you will need the value of ADMIN_PASSWORD to login to the default admin account.

You can login to the newly started application with the default created admin account, 
which is authenticated with ADMIN_EMAIL and ADMIN_PASSWORD.

## Stripe

Running with Stripe requires that you first make a Stripe account, and then get your secret key and publishable key.

```bash
BUILD=true ENABLE_STRIPE=true STRIPE_SECRET_KEY='{YOUR SECRET KEY}' STRIPE_PUBLISHABLE_KEY='{YOUR PUBLISHABLE KEY}' ./run-dev.sh
```
