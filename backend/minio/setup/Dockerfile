FROM alpine:latest

ARG MINIO_HOST
RUN apk update
RUN apk add curl bash
RUN curl https://dl.min.io/client/mc/release/linux-amd64/mc -o /bin/mc
RUN chmod +x /bin/mc

COPY /startup.sh /bin/startup.sh
COPY /user-policy.json /user-policy.json
RUN chmod +x /bin/startup.sh
ENV MINIO_HOST=$MINIO_HOST
ENTRYPOINT '/bin/bash'
CMD '/bin/startup.sh'
