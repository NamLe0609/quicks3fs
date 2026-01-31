#!/bin/sh

SERVER_IP=$(hostname -i | awk '{print $1}')
SERVER_NAME=$(grep "$(hostname -i)" /etc/hosts | awk '{print $2}')

openssl req -x509 -newkey ed25519 -days 3650 \
  -noenc -keyout example.com.key -out example.com.crt -subj "/CN=${SERVER_NAME}" \
  -keyout certs/privkey.pem \
  -out certs/fullchain.pem \
  -addext "subjectAltName=DNS:${SERVER_NAME},DNS:*.${SERVER_NAME},IP:${SERVER_IP}"
