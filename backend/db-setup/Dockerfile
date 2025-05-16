FROM alpine:3.21.3

RUN apk add postgresql-client bash argon2 openssl


COPY /create-admin.sh /bin/create-admin.sh
RUN chmod +x /bin/create-admin.sh

CMD /bin/create-admin.sh
