FROM alpine:latest AS builder
RUN apk add npm
RUN npm install -g typescript 
WORKDIR /app
COPY tsconfig.json tsconfig.json
COPY package.json package.json
RUN npm install
COPY src src
RUN tsc

FROM nginx:1.28.0-alpine3.21-slim AS server
RUN apk add curl bash --no-cache
COPY healthcheck.sh /bin/healthcheck.sh
RUN chmod +x /bin/healthcheck.sh
COPY --from=builder /app/src/ /usr/share/nginx/html/
RUN rm -rf /usr/share/nginx/html/ts # prevent accidentally leaking TS source in case of comments etc
