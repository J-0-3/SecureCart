#!/usr/bin/env bash

EMAIL=$(cat /run/secrets/admin_email)
FORENAME="Administrator"
SURNAME="Administrator"
ROLE="Administrator"
ADDRESS="21 Fake Street"
PASSWORD=$(cat /run/secrets/admin_password)

ENCRYPTION_KEY=$(cat /run/secrets/db_encryption_key)

export PGPASSWORD="$(cat /run/secrets/db_password)"

USER_ID=$(psql -h "$DB_HOST" -U "$DB_USERNAME" -d "$DB_DATABASE" -w -t -c "
  INSERT INTO AppUser (email, forename, surname, address, role)
  VALUES ('$EMAIL', pgp_sym_encrypt('$FORENAME', '$ENCRYPTION_KEY'),
  pgp_sym_encrypt('$SURNAME', '$ENCRYPTION_KEY'), pgp_sym_encrypt('$ADDRESS', '$ENCRYPTION_KEY'),'$ROLE')
  RETURNING id;
" | head -n 1 | tr -d ' ')

echo "[✔] Created Administrator user."
echo "[.] Setting password for Administrator."

SALT=$(openssl rand -hex 16)

HASHED_PASSWORD=$(echo -n "$PASSWORD" | argon2 "$SALT" -id -t 3 -m 14 -p 1 | grep "Encoded:" | awk '{print $2}')

psql -h "$DB_HOST" -U "$DB_USERNAME" -d "$DB_DATABASE" -c "
  INSERT INTO Password (user_id, password)
  VALUES ('$USER_ID', '$HASHED_PASSWORD');
  "

echo "[✔] Successfully set password for Administrator."
