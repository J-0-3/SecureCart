FROM alpine:latest AS certgenerator
RUN apk update && apk upgrade
COPY /ssl /ssl
RUN apk add openssl3 bash --no-cache
RUN if [ -f "/ssl/securecart.crt" ] -a [ -f "/ssl/securecart.key" ]; then \
        true; \
    else \
        /bin/bash /ssl/genkey.sh /ssl/securecart.key /ssl/securecart.crt; \
    fi

FROM nginx:1.28.0
COPY nginx.conf /etc/nginx/nginx.conf
COPY healthcheck.sh /bin/healthcheck.sh
RUN chmod +x /bin/healthcheck.sh
COPY --from=certgenerator /ssl /ssl

