#!/bin/sh

CERTS_DIR=$(git rev-parse --show-toplevel)/certs
SERVER_IP=$(hostname -i | awk '{print $1}')
SERVER_NAME=$(grep "$(hostname -i)" /etc/hosts | awk '{print $2}')

if [ -z "$SERVER_NAME" ]; then
    SERVER_NAME=$(hostname -i)
fi

openssl req -x509 -newkey rsa -days 3650 \
  -noenc -subj "/CN=${SERVER_NAME}" \
  -keyout $CERTS_DIR/privkey.pem \
  -out $CERTS_DIR/fullchain.pem \
  -addext "basicConstraints=critical,CA:FALSE" \
  -addext "subjectAltName=DNS:${SERVER_NAME},DNS:*.${SERVER_NAME},IP:${SERVER_IP}"

cat $CERTS_DIR/fullchain.pem $CERTS_DIR/privkey.pem > $CERTS_DIR/combined.pem
chmod 644 $CERTS_DIR/combined.pem
