FROM alpine:latest AS builder
RUN apk add npm
RUN npm install -g typescript 
WORKDIR /app
COPY tsconfig.json tsconfig.json
COPY package.json package.json
RUN npm install
COPY src src
RUN tsc

FROM nginx AS server
COPY healthcheck.sh /bin/healthcheck.sh
RUN chmod +x /bin/healthcheck.sh
COPY --from=builder /app/src/ /usr/share/nginx/html/
RUN rm -rf /app/src/ts # prevent accidentally leaking TS source in case of comments etc
