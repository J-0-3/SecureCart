FROM alpine:latest

ARG MINIO_HOST
RUN apk update
RUN apk add curl bash
RUN curl https://dl.min.io/client/mc/release/linux-amd64/mc -o /bin/mc
RUN chmod +x /bin/mc

COPY /startup.sh /bin/startup.sh
RUN chmod +x /bin/startup.sh
COPY /user-policy.json /user-policy.json
ENV MINIO_HOST=$MINIO_HOST
CMD '/bin/startup.sh'
