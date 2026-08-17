#!/usr/bin/env sh
set -eu
mkdir -p .secrets
if [ ! -s .secrets/private.pem ]; then
  openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out .secrets/private.pem
  openssl rsa -pubout -in .secrets/private.pem -out .secrets/public.pem
  chmod 600 .secrets/private.pem
fi
printf 'Chaves RS256 de desenvolvimento disponíveis em .secrets/\n'
