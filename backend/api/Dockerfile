FROM rust:alpine AS builder

RUN apk add musl-dev
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=false
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest
RUN apk add curl

ARG DB_HOST
ARG REDIS_HOST
ARG DB_USERNAME
ARG DB_DATABASE

COPY /healthcheck.sh /bin/healthcheck.sh
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/securecart-api /bin/securecart-api
RUN chmod +x /bin/healthcheck.sh

CMD ["./bin/securecart-api"]
