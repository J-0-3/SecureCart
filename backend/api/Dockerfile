FROM rust:alpine AS builder

ARG ENABLE_STRIPE

RUN apk add musl-dev openssl-dev openssl-libs-static
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=false

RUN if [ "$ENABLE_STRIPE" = "true" ]; then \
        cargo build --release --target=x86_64-unknown-linux-musl --features stripe; \
    else \
        cargo build --release --target=x86_64-unknown-linux-musl; \
    fi

FROM alpine:latest
RUN apk add curl

COPY /healthcheck.sh /bin/healthcheck.sh
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/securecart-api /bin/securecart-api
RUN chmod +x /bin/healthcheck.sh

CMD ["./bin/securecart-api"]
