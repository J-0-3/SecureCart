#!/bin/bash

openssl req -x509 -nodes -newkey ec:<(openssl ecparam -name secp521r1) -keyout $1 -out $2 -sha256 -days 365 -subj "/C=GB/ST=Warwickshire/L=Coventry/O=SecureCart/OU=SecureCart/CN=securecart.local"
