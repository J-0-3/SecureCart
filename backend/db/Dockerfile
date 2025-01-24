FROM postgres
COPY /schema.sql /docker-entrypoint-initdb.d/schema.sql
ENV POSTGRES_PASSWORD_FILE=/run/secrets/db_password
ENV POSTGRES_USER=securecart
ENV POSTGRES_DB=securecart
