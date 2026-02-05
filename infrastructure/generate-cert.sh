#!/bin/sh

SERVER_IP=$(hostname -i | awk '{print $1}')
SERVER_NAME=$(grep "$(hostname -i)" /etc/hosts | awk '{print $2}')

openssl req -x509 -newkey rsa -days 3650 \
  -noenc -subj "/CN=${SERVER_NAME}" \
  -keyout certs/privkey.pem \
  -out certs/fullchain.pem \
  -addext "basicConstraints=critical,CA:FALSE" \
  -addext "subjectAltName=DNS:${SERVER_NAME},DNS:*.${SERVER_NAME},IP:${SERVER_IP}"

cat certs/fullchain.pem certs/privkey.pem > certs/combined.pem
chmod 600 certs/combined.pem

